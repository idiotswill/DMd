//! SRD Falling source primitives. These derive requests and damage, not permission to
//! mutate a campaign. The shared tactical resolver owns timing, Reaction expenditure,
//! retained work, authoritative raw-roll history and actual landing/vitality commits.
use crate::{ResolveRoll, RulesError};
use dmd_domain::*;

fn invalid(message: impl Into<String>) -> RulesError {
    RulesError::Invalid(message.into())
}

pub fn fall_distance(path: &SpatialFall) -> Result<u32, RulesError> {
    path.from.validate().map_err(invalid)?;
    path.to.validate().map_err(invalid)?;
    if path.from.x != path.to.x || path.from.y != path.to.y || path.to.z >= path.from.z {
        return Err(invalid("fall must descend vertically"));
    }
    u32::try_from(i64::from(path.from.z) - i64::from(path.to.z))
        .map_err(|_| invalid("fall distance overflow"))
}

/// Full tens of feet, capped at 20d6. A shorter fall creates no fabricated zero-die
/// request. Half-foot geometry units must not accidentally become feet of damage.
pub fn fall_damage_request(
    path: &SpatialFall,
    actor: EntityId,
    id: RollRequestId,
    visibility: RollVisibility,
) -> Result<Option<RollRequest>, RulesError> {
    if id.0.is_nil() || actor.0.is_nil() {
        return Err(invalid("fall request/actor identity is empty"));
    }
    let count = (fall_distance(path)? / (10 * SPATIAL_UNITS_PER_FOOT)).min(20) as u16;
    Ok((count > 0).then(|| RollRequest {
        id,
        roller: Some(actor),
        dice: vec![DieSpec { count, sides: 6 }],
        modifier: 0,
        mode: RollMode::Normal,
        visibility,
        reason: "Falling damage".into(),
    }))
}

pub fn liquid_landing_kind(choice: LiquidLandingChoice) -> TestKind {
    let (ability, skill) = match choice {
        LiquidLandingChoice::Athletics => (Ability::Strength, Skill::Athletics),
        LiquidLandingChoice::Acrobatics => (Ability::Dexterity, Skill::Acrobatics),
    };
    TestKind::Check {
        ability,
        skill: Some(skill),
    }
}

pub fn can_attempt_liquid_landing(
    state: &CampaignState,
    actor: EntityId,
) -> Result<bool, RulesError> {
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    Ok(crate::tactical_conditions::can_act(rules, actor)?
        && rules
            .timing
            .as_ref()
            .is_some_and(|timing| !timing.reactions_spent.contains(&actor)))
}

/// Request derivation deliberately does not re-spend/re-test an unused Reaction:
/// the retained accepted choice already paid it before this raw request is pending.
pub fn liquid_landing_request(
    state: &CampaignState,
    actor: EntityId,
    choice: LiquidLandingChoice,
    id: RollRequestId,
    visibility: RollVisibility,
) -> Result<RollRequest, RulesError> {
    if id.0.is_nil() {
        return Err(invalid("landing request identity is empty"));
    }
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let entity = rules
        .entities
        .get(&actor)
        .ok_or_else(|| invalid("landing actor absent"))?;
    if !crate::tactical_conditions::can_act(rules, actor)? {
        return Err(RulesError::Prerequisite(
            "actor cannot use a landing Reaction".into(),
        ));
    }
    let kind = liquid_landing_kind(choice);
    let modifier = match rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(actor))
    {
        Some(profile) => crate::tactical_creatures::creature_test_modifier(profile, entity, &kind)
            .map_err(|e| invalid(e.to_string()))?,
        None => crate::test_modifier(entity, &kind),
    };
    let mut fear_visible = false;
    if let Some(encounter) = &state.encounter {
        for effect in crate::tactical_effect_adapter::condition_effects(rules) {
            if effect.target == actor
                && effect.condition == Some(Condition::Frightened)
                && encounter.participant(effect.source).is_some()
            {
                fear_visible |= crate::spatial::perceive(encounter, state, actor, effect.source)
                    .map_err(|e| invalid(e.to_string()))?
                    .sees;
            }
        }
    }
    let crate::tactical_conditions::TestDisposition::Roll(mode) =
        crate::tactical_conditions::check_conditions(
            rules,
            actor,
            false,
            false,
            fear_visible,
            Circumstances::default(),
        )?
    else {
        return Err(RulesError::Prerequisite(
            "actor cannot perform a landing check".into(),
        ));
    };
    Ok(RollRequest {
        id,
        roller: Some(actor),
        dice: vec![DieSpec {
            count: 1,
            sides: 20,
        }],
        modifier,
        mode,
        visibility,
        reason: match choice {
            LiquidLandingChoice::Athletics => "Strength (Athletics) liquid landing check",
            LiquidLandingChoice::Acrobatics => "Dexterity (Acrobatics) liquid landing check",
        }
        .into(),
    })
}

/// This proof is produced from an actual source request/result. It is neither
/// deserializable nor a client-supplied success flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiquidLandingOutcome {
    actor: EntityId,
    path: SpatialFall,
    successful: bool,
}
impl LiquidLandingOutcome {
    pub fn successful(&self) -> bool {
        self.successful
    }
}

pub fn resolve_liquid_landing(
    state: &CampaignState,
    actor: EntityId,
    path: &SpatialFall,
    choice: LiquidLandingChoice,
    request_id: RollRequestId,
    visibility: RollVisibility,
    result: &RollResult,
) -> Result<LiquidLandingOutcome, RulesError> {
    fall_distance(path)?;
    if !matches!(path.surface, FallSurface::Liquid { .. }) {
        return Err(invalid("landing Reaction requires liquid"));
    }
    let request = liquid_landing_request(state, actor, choice, request_id, visibility)?;
    Ok(LiquidLandingOutcome {
        actor,
        path: path.clone(),
        successful: request.resolve(result)?.total >= 15,
    })
}

pub fn fall_damage_packet(
    path: &SpatialFall,
    actor: EntityId,
    request_id: RollRequestId,
    result: Option<&RollResult>,
    landing: Option<&LiquidLandingOutcome>,
) -> Result<DamagePacket, RulesError> {
    if landing.is_some_and(|landing| landing.path != *path || landing.actor != actor) {
        return Err(invalid("landing check belongs to another fall"));
    }
    let request = fall_damage_request(path, actor, request_id, RollVisibility::Public)?;
    let total = match (request, result) {
        (Some(request), Some(result)) => u32::try_from(request.resolve(result)?.total)
            .map_err(|_| invalid("fall damage total is negative"))?,
        (None, None) => 0,
        _ => return Err(invalid("fall damage requires exactly its derived raw dice")),
    };
    Ok(DamagePacket {
        cause: DamageCause::Other,
        components: vec![DamageComponent {
            damage_type: DamageType::Bludgeoning,
            amounts: vec![total],
            adjustments: if landing.is_some_and(LiquidLandingOutcome::successful) {
                vec![DamageAdjustment::Multiply {
                    numerator: 1,
                    denominator: 2,
                }]
            } else {
                vec![]
            },
        }],
    })
}
