//! Pure preflight for composed table/rules restores. Raw recovery remains content-agnostic.

use std::collections::{BTreeMap, HashMap, HashSet};

use dmd_domain::{
    AgentRef, CampaignId, CampaignState, CommandId, CommandIssuer, CommandMeta, PendingPurpose,
    PlaySession, PlaySessionId, PlaySessionStatus, RollSource, Ruling, RulingRecord,
};
use dmd_persistence::{
    CampaignExport, CampaignStateSnapshotCodec, CommandAuditRow, EventJournalRow, SessionChange,
};
use dmd_rules::tactical::{
    TACTICAL_EVENT_KIND, TACTICAL_EVENT_VERSION, TacticalAction, TacticalEvent, TacticalOutcome,
};
use dmd_rules::{RULES_EVENT_KIND, RULES_EVENT_VERSION, RulesAction, RulesEvent, RulesPack};

use crate::table_engine::{replay_table, validate_table};
use crate::{TABLE_EVENT_KIND, TABLE_EVENT_VERSION, TableAction, TableEvent, TableOutcome};

enum RecoveryEvent {
    Rules(Box<RulesEvent>),
    Tactical(Box<TacticalEvent>),
    Table(Box<TableEvent>),
}

impl RecoveryEvent {
    fn meta(&self) -> &CommandMeta {
        match self {
            Self::Rules(event) => &event.meta,
            Self::Tactical(event) => &event.meta,
            Self::Table(event) => &event.meta,
        }
    }

    fn rules_event(&self) -> Option<&RulesEvent> {
        match self {
            Self::Rules(event) => Some(event),
            Self::Tactical(_) => None,
            Self::Table(event) => event.rules_event.as_ref(),
        }
    }

    fn command_kind(&self) -> &'static str {
        match self {
            Self::Rules(_) => "rules.action",
            Self::Tactical(_) => "tactical.action",
            Self::Table(_) => "table.action",
        }
    }
}

/// The caller first upgrades and structurally validates the export and resolves its exact pack.
/// This adds gameplay-history checks without touching the export or opening a write transaction.
pub(crate) fn validate_rules_export(
    export: &CampaignExport,
    pack: &RulesPack,
) -> Result<(), String> {
    let current = CampaignState::decode_json(&export.current_state.state_json)
        .map_err(|error| format!("current rules state: {error}"))?;
    validate_table(&current, pack)?;
    for observation in &export.observations {
        if observation.record.kind.starts_with("table.") {
            crate::table_runtime::validate_table_observation(&observation.record)?;
        }
    }
    let codec = CampaignStateSnapshotCodec::new();
    let mut snapshots = BTreeMap::new();
    for row in &export.snapshots {
        let sequence = nonnegative(row.event_sequence, "snapshot sequence")?;
        let version = u32::try_from(row.state_schema_version)
            .map_err(|_| "invalid snapshot schema version".to_owned())?;
        let state = codec
            .decode_state(version, &row.state_json)
            .map_err(|error| format!("rules snapshot {sequence}: {error}"))?;
        validate_table(&state, pack)
            .map_err(|error| format!("rules snapshot {sequence}: {error}"))?;
        if state.campaign_id() != current.campaign_id()
            || state.applied_event_sequence != sequence
            || sequence > current.applied_event_sequence
            || snapshots.insert(sequence, state).is_some()
        {
            return Err("rules snapshot identity/sequence mismatch".into());
        }
    }
    let (&anchor_sequence, anchor) = snapshots
        .first_key_value()
        .ok_or_else(|| "rules export has no recovery anchor".to_owned())?;
    // New tactical authority has always been event-sourced. Unlike pre-journal legacy
    // mechanics, it cannot be authenticated by trusting an initial snapshot of itself.
    // Keep the original pre-tactical anchor so every source-derived payload is replayed.
    if anchor
        .encounter
        .as_ref()
        .is_some_and(|encounter| encounter.flow.is_some())
        || anchor.rules.as_ref().is_some_and(|rules| {
            rules.tactical_effects.is_some() || rules.tactical_inventory.is_some()
        })
    {
        return Err("tactical recovery requires its original pre-tactical anchor".into());
    }

    let mut audits = HashMap::new();
    for row in &export.command_audit {
        let meta = audit_meta(row)?;
        if audits.insert(meta.id, (row, meta)).is_some() {
            return Err("duplicate rules command audit identity".into());
        }
    }
    let mut events = BTreeMap::new();
    let mut rules_commands = HashSet::new();
    for row in &export.event_journal {
        let sequence = nonnegative(row.sequence, "event sequence")?;
        if row.event_kind != RULES_EVENT_KIND
            && row.event_kind != TABLE_EVENT_KIND
            && row.event_kind != TACTICAL_EVENT_KIND
        {
            if sequence > anchor_sequence
                || row.event_kind.starts_with("rules.")
                || row.event_kind.starts_with("table.")
                || row.event_kind.starts_with("tactical.")
            {
                return Err(format!(
                    "unsupported rules recovery event {}@{} at {sequence}",
                    row.event_kind, row.event_schema_version
                ));
            }
            // A backfilled anchor may follow older generic event families. Their opaque payloads
            // have passed persistence validation; no unrecorded historical image is invented here.
            continue;
        }
        let expected_version = if row.event_kind == RULES_EVENT_KIND {
            RULES_EVENT_VERSION
        } else if row.event_kind == TACTICAL_EVENT_KIND {
            TACTICAL_EVENT_VERSION
        } else {
            TABLE_EVENT_VERSION
        };
        if row.event_schema_version != i64::from(expected_version) {
            return Err(format!(
                "unsupported rules event version {} at {sequence}",
                row.event_schema_version
            ));
        }
        let event = if row.event_kind == RULES_EVENT_KIND {
            RecoveryEvent::Rules(
                serde_json::from_str(&row.payload_json)
                    .map_err(|error| format!("rules event {sequence}: {error}"))?,
            )
        } else if row.event_kind == TACTICAL_EVENT_KIND {
            RecoveryEvent::Tactical(
                serde_json::from_str(&row.payload_json)
                    .map_err(|error| format!("tactical event {sequence}: {error}"))?,
            )
        } else {
            RecoveryEvent::Table(
                serde_json::from_str(&row.payload_json)
                    .map_err(|error| format!("table event {sequence}: {error}"))?,
            )
        };
        let (audit, meta) = audits
            .get(&event.meta().id)
            .ok_or_else(|| "rules event has no matching command audit".to_owned())?;
        validate_event(row, &event, audit, meta)?;
        if snapshots
            .get(&sequence)
            .is_some_and(|snapshot| snapshot.clock.now.0 != row.occurred_at_world)
        {
            return Err(format!("rules snapshot/event time mismatch at {sequence}"));
        }
        if !rules_commands.insert(event.meta().id)
            || events.insert(sequence, (row, event)).is_some()
        {
            return Err("rules command must produce exactly one event at a unique sequence".into());
        }
    }
    for (id, (audit, _)) in &audits {
        if (audit.command_kind.starts_with("rules.")
            || audit.command_kind.starts_with("table.")
            || audit.command_kind.starts_with("tactical."))
            && (!matches!(
                audit.command_kind.as_str(),
                "rules.action" | "table.action" | "tactical.action"
            ) || audit.command_schema_version != 1
                || !rules_commands.contains(id))
        {
            return Err("rules command audit lacks its supported typed event".into());
        }
    }

    let checked_commands = events
        .values()
        .map(|(_, event)| (event.meta().id, event))
        .collect::<HashMap<_, _>>();

    let anchor_rulings = anchor
        .rules
        .as_ref()
        .map_or(&[][..], |rules| &rules.rulings);
    let anchor_origins = command_origins(anchor);
    for snapshot in snapshots.values() {
        validate_rulings(
            snapshot,
            anchor_rulings,
            anchor_sequence,
            &audits,
            &checked_commands,
        )?;
        validate_origins(
            snapshot,
            &anchor_origins,
            anchor_sequence,
            &audits,
            &checked_commands,
        )?;
    }
    validate_rulings(
        &current,
        anchor_rulings,
        anchor_sequence,
        &audits,
        &checked_commands,
    )?;
    validate_origins(
        &current,
        &anchor_origins,
        anchor_sequence,
        &audits,
        &checked_commands,
    )?;

    let mut replayed = anchor.clone();
    let mut session_ledger = HashMap::new();
    if let Some(binding) = anchor
        .table
        .as_ref()
        .and_then(|table| table.active_session.as_ref())
    {
        session_ledger.insert(binding.session_id, binding.as_session(anchor.campaign_id()));
    }
    let mut seen_sessions = export
        .event_journal
        .iter()
        .filter(|row| row.sequence <= anchor_sequence as i64)
        .filter_map(|row| row.session_id.as_deref())
        .map(|id| id.to_owned())
        .collect::<HashSet<_>>();
    seen_sessions.extend(session_ledger.keys().map(|id| id.0.to_string()));
    for (&sequence, (row, event)) in events.range((anchor_sequence.saturating_add(1))..) {
        let expected = replayed
            .applied_event_sequence
            .checked_add(1)
            .ok_or_else(|| "rules replay sequence overflow".to_owned())?;
        if sequence != expected {
            return Err(format!(
                "rules replay gap: expected {expected}, found {sequence}"
            ));
        }
        replayed = match event {
            RecoveryEvent::Tactical(event) => {
                if replayed.table.is_some() {
                    return Err("raw tactical event bypasses the table command boundary".into());
                }
                dmd_rules::tactical::replay_tactical(&replayed, event, pack)
                    .map_err(|error| format!("tactical replay at {sequence}: {error}"))?
                    .next_state
            }
            RecoveryEvent::Rules(event) => {
                if replayed.table.is_some() {
                    return Err("raw rules event bypasses the table command boundary".into());
                }
                dmd_rules::replay(&replayed, event, pack)
                    .map_err(|error| format!("rules replay at {sequence}: {error}"))?
                    .next_state
            }
            RecoveryEvent::Table(event) => {
                let transition = replay_table(&replayed, event, pack)
                    .map_err(|error| format!("table replay at {sequence}: {error}"))?;
                if let Some(change) = transition.session_change {
                    match change {
                        SessionChange::Start { session } => {
                            if !seen_sessions.insert(session.id.0.to_string()) {
                                return Err(
                                    "table replay reuses an earlier session identity".into()
                                );
                            }
                            session_ledger.insert(session.id, session);
                        }
                        SessionChange::Replace { expected, next } => {
                            if session_ledger.get(&expected.id) != Some(&expected)
                                || expected.status != PlaySessionStatus::Active
                            {
                                return Err(
                                    "table replay session replacement lacks its exact prior image"
                                        .into(),
                                );
                            }
                            session_ledger.insert(next.id, next);
                        }
                    }
                }
                transition.state
            }
        };
        replayed.applied_event_sequence = sequence;
        if replayed.clock.now.0 != row.occurred_at_world || validate_table(&replayed, pack).is_err()
        {
            return Err(format!("rules replay time/domain mismatch at {sequence}"));
        }
        if let Some(snapshot) = snapshots.get(&sequence)
            && snapshot != &replayed
        {
            return Err(format!(
                "rules snapshot disagrees with replay at {sequence}"
            ));
        }
    }
    if replayed != current {
        return Err("current rules state disagrees with anchor-and-journal replay".into());
    }
    validate_replayed_sessions(export, &session_ledger)?;
    Ok(())
}

fn nonnegative(value: i64, field: &str) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("invalid {field}"))
}

fn actor(kind: Option<&str>, id: Option<&str>) -> Result<Option<AgentRef>, String> {
    match (kind, id) {
        (None, None) => Ok(None),
        (Some("entity"), Some(id)) => serde_json::from_value(serde_json::json!(id))
            .map(AgentRef::Entity)
            .map(Some)
            .map_err(|error| format!("invalid entity actor: {error}")),
        (Some("faction"), Some(id)) => serde_json::from_value(serde_json::json!(id))
            .map(AgentRef::Faction)
            .map(Some)
            .map_err(|error| format!("invalid faction actor: {error}")),
        _ => Err("invalid actor metadata".into()),
    }
}

fn session(id: Option<&str>) -> Result<Option<PlaySessionId>, String> {
    id.map(|id| {
        serde_json::from_value(serde_json::json!(id))
            .map_err(|error| format!("invalid session identity: {error}"))
    })
    .transpose()
}

fn audit_meta(row: &CommandAuditRow) -> Result<CommandMeta, String> {
    let issuer = match (row.issuer_kind.as_str(), row.issuer_player_id.as_deref()) {
        ("player", Some(id)) => CommandIssuer::Player(
            serde_json::from_value(serde_json::json!(id))
                .map_err(|error| format!("invalid player issuer: {error}"))?,
        ),
        ("system", None) => CommandIssuer::System,
        ("admin", None) => CommandIssuer::Admin,
        ("import", None) => CommandIssuer::Import,
        _ => return Err("invalid rules command issuer metadata".into()),
    };
    Ok(CommandMeta {
        id: serde_json::from_value(serde_json::json!(row.id))
            .map_err(|error| format!("invalid command identity: {error}"))?,
        campaign_id: serde_json::from_value(serde_json::json!(row.campaign_id))
            .map_err(|error| format!("invalid command campaign: {error}"))?,
        session_id: session(row.session_id.as_deref())?,
        issuer,
        actor: actor(row.actor_kind.as_deref(), row.actor_id.as_deref())?,
        expected_event_sequence: nonnegative(row.expected_event_sequence, "command sequence")?,
    })
}

fn validate_event(
    row: &EventJournalRow,
    event: &RecoveryEvent,
    audit: &CommandAuditRow,
    audit_meta: &CommandMeta,
) -> Result<(), String> {
    let campaign_id: CampaignId = serde_json::from_value(serde_json::json!(row.campaign_id))
        .map_err(|error| format!("invalid event campaign: {error}"))?;
    let command_id: CommandId = serde_json::from_value(serde_json::json!(row.command_id))
        .map_err(|error| format!("invalid event command: {error}"))?;
    let sequence = nonnegative(row.sequence, "event sequence")?;
    let meta = event.meta();
    if meta != audit_meta
        || meta.campaign_id != campaign_id
        || meta.id != command_id
        || meta.session_id != session(row.session_id.as_deref())?
        || meta.actor != actor(row.actor_kind.as_deref(), row.actor_id.as_deref())?
        || row.source != "rule_resolution"
        || meta.expected_event_sequence.checked_add(1) != Some(sequence)
        || nonnegative(audit.resulting_event_sequence, "audit resulting sequence")? != sequence
        || audit.accepted != 1
        || audit.command_kind != event.command_kind()
        || audit.command_schema_version != 1
    {
        return Err(format!(
            "rules event/audit/envelope metadata mismatch at {sequence}"
        ));
    }
    match event {
        RecoveryEvent::Tactical(event) => {
            let action: TacticalAction = serde_json::from_str(&audit.payload_json)
                .map_err(|e| format!("invalid tactical command action: {e}"))?;
            let outcome: TacticalOutcome = serde_json::from_str(&audit.resolution_explanation)
                .map_err(|e| format!("invalid tactical audit outcome: {e}"))?;
            if action != event.action || outcome != event.outcome {
                return Err(format!(
                    "tactical event action/outcome disagrees with audit at {sequence}"
                ));
            }
        }
        RecoveryEvent::Rules(event) => {
            let action: RulesAction = serde_json::from_str(&audit.payload_json)
                .map_err(|error| format!("invalid rules command action: {error}"))?;
            let outcome: dmd_rules::RulesOutcome =
                serde_json::from_str(&audit.resolution_explanation)
                    .map_err(|error| format!("invalid rules audit outcome: {error}"))?;
            if action != event.action || outcome != event.outcome {
                return Err(format!(
                    "rules event action/outcome disagrees with audit at {sequence}"
                ));
            }
        }
        RecoveryEvent::Table(event) => {
            let action: TableAction = serde_json::from_str(&audit.payload_json)
                .map_err(|error| format!("invalid table command action: {error}"))?;
            let outcome: TableOutcome = serde_json::from_str(&audit.resolution_explanation)
                .map_err(|error| format!("invalid table audit outcome: {error}"))?;
            if action != event.action || outcome != event.outcome {
                return Err(format!(
                    "table event action/outcome disagrees with audit at {sequence}"
                ));
            }
            validate_nested_rules(event)?;
        }
    }
    Ok(())
}

/// Context before an imported anchor cannot be re-created. Even there, a table envelope
/// may contain only its defined nested mechanics, exact authority and matching outcome.
fn validate_nested_rules(event: &TableEvent) -> Result<(), String> {
    if let TableAction::PrepareBattlefield { setup } = &event.action {
        let matched = event.tactical_event.as_ref().is_some_and(|nested| {
            nested.meta == event.meta && matches!(&nested.action,
                TacticalAction::Establish { encounter } if encounter.id == setup.encounter_id
                    && encounter.scene_id == setup.scene_id && encounter.battlefield == setup.battlefield
                    && encounter.geometry_ruling == setup.geometry_ruling && encounter.origin == event.meta
                    && encounter.flow.is_none() && encounter.knowledge.is_empty())
        });
        if !matches!(
            event.meta.issuer,
            CommandIssuer::Admin | CommandIssuer::System
        ) || event.meta.actor.is_some()
            || event.meta.session_id.is_none()
            || event.rules_event.is_some()
            || event.outcome.mechanics.is_some()
            || !matched
        {
            return Err("battlefield setup disagrees with its nested authority".into());
        }
        return Ok(());
    }
    if let TableAction::Tactical { action } = &event.action {
        let valid_authority = match event.meta.issuer {
            CommandIssuer::Player(_) => matches!(event.meta.actor, Some(AgentRef::Entity(_))),
            CommandIssuer::Admin | CommandIssuer::System => event.meta.actor.is_none(),
            CommandIssuer::Import => false,
        };
        if !valid_authority
            || event.meta.session_id.is_none()
            || event.rules_event.is_some()
            || event.outcome.mechanics.is_some()
            || event
                .tactical_event
                .as_ref()
                .is_none_or(|nested| nested.meta != event.meta || &nested.action != action)
        {
            return Err("table tactical action disagrees with its nested authority".into());
        }
        return Ok(());
    }
    if event.tactical_event.is_some() {
        return Err("non-tactical table action contains unsolicited encounter authority".into());
    }
    let player_action = matches!(
        event.action,
        TableAction::Declare { .. }
            | TableAction::Correct { .. }
            | TableAction::CancelDecision { .. }
            | TableAction::SubmitPhysical { .. }
    );
    let valid_authority = if player_action {
        matches!(event.meta.issuer, CommandIssuer::Player(_))
            && matches!(event.meta.actor, Some(AgentRef::Entity(_)))
            && event.meta.session_id.is_some()
    } else {
        matches!(
            event.meta.issuer,
            CommandIssuer::Admin | CommandIssuer::System
        ) && event.meta.actor.is_none()
    };
    if !valid_authority
        || matches!(&event.action, TableAction::StartSession { id, .. } if event.meta.session_id != Some(*id))
        || (matches!(
            event.action,
            TableAction::EndSession | TableAction::Adjudicate { .. }
        ) && event.meta.session_id.is_none())
    {
        return Err("table action authority/session shape is invalid".into());
    }
    let mechanical = matches!(
        event.action,
        TableAction::CreateCharacter { .. }
            | TableAction::Adjudicate { .. }
            | TableAction::SubmitPhysical { .. }
    );
    let Some(nested) = &event.rules_event else {
        if mechanical || event.outcome.mechanics.is_some() {
            return Err("table action is missing its nested rules event".into());
        }
        return Ok(());
    };
    if nested.meta != event.meta || event.outcome.mechanics.as_ref() != Some(&nested.outcome) {
        return Err("nested rules authority/outcome disagrees with table envelope".into());
    }
    let matched = match (&event.action, &nested.action) {
        (
            TableAction::CreateCharacter {
                entity_id, input, ..
            },
            RulesAction::CreateCharacter {
                entity_id: nested_id,
                input: nested_input,
            },
        ) => entity_id == nested_id && input == nested_input,
        (
            TableAction::Adjudicate { request_id, .. },
            RulesAction::RequestTest {
                request_id: nested_id,
                visibility,
                circumstances,
                ruling,
                kind,
                ..
            },
        ) => {
            request_id == nested_id
                && *visibility == dmd_domain::RollVisibility::Public
                && *circumstances == dmd_domain::Circumstances::default()
                && ruling.basis == dmd_domain::RulingBasis::GmAdjudication
                && matches!(kind, dmd_domain::TestKind::Check { .. })
        }
        (
            TableAction::Adjudicate { request_id, .. },
            RulesAction::SecondWind {
                request_id: nested_id,
                ..
            },
        ) => request_id == nested_id,
        (TableAction::SubmitPhysical { request_id, faces }, RulesAction::SubmitRoll { result }) => {
            *request_id == result.request_id
                && result.source == RollSource::Physical
                && result
                    .dice
                    .iter()
                    .map(|die| die.value)
                    .eq(faces.iter().copied())
        }
        _ => false,
    };
    if !matched {
        return Err("table action disagrees with its nested rules action".into());
    }
    Ok(())
}

fn validate_replayed_sessions(
    export: &CampaignExport,
    sessions: &HashMap<PlaySessionId, PlaySession>,
) -> Result<(), String> {
    for session in sessions.values() {
        let id = session.id.0.to_string();
        let row = export
            .play_sessions
            .iter()
            .find(|row| row.id == id)
            .ok_or("replayed session is absent from the exported ledger")?;
        if row.campaign_id != session.campaign_id.0.to_string()
            || row.display_name != session.display_name
            || row.started_at_world != session.started_at_world.0
            || row.ended_at_world != session.ended_at_world.map(|time| time.0)
            || row.status
                != match session.status {
                    PlaySessionStatus::Active => "active",
                    PlaySessionStatus::Closed => "closed",
                }
        {
            return Err("exported session history disagrees with table replay".into());
        }
        let mut rows = export
            .play_session_participants
            .iter()
            .filter(|row| row.session_id == id)
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row.ordinal);
        if rows.len() != session.participants.len()
            || rows.iter().zip(&session.participants).enumerate().any(
                |(index, (row, participant))| {
                    row.ordinal != index as i64
                        || row.player_id != participant.player_id.0.to_string()
                        || row.character_id != participant.character_id.map(|id| id.0.to_string())
                        || row.attendance
                            != match participant.attendance {
                                dmd_domain::AttendanceStatus::Present => "present",
                                dmd_domain::AttendanceStatus::Absent => "absent",
                            }
                },
            )
        {
            return Err("exported attendance disagrees with table replay".into());
        }
    }
    Ok(())
}

fn validate_rulings(
    state: &CampaignState,
    anchor_rulings: &[RulingRecord],
    anchor_sequence: u64,
    audits: &HashMap<CommandId, (&CommandAuditRow, CommandMeta)>,
    commands: &HashMap<CommandId, &RecoveryEvent>,
) -> Result<(), String> {
    let Some(rules) = &state.rules else {
        return Ok(());
    };
    let mut seen = HashSet::new();
    for record in &rules.rulings {
        if !matches!(
            record.command.issuer,
            CommandIssuer::System | CommandIssuer::Admin
        ) || record.command.campaign_id != state.campaign_id()
            || !seen.insert(record.command.id)
        {
            return Err("invalid or duplicate privileged ruling provenance".into());
        }
        if let Some((audit, meta)) = audits.get(&record.command.id) {
            if meta != &record.command
                || !matches!(audit.command_kind.as_str(), "rules.action" | "table.action")
                || audit.command_schema_version != 1
                || audit.accepted != 1
                || nonnegative(audit.resulting_event_sequence, "ruling sequence")?
                    > state.applied_event_sequence
            {
                return Err("ruling provenance disagrees with accepted audit".into());
            }
            let action = commands
                .get(&record.command.id)
                .and_then(|event| event.rules_event())
                .map(|event| &event.action)
                .ok_or("ruling has no checked typed rules action")?;
            if action_ruling(action) != Some(&record.ruling) {
                return Err("ruling disagrees with its originating typed action".into());
            }
        } else if record.command.expected_event_sequence > anchor_sequence
            || !anchor_rulings.contains(record)
        {
            return Err("post-anchor ruling has no authoritative command audit".into());
        }
    }
    Ok(())
}

fn action_ruling(action: &RulesAction) -> Option<&Ruling> {
    match action {
        RulesAction::Initialize { ruling, .. }
        | RulesAction::RequestTest { ruling, .. }
        | RulesAction::AuthorizeAttack { ruling, .. }
        | RulesAction::AuthorizeSpell { ruling, .. }
        | RulesAction::GrantInspiration { ruling, .. }
        | RulesAction::CancelRoll { ruling }
        | RulesAction::ApplyDamage { ruling, .. }
        | RulesAction::Heal { ruling, .. }
        | RulesAction::GrantTemporaryHp { ruling, .. }
        | RulesAction::ApplyEffect { ruling, .. }
        | RulesAction::RemoveEffect { ruling, .. }
        | RulesAction::SetExhaustion { ruling, .. }
        | RulesAction::SetProne { ruling, .. }
        | RulesAction::RecoverResource { ruling, .. }
        | RulesAction::StartRest { ruling, .. }
        | RulesAction::InterruptRest { ruling, .. }
        | RulesAction::FinishRest { ruling, .. }
        | RulesAction::AdvanceTime { ruling, .. }
        | RulesAction::StartCombat { ruling, .. }
        | RulesAction::EndCombat { ruling }
        | RulesAction::UseReaction { ruling, .. }
        | RulesAction::UseBonusAction { ruling, .. } => Some(ruling),
        RulesAction::Attack { .. }
        | RulesAction::CastSpell { .. }
        | RulesAction::SubmitRoll { .. }
        | RulesAction::SubmitRollWithInspiration { .. }
        | RulesAction::SpendResource { .. }
        | RulesAction::SpendHitDie { .. }
        | RulesAction::EndTurn { .. }
        | RulesAction::EndConcentration { .. }
        | RulesAction::CreateCharacter { .. }
        | RulesAction::SecondWind { .. }
        | RulesAction::ResolveInspirationTransfer { .. }
        | RulesAction::SubmitSavageAttacker { .. } => None,
    }
}

fn command_origins(state: &CampaignState) -> Vec<&CommandMeta> {
    let mut origins = state
        .table
        .as_ref()
        .and_then(|table| table.pending.as_ref())
        .map(|pending| vec![&pending.origin])
        .unwrap_or_default();
    if let Some(encounter) = &state.encounter {
        origins.push(&encounter.origin);
        if let Some(flow) = &encounter.flow {
            origins.push(&flow.origin);
        }
        for knowledge in &encounter.knowledge {
            origins.extend(knowledge.contacts.iter().map(|c| &c.origin));
            origins.extend(knowledge.terrain.iter().map(|c| &c.origin));
        }
    }
    let Some(rules) = &state.rules else {
        return origins;
    };
    if let Some(inventory) = &rules.tactical_inventory {
        origins.extend(inventory.receipts.iter().map(|receipt| &receipt.command));
        origins.extend(inventory.loadouts.iter().map(|loadout| &loadout.command));
    }
    if let Some(effects) = &rules.tactical_effects {
        origins.extend(effects.groups.iter().map(|g| &g.source.command));
        for effect in &effects.effects {
            origins.push(&effect.source.command);
            if let Some(stamp) = &effect.established_at {
                origins.push(&stamp.command);
            }
        }
        if let Some(stamp) = &effects.last_operation {
            origins.push(&stamp.command);
        }
        for trigger in &effects.pending {
            origins.push(&trigger.source.command);
            origins.push(&trigger.origin.command);
        }
        origins.extend(
            effects
                .trigger_uses
                .iter()
                .map(|usage| &usage.origin.command),
        );
    }
    origins.extend(
        rules
            .rulings
            .iter()
            .map(|record| &record.command)
            .collect::<Vec<_>>(),
    );
    if let Some(permission) = &rules.permission {
        origins.push(&permission.issued_by);
    }
    if let Some(pending) = &rules.pending {
        origins.push(&pending.issued_by);
        if let Some(meta) = purpose_permission_origin(&pending.purpose) {
            origins.push(meta);
        }
    }
    for roll in &rules.rolls {
        origins.push(&roll.issued_by);
        origins.push(&roll.accepted_by);
        if let Some(meta) = purpose_permission_origin(&roll.purpose) {
            origins.push(meta);
        }
    }
    origins
}

fn purpose_permission_origin(purpose: &PendingPurpose) -> Option<&CommandMeta> {
    match purpose {
        PendingPurpose::Attack { permission, .. } | PendingPurpose::Healing { permission, .. } => {
            Some(&permission.issued_by)
        }
        _ => None,
    }
}

fn validate_origins(
    state: &CampaignState,
    anchor_origins: &[&CommandMeta],
    anchor_sequence: u64,
    audits: &HashMap<CommandId, (&CommandAuditRow, CommandMeta)>,
    commands: &HashMap<CommandId, &RecoveryEvent>,
) -> Result<(), String> {
    let pending = state
        .table
        .as_ref()
        .and_then(|table| table.pending.as_ref());
    for origin in command_origins(state) {
        if let Some((audit, meta)) = audits.get(&origin.id) {
            if origin != meta
                || audit.accepted != 1
                || !matches!(
                    audit.command_kind.as_str(),
                    "rules.action" | "table.action" | "tactical.action"
                )
                || !commands.contains_key(&origin.id)
                || (pending.map(|pending| &pending.origin) != Some(origin)
                    && !commands
                        .get(&origin.id)
                        .is_some_and(|e| matches!(e, RecoveryEvent::Tactical(_)))
                    && !commands.get(&origin.id).is_some_and(|event| matches!(event,
                        RecoveryEvent::Table(event) if matches!(event.action, TableAction::PrepareEquipment { .. } | TableAction::PrepareBattlefield { .. } | TableAction::Tactical { .. })))
                    && commands
                        .get(&origin.id)
                        .and_then(|event| event.rules_event())
                        .is_none())
                || audit.command_schema_version != 1
                || nonnegative(audit.resulting_event_sequence, "rules origin sequence")?
                    > state.applied_event_sequence
            {
                return Err("rules request/result/permission origin disagrees with audit".into());
            }
        } else if origin.expected_event_sequence > anchor_sequence
            || !anchor_origins.contains(&origin)
        {
            return Err(
                "rules request/result/permission origin lacks an authoritative audit".into(),
            );
        }
    }
    if let Some(pending) = pending
        && let Some(event) = commands.get(&pending.origin.id)
    {
        let matches = match event {
            RecoveryEvent::Table(event) => match &event.action {
                TableAction::Declare { text } => {
                    pending.id == event.meta.id
                        && pending.revision == 0
                        && pending.text == text.trim()
                }
                TableAction::Correct {
                    pending_id,
                    revision,
                    text,
                } => {
                    pending.id == *pending_id
                        && revision.checked_add(1) == Some(pending.revision)
                        && pending.text == text.trim()
                }
                _ => false,
            },
            RecoveryEvent::Rules(_) | RecoveryEvent::Tactical(_) => false,
        };
        if !matches {
            return Err("table pending decision disagrees with its originating action".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dmd_domain::{
        Campaign, CampaignStatus, EntityExistence, EntityId, EntityKind, MechanicalEntity,
        RulingBasis, VersionedRef, WorldClock, WorldEntity, WorldInstant,
    };
    use dmd_persistence::{
        CAMPAIGN_EXPORT_FORMAT_VERSION, CampaignLifecycleSummary, CampaignStorageStatus,
        CurrentStateRow, SnapshotRow,
    };
    use dmd_rules::RulesOutcome;

    fn fixture() -> (CampaignExport, RulesPack, EntityId) {
        let pack = RulesPack::from_json(include_str!("../../../content/srd-5.2.1/kernel.json"))
            .expect("shipped rules definitions");
        let campaign_id = CampaignId::new();
        let actor = EntityId::new();
        let mut initial = CampaignState::empty(
            Campaign {
                id: campaign_id,
                display_name: "Restore preflight".into(),
                status: CampaignStatus::Active,
                world_seed: 41,
                ruleset: VersionedRef {
                    id: pack.id.clone(),
                    version: pack.version.clone(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        initial.entities.insert(
            actor,
            WorldEntity {
                id: actor,
                campaign_id,
                display_name: "Independent creature".into(),
                kind: EntityKind::Creature,
                existence: EntityExistence::Present,
                location_id: None,
            },
        );
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id,
            session_id: None,
            issuer: CommandIssuer::System,
            actor: None,
            expected_event_sequence: 0,
        };
        let action = RulesAction::Initialize {
            entities: vec![MechanicalEntity::basic(actor)],
            house_rules: Default::default(),
            ruling: Ruling {
                basis: RulingBasis::Srd { page: 7 },
                reason: "Validated initialization".into(),
            },
        };
        let transition = dmd_rules::resolve(&initial, &meta, &action, &pack).expect("initialize");
        let mut current = transition.next_state;
        current.applied_event_sequence = 1;
        let export = CampaignExport {
            format_version: CAMPAIGN_EXPORT_FORMAT_VERSION,
            state_schema_version: current.schema_version,
            campaign_id: campaign_id.0.to_string(),
            exported_at_utc: "2026-09-24 00:00:00".into(),
            lifecycle: CampaignLifecycleSummary {
                campaign_id: campaign_id.0.to_string(),
                display_name: "Restore preflight".into(),
                storage_status: CampaignStorageStatus::Active,
                state_schema_version: current.schema_version,
                created_at_utc: "2026-09-24 00:00:00".into(),
                archived_at_utc: None,
            },
            current_state: CurrentStateRow {
                campaign_id: campaign_id.0.to_string(),
                schema_version: i64::from(current.schema_version),
                applied_event_sequence: 1,
                state_json: current.encode_json().expect("current state"),
            },
            play_sessions: vec![],
            play_session_participants: vec![],
            observations: vec![],
            command_audit: vec![CommandAuditRow {
                id: meta.id.0.to_string(),
                campaign_id: campaign_id.0.to_string(),
                session_id: None,
                issuer_kind: "system".into(),
                issuer_player_id: None,
                actor_kind: None,
                actor_id: None,
                expected_event_sequence: 0,
                command_kind: "rules.action".into(),
                command_schema_version: 1,
                payload_json: serde_json::to_string(&action).expect("action"),
                accepted: 1,
                resolution_explanation: serde_json::to_string(&transition.outcome)
                    .expect("outcome"),
                resulting_event_sequence: 1,
            }],
            event_journal: vec![EventJournalRow {
                id: dmd_domain::EventId::new().0.to_string(),
                campaign_id: campaign_id.0.to_string(),
                sequence: 1,
                session_id: None,
                occurred_at_world: 0,
                source: "rule_resolution".into(),
                actor_kind: None,
                actor_id: None,
                command_id: meta.id.0.to_string(),
                event_kind: RULES_EVENT_KIND.into(),
                event_schema_version: i64::from(RULES_EVENT_VERSION),
                payload_json: serde_json::to_string(&transition.event).expect("event"),
            }],
            event_causes: vec![],
            snapshots: vec![SnapshotRow {
                campaign_id: campaign_id.0.to_string(),
                event_sequence: 0,
                state_schema_version: i64::from(initial.schema_version),
                state_json: initial.encode_json().expect("initial state"),
                created_at_utc: "2026-09-24 00:00:00".into(),
            }],
        };
        (export, pack, actor)
    }

    #[test]
    fn valid_export_replays_without_mutating_source() {
        let (export, pack, _) = fixture();
        let original = export.clone();
        validate_rules_export(&export, &pack).expect("valid complete rules history");
        assert_eq!(export, original);
    }

    fn append_action(
        export: &mut CampaignExport,
        pack: &RulesPack,
        actor: EntityId,
        action: RulesAction,
    ) {
        let state = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::System,
            actor: Some(AgentRef::Entity(actor)),
            expected_event_sequence: state.applied_event_sequence,
        };
        let transition = dmd_rules::resolve(&state, &meta, &action, pack).unwrap();
        let mut current = transition.next_state;
        current.applied_event_sequence += 1;
        let sequence = i64::try_from(current.applied_event_sequence).unwrap();
        let mut audit = export.command_audit[0].clone();
        audit.id = meta.id.0.to_string();
        audit.actor_kind = Some("entity".into());
        audit.actor_id = Some(actor.0.to_string());
        audit.expected_event_sequence = sequence - 1;
        audit.resulting_event_sequence = sequence;
        audit.payload_json = serde_json::to_string(&action).unwrap();
        audit.resolution_explanation = serde_json::to_string(&transition.outcome).unwrap();
        let mut event = export.event_journal[0].clone();
        event.id = dmd_domain::EventId::new().0.to_string();
        event.command_id = audit.id.clone();
        event.actor_kind.clone_from(&audit.actor_kind);
        event.actor_id.clone_from(&audit.actor_id);
        event.sequence = sequence;
        event.occurred_at_world = current.clock.now.0;
        event.payload_json = serde_json::to_string(&transition.event).unwrap();
        export.command_audit.push(audit);
        export.event_journal.push(event);
        export.current_state.applied_event_sequence = sequence;
        export.current_state.state_json = current.encode_json().unwrap();
    }

    #[test]
    fn prone_and_bonus_action_history_restores_with_origin_audits() {
        use dmd_domain::{
            Circumstances, DieResult, InitiativeEntry, RollRequestId, RollResult, RollSource,
            RollVisibility, TestKind,
        };

        let (mut export, pack, actor) = fixture();
        let ruling = Ruling {
            basis: RulingBasis::Srd { page: 15 },
            reason: "Authoritative combat context".into(),
        };
        let request_id = RollRequestId::new();
        for action in [
            RulesAction::SetProne {
                target: actor,
                prone: true,
                ruling: ruling.clone(),
            },
            RulesAction::RequestTest {
                actor,
                kind: TestKind::Initiative,
                dc: 0,
                visibility: RollVisibility::Public,
                circumstances: Circumstances::default(),
                ruling: ruling.clone(),
                request_id,
            },
            RulesAction::SubmitRoll {
                result: RollResult {
                    request_id,
                    source: RollSource::Digital,
                    dice: vec![DieResult {
                        sides: 20,
                        value: 10,
                    }],
                },
            },
            RulesAction::StartCombat {
                participants: vec![InitiativeEntry {
                    actor,
                    total: 10,
                    tie_break: 0,
                }],
                ruling: ruling.clone(),
            },
            RulesAction::UseBonusAction {
                actor,
                feature_id: "supported-feature".into(),
                ruling,
            },
        ] {
            append_action(&mut export, &pack, actor, action);
            validate_rules_export(&export, &pack).expect("valid combat history");
        }
    }

    #[test]
    fn current_state_and_latest_snapshot_cannot_hide_replay_disagreement() {
        let (mut export, pack, actor) = fixture();
        let mut current = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        current
            .rules
            .as_mut()
            .unwrap()
            .entities
            .get_mut(&actor)
            .unwrap()
            .hp -= 1;
        export.current_state.state_json = current.encode_json().unwrap();
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("current rules state disagrees")
        );
        export.snapshots.push(SnapshotRow {
            campaign_id: export.campaign_id.clone(),
            event_sequence: 1,
            state_schema_version: i64::from(current.schema_version),
            state_json: export.current_state.state_json.clone(),
            created_at_utc: "2026-09-24 00:00:01".into(),
        });
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("snapshot disagrees")
        );
    }

    #[test]
    fn all_rules_envelopes_and_audits_are_checked_even_before_the_anchor() {
        let (mut base, pack, _) = fixture();
        base.snapshots[0].event_sequence = 1;
        base.snapshots[0].state_json = base.current_state.state_json.clone();
        validate_rules_export(&base, &pack).expect("real backfilled current anchor");
        for corruption in [
            "issuer",
            "action",
            "version",
            "source",
            "actor",
            "audit_kind",
        ] {
            let mut export = base.clone();
            match corruption {
                "issuer" => export.command_audit[0].issuer_kind = "admin".into(),
                "action" => {
                    export.command_audit[0].payload_json =
                        serde_json::to_string(&RulesAction::EndConcentration {
                            actor: EntityId::new(),
                        })
                        .unwrap()
                }
                "version" => export.event_journal[0].event_schema_version += 1,
                "source" => export.event_journal[0].source = "import".into(),
                "actor" => {
                    export.event_journal[0].actor_kind = Some("entity".into());
                    export.event_journal[0].actor_id = Some(EntityId::new().0.to_string());
                }
                "audit_kind" => export.command_audit[0].command_kind = "generic.action".into(),
                _ => unreachable!(),
            }
            assert!(
                validate_rules_export(&export, &pack).is_err(),
                "{corruption}"
            );
        }
    }

    #[test]
    fn unknown_post_anchor_events_and_forged_outcomes_fail_closed() {
        let (mut export, pack, _) = fixture();
        export.event_journal[0].event_kind = "generic.changed".into();
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("unsupported")
        );
        export.event_journal[0].event_kind = RULES_EVENT_KIND.into();
        let mut event: RulesEvent =
            serde_json::from_str(&export.event_journal[0].payload_json).unwrap();
        event.outcome = RulesOutcome::Healed { regained: 1 };
        export.event_journal[0].payload_json = serde_json::to_string(&event).unwrap();
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("audit")
        );
        export.command_audit[0].resolution_explanation =
            serde_json::to_string(&event.outcome).unwrap();
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("replay")
        );
    }

    #[test]
    fn new_ruling_provenance_must_have_an_accepted_audit() {
        let (mut export, pack, _) = fixture();
        let mut current = CampaignState::decode_json(&export.current_state.state_json).unwrap();
        let mut invented = current.rules.as_ref().unwrap().rulings[0].clone();
        invented.command.id = CommandId::new();
        current.rules.as_mut().unwrap().rulings.push(invented);
        export.current_state.state_json = current.encode_json().unwrap();
        assert!(
            validate_rules_export(&export, &pack)
                .unwrap_err()
                .contains("no authoritative command audit")
        );
    }
}
