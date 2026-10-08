//! Consumers of the unchanged pre-mass969 oracle. No positive state is manufactured.
use dmd_app::*;
use dmd_domain::*;
use dmd_persistence::{
    CampaignExport, ProjectionAudience, ProjectionChange, ProjectionRevision,
    TableTransportBinding, TransportAcceptance, export_campaign, open_sqlite_path,
};
use dmd_rules::tactical::TacticalAction;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

mod archive;
mod compare;
mod details;
mod driver;
mod mechanics;
mod negatives;
mod play;

fn audience(channel: &TableTransportChannel) -> ProjectionAudience {
    match channel {
        TableTransportChannel::Host => ProjectionAudience::Host,
        TableTransportChannel::Player { player_id, .. }
        | TableTransportChannel::SourceCreature { player_id, .. } => {
            ProjectionAudience::Player(*player_id)
        }
    }
}
fn viewer(channel: &TableTransportChannel) -> TableViewer {
    match audience(channel) {
        ProjectionAudience::Host => TableViewer::Host,
        ProjectionAudience::Player(id) => TableViewer::Player(id),
    }
}
fn tactical(action: TacticalAction) -> TableTransportInput {
    TableTransportInput::Action(Box::new(TableAction::Tactical { action }))
}
fn decoded(export: &CampaignExport) -> CampaignState {
    serde_json::from_str(&export.current_state.state_json).unwrap()
}
fn latest(export: &CampaignExport, who: ProjectionAudience) -> &ProjectionChange {
    export
        .table_projection_history
        .iter()
        .rev()
        .flat_map(|r| &r.changes)
        .find(|c| c.audience == who)
        .unwrap()
}

pub async fn run(name: &str) {
    let archive = archive::load(name);
    mechanics::captured(&archive);
    archive::packaging_negatives(name);
    for (index, cut) in archive.cuts.iter().enumerate() {
        eprintln!(
            "historical969 {name} cut {index:02} {}: restore-before",
            cut.label
        );
        for (phase, export, views) in [
            ("before", &cut.before, &cut.before_views),
            ("after", &cut.after, &cut.after_views),
        ] {
            let mut f = Box::pin(driver::Fixture::restore(export)).await;
            Box::pin(f.exact_views(views)).await;
            Box::pin(f.exact_image(export)).await;
            Box::pin(f.retained(export, &archive.retained)).await;
            if phase == "after" {
                Box::pin(f.retry(&cut.request, &cut.response_bytes)).await;
            }
            Box::pin(f.premature_mass()).await;
            Box::pin(f.reopen()).await;
            Box::pin(f.exact_views(views)).await;
            Box::pin(f.exact_image(export)).await;
            Box::pin(f.retained(export, &archive.retained)).await;
            if phase == "after" {
                Box::pin(f.retry(&cut.request, &cut.response_bytes)).await;
            }
            let portable = Box::pin(driver::Fixture::restore(&f.export().await)).await;
            Box::pin(portable.exact_views(views)).await;
            Box::pin(portable.exact_image(export)).await;
            Box::pin(portable.retained(export, &archive.retained)).await;
            if phase == "after" {
                Box::pin(portable.retry(&cut.request, &cut.response_bytes)).await;
            }
            Box::pin(portable.close()).await;
            Box::pin(f.close()).await;
            eprintln!(
                "historical969 {name} cut {index:02}: {phase} exact cold/portable/retry PASS"
            );
        }
        let mut branch = Box::pin(driver::Fixture::restore(&cut.before)).await;
        Box::pin(negatives::live_cut(&branch, cut)).await;
        let channels = cut
            .after_views
            .iter()
            .map(|v| v.channel.clone())
            .collect::<Vec<_>>();
        let response = Box::pin(branch.accept(cut.request.clone(), &channels)).await;
        let after = branch.export().await;
        compare::fresh(&cut.before, &cut.after, &after, &cut.request);
        assert_eq!(cut.event, compare::event(&after, cut.request.command_id));
        assert_eq!(cut.binding.meta.id, cut.request.command_id);
        compare::receipt(
            &cut.response,
            &response,
            latest(&after, audience(&cut.request.channel)).revision,
        );
        let actual_views = Box::pin(branch.views(&channels)).await;
        compare::fresh_views(
            &cut.before,
            &cut.after,
            &after,
            &cut.after_views,
            &actual_views,
        );
        compare::private_award(&cut.before_views, &cut.after_views, &cut.label, name);
        Box::pin(branch.retained(&cut.before, &archive.retained)).await;
        Box::pin(branch.close()).await;
        eprintln!("historical969 {name} cut {index:02}: genuine next acceptance PASS");
    }
    Box::pin(negatives::restore_cases(&archive)).await;
    Box::pin(play::old_pending(&archive)).await;
    Box::pin(play::new_version(&archive)).await;
    eprintln!(
        "historical969 {} COMPLETE: {} cuts, all retained retries, negatives, pending completion and genuine v5 route",
        archive.name,
        archive.cuts.len()
    );
}
