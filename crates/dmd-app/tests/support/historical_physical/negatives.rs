//! Hostile copies only. Every rejection leaves a real unrelated campaign usable.
use super::*;
use driver::Fixture;

const UNAUTHORIZED: &str = "issuer is not authorized for this action";
const NO_RAW_CAPABILITY: &str = "That roll is not available in this view.";

fn raw_result(input: &TableTransportInput) -> Option<&RollResult> {
    match input {
        TableTransportInput::Action(action) => match action.as_ref() {
            TableAction::Tactical {
                action:
                    TacticalAction::SubmitRoll { result }
                    | TacticalAction::SubmitRollWithInspiration { result, .. },
            } => Some(result),
            _ => None,
        },
        _ => None,
    }
}
fn replace_raw_handle(input: &mut TableTransportInput, id: RollRequestId) {
    match input {
        TableTransportInput::Action(action) => match action.as_mut() {
            TableAction::Tactical {
                action:
                    TacticalAction::SubmitRoll { result }
                    | TacticalAction::SubmitRollWithInspiration { result, .. },
            } => result.request_id = id,
            _ => panic!("actual raw action required"),
        },
        _ => panic!("actual raw transport required"),
    }
}
fn raw_capability(
    export: &CampaignExport,
    channel: &TableTransportChannel,
    opaque: RollRequestId,
) -> RollRequestId {
    let handle = latest(export, audience(channel))
        .handles
        .iter()
        .find(|h| h.opaque == opaque.0)
        .unwrap();
    let dmd_persistence::ProjectionCapability::Roll { canonical } = handle.capability else {
        panic!("offered key must bind exactly a Roll")
    };
    canonical
}
/// Valid Host translation must reach the owned Grapple save guard. Unrelated
/// players have no offer; those probes deliberately stop at the capability layer.
pub async fn inspired_authority(
    f: &Fixture,
    owner: &TableTransportRequest,
    other: &TableTransportChannel,
) {
    assert!(
        matches!(&owner.input,TableTransportInput::Action(a) if matches!(a.as_ref(),TableAction::Tactical {action:TacticalAction::SubmitRollWithInspiration {..}}))
    );
    assert_ne!(audience(&owner.channel), audience(other));
    let export = f.export().await;
    let state = decoded(&export);
    let pending = state.rules.as_ref().unwrap().pending.as_ref().unwrap();
    assert!(
        matches!(pending.purpose,PendingPurpose::TacticalResolution {key,..} if key.role==TacticalRollRole::GrappleSave)
    );
    let original = raw_result(&owner.input).unwrap();
    let owner_view = f.view(&owner.channel).await;
    let owner_roll = owner_view.roll.as_ref().unwrap();
    assert_eq!(original.request_id, owner_roll.id);
    assert_eq!(
        raw_capability(&export, &owner.channel, owner_roll.id),
        pending.request.id
    );
    let host = f.view(&TableTransportChannel::Host).await;
    let host_roll = host.roll.as_ref().unwrap();
    assert_ne!(host_roll.id, owner_roll.id);
    assert_eq!(
        raw_capability(&export, &TableTransportChannel::Host, host_roll.id),
        pending.request.id
    );
    let mut related = host_roll.clone();
    related.id = owner_roll.id;
    assert_eq!(&related, owner_roll);
    let other_view = f.view(other).await;
    assert!(
        other_view.roll.is_none(),
        "uninvolved Player has no raw offer"
    );
    assert!(!latest(&export,audience(other)).handles.iter().any(|h|matches!(h.capability,dmd_persistence::ProjectionCapability::Roll {canonical} if canonical==pending.request.id)));
    for channel in [TableTransportChannel::Host, other.clone()] {
        let bad = f.request(channel, owner.input.clone()).await;
        assert_eq!(bad.version, owner.version);
        assert_ne!(&bad, owner);
        assert_eq!(
            Box::pin(f.refuse(bad)).await,
            NO_RAW_CAPABILITY,
            "foreign/missing opaque raw capability"
        );
    }
    let mut input = owner.input.clone();
    replace_raw_handle(&mut input, host_roll.id);
    let bad = f.request(TableTransportChannel::Host, input).await;
    assert_eq!(bad.version, owner.version);
    assert_ne!(&bad, owner);
    assert_eq!(bad.revision, host.revision);
    assert_eq!(raw_result(&bad.input).unwrap().dice, original.dice);
    assert_eq!(raw_result(&bad.input).unwrap().source, original.source);
    assert_eq!(
        Box::pin(f.refuse(bad)).await,
        UNAUTHORIZED,
        "Host valid current raw capability must reach body-owner authorization"
    );
}
pub async fn source_alias(
    f: &Fixture,
    source: &TableTransportRequest,
    ordinary: &TableTransportChannel,
) {
    assert!(matches!(
        source.channel,
        TableTransportChannel::SourceCreature { .. }
    ));
    assert!(matches!(ordinary, TableTransportChannel::Player { .. }));
    assert_eq!(audience(&source.channel), audience(ordinary));
    let export = f.export().await;
    let state = decoded(&export);
    let source_view = f.view(&source.channel).await;
    let ordinary_view = f.view(ordinary).await;
    assert_eq!(source_view.revision, ordinary_view.revision);
    assert_eq!(source_view.roll, ordinary_view.roll);
    let result = raw_result(&source.input).unwrap();
    assert_eq!(result.request_id, source_view.roll.as_ref().unwrap().id);
    assert_eq!(
        raw_capability(&export, ordinary, result.request_id),
        state
            .rules
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .request
            .id
    );
    let bad = f.request(ordinary.clone(), source.input.clone()).await;
    assert_eq!(bad.version, source.version);
    assert_ne!(&bad, source);
    assert_eq!(
        Box::pin(f.refuse(bad)).await,
        UNAUTHORIZED,
        "same audience capability does not grant another actor's source authority"
    );
}

pub async fn live_cut(f: &Fixture, cut: &archive::Cut) {
    let original = &cut.request;
    let mut stale = original.clone();
    stale.command_id = CommandId::new();
    stale.revision = ProjectionRevision(uuid::Uuid::new_v4());
    assert_ne!(&stale, original);
    let message = Box::pin(f.refuse(stale)).await;
    assert!(
        message.to_ascii_lowercase().contains("revision")
            || message.to_ascii_lowercase().contains("changed"),
        "{message}"
    );
    let mut foreign = original.clone();
    foreign.command_id = CommandId::new();
    let capability_error = match &mut foreign.input {
        TableTransportInput::GrappleChoice { handle } => {
            *handle = CommandId::new();
            Some("That Grapple choice is not available in this view.")
        }
        TableTransportInput::InspirationTransfer { handle } => {
            *handle = CommandId::new();
            Some("That Inspiration choice is not available in this view.")
        }
        TableTransportInput::MoveGrappled { option, .. } => {
            *option = CommandId::new();
            Some("That ground drag choice is not available in this view.")
        }
        TableTransportInput::Action(action) => match action.as_mut() {
            TableAction::Tactical {
                action:
                    TacticalAction::SubmitRoll { result }
                    | TacticalAction::SubmitRollWithInspiration { result, .. },
            } => {
                result.request_id = RollRequestId::new();
                Some(NO_RAW_CAPABILITY)
            }
            _ => None,
        },
        _ => None,
    };
    if let Some(expected) = capability_error {
        assert_ne!(&foreign, original);
        assert_eq!(
            Box::pin(f.refuse(foreign)).await,
            expected,
            "foreign opaque capability"
        );
    }
    let other = cut
        .before_views
        .iter()
        .find(|v| {
            matches!(v.channel, TableTransportChannel::Player { .. })
                && audience(&v.channel) != audience(&original.channel)
        })
        .unwrap();
    let mut wrong = f
        .request(other.channel.clone(), original.input.clone())
        .await;
    wrong.version = original.version;
    assert_ne!(&wrong, original);
    let error = Box::pin(f.refuse(wrong)).await;
    if let Some(expected) = capability_error {
        assert_eq!(
            error, expected,
            "another audience cannot translate this opaque capability"
        );
    } else {
        eprintln!("historical969 non-opaque wrong-channel refusal: {error}");
    }
    if matches!(
        original.input,
        TableTransportInput::InspirationTransfer { .. }
    ) {
        let mut wrong = f
            .request(TableTransportChannel::Host, original.input.clone())
            .await;
        wrong.version = original.version;
        assert_eq!(
            Box::pin(f.refuse(wrong)).await,
            "Select your current Inspiration choice.",
            "Host has no Player Inspiration choice channel"
        );
    }
    if matches!(&original.input,TableTransportInput::Action(a) if matches!(a.as_ref(),TableAction::Tactical {action:TacticalAction::SubmitRollWithInspiration {..}}))
    {
        Box::pin(inspired_authority(f, original, &other.channel)).await;
    }
    if matches!(
        original.channel,
        TableTransportChannel::SourceCreature { .. }
    ) && raw_result(&original.input).is_some()
    {
        let ordinary = cut
            .before_views
            .iter()
            .find(|v| {
                matches!(v.channel, TableTransportChannel::Player { .. })
                    && audience(&v.channel) == audience(&original.channel)
            })
            .unwrap();
        Box::pin(source_alias(f, original, &ordinary.channel)).await;
    }
    // The valid saved request is accepted by the caller immediately afterward.
}

enum Layer {
    Portable(&'static str),
    Semantic(&'static [&'static str]),
}
fn matched_current(original: &CampaignExport, state: &CampaignState) -> CampaignExport {
    assert_ne!(
        &decoded(original),
        state,
        "hostile image must actually differ"
    );
    let mut bad = original.clone();
    bad.current_state.state_json = serde_json::to_string(state).unwrap();
    let mut snapshot = bad.snapshots.last().unwrap().clone();
    snapshot.event_sequence = bad.current_state.applied_event_sequence;
    snapshot.state_json = bad.current_state.state_json.clone();
    if bad.snapshots.last().unwrap().event_sequence == snapshot.event_sequence {
        *bad.snapshots.last_mut().unwrap() = snapshot;
    } else {
        bad.snapshots.push(snapshot);
    }
    assert_eq!(
        bad.snapshots.last().unwrap().state_json,
        bad.current_state.state_json
    );
    assert_eq!(
        bad.snapshots.first(),
        original.snapshots.first(),
        "original recovery anchor remains literal"
    );
    bad
}
async fn rejected(original: &CampaignExport, bad: CampaignExport, label: &str, layer: Layer) {
    assert_ne!(&bad, original, "negative {label} must be distinct");
    // Pin the expected public boundary before touching the destination. A bad
    // package is not evidence of a successful original-history semantic check.
    match &layer {
        Layer::Portable(fragment) => {
            let message = bad.upgraded().unwrap_err().to_string();
            assert!(
                message.contains(fragment),
                "{label}: portable preflight {message}"
            );
        }
        Layer::Semantic(_) => {
            bad.upgraded().unwrap_or_else(|e| {
                panic!("{label}: unexpectedly failed before owned replay: {e}")
            });
        }
    }
    let destination = Box::pin(Fixture::blank(decoded(original).campaign_id())).await;
    let sentinel_id = CampaignId::new();
    destination
        .runtime
        .create_table_campaign(
            sentinel_id,
            "Unrelated preserved destination",
            TableContract::default(),
        )
        .await
        .unwrap();
    let sentinel = export_campaign(&destination.pool, sentinel_id)
        .await
        .unwrap();
    let cells = driver::rows(&destination.pool).await;
    let error = Box::pin(destination.runtime.restore_campaign(&bad))
        .await
        .unwrap_err();
    match (&layer, error) {
        (Layer::Portable(fragment), RunnableCampaignError::Lifecycle(error)) => {
            assert!(error.to_string().contains(fragment), "{label}: {error}")
        }
        (Layer::Semantic(fragments), RunnableCampaignError::RulesContent(message)) => assert!(
            fragments.iter().any(|s| message.contains(s)),
            "{label}: unexpected semantic guard: {message}"
        ),
        (_, other) => panic!("{label}: wrong rejection layer {other:?}"),
    }
    assert_eq!(
        driver::rows(&destination.pool).await,
        cells,
        "{label}: rejected import altered logical cells"
    );
    compare::exact_export(
        &sentinel,
        &export_campaign(&destination.pool, sentinel_id)
            .await
            .unwrap(),
    );
    assert_eq!(
        destination
            .runtime
            .open_campaign(sentinel_id)
            .await
            .unwrap()
            .state(),
        &decoded(&sentinel)
    );
    assert!(matches!(
        export_campaign(&destination.pool, destination.campaign)
            .await
            .unwrap_err(),
        dmd_persistence::LifecycleError::CampaignNotFound
    ));
    Box::pin(destination.runtime.restore_campaign(original))
        .await
        .unwrap();
    Box::pin(destination.exact_image(original)).await;
    let accepted = driver::rows(&destination.pool).await;
    assert!(matches!(
        Box::pin(destination.runtime.restore_campaign(original))
            .await
            .unwrap_err(),
        RunnableCampaignError::Lifecycle(dmd_persistence::LifecycleError::CampaignAlreadyExists)
    ));
    assert_eq!(driver::rows(&destination.pool).await, accepted);
    compare::exact_export(
        &sentinel,
        &export_campaign(&destination.pool, sentinel_id)
            .await
            .unwrap(),
    );
    Box::pin(destination.close()).await;
    eprintln!(
        "historical969 hostile copy {label}: intended layer, all-table atomicity and subsequent valid restore PASS"
    );
}

pub async fn restore_cases(archive: &archive::Archive) {
    let original = &archive.cuts.last().unwrap().after;
    // Delete actual producers individually, leaving their retained effects and
    // envelope references. These are intentionally structural missing-history
    // refusals, not claims about a later geometry or original-owner validator.
    let creation = if archive.name == "ground-v4" {
        "CreateCharacter"
    } else {
        "CreateCharacterFromSource"
    };
    for kind in [
        creation,
        "EnableGrappleAccess",
        "EnableGrappleTransport",
        "SetSourceCreatureController",
        "AwardHeroicInspiration",
        "AwardExcessInspiration",
        "SubmitRollWithInspiration",
    ] {
        let rows = original
            .event_journal
            .iter()
            .filter(|r| r.payload_json.contains(kind))
            .collect::<Vec<_>>();
        let applicable = match kind {
            "AwardHeroicInspiration" | "AwardExcessInspiration" | "SubmitRollWithInspiration" => {
                archive.name != "ground-v4"
            }
            _ => true,
        };
        if !applicable {
            assert!(rows.is_empty(), "unexpected original {kind}");
            continue;
        }
        assert!(!rows.is_empty(), "required genuine producer {kind}");
        let id = &rows[0].id;
        let mut bad = original.clone();
        bad.event_journal.retain(|r| &r.id != id);
        assert_eq!(bad.event_journal.len() + 1, original.event_journal.len());
        Box::pin(rejected(
            original,
            bad,
            kind,
            Layer::Portable("journal count does not match current-state head"),
        ))
        .await;
    }
    let mut bad = original.clone();
    bad.format_version = 2;
    Box::pin(rejected(
        original,
        bad,
        "legacy format with current authority",
        Layer::Portable("legacy export cannot contain table projection or transport authority"),
    ))
    .await;
    let mut bad = original.clone();
    bad.state_schema_version = 7;
    Box::pin(rejected(
        original,
        bad,
        "unimplemented state schema",
        Layer::Portable("campaign state schema 7 is incompatible"),
    ))
    .await;
    let mut bad = original.clone();
    bad.table_projection_history.last_mut().unwrap().version = 6;
    Box::pin(rejected(
        original,
        bad,
        "unimplemented presentation version",
        Layer::Portable("invalid table projection/transport history"),
    ))
    .await;
    let mut bad = original.clone();
    let digest = &mut bad.table_projection_history.last_mut().unwrap().changes[0].visible_digest;
    let alternate = if digest.starts_with('0') { "1" } else { "0" };
    digest.replace_range(0..1, alternate);
    Box::pin(rejected(
        original,
        bad,
        "altered historical visible digest",
        Layer::Semantic(&["presentation revision does not match its historical audience"]),
    ))
    .await;
    let mut bad = original.clone();
    bad.table_projection_history.last_mut().unwrap().changes[0].previous =
        Some(ProjectionRevision(uuid::Uuid::new_v4()));
    Box::pin(rejected(
        original,
        bad,
        "foreign prior audience revision",
        Layer::Portable("invalid table projection/transport history"),
    ))
    .await;
    let mut bad = original.clone();
    let (record, change) = bad
        .table_projection_history
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, r)| {
            r.changes
                .iter()
                .position(|c| !c.handles.is_empty())
                .map(|j| (i, j))
        })
        .unwrap();
    let owner = bad.table_projection_history[record].changes[change].audience;
    let borrowed = bad.table_projection_history[..record]
        .iter()
        .flat_map(|p| &p.changes)
        .filter(|c| c.audience != owner)
        .flat_map(|c| &c.handles)
        .next()
        .unwrap()
        .opaque;
    assert_ne!(
        bad.table_projection_history[record].changes[change].handles[0].opaque,
        borrowed
    );
    bad.table_projection_history[record].changes[change].handles[0].opaque = borrowed;
    Box::pin(rejected(
        original,
        bad,
        "handle borrows another audience identity",
        Layer::Portable("invalid table projection/transport history"),
    ))
    .await;
    let mut bad = original.clone();
    let visibility = bad
        .table_projection_history
        .iter_mut()
        .rev()
        .find_map(|p| p.transcript.first_mut())
        .unwrap();
    assert!(!visibility.audiences.is_empty());
    visibility.audiences.clear();
    Box::pin(rejected(
        original,
        bad,
        "erased original visibility",
        Layer::Portable("invalid table projection/transport history"),
    ))
    .await;
    let mut bad = original.clone();
    let visibility = bad
        .table_projection_history
        .iter_mut()
        .rev()
        .flat_map(|p| &mut p.transcript)
        .find(|v| v.audiences.len() > 1)
        .unwrap();
    visibility.audiences.pop();
    Box::pin(rejected(
        original,
        bad,
        "removed one legitimate audience from visible event",
        Layer::Semantic(&["presentation history/cause visibility mismatch"]),
    ))
    .await;
    let mut bad = original.clone();
    let binding = bad
        .table_transport_bindings
        .iter_mut()
        .find(|b| matches!(b.audience, ProjectionAudience::Player(_)))
        .unwrap();
    binding.audience = ProjectionAudience::Host;
    Box::pin(rejected(
        original,
        bad,
        "binding borrows Host audience",
        Layer::Portable("invalid table projection/transport history"),
    ))
    .await;

    // Even a matching current/latest snapshot cannot certify invented resources.
    let mut state = decoded(original);
    let actor = state.rules.as_ref().unwrap().timing.as_ref().unwrap().order[0].actor;
    let entity = state
        .rules
        .as_mut()
        .unwrap()
        .entities
        .get_mut(&actor)
        .unwrap();
    entity.heroic_inspiration = !entity.heroic_inspiration;
    Box::pin(rejected(
        original,
        matched_current(original, &state),
        "forged resource plus matching latest snapshot",
        Layer::Semantic(&["rules snapshot disagrees with replay"]),
    ))
    .await;
    if archive.name == "ground-v4" {
        let original = &archive.cuts[5].after; // released, but its paid OA raw is still pending
        let mut state = decoded(original);
        let transport = state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .grapple
            .as_mut()
            .unwrap()
            .transport
            .as_mut()
            .unwrap();
        transport.steps[0].target.to.x += 1;
        Box::pin(rejected(
            original,
            matched_current(original, &state),
            "forged immutable paired prefix",
            Layer::Portable(
                "current state: decoded snapshot violates campaign-state domain invariants",
            ),
        ))
        .await;
        let mut state = decoded(original);
        let pending = state.rules.as_mut().unwrap().pending.as_mut().unwrap();
        assert_ne!(pending.request.roller, Some(actor));
        pending.request.roller = Some(actor);
        Box::pin(rejected(
            original,
            matched_current(original, &state),
            "source raw rebound to PC",
            Layer::Semantic(&["rules snapshot disagrees with replay"]),
        ))
        .await;
        let mut state = decoded(original);
        let ends = &mut state
            .encounter
            .as_mut()
            .unwrap()
            .flow
            .as_mut()
            .unwrap()
            .resolution
            .as_mut()
            .unwrap()
            .grapple
            .as_mut()
            .unwrap()
            .ends;
        assert_eq!(ends.len(), 1);
        ends.clear();
        Box::pin(rejected(
            original,
            matched_current(original, &state),
            "erased free-release receipt",
            Layer::Portable(
                "current state: decoded snapshot violates campaign-state domain invariants",
            ),
        ))
        .await;
    } else {
        let original = &archive.cuts[3].before;
        let mut state = decoded(original);
        let origin = &mut state
            .table
            .as_mut()
            .unwrap()
            .inspiration_transfer
            .as_mut()
            .unwrap()
            .origin;
        assert_eq!(origin.issuer, CommandIssuer::Admin);
        origin.issuer = CommandIssuer::System;
        // This one is a deliberately malformed owner receipt; table validation
        // rejects it before original replay, a separate boundary from forgery above.
        Box::pin(rejected(
            original,
            matched_current(original, &state),
            "altered Inspiration owner receipt",
            Layer::Portable(
                "current state: decoded snapshot violates campaign-state domain invariants",
            ),
        ))
        .await;
        let original = &archive.cuts[4].after;
        let mut state = decoded(original);
        let roll = state.rules.as_mut().unwrap().rolls.last_mut().unwrap();
        assert_eq!(roll.original_result.as_ref().unwrap().dice[0].value, 1);
        roll.original_result.as_mut().unwrap().dice[0].value = 2;
        Box::pin(rejected(
            original,
            matched_current(original, &state),
            "altered consumed original physical face",
            Layer::Semantic(&["rules snapshot disagrees with replay"]),
        ))
        .await;
    }
}

pub async fn post_mass(f: &Fixture, before_activation: &CampaignExport) {
    let original = f.export().await;
    let mut older = decoded(before_activation);
    assert!(older.physical_facts.is_none());
    let mut injected = decoded(&original).physical_facts.unwrap();
    injected.records.clear();
    let prior_event: TableEvent =
        serde_json::from_str(&before_activation.event_journal.last().unwrap().payload_json)
            .unwrap();
    assert!(matches!(
        prior_event.action,
        TableAction::Tactical {
            action: TacticalAction::FinishEncounter
        }
    ));
    injected.origin = prior_event.meta;
    older.physical_facts = Some(injected);
    Box::pin(rejected(
        before_activation,
        matched_current(before_activation, &older),
        "mass attachment without an accepted activation",
        Layer::Semantic(&["Physical activation lacks its exact accepted Host command."]),
    ))
    .await;
    let mut state = decoded(&original);
    let facts = state.physical_facts.as_mut().unwrap();
    let record = facts.records.first().unwrap();
    assert_ne!(facts.origin, record.origin);
    facts.origin = record.origin.clone();
    Box::pin(rejected(
        &original,
        matched_current(&original, &state),
        "body command substituted as activation origin",
        Layer::Semantic(&["Physical activation lacks its exact accepted Host command."]),
    ))
    .await;
    let mut state = decoded(&original);
    let facts = state.physical_facts.as_mut().unwrap();
    let record = facts.records.first_mut().unwrap();
    let PhysicalFactValue::Body { mass, .. } = &mut record.value else {
        panic!("genuine explicit unknown body fact required")
    };
    assert!(mass.is_none());
    *mass = Some(PhysicalMass(1));
    Box::pin(rejected(
        &original,
        matched_current(&original, &state),
        "invented body mass with matching latest snapshot",
        Layer::Semantic(&["Physical fact differs from its original accepted value or scope."]),
    ))
    .await;
    let mut bad = original.clone();
    let activation = decoded(&original)
        .physical_facts
        .unwrap()
        .origin
        .id
        .0
        .to_string();
    let row = bad
        .command_audit
        .iter_mut()
        .find(|row| row.id == activation)
        .unwrap();
    assert_eq!(row.command_schema_version, 6);
    row.command_schema_version = 7;
    Box::pin(rejected(
        &original,
        bad,
        "unsupported mass command envelope",
        Layer::Semantic(&["rules event/audit/envelope metadata mismatch"]),
    ))
    .await;
    let mut bad = original.clone();
    let needle = "\"physical_facts\":";
    assert_eq!(bad.current_state.state_json.matches(needle).count(), 1);
    bad.current_state.state_json = bad.current_state.state_json.replacen(
        needle,
        "\"physical_facts\":null,\"physical_facts\":",
        1,
    );
    Box::pin(rejected(
        &original,
        bad,
        "duplicate authoritative mass attachment",
        Layer::Portable("current state:"),
    ))
    .await;
}
