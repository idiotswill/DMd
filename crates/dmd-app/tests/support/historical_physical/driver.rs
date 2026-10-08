use super::*;
use sqlx::Row;
use std::time::Duration;

pub struct Fixture {
    pub pool: sqlx::SqlitePool,
    pub runtime: CampaignRuntime,
    pub campaign: CampaignId,
    path: PathBuf,
    directory: PathBuf,
}
fn runtime(pool: sqlx::SqlitePool) -> CampaignRuntime {
    CampaignRuntime::from_content_root(
        pool,
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"),
    )
}
pub async fn rows(pool: &sqlx::SqlitePool) -> Vec<(String, Vec<Vec<String>>)> {
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let mut all = Vec::new();
    for table in tables {
        let quoted = format!("\"{}\"", table.replace('"', "\"\""));
        let columns = sqlx::query(&format!("PRAGMA table_info({quoted})"))
            .fetch_all(pool)
            .await
            .unwrap();
        let expressions = columns
            .iter()
            .map(|c| {
                let name: String = c.get("name");
                let name = format!("\"{}\"", name.replace('"', "\"\""));
                format!("typeof({name}) || ':' || hex(CAST({name} AS BLOB))")
            })
            .collect::<Vec<_>>();
        let mut cells = sqlx::query(&format!("SELECT {} FROM {quoted}", expressions.join(",")))
            .fetch_all(pool)
            .await
            .unwrap()
            .iter()
            .map(|r| {
                (0..expressions.len())
                    .map(|i| r.get::<String, _>(i))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        cells.sort();
        all.push((table, cells));
    }
    all
}
impl Fixture {
    pub async fn blank(campaign: CampaignId) -> Self {
        let directory =
            std::env::temp_dir().join(format!("dmd-physical-history-{}", CommandId::new().0));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("campaign.sqlite");
        let pool = open_sqlite_path(&path).await.unwrap();
        Self {
            runtime: runtime(pool.clone()),
            pool,
            campaign,
            path,
            directory,
        }
    }
    pub async fn restore(export: &CampaignExport) -> Self {
        let f = Self::blank(decoded(export).campaign_id()).await;
        Box::pin(f.runtime.restore_campaign(export)).await.unwrap();
        Box::pin(f.exact_image(export)).await;
        f
    }
    pub async fn state(&self) -> CampaignState {
        self.runtime
            .open_campaign(self.campaign)
            .await
            .unwrap()
            .state()
            .clone()
    }
    pub async fn export(&self) -> CampaignExport {
        export_campaign(&self.pool, self.campaign).await.unwrap()
    }
    pub async fn reopen(&mut self) {
        self.pool.close().await;
        self.pool = open_sqlite_path(&self.path).await.unwrap();
        self.runtime = runtime(self.pool.clone());
    }
    pub async fn exact_image(&self, expected: &CampaignExport) {
        let cells = rows(&self.pool).await;
        assert_eq!(self.state().await, decoded(expected));
        assert_eq!(
            Box::pin(self.runtime.replay_rules(self.campaign))
                .await
                .unwrap(),
            decoded(expected)
        );
        compare::exact_export(expected, &self.export().await);
        assert_eq!(
            rows(&self.pool).await,
            cells,
            "read/replay mutated a logical cell"
        );
    }
    pub async fn view(&self, channel: &TableTransportChannel) -> TablePresentedView {
        self.runtime
            .presented_table_view(self.campaign, viewer(channel))
            .await
            .unwrap()
    }
    pub async fn views(&self, channels: &[TableTransportChannel]) -> Vec<archive::Audience> {
        let before = self.export().await;
        let cells = rows(&self.pool).await;
        let mut views = Vec::new();
        for channel in channels {
            let raw = self
                .runtime
                .table_view(self.campaign, viewer(channel))
                .await
                .unwrap();
            let presented = self.view(channel).await;
            let roll_options = match &presented.roll {
                Some(roll) => Some(
                    self.runtime
                        .table_roll_options(TableRollOptionsRequest {
                            campaign_id: self.campaign,
                            channel: channel.clone(),
                            revision: presented.revision,
                            roll_id: roll.id,
                        })
                        .await
                        .map_err(|e| e.to_string()),
                ),
                None => None,
            };
            views.push(archive::Audience {
                channel: channel.clone(),
                raw,
                presented,
                roll_options,
            });
        }
        Box::pin(super::details::assert_reads(self, &views)).await;
        compare::exact_export(&before, &self.export().await);
        assert_eq!(rows(&self.pool).await, cells);
        views
    }
    pub async fn exact_views(&self, expected: &[archive::Audience]) {
        let channels = expected
            .iter()
            .map(|v| v.channel.clone())
            .collect::<Vec<_>>();
        let actual = self.views(&channels).await;
        for (old, new) in expected.iter().zip(actual) {
            assert_eq!(old.channel, new.channel);
            assert_eq!(
                serde_json::to_vec(&old.raw).unwrap(),
                serde_json::to_vec(&new.raw).unwrap(),
                "original raw DTO bytes"
            );
            assert_eq!(
                serde_json::to_vec(&old.presented).unwrap(),
                serde_json::to_vec(&new.presented).unwrap(),
                "original presented DTO bytes"
            );
            assert_eq!(
                old.roll_options, new.roll_options,
                "original channel-specific options"
            );
        }
    }
    pub async fn request(
        &self,
        channel: TableTransportChannel,
        input: TableTransportInput,
    ) -> TableTransportRequest {
        let view = self.view(&channel).await;
        TableTransportRequest {
            version: if view.physical.is_some()
                || matches!(input, TableTransportInput::EnablePhysicalFacts)
            {
                5
            } else {
                view.grapple.as_ref().map_or(1, |g| g.version)
            },
            command_id: CommandId::new(),
            campaign_id: self.campaign,
            session_id: view.active_session.as_ref().map(|s| s.session_id),
            channel,
            revision: view.revision,
            input,
        }
    }
    pub async fn retry(&self, request: &TableTransportRequest, expected: &[u8]) {
        let before = self.export().await;
        let cells = rows(&self.pool).await;
        let result = Box::pin(self.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&result).unwrap(),
            expected,
            "literal accepted retry"
        );
        compare::exact_export(&before, &self.export().await);
        assert_eq!(rows(&self.pool).await, cells);
    }
    pub async fn retained(&self, checkpoint: &CampaignExport, retained: &[TableTransportBinding]) {
        let before = self.export().await;
        for binding in compare::retained_bindings(checkpoint, &before, retained) {
            let request: TableTransportRequest =
                serde_json::from_str(&binding.request_json).unwrap();
            assert_eq!(
                serde_json::to_string(&request).unwrap(),
                binding.request_json
            );
            self.retry(&request, binding.response_json.as_bytes()).await;
        }
    }
    pub async fn refuse(&self, request: TableTransportRequest) -> String {
        let before = self.export().await;
        let cells = rows(&self.pool).await;
        let error = match Box::pin(self.runtime.submit_presented_table(request))
            .await
            .unwrap_err()
        {
            RunnableCampaignError::TableRejected(message) => message,
            other => panic!("expected atomic input rejection, got {other:?}"),
        };
        compare::exact_export(&before, &self.export().await);
        assert_eq!(rows(&self.pool).await, cells);
        error
    }
    pub async fn premature_mass(&self) {
        let state = self.state().await;
        assert!(state.physical_facts.is_none());
        assert_eq!(
            state.encounter.unwrap().flow.unwrap().phase,
            TacticalPhase::Active
        );
        let host = self.view(&TableTransportChannel::Host).await;
        assert!(host.physical.as_ref().is_none_or(|p| p.controls.is_empty()));
        let request = self
            .request(
                TableTransportChannel::Host,
                TableTransportInput::EnablePhysicalFacts,
            )
            .await;
        assert_eq!(request.version, 5);
        let error = self.refuse(request).await;
        assert_eq!(
            error,
            "invalid mechanics: This physical-fact control is no longer current."
        );
    }
    /// Genuine acceptance after cold reopen, plus independent portable acceptance.
    /// Each branch retains its own fresh opaque IDs and exact retry response.
    pub async fn accept(
        &mut self,
        request: TableTransportRequest,
        channels: &[TableTransportChannel],
    ) -> TableTransportResult {
        let before = self.export().await;
        let mut mirror = Box::pin(Self::restore(&before)).await;
        Box::pin(self.reopen()).await;
        let actual = Box::pin(self.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        let other = Box::pin(mirror.runtime.submit_presented_table(request.clone()))
            .await
            .unwrap();
        let after = self.export().await;
        let mirrored = mirror.export().await;
        assert_eq!(
            after.command_audit.last().unwrap().command_schema_version,
            match request.version {
                4 => 5,
                5 => 6,
                _ => panic!("unreviewed consumer producer version"),
            }
        );
        assert_eq!(
            after.table_projection_history.last().unwrap().version,
            request.version
        );
        assert_eq!(
            after
                .table_transport_bindings
                .iter()
                .find(|b| b.meta.id == request.command_id)
                .unwrap()
                .version,
            request.version
        );
        compare::fresh(&before, &after, &mirrored, &request);
        compare::receipt(
            &actual,
            &other,
            latest(&mirrored, audience(&request.channel)).revision,
        );
        let actual_views = Box::pin(self.views(channels)).await;
        let mirror_views = Box::pin(mirror.views(channels)).await;
        compare::fresh_views(&before, &after, &mirrored, &actual_views, &mirror_views);
        Box::pin(mirror.retry(&request, &serde_json::to_vec(&other).unwrap())).await;
        Box::pin(mirror.reopen()).await;
        Box::pin(mirror.retry(&request, &serde_json::to_vec(&other).unwrap())).await;
        Box::pin(mirror.exact_image(&mirrored)).await;
        Box::pin(mirror.close()).await;
        Box::pin(self.retry(&request, &serde_json::to_vec(&actual).unwrap())).await;
        Box::pin(self.reopen()).await;
        Box::pin(self.retry(&request, &serde_json::to_vec(&actual).unwrap())).await;
        Box::pin(self.exact_image(&after)).await;
        let portable = Box::pin(Self::restore(&after)).await;
        Box::pin(portable.retry(&request, &serde_json::to_vec(&actual).unwrap())).await;
        Box::pin(portable.exact_views(&actual_views)).await;
        Box::pin(portable.close()).await;
        let mut changed = request.clone();
        changed.input = if changed.input == tactical(TacticalAction::Dodge) {
            tactical(TacticalAction::Dash {
                speed: DashSpeed::Speed,
            })
        } else {
            tactical(TacticalAction::Dodge)
        };
        assert_ne!(changed, request);
        Box::pin(self.refuse(changed)).await;
        actual
    }
    pub async fn close(self) {
        self.pool.close().await;
        let directory = self.directory.clone();
        drop(self);
        let resolved = directory.canonicalize().unwrap();
        assert_eq!(
            resolved.parent(),
            Some(std::env::temp_dir().canonicalize().unwrap().as_path())
        );
        assert!(
            resolved
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("dmd-physical-history-")
        );
        for attempt in 0..20 {
            match std::fs::remove_dir_all(&resolved) {
                Ok(()) => return,
                Err(e)
                    if cfg!(windows)
                        && matches!(e.raw_os_error(), Some(32 | 33))
                        && attempt < 19 =>
                {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => panic!("closed historical fixture cleanup: {e}"),
            }
        }
        unreachable!()
    }
}
