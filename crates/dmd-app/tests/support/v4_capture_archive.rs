//! Optional write-once artifacts from genuine accepted application transitions.
use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

const BASE: &str = "58696ac1d0c55ef7f71cfb546fb92e97747ee437";
const CAPTURE_PATHS: [&str; 7] = [
    "crates/dmd-app/tests/table_grapple_public.rs",
    "crates/dmd-app/tests/support/table_grapple_ground_transport.rs",
    "crates/dmd-app/tests/support/table_inspiration_transfer.rs",
    "crates/dmd-app/tests/support/v4_capture_archive.rs",
    "crates/dmd-app/tests/support/v4_capture_ground.rs",
    "crates/dmd-app/tests/support/v4_capture_inspiration.rs",
    "docs/exec-plans/active/gate4-v4-compatibility-captures.md",
];

#[derive(Serialize)]
struct AudienceSnapshot {
    channel: TableTransportChannel,
    raw: TableView,
    presented: TablePresentedView,
    roll_options: Option<Result<TableRollOptions, String>>,
}

struct Snapshot {
    export: CampaignExport,
    audiences: Vec<AudienceSnapshot>,
}

#[derive(Serialize)]
struct Artifact {
    path: String,
    bytes: usize,
    sha256: String,
}

#[derive(Serialize)]
struct Cut {
    label: String,
    command: CommandId,
    version: u32,
    before_sequence: i64,
    after_sequence: i64,
}

pub(crate) struct Capture {
    scenario: &'static str,
    directory: Option<PathBuf>,
    source: Option<serde_json::Value>,
    source_channel: Option<TableTransportChannel>,
    artifacts: Vec<Artifact>,
    cuts: Vec<Cut>,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("capture provenance requires Git");
    assert!(
        output.status.success(),
        "Git provenance failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn source_manifest(root: &Path) -> serde_json::Value {
    assert!(git(root, &["status", "--porcelain"]).trim().is_empty());
    let head = git(root, &["rev-parse", "HEAD"]);
    let tree = git(root, &["rev-parse", "HEAD^{tree}"]);
    let changed = git(root, &["diff", "--name-only", BASE, "HEAD"]);
    assert!(
        changed.lines().all(|path| CAPTURE_PATHS.contains(&path)),
        "capture binary requires unchanged baseline production/content"
    );
    // Bind the compiled capture implementation and its containing helper modules
    // to the source being labeled. These are bytes, not an execution authority.
    for (path, compiled) in [
        (
            CAPTURE_PATHS[0],
            include_bytes!("../table_grapple_public.rs").as_slice(),
        ),
        (
            CAPTURE_PATHS[1],
            include_bytes!("table_grapple_ground_transport.rs").as_slice(),
        ),
        (
            CAPTURE_PATHS[2],
            include_bytes!("table_inspiration_transfer.rs").as_slice(),
        ),
        (
            CAPTURE_PATHS[3],
            include_bytes!("v4_capture_archive.rs").as_slice(),
        ),
        (
            CAPTURE_PATHS[4],
            include_bytes!("v4_capture_ground.rs").as_slice(),
        ),
        (
            CAPTURE_PATHS[5],
            include_bytes!("v4_capture_inspiration.rs").as_slice(),
        ),
    ] {
        assert_eq!(std::fs::read(root.join(path)).unwrap().as_slice(), compiled);
    }
    let entries = git(root, &["ls-tree", "-r", "--full-tree", "HEAD"])
        .lines()
        .map(|line| {
            let (entry, path) = line.split_once('\t').unwrap();
            let bytes = std::fs::read(root.join(path)).unwrap();
            serde_json::json!({"path":path,"git_entry":entry,"working_bytes":bytes.len(),"working_sha256":sha(&bytes)})
        })
        .collect::<Vec<_>>();
    assert_eq!(git(root, &["rev-parse", "HEAD"]), head);
    assert!(git(root, &["status", "--porcelain"]).trim().is_empty());
    serde_json::json!({
        "base":BASE,"head":head.trim(),"tree":tree.trim(),
        "changed_paths":changed.lines().collect::<Vec<_>>(),"entries":entries,
        "compiled_capture_bytes_match":true,
        "limits":"Runner provenance must separately pin toolchain, command, environment, executable and complete log. No native/human-dice evidence."
    })
}

fn same_export(left: &CampaignExport, right: &CampaignExport) {
    let mut right = right.clone();
    right.exported_at_utc.clone_from(&left.exported_at_utc);
    assert_eq!(&right, left, "read or exact retry changed durable history");
}

impl Capture {
    pub(crate) fn new(scenario: &'static str) -> Self {
        let (directory, source) = match std::env::var_os("DMD_V4_CAPTURE_DIR") {
            None => (None, None),
            Some(path) => {
                let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .canonicalize()
                    .unwrap();
                let output = PathBuf::from(path);
                assert!(output.is_absolute(), "capture output must be absolute");
                let output = output
                    .canonicalize()
                    .expect("create the external parent first");
                assert!(
                    !output.starts_with(&root),
                    "capture output must be outside checkout"
                );
                let source = source_manifest(&root);
                let directory = output.join(scenario);
                std::fs::create_dir(&directory).expect("capture scenario must not already exist");
                (Some(directory), Some(source))
            }
        };
        Self {
            scenario,
            directory,
            source,
            source_channel: None,
            artifacts: vec![],
            cuts: vec![],
        }
    }

    pub(crate) fn include_source(&mut self, channel: TableTransportChannel) {
        assert!(matches!(
            channel,
            TableTransportChannel::SourceCreature { .. }
        ));
        self.source_channel = Some(channel);
    }

    async fn snapshot(&self, f: &Fixture) -> Snapshot {
        let before = export_campaign(&f.pool, f.campaign).await.unwrap();
        let mut audiences = vec![];
        for channel in [TableTransportChannel::Host, f.pc(0), f.pc(1)]
            .into_iter()
            .chain(self.source_channel.clone())
        {
            let viewer = match &channel {
                TableTransportChannel::Host => TableViewer::Host,
                TableTransportChannel::Player { player_id, .. }
                | TableTransportChannel::SourceCreature { player_id, .. } => {
                    TableViewer::Player(*player_id)
                }
            };
            let raw = f.runtime.table_view(f.campaign, viewer).await.unwrap();
            let presented = f.view(channel.clone()).await;
            let roll_options = if let Some(roll) = &presented.roll {
                Some(
                    f.runtime
                        .table_roll_options(TableRollOptionsRequest {
                            campaign_id: f.campaign,
                            channel: channel.clone(),
                            revision: presented.revision,
                            roll_id: roll.id,
                        })
                        .await
                        .map_err(|error| error.to_string()),
                )
            } else {
                None
            };
            audiences.push(AudienceSnapshot {
                channel,
                raw,
                presented,
                roll_options,
            });
        }
        let after = export_campaign(&f.pool, f.campaign).await.unwrap();
        same_export(&before, &after);
        Snapshot {
            export: before,
            audiences,
        }
    }

    fn write(&mut self, name: &str, bytes: &[u8]) {
        if let Some(directory) = &self.directory {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(name))
                .unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        self.artifacts.push(Artifact {
            path: name.into(),
            bytes: bytes.len(),
            sha256: sha(bytes),
        });
    }

    fn json(&mut self, name: &str, value: &impl Serialize) {
        self.write(name, &serde_json::to_vec_pretty(value).unwrap());
    }

    /// This cut's advertised next request is actually accepted by the original
    /// cold/portable/replay helper. No synthetic expected result is installed.
    pub(crate) async fn accept(
        &mut self,
        f: &mut Fixture,
        label: &str,
        request: TableTransportRequest,
    ) {
        assert!(label.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'));
        assert!(self.cuts.iter().all(|cut| cut.label != label));
        let before = self.snapshot(f).await;
        assert_eq!(request.campaign_id, f.campaign);
        assert!(matches!(request.version, 3 | 4));
        Box::pin(f.cold(request.clone())).await;
        let response = Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        let after = self.snapshot(f).await;
        let bindings = after
            .export
            .table_transport_bindings
            .iter()
            .filter(|binding| binding.meta.id == request.command_id)
            .collect::<Vec<_>>();
        assert_eq!(bindings.len(), 1);
        let binding = bindings[0];
        assert_eq!(
            binding.request_json,
            serde_json::to_string(&request).unwrap()
        );
        assert_eq!(
            binding.response_json,
            serde_json::to_string(&response).unwrap()
        );
        let events = after
            .export
            .event_journal
            .iter()
            .filter(|event| event.command_id == request.command_id.0.to_string())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        let event: TableEvent = serde_json::from_str(&events[0].payload_json).unwrap();
        assert_eq!(event.meta.id, request.command_id);
        assert_eq!(
            after.export.current_state.applied_event_sequence,
            before.export.current_state.applied_event_sequence + 1
        );
        assert!(
            after
                .export
                .command_audit
                .starts_with(&before.export.command_audit)
        );
        assert!(
            after
                .export
                .event_journal
                .starts_with(&before.export.event_journal)
        );
        assert!(
            after
                .export
                .table_projection_history
                .starts_with(&before.export.table_projection_history)
        );
        // Bindings are stored in command UUID order, not acceptance order.
        // Remove only the new identity and require the entire old vector exact.
        assert_eq!(
            after
                .export
                .table_transport_bindings
                .iter()
                .filter(|binding| binding.meta.id != request.command_id)
                .collect::<Vec<_>>(),
            before
                .export
                .table_transport_bindings
                .iter()
                .collect::<Vec<_>>()
        );
        assert!(after.export.snapshots.starts_with(&before.export.snapshots));
        assert_eq!(
            after
                .export
                .event_causes
                .iter()
                .filter(|cause| cause.event_id != events[0].id)
                .collect::<Vec<_>>(),
            before.export.event_causes.iter().collect::<Vec<_>>()
        );
        assert_eq!(after.export.observations, before.export.observations);
        let stem = format!("{:02}-{label}", self.cuts.len());
        self.json(&format!("{stem}-before-export.json"), &before.export);
        self.json(&format!("{stem}-before-audiences.json"), &before.audiences);
        self.write(
            &format!("{stem}-next-request.json"),
            binding.request_json.as_bytes(),
        );
        self.write(
            &format!("{stem}-accepted-response.json"),
            binding.response_json.as_bytes(),
        );
        self.write(
            &format!("{stem}-accepted-event.json"),
            events[0].payload_json.as_bytes(),
        );
        self.json(&format!("{stem}-binding.json"), binding);
        self.json(&format!("{stem}-expected-after-export.json"), &after.export);
        self.json(
            &format!("{stem}-expected-after-audiences.json"),
            &after.audiences,
        );
        self.cuts.push(Cut {
            label: label.into(),
            command: request.command_id,
            version: request.version,
            before_sequence: before.export.current_state.applied_event_sequence,
            after_sequence: after.export.current_state.applied_event_sequence,
        });
    }

    pub(crate) async fn retained_retry(
        &mut self,
        f: &Fixture,
        label: &str,
        request: &TableTransportRequest,
    ) {
        let before = export_campaign(&f.pool, f.campaign).await.unwrap();
        let binding = before
            .table_transport_bindings
            .iter()
            .find(|binding| binding.meta.id == request.command_id)
            .unwrap();
        let response = Box::pin(f.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_string(&response).unwrap(),
            binding.response_json
        );
        same_export(
            &before,
            &export_campaign(&f.pool, f.campaign).await.unwrap(),
        );
        self.json(&format!("retained-{label}.json"), binding);
    }

    pub(crate) fn finish(mut self, expected: &[&str]) {
        assert_eq!(
            self.cuts
                .iter()
                .map(|cut| cut.label.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        let manifest = serde_json::json!({
            "schema_version":1,"scenario":self.scenario,
            "status":"COMPLETE_GENUINE_APPLICATION_CAPTURES",
            "source":&self.source,"cuts":&self.cuts,"artifacts":&self.artifacts,
            "limits":[
                "SourceCreature aliases the controlling Player audience; it is not a separate projection audience.",
                "Physical roll faces are controlled test values; no human throw or native UI evidence.",
                "Each before image has an actually accepted next request; expected-after images are comparison results.",
                "No full Gate 4 acceptance. Future-version compatibility still requires independent consumer verification."
            ]
        });
        if self.directory.is_some() {
            self.json("manifest.json", &manifest);
            println!(
                "v4 capture {} complete: {}",
                self.scenario,
                self.directory.as_ref().unwrap().display()
            );
        }
    }
}
