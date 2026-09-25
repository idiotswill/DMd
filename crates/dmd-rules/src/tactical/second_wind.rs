//! Fighter Second Wind uses the ordinary tactical continuation and vitality path.
use super::turns::*;
use super::*;

fn source(
    state: &CampaignState,
    actor: EntityId,
    pack: &RulesPack,
) -> Result<(u8, u8), RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("Fighter absent"))?;
    let profile = state
        .table
        .as_ref()
        .and_then(|table| {
            table
                .character_profiles
                .values()
                .find(|p| p.entity_id == actor)
        })
        .ok_or_else(|| prerequisite("Second Wind requires a created Fighter profile"))?;
    if !flow(state)?
        .combatants
        .iter()
        .any(|c| c.actor == actor && c.source == TacticalSource::Character)
    {
        return Err(prerequisite(
            "Second Wind requires the character's encounter source",
        ));
    }
    crate::validate_character_intrinsics(profile, entity, pack)?;
    let features = entity
        .character_features
        .as_ref()
        .ok_or_else(|| prerequisite("Second Wind is not granted"))?;
    if profile.class_id != "fighter"
        || profile.level != 1
        || features.fighter_level != profile.level
        || features.second_wind_remaining > 2
    {
        return Err(invalid(
            "Second Wind source or use count differs from the supported Fighter",
        ));
    }
    Ok((features.fighter_level, features.second_wind_remaining))
}

pub(super) fn begin(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    if flow(state)?.phase != TacticalPhase::Active || flow(state)?.resolution.is_some() {
        return Err(RulesError::Pending);
    }
    let actor = active(state)?;
    authorize(state, meta, actor)?;
    let (_, uses_before) = source(state, actor, pack)?;
    if uses_before == 0 {
        return Err(prerequisite("Second Wind is exhausted"));
    }
    let rules = state.rules.as_mut().ok_or(RulesError::Uninitialized)?;
    crate::tactical_budget::spend_cost(
        rules,
        actor,
        crate::tactical_budget::TacticalCost::BonusAction,
    )?;
    rules
        .entities
        .get_mut(&actor)
        .unwrap()
        .character_features
        .as_mut()
        .unwrap()
        .second_wind_remaining -= 1;
    let turn_number = rules.timing.as_ref().unwrap().turn_number;
    let flow = flow_mut(state)?;
    flow.budget.movement_progress = None;
    flow.budget.movement_origin = None;
    flow.resolution = Some(Box::new(TacticalResolution {
        origin: meta.clone(),
        turn_actor: actor,
        turn_number,
        boundary: TurnBoundary::Start,
        frames: vec![],
        pending: None,
        failed_save: None,
        legendary_window: None,
        attack: None,
        movement: None,
        casts: vec![],
        falls: vec![],
        areas: vec![],
        next_occurrence: 0,
    }));
    push_frame(
        state,
        vec![TacticalWorkKind::SecondWind { actor, uses_before }],
    )?;
    pump(state, meta)
}

pub(super) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let TacticalWorkKind::SecondWind { actor, uses_before } = work.kind else {
        return Err(invalid("not Second Wind work"));
    };
    let r = resolution(state)?;
    let pack = RulesPack::from_json(include_str!("../../../../content/srd-5.2.1/kernel.json"))?;
    let (_, remaining) = source(state, actor, &pack)?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let timing = rules
        .timing
        .as_ref()
        .ok_or_else(|| invalid("Second Wind has no turn"))?;
    if actor != r.turn_actor
        || actor != active(state)?
        || work.occurrence != 0
        || r.next_occurrence != 1
        || r.boundary != TurnBoundary::Start
        || !timing.bonus_action_spent
        || !(1..=2).contains(&uses_before)
        || remaining.checked_add(1) != Some(uses_before)
        || r.attack.is_some()
        || r.movement.is_some()
        || !r.casts.is_empty()
        || !r.falls.is_empty()
        || !r.areas.is_empty()
        || r.frames.iter().flatten().any(|w| w != work)
        || r.failed_save.is_some()
        || r.legendary_window.is_some()
        || !crate::tactical_conditions::can_act(rules, actor)?
    {
        return Err(invalid(
            "Second Wind work differs from its paid source action",
        ));
    }
    authorize(state, &r.origin, actor)?;
    Ok(actor)
}

pub(super) fn request(
    state: &CampaignState,
    work: &TacticalWorkItem,
    key: TacticalRollKey,
) -> Result<Option<RollRequest>, RulesError> {
    let actor = validate_work(state, work)?;
    let level = state.rules.as_ref().unwrap().entities[&actor]
        .character_features
        .as_ref()
        .unwrap()
        .fighter_level;
    Ok(Some(RollRequest {
        id: key.request_id(),
        roller: Some(actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 10,
        }],
        modifier: i32::from(level),
        mode: RollMode::Normal,
        visibility: if controller(state, actor).is_some() {
            RollVisibility::Public
        } else {
            RollVisibility::Secret
        },
        reason: "Second Wind healing".into(),
    }))
}
