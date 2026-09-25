use super::*;

pub fn creature_legendary_resistance_available(
    state: &CampaignState,
    current: &TacticalCreatures,
    actor: EntityId,
) -> Result<bool, CreatureError> {
    validate_tactical_creatures(state, current)?;
    let profile = current
        .profile(actor)
        .ok_or_else(|| invalid("unknown creature"))?;
    let runtime = current
        .runtime(actor)
        .ok_or_else(|| invalid("unknown creature runtime"))?;
    // Legendary Resistance is not an action; Incapacitated does not prohibit it.
    Ok(!rules(state)?.entities[&actor].death.dead
        && runtime.legendary_resistance_spent
            < resistance_max(source_for_profile(profile)?, runtime.in_lair))
}

/// Sealed failed-save evidence. Only rules' live continuation validator constructs
/// this after checking the pending occurrence against recorded raw/voluntary input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureFailedSaveProof {
    pub(crate) campaign_id: CampaignId,
    pub(crate) expected_event_sequence: u64,
    pub(crate) actor: EntityId,
    pub(crate) request_id: RollRequestId,
}
impl CreatureFailedSaveProof {
    // The turn adapter lands in a separate reviewed slice; external callers can
    // neither construct nor deserialize this proof.
    #[allow(
        dead_code,
        reason = "called by the source-save continuation adapter at integration"
    )]
    pub(crate) fn from_validated_resolution(
        campaign_id: CampaignId,
        expected_event_sequence: u64,
        actor: EntityId,
        request_id: RollRequestId,
    ) -> Self {
        Self {
            campaign_id,
            expected_event_sequence,
            actor,
            request_id,
        }
    }
}
/// Returns only the updated source budget. The caller must atomically turn the
/// exact still-paused failed save into success and clear its decision slot.
pub fn use_creature_legendary_resistance(
    state: &CampaignState,
    current: &TacticalCreatures,
    meta: &CommandMeta,
    proof: &CreatureFailedSaveProof,
) -> Result<TacticalCreatures, CreatureError> {
    validate_tactical_creatures(state, current)?;
    if meta.expected_event_sequence != state.applied_event_sequence
        || proof.expected_event_sequence != state.applied_event_sequence
    {
        return Err(CreatureError::Stale);
    }
    if proof.campaign_id != state.campaign_id() || proof.request_id.0.is_nil() {
        return Err(invalid("failed save proof is foreign or malformed"));
    }
    let runtime = current
        .runtime(proof.actor)
        .ok_or_else(|| invalid("unknown creature"))?;
    authorize(state, runtime, meta)?;
    if runtime
        .legendary_resistance_rolls
        .contains(&proof.request_id)
        || !creature_legendary_resistance_available(state, current, proof.actor)?
    {
        return Err(CreatureError::Unavailable(
            "Legendary Resistance exhausted or occurrence already changed".into(),
        ));
    }
    let mut next = current.clone();
    let runtime = next
        .runtime
        .iter_mut()
        .find(|r| r.actor == proof.actor)
        .expect("validated actor");
    runtime.legendary_resistance_spent += 1;
    runtime.legendary_resistance_rolls.push(proof.request_id);
    runtime.last_operation = meta.clone();
    validate_tactical_creatures(state, &next)?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (CampaignState, TacticalCreatures, CommandMeta, EntityId) {
        let actor = EntityId::new();
        let mut state = CampaignState::empty(
            Campaign {
                id: CampaignId::new(),
                display_name: "Resistance".into(),
                status: CampaignStatus::Active,
                world_seed: 1,
                ruleset: VersionedRef {
                    id: "srd-5.2".into(),
                    version: "5.2.1".into(),
                },
                content_packs: vec![],
            },
            WorldClock {
                now: WorldInstant(0),
                calendar_id: "seconds".into(),
            },
        );
        state.applied_event_sequence = 1;
        state.entities.insert(
            actor,
            WorldEntity {
                id: actor,
                campaign_id: state.campaign_id(),
                display_name: "Dragon".into(),
                kind: EntityKind::Creature,
                existence: EntityExistence::Present,
                location_id: None,
            },
        );
        let meta = CommandMeta {
            id: CommandId::new(),
            campaign_id: state.campaign_id(),
            session_id: None,
            issuer: CommandIssuer::Admin,
            actor: None,
            expected_event_sequence: 1,
        };
        let built = build_creature(
            &state,
            &meta,
            actor,
            &CreatureBuildChoice {
                definition_id: "adult-red-dragon".into(),
                size: CreatureSize::Huge,
                additional_languages: vec![],
                hit_points: CreatureHitPointChoice::Average,
                controller: CreatureController::Autonomous,
                in_lair: false,
            },
        )
        .unwrap();
        let mut rules:RulesState=serde_json::from_value(serde_json::json!({"pack_id":"srd-5.2","pack_version":"5.2.1","entities":{},"house_rules":{"ability_test_natural_extremes":false},"effects":[],"pending":null,"rolls":[],"cancelled_roll_ids":[],"rulings":[],"timing":null,"rests":[],"completed_short_rests":[],"permission":null})).unwrap();
        rules.entities.insert(actor, built.mechanics);
        state.rules = Some(rules);
        (
            state,
            TacticalCreatures {
                schema_version: 1,
                profiles: vec![built.profile],
                runtime: vec![built.runtime],
            },
            meta,
            actor,
        )
    }
    #[test]
    fn resistance_consumes_each_verified_occurrence_once_and_tracks_spent_across_lair_changes() {
        let (state, mut current, meta, actor) = fixture();
        for _ in 0..3 {
            let proof = CreatureFailedSaveProof::from_validated_resolution(
                state.campaign_id(),
                1,
                actor,
                RollRequestId::new(),
            );
            current = use_creature_legendary_resistance(&state, &current, &meta, &proof).unwrap();
            assert!(use_creature_legendary_resistance(&state, &current, &meta, &proof).is_err());
        }
        assert!(!creature_legendary_resistance_available(&state, &current, actor).unwrap());
        current = apply_creature_schedule(
            &state,
            &current,
            &meta,
            &CreatureScheduleOperation::SetContext {
                actor,
                controller: CreatureController::Autonomous,
                in_lair: true,
            },
        )
        .unwrap()
        .next;
        assert!(creature_legendary_resistance_available(&state, &current, actor).unwrap());
        let proof = CreatureFailedSaveProof::from_validated_resolution(
            state.campaign_id(),
            1,
            actor,
            RollRequestId::new(),
        );
        current = use_creature_legendary_resistance(&state, &current, &meta, &proof).unwrap();
        assert_eq!(current.runtime[0].legendary_resistance_spent, 4);
        let before = current.clone();
        for invalid_proof in [
            CreatureFailedSaveProof::from_validated_resolution(
                CampaignId::new(),
                1,
                actor,
                RollRequestId::new(),
            ),
            CreatureFailedSaveProof::from_validated_resolution(
                state.campaign_id(),
                0,
                actor,
                RollRequestId::new(),
            ),
        ] {
            assert!(
                use_creature_legendary_resistance(&state, &current, &meta, &invalid_proof).is_err()
            );
            assert_eq!(current, before);
        }
        current = apply_creature_schedule(
            &state,
            &current,
            &meta,
            &CreatureScheduleOperation::SetContext {
                actor,
                controller: CreatureController::Autonomous,
                in_lair: false,
            },
        )
        .unwrap()
        .next;
        assert_eq!(current.runtime[0].legendary_resistance_spent, 4);
        assert!(!creature_legendary_resistance_available(&state, &current, actor).unwrap());
    }
}
