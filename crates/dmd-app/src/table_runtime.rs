//! Durable desktop application service. UI and local interpretation never write projections.
use crate::{
    CampaignRuntime, CharacterCreationOptions, RunnableCampaignError, TableAction,
    TableCampaignSummary, TableObservationBody, TableReceipt, TableSaveBonus, TableSheetDetails,
    TableSkillBonus, TableTextResult, TableView, TableViewer,
    rules_runtime::load_rules_pack,
    table_engine::{player_channel, table},
};
use dmd_conversation::{LocalText, interpret_local_text};
use dmd_domain::*;
use dmd_persistence::{
    load_campaign_observations, load_command_audit, load_journal_events, load_session_observation,
};
use dmd_rules::{RulesAnswer, RulesQuery};

fn invalid(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::TableRejected(error.to_string())
}
fn recovery(error: impl ToString) -> RunnableCampaignError {
    RunnableCampaignError::Table(error.to_string())
}

pub(crate) fn roll_label(purpose: &PendingPurpose, state: &CampaignState) -> String {
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
            TacticalRollRole::SecondWind => "Second Wind healing",
            TacticalRollRole::DeathSave => "Death saving throw",
            TacticalRollRole::EffectSave
            | TacticalRollRole::SpellSave
            | TacticalRollRole::AreaSave => "Saving throw",
            TacticalRollRole::AreaDamage => "Shared area damage",
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

pub(crate) fn sheet_details(rules: &RulesState, entity: &MechanicalEntity) -> TableSheetDetails {
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
    if record.kind != "table.conversation" {
        return Err("Unsupported table observation.".into());
    }
    let body: TableObservationBody = match record.payload_schema_version {
        1 => serde_json::from_str(&record.payload_json).map_err(|error| error.to_string())?,
        2 => {
            serde_json::from_str::<crate::table_transport::TransportedTableObservation>(
                &record.payload_json,
            )
            .map_err(|error| error.to_string())?
            .body
        }
        _ => return Err("Unsupported table observation.".into()),
    };
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
        self.execute_legacy_table(meta, action, true).await
    }

    /// Questions and table chatter have their own append-only observation history, not game events.
    pub async fn observe_table_text(
        &self,
        meta: CommandMeta,
        id: ObservationId,
        text: &str,
    ) -> Result<TableObservationBody, RunnableCampaignError> {
        self.observe_legacy_table(meta, id, text, true).await
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
        let pack = load_rules_pack(runnable.content())?;
        let events = load_journal_events(&self.pool, campaign_id, 0)
            .await
            .map_err(recovery)?;
        let mut observations = Vec::new();
        let mut after = 0;
        loop {
            let page = load_campaign_observations(&self.pool, campaign_id, after, 500)
                .await
                .map_err(recovery)?;
            let done = page.len() < 500;
            if let Some(last) = page.last() {
                after = last.ordinal;
            }
            observations.extend(page);
            if done {
                break;
            }
        }
        let events = events
            .iter()
            .map(crate::table_projection::ProjectionEvent::from)
            .collect::<Vec<_>>();
        crate::table_projection::project_table(
            runnable.state(),
            viewer,
            &pack,
            &events,
            &observations,
            None,
        )
    }
}

pub(crate) fn propose_observation(
    state: &CampaignState,
    pack: &dmd_rules::RulesPack,
    meta: &CommandMeta,
    id: ObservationId,
    text: &str,
) -> Result<(TableObservationBody, NewSessionObservation), RunnableCampaignError> {
    bounded_text(text, 8000).map_err(invalid)?;

    let (player, _, actor, _) = player_channel(state, meta).map_err(invalid)?;
    if state.applied_event_sequence != meta.expected_event_sequence {
        return Err(invalid("The table changed. Refresh before asking again."));
    }
    let table = table(state).map_err(invalid)?;

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
                dmd_rules::query(state, meta.issuer, &RulesQuery::Character { actor }, pack)?;
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
                "No situation has been established yet. Ask the host to describe the scene.".into()
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

    Ok((body, record))
}
