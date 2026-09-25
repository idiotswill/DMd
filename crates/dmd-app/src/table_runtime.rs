//! Durable desktop application service. UI and local interpretation never write projections.
use crate::{
    CampaignRuntime, CharacterCreationOptions, RunnableCampaignError, TABLE_EVENT_KIND,
    TABLE_EVENT_VERSION, TableAction, TableCampaignSummary, TableCharacterView, TableEvent,
    TableObservationBody, TableReceipt, TableSaveBonus, TableSheetDetails, TableSkillBonus,
    TableTextResult, TableTranscriptEntry, TableView, TableViewer,
    rules_runtime::load_rules_pack,
    table_engine::{player_channel, resolve_table, table},
};
use dmd_conversation::{LocalText, interpret_local_text};
use dmd_domain::*;
use dmd_persistence::{
    append_session_observations, commit_campaign_transition_with_session,
    load_campaign_observations, load_command_audit, load_journal_events, load_session_observation,
};
use dmd_rules::{RulesAnswer, RulesQuery};

fn invalid(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::TableRejected(error.to_string())
}
fn recovery(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::Table(error.to_string())
}

fn roll_label(purpose: &PendingPurpose, state: &CampaignState) -> String {
    let ability_name = |ability: &Ability| match ability {
        Ability::Strength => "Strength",
        Ability::Dexterity => "Dexterity",
        Ability::Constitution => "Constitution",
        Ability::Intelligence => "Intelligence",
        Ability::Wisdom => "Wisdom",
        Ability::Charisma => "Charisma",
    };
    let skill_name = |skill: &Skill| match skill {
        Skill::Acrobatics => "Acrobatics",
        Skill::AnimalHandling => "Animal Handling",
        Skill::Arcana => "Arcana",
        Skill::Athletics => "Athletics",
        Skill::Deception => "Deception",
        Skill::History => "History",
        Skill::Insight => "Insight",
        Skill::Intimidation => "Intimidation",
        Skill::Investigation => "Investigation",
        Skill::Medicine => "Medicine",
        Skill::Nature => "Nature",
        Skill::Perception => "Perception",
        Skill::Performance => "Performance",
        Skill::Persuasion => "Persuasion",
        Skill::Religion => "Religion",
        Skill::SleightOfHand => "Sleight of Hand",
        Skill::Stealth => "Stealth",
        Skill::Survival => "Survival",
    };
    match purpose {
        PendingPurpose::TacticalInitiative { .. } => "Initiative".into(),
        PendingPurpose::TacticalResolution { key, .. } => match key.role {
            TacticalRollRole::DeathSave => "Death saving throw",
            TacticalRollRole::EffectSave | TacticalRollRole::SpellSave => "Saving throw",
            TacticalRollRole::SpellAmount => "Spell effect roll",
            TacticalRollRole::EffectDamage => "Effect damage",
            TacticalRollRole::Concentration => {
                "Constitution saving throw to maintain concentration"
            }
            TacticalRollRole::StableRecovery => "Stable recovery time",
            TacticalRollRole::CreatureRecharge => "Ability recharge",
            TacticalRollRole::Attack => "Attack roll",
            TacticalRollRole::AttackDamage => "Attack damage",
            TacticalRollRole::FallDamage => "Falling damage",
            TacticalRollRole::LiquidLandingCheck => {
                let landing = state
                    .encounter
                    .as_ref()
                    .and_then(|encounter| encounter.flow.as_ref())
                    .and_then(|flow| flow.resolution.as_ref())
                    .and_then(|resolution| {
                        let pending = resolution
                            .pending
                            .as_ref()
                            .filter(|pending| pending.key == *key)?;
                        let TacticalWorkKind::LiquidLandingCheck { fall } = pending.work.kind
                        else {
                            return None;
                        };
                        resolution.falls.get(usize::from(fall))
                    });
                match landing.map(|fall| &fall.stage) {
                    Some(TacticalFallStage::LandingCheck {
                        choice: LiquidLandingChoice::Athletics,
                        ..
                    }) => "Strength (Athletics) liquid landing check",
                    Some(TacticalFallStage::LandingCheck {
                        choice: LiquidLandingChoice::Acrobatics,
                        ..
                    }) => "Dexterity (Acrobatics) liquid landing check",
                    _ => "Liquid landing check",
                }
            }
        }
        .into(),
        PendingPurpose::Test { kind, .. } => match kind {
            TestKind::Check { ability, skill } => match skill {
                Some(skill) => format!("{} ({}) check", ability_name(ability), skill_name(skill)),
                None => format!("{} check", ability_name(ability)),
            },
            TestKind::Save { ability } => format!("{} saving throw", ability_name(ability)),
            TestKind::Initiative => "Initiative".into(),
            TestKind::DeathSave => "Death saving throw".into(),
        },
        PendingPurpose::Attack { .. } => "Attack roll".into(),
        PendingPurpose::Damage { critical: true, .. } => "Critical hit damage".into(),
        PendingPurpose::Damage { .. } => "Damage".into(),
        PendingPurpose::Healing { .. } => "Healing".into(),
        PendingPurpose::Concentration { .. } => {
            "Constitution saving throw to maintain concentration".into()
        }
        PendingPurpose::RestHitDie => "Short rest healing".into(),
        PendingPurpose::SecondWind => "Second Wind healing".into(),
    }
}

fn sheet_details(rules: &RulesState, entity: &MechanicalEntity) -> TableSheetDetails {
    let mut conditions = dmd_rules::active_conditions(rules, entity.entity_id)
        .into_iter()
        .collect::<Vec<_>>();
    conditions.sort();
    TableSheetDetails {
        ability_scores: entity.ability_scores,
        hit_dice: entity.hit_dice.clone(),
        heroic_inspiration: entity.heroic_inspiration,
        saving_throws: Ability::ALL
            .into_iter()
            .map(|ability| TableSaveBonus {
                ability,
                modifier: dmd_rules::test_modifier(entity, &TestKind::Save { ability }),
                proficient: entity.saving_proficiencies.contains(&ability),
            })
            .collect(),
        skills: dmd_rules::STANDARD_SKILL_ABILITIES
            .into_iter()
            .map(|(skill, ability)| TableSkillBonus {
                skill,
                ability,
                modifier: dmd_rules::test_modifier(
                    entity,
                    &TestKind::Check {
                        ability,
                        skill: Some(skill),
                    },
                ),
                proficiency: entity.skill_proficiencies.get(&skill).copied(),
            })
            .collect(),
        conditions,
        exhaustion: entity.exhaustion,
        death: entity.death.clone(),
    }
}

pub(crate) fn validate_table_observation(
    record: &NewSessionObservation,
) -> Result<TableObservationBody, String> {
    if record.kind != "table.conversation" || record.payload_schema_version != 1 {
        return Err("Unsupported table observation.".into());
    }
    let body: TableObservationBody =
        serde_json::from_str(&record.payload_json).map_err(|error| error.to_string())?;
    if body.meta.id.0 != record.id.0
        || body.meta.campaign_id != record.campaign_id
        || body.meta.issuer != record.issuer
        || body.meta.session_id != record.session_id
        || body.meta.expected_event_sequence != record.observed_event_sequence
    {
        return Err("Table observation body and provenance disagree.".into());
    }
    bounded_text(&body.text, 8000)?;
    bounded_text(&body.answer, 20000)?;
    Ok(body)
}

impl CampaignRuntime {
    pub async fn character_creation_options(
        &self,
        campaign_id: CampaignId,
    ) -> Result<CharacterCreationOptions, RunnableCampaignError> {
        let runnable = self.open_campaign(campaign_id).await?;
        load_rules_pack(runnable.content())?;
        Ok(CharacterCreationOptions {
            catalog: dmd_rules::starter_catalog(),
            fighter_skills: dmd_rules::FIGHTER_SKILLS.to_vec(),
            fighter_masteries: dmd_rules::fighter_mastery_choices()?,
            standard_languages: dmd_rules::STANDARD_LANGUAGES
                .iter()
                .map(|language| (*language).to_owned())
                .collect(),
            alignments: dmd_rules::ALIGNMENTS
                .iter()
                .map(|alignment| (*alignment).to_owned())
                .collect(),
        })
    }

    pub async fn submit_table_text(
        &self,
        meta: CommandMeta,
        text: &str,
    ) -> Result<TableTextResult, RunnableCampaignError> {
        bounded_text(text, 8000).map_err(invalid)?;
        let runnable = self.open_campaign(meta.campaign_id).await?;
        // Resolve a previously accepted submission before consulting current pending state.
        if let Some(audit) = load_command_audit(&self.pool, meta.id)
            .await
            .map_err(recovery)?
        {
            let action: TableAction =
                serde_json::from_str(&audit.payload.json).map_err(recovery)?;
            let submitted = match &action {
                TableAction::Declare { text: submitted } => submitted == text,
                TableAction::Correct {
                    text: submitted, ..
                } => match interpret_local_text(text, &TableSituation::default()) {
                    LocalText::Correction(value) => &value == submitted,
                    _ => false,
                },
                _ => false,
            };
            if !submitted {
                return Err(invalid(
                    "This request identity already belongs to different input.",
                ));
            }
            return self
                .execute_table(meta, action)
                .await
                .map(TableTextResult::Accepted);
        }
        let id = ObservationId(meta.id.0);
        if load_session_observation(&self.pool, meta.campaign_id, id)
            .await
            .map_err(recovery)?
            .is_some()
        {
            return self
                .observe_table_text(meta, id, text)
                .await
                .map(TableTextResult::Observed);
        }
        let current = table(runnable.state()).map_err(invalid)?;
        match interpret_local_text(text, &current.situation) {
            LocalText::RulesQuestion
            | LocalText::CharacterQuestion
            | LocalText::WorldQuestion
            | LocalText::TableChat => self
                .observe_table_text(meta, id, text)
                .await
                .map(TableTextResult::Observed),
            LocalText::Correction(text) => {
                let pending=current.pending.as_ref().ok_or_else(||invalid("There is no uncommitted declaration to correct. Accepted history cannot be rewritten."))?;
                let action = TableAction::Correct {
                    pending_id: pending.id,
                    revision: pending.revision,
                    text,
                };
                self.execute_table(meta, action)
                    .await
                    .map(TableTextResult::Accepted)
            }
            LocalText::Declaration(_) => self
                .execute_table(meta, TableAction::Declare { text: text.into() })
                .await
                .map(TableTextResult::Accepted),
        }
    }
    pub async fn create_table_campaign(
        &self,
        id: CampaignId,
        name: &str,
        contract: TableContract,
    ) -> Result<TableView, RunnableCampaignError> {
        bounded_text(name, 200).map_err(invalid)?;
        contract.validate().map_err(invalid)?;
        if self.matches_existing_creation(id, name, &contract).await? {
            return self.table_view(id, TableViewer::Host).await;
        }
        let mut state = CampaignState::empty(
            Campaign {
                id,
                display_name: name.trim().into(),
                status: CampaignStatus::Active,
                world_seed: rand::random(),
                ruleset: contract.ruleset.clone(),
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        state.table = Some(TableState::new(contract));
        if let Err(error) = self.create_campaign(&state).await {
            if self
                .matches_existing_creation(
                    id,
                    name,
                    &state.table.as_ref().expect("new table").contract,
                )
                .await?
            {
                return self.table_view(id, TableViewer::Host).await;
            }
            return Err(error);
        }
        self.table_view(id, TableViewer::Host).await
    }

    async fn matches_existing_creation(
        &self,
        id: CampaignId,
        name: &str,
        contract: &TableContract,
    ) -> Result<bool, RunnableCampaignError> {
        let initial = dmd_persistence::load_campaign_snapshot_at_or_before(
            &self.pool,
            id,
            0,
            &dmd_persistence::CampaignStateSnapshotCodec::new(),
        )
        .await
        .map_err(recovery)?;
        let Some(initial) = initial else {
            return Ok(false);
        };
        if initial.campaign.display_name != name.trim()
            || initial.table.as_ref().map(|table| &table.contract) != Some(contract)
        {
            return Err(invalid(
                "That campaign identity was already created with different setup. Open the existing campaign or start a new one.",
            ));
        }
        Ok(true)
    }

    pub async fn list_table_campaigns(
        &self,
    ) -> Result<Vec<TableCampaignSummary>, RunnableCampaignError> {
        let rows = dmd_persistence::list_campaigns(&self.pool).await?;
        rows.into_iter()
            .filter(|row| row.storage_status == dmd_persistence::CampaignStorageStatus::Active)
            .map(|row| {
                Ok(TableCampaignSummary {
                    id: serde_json::from_value(serde_json::json!(row.campaign_id))
                        .map_err(recovery)?,
                    name: row.display_name,
                })
            })
            .collect()
    }

    /// Stable command IDs make retry after a lost response safe, including after session end.
    pub async fn execute_table(
        &self,
        meta: CommandMeta,
        action: TableAction,
    ) -> Result<TableReceipt, RunnableCampaignError> {
        let runnable = self.open_campaign(meta.campaign_id).await?;
        if let Some(receipt) = self.accepted_table_receipt(&meta, &action).await? {
            return Ok(receipt);
        }
        let pack = load_rules_pack(runnable.content())?;
        let mut transition =
            resolve_table(runnable.state(), &meta, &action, &pack).map_err(invalid)?;
        transition.state.applied_event_sequence = meta
            .expected_event_sequence
            .checked_add(1)
            .ok_or_else(|| invalid("Event sequence exhausted."))?;
        let payload = SerializedRecord::encode("table.action", 1, &action).map_err(invalid)?;
        let event = PendingEvent {
            id: EventId::new(),
            occurred_at: transition.state.clock.now,
            source: EventSource::RuleResolution,
            actor: meta.actor,
            caused_by_event_ids: vec![],
            payload: transition.event.clone(),
        }
        .encode(TABLE_EVENT_KIND, TABLE_EVENT_VERSION)
        .map_err(invalid)?;
        let explanation = serde_json::to_string(&transition.event.outcome).map_err(invalid)?;
        let result = commit_campaign_transition_with_session(
            &self.pool,
            &meta,
            &payload,
            &transition.state,
            &[event],
            &explanation,
            transition.session_change.as_ref(),
        )
        .await;
        match result {
            Ok(commit) => Ok(TableReceipt {
                command_id: meta.id,
                event_sequence: commit.resulting_event_sequence,
                outcome: transition.event.outcome,
                already_accepted: false,
            }),
            Err(error) => {
                // Another delivery can win the same stable ID between lookup and commit.
                if let Some(receipt) = self.accepted_table_receipt(&meta, &action).await? {
                    return Ok(receipt);
                }
                Err(RunnableCampaignError::Journal(Box::new(error)))
            }
        }
    }

    async fn accepted_table_receipt(
        &self,
        meta: &CommandMeta,
        action: &TableAction,
    ) -> Result<Option<TableReceipt>, RunnableCampaignError> {
        let Some(audit) = load_command_audit(&self.pool, meta.id)
            .await
            .map_err(recovery)?
        else {
            return Ok(None);
        };
        if !audit.accepted
            || &audit.meta != meta
            || audit.payload.kind != "table.action"
            || audit.payload.schema_version != 1
            || serde_json::from_str::<TableAction>(&audit.payload.json).map_err(recovery)?
                != *action
        {
            return Err(invalid(
                "This request identity was already used for different input. Refresh before continuing.",
            ));
        }
        Ok(Some(TableReceipt {
            command_id: meta.id,
            event_sequence: audit.resulting_event_sequence,
            outcome: serde_json::from_str(&audit.resolution_explanation).map_err(recovery)?,
            already_accepted: true,
        }))
    }

    /// Questions and table chatter have their own append-only observation history, not game events.
    pub async fn observe_table_text(
        &self,
        meta: CommandMeta,
        id: ObservationId,
        text: &str,
    ) -> Result<TableObservationBody, RunnableCampaignError> {
        bounded_text(text, 8000).map_err(invalid)?;
        let runnable = self.open_campaign(meta.campaign_id).await?;
        if let Some(saved) = load_session_observation(&self.pool, meta.campaign_id, id)
            .await
            .map_err(recovery)?
        {
            let body = validate_table_observation(&saved.record).map_err(recovery)?;
            if body.meta != meta
                || saved.record.issuer != meta.issuer
                || saved.record.session_id != meta.session_id
                || saved.record.observed_event_sequence != meta.expected_event_sequence
                || body.text != text
                || saved.record.kind != "table.conversation"
                || saved.record.payload_schema_version != 1
            {
                return Err(invalid(
                    "This conversation request identity was already used for different input.",
                ));
            }
            return Ok(body);
        }
        let state = runnable.state();
        let (player, _, actor, _) = player_channel(state, &meta).map_err(invalid)?;
        if state.applied_event_sequence != meta.expected_event_sequence {
            return Err(invalid("The table changed. Refresh before asking again."));
        }
        let table = table(state).map_err(invalid)?;
        let pack = load_rules_pack(runnable.content())?;
        let (audience, answer) = match interpret_local_text(text, &table.situation) {
            LocalText::RulesQuestion => (
                ObservationAudience::Player(player),
                format!(
                    "For an ability check, report the raw d20 face; the rules engine applies the character's ability and proficiency. Advantage uses the higher of two d20s; disadvantage uses the lower. {} Second Wind uses a d10 plus Fighter level and a limited use. This local rules helper covers these questions; ask for clarification for rules outside that scope.",
                    if table.contract.house_rules.ability_test_natural_extremes {
                        "Your table explicitly enabled the house rule that makes natural 1 and 20 automatic ability-test failure and success."
                    } else {
                        "The selected SRD 5.2.1 rules do not make every natural 1 or 20 an automatic check failure or success."
                    }
                ),
            ),
            LocalText::CharacterQuestion => {
                let answer =
                    dmd_rules::query(state, meta.issuer, &RulesQuery::Character { actor }, &pack)?;
                let RulesAnswer::Character {
                    hp,
                    max_hp,
                    temporary_hp,
                    armor_class,
                    proficiency_bonus,
                    ..
                } = answer
                else {
                    return Err(invalid("Character answer unavailable."));
                };
                (
                    ObservationAudience::Player(player),
                    format!(
                        "Your character has {hp}/{max_hp} HP, {temporary_hp} temporary HP, armor class {armor_class}, and proficiency bonus {proficiency_bonus}."
                    ),
                )
            }
            LocalText::WorldQuestion => (
                ObservationAudience::Player(player),
                if table.situation.description.is_empty() {
                    "No situation has been established yet. Ask the host to describe the scene."
                        .into()
                } else {
                    format!(
                        "Established situation: {}. {} Further details or hypothetical actions need clarification; this question has not changed the world.",
                        table.situation.title, table.situation.description
                    )
                },
            ),
            LocalText::TableChat => (
                ObservationAudience::Party,
                "Table chat recorded. No game action was taken.".into(),
            ),
            _ => {
                return Err(invalid(
                    "This text proposes an action or correction; use the declaration path.",
                ));
            }
        };
        let body = TableObservationBody {
            meta: meta.clone(),
            text: text.into(),
            answer,
        };
        let record = NewSessionObservation {
            id,
            campaign_id: meta.campaign_id,
            session_id: meta.session_id,
            issuer: meta.issuer,
            audience,
            observed_event_sequence: state.applied_event_sequence,
            kind: "table.conversation".into(),
            payload_schema_version: 1,
            payload_json: serde_json::to_string(&body).map_err(invalid)?,
        };
        append_session_observations(&self.pool, meta.campaign_id, &[record])
            .await
            .map_err(recovery)?;
        Ok(body)
    }

    pub async fn read_host_situation(
        &self,
        campaign_id: CampaignId,
    ) -> Result<TableSituation, RunnableCampaignError> {
        let runnable = self.open_campaign(campaign_id).await?;
        Ok(table(runnable.state()).map_err(invalid)?.situation.clone())
    }

    pub async fn table_view(
        &self,
        campaign_id: CampaignId,
        viewer: TableViewer,
    ) -> Result<TableView, RunnableCampaignError> {
        let runnable = self.open_campaign(campaign_id).await?;
        let state = runnable.state();
        let table = table(state).map_err(invalid)?;
        if let TableViewer::Player(player) = viewer
            && !state.players.contains_key(&player)
        {
            return Err(invalid("Unknown player view."));
        }
        let pack = load_rules_pack(runnable.content())?;
        let issuer = match viewer {
            TableViewer::Host => CommandIssuer::Admin,
            TableViewer::Player(player) => CommandIssuer::Player(player),
        };
        let mut players = state.players.values().cloned().collect::<Vec<_>>();
        players.sort_by_key(|player| player.id.0);
        let mut characters = Vec::new();
        for character in state.characters.values() {
            let may_see = matches!(viewer, TableViewer::Host)
                || matches!(viewer,TableViewer::Player(player) if character.controlling_player_id==Some(player));
            let sheet = if may_see
                && state
                    .rules
                    .as_ref()
                    .is_some_and(|rules| rules.entities.contains_key(&character.entity_id))
            {
                Some(dmd_rules::query(
                    state,
                    issuer,
                    &RulesQuery::Character {
                        actor: character.entity_id,
                    },
                    &pack,
                )?)
            } else {
                None
            };
            characters.push(TableCharacterView {
                character_id: character.id,
                player_id: character.controlling_player_id,
                entity_id: character.entity_id,
                name: character.display_name.clone(),
                profile: may_see
                    .then(|| table.character_profiles.get(&character.id).cloned())
                    .flatten(),
                sheet,
                equipment: if may_see && table.character_profiles.contains_key(&character.id) {
                    Some(
                        crate::table_equipment::view(
                            state,
                            character.id,
                            &pack,
                            matches!(viewer, TableViewer::Host),
                        )
                        .map_err(invalid)?,
                    )
                } else {
                    None
                },
                second_wind_remaining: if may_see {
                    state
                        .rules
                        .as_ref()
                        .and_then(|r| r.entities.get(&character.entity_id))
                        .and_then(|e| e.character_features.as_ref())
                        .map(|f| f.second_wind_remaining)
                } else {
                    None
                },
                details: if may_see {
                    state.rules.as_ref().and_then(|rules| {
                        rules
                            .entities
                            .get(&character.entity_id)
                            .map(|entity| sheet_details(rules, entity))
                    })
                } else {
                    None
                },
            });
        }
        characters.sort_by_key(|character| character.character_id.0);
        let roll = if state.rules.is_some() {
            match dmd_rules::query(state, issuer, &RulesQuery::PendingRoll, &pack)? {
                RulesAnswer::PendingRoll(Some(mut request)) => {
                    let pending = state
                        .rules
                        .as_ref()
                        .and_then(|rules| rules.pending.as_ref())
                        .filter(|pending| pending.request.id == request.id)
                        .ok_or_else(|| recovery("Visible roll has no matching pending purpose."))?;
                    // Presentation only: leave the persisted request and its replay inputs intact.
                    request.reason = roll_label(&pending.purpose, state);
                    Some(request)
                }
                _ => None,
            }
        } else {
            None
        };
        let pending = table
            .pending
            .as_ref()
            .filter(|pending| {
                matches!(viewer, TableViewer::Host)
                    || matches!(viewer,TableViewer::Player(player) if pending.player_id==player)
            })
            .cloned();
        let events = load_journal_events(&self.pool, campaign_id, 0)
            .await
            .map_err(recovery)?;
        let mut transcript = Vec::new();
        let mut recap = Vec::new();
        for event in events {
            if event.meta.sequence > state.applied_event_sequence {
                break;
            }
            if event.payload.kind != TABLE_EVENT_KIND {
                continue;
            }
            if event.payload.schema_version != TABLE_EVENT_VERSION {
                return Err(recovery("Unsupported table transcript event version."));
            }
            let event_body: TableEvent =
                serde_json::from_str(&event.payload.json).map_err(recovery)?;
            let (kind, text) = match &event_body.action {
                TableAction::Declare { text } => (
                    "declaration",
                    format!("Proposed: {text}\n{}", event_body.outcome.message),
                ),
                TableAction::Correct { text, .. } => (
                    "correction",
                    format!("Correction: {text}\n{}", event_body.outcome.message),
                ),
                TableAction::SubmitPhysical { .. } => {
                    ("outcome", event_body.outcome.message.clone())
                }
                _ => ("table", event_body.outcome.message.clone()),
            };
            if kind == "outcome" {
                recap.push(text.clone());
            }
            let speaker = match event_body.meta.issuer {
                CommandIssuer::Player(player) => state
                    .players
                    .get(&player)
                    .map_or("Player", |p| p.display_name.as_str())
                    .to_owned(),
                _ => "Table".into(),
            };
            transcript.push((
                event.meta.sequence,
                0,
                TableTranscriptEntry {
                    id: event.meta.id.0.to_string(),
                    event_sequence: event.meta.sequence,
                    kind: kind.into(),
                    speaker,
                    text,
                },
            ));
        }
        let mut after = 0;
        loop {
            let page = load_campaign_observations(&self.pool, campaign_id, after, 500)
                .await
                .map_err(recovery)?;
            if page.is_empty() {
                break;
            }
            for observation in &page {
                after = observation.ordinal;
                let record = &observation.record;
                let visible = matches!(record.audience, ObservationAudience::Party)
                    || matches!(viewer, TableViewer::Host)
                    || matches!((&viewer,&record.audience),(TableViewer::Player(a),ObservationAudience::Player(b)) if a==b);
                if !visible || record.observed_event_sequence > state.applied_event_sequence {
                    continue;
                }
                if record.kind != "table.conversation" || record.payload_schema_version != 1 {
                    continue;
                }
                let body = validate_table_observation(record).map_err(recovery)?;
                let speaker = match record.issuer {
                    CommandIssuer::Player(player) => state
                        .players
                        .get(&player)
                        .map_or("Player", |p| p.display_name.as_str())
                        .to_owned(),
                    _ => "Table".into(),
                };
                transcript.push((
                    record.observed_event_sequence,
                    observation.ordinal,
                    TableTranscriptEntry {
                        id: record.id.0.to_string(),
                        event_sequence: record.observed_event_sequence,
                        kind: "conversation".into(),
                        speaker,
                        text: format!("{}\n{}", body.text, body.answer),
                    },
                ));
            }
            if page.len() < 500 {
                break;
            }
        }
        transcript.sort_by_key(|(sequence, ordinal, _)| (*sequence, *ordinal));
        let start = transcript.len().saturating_sub(500);
        let transcript = transcript
            .into_iter()
            .skip(start)
            .map(|(_, _, entry)| entry)
            .collect();
        let recap = recap
            .into_iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        Ok(TableView {
            campaign_id,
            name: state.campaign.display_name.clone(),
            event_sequence: state.applied_event_sequence,
            contract: table.contract.clone(),
            players,
            characters,
            active_session: table.active_session.clone(),
            pending,
            roll,
            tactical: crate::table_tactical::view(state, &viewer).map_err(invalid)?,
            creature_setup: crate::table_creatures::view(
                state,
                matches!(viewer, TableViewer::Host),
            )
            .map_err(invalid)?,
            situation_title: table.situation.title.clone(),
            situation_description: table.situation.description.clone(),
            transcript,
            recap,
        })
    }
}
