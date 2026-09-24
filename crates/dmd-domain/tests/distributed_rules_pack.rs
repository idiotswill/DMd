use std::{collections::BTreeSet, path::Path};

use dmd_domain::{Campaign, CampaignId, CampaignStatus, ContentCatalog, VersionedRef};

#[test]
fn distributed_srd_manifest_verifies_kernel_source_and_license_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/srd-5.2.1");
    let catalog = ContentCatalog::load_from_roots(&[root]).expect("shipped integrity checks pass");
    let campaign = Campaign {
        id: CampaignId::new(),
        display_name: "Distribution validation".into(),
        status: CampaignStatus::Active,
        world_seed: 1,
        ruleset: VersionedRef {
            id: "srd-5.2".into(),
            version: "5.2.1".into(),
        },
        content_packs: vec![],
    };
    let content = catalog
        .resolve_campaign(&campaign)
        .expect("exact rules version resolves");
    assert_eq!(
        content
            .ruleset
            .manifest
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["NOTICE.md", "source.json", "kernel.json"])
    );
}
