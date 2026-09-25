//! Source-bound encounter admission. The returned proof has private fields and cannot
//! be supplied through Serde/IPC. Admission is read-only and precedes all cast costs.
use super::*;
use crate::spatial::{cover_from, participant_distance, perceive};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableSpellKind {
    Healing,
    SavingThrowCondition,
    AutomaticDamage,
    AttackDamage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSpellTarget {
    actor: EntityId,
    /// Internal source truth. Never expose this bit as the explanation for an
    /// apparent successful saving throw/no effect on an invalid type (SRD106).
    valid_type: bool,
}
impl BoundSpellTarget {
    pub fn actor(&self) -> EntityId {
        self.actor
    }
    pub fn valid_type(&self) -> bool {
        self.valid_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSpell {
    plan: Box<SpellCastPlan>,
    kind: ExecutableSpellKind,
    targets: Vec<BoundSpellTarget>,
    consumed_material: Option<ItemId>,
    completed: Vec<SpellProgramOccurrence>,
}
impl BoundSpell {
    pub(super) fn plan(&self) -> &SpellCastPlan {
        &self.plan
    }
    pub(super) fn from_retained(record: &TacticalCasting, kind: ExecutableSpellKind) -> Self {
        Self {
            plan: Box::new(record.cast.plan.clone()),
            kind,
            targets: record
                .targets
                .iter()
                .map(|target| BoundSpellTarget {
                    actor: target.actor,
                    valid_type: target.source_type_matches,
                })
                .collect(),
            consumed_material: record.consumed_material,
            completed: record.completed.clone(),
        }
    }
    pub fn kind(&self) -> ExecutableSpellKind {
        self.kind
    }
    /// The chosen order is retained. Repeated ray/dart targets are separate source
    /// occurrences, while a multiple-creature selector must contain distinct IDs.
    pub fn targets(&self) -> &[BoundSpellTarget] {
        &self.targets
    }
    pub fn consumed_material(&self) -> Option<ItemId> {
        self.consumed_material
    }
}

fn spatial(error: crate::spatial::SpatialError) -> RulesError {
    invalid(&error.to_string())
}

/// A program must have a complete executable path before spending anything. This
/// closed first slice does not silently discard lights, object ignition, movement,
/// behavior commands, protection or reaction clauses. Their work remains Gate 4.
pub fn executable_spell_kind(plan: &SpellCastPlan) -> Result<ExecutableSpellKind, RulesError> {
    validate_spell_plan(plan)?;
    match plan.program.nodes.as_slice() {
        [SpellProgramNode::Heal { .. }] => Ok(ExecutableSpellKind::Healing),
        [SpellProgramNode::SaveCondition { .. }] => Ok(ExecutableSpellKind::SavingThrowCondition),
        [SpellProgramNode::AutomaticDamage { .. }] => Ok(ExecutableSpellKind::AutomaticDamage),
        // Ignition applies to the object alternative only. Binding below explicitly
        // limits this path to creatures before any expenditure (e.g. Fire Bolt).
        [SpellProgramNode::AttackDamage { .. }] => Ok(ExecutableSpellKind::AttackDamage),
        _ => Err(unavailable(
            "this source program still requires its complete tactical execution path",
        )),
    }
}

fn material_fact(
    state: &CampaignState,
    choice: SpellMaterialChoice,
) -> Result<Option<SpellMaterialFact>, RulesError> {
    match choice {
        SpellMaterialChoice::None => Ok(None),
        SpellMaterialChoice::Material { item } => {
            let physical = state
                .items
                .get(&item)
                .ok_or_else(|| unavailable("specified material is unavailable"))?;
            let source = crate::tactical_creature_equipment::source_spell_material(physical)?
                .ok_or_else(|| {
                    unavailable("specified material has no canonical source identity")
                })?;
            Ok(Some(SpellMaterialFact::Specified {
                item,
                spell_id: source.spell_id,
                value_cp: source.value_cp,
            }))
        }
        // A display name, a carried holy symbol, or an old free_hand boolean does
        // not establish the class-specific focus grant or a physical pouch.
        SpellMaterialChoice::Focus { .. } | SpellMaterialChoice::ComponentPouch { .. } => Err(
            unavailable("this component substitute requires a source-backed physical grant"),
        ),
    }
}

fn creature_type(state: &CampaignState, actor: EntityId) -> Result<CreatureType, RulesError> {
    if let Some(profile) = state
        .rules
        .as_ref()
        .and_then(|r| r.tactical_creatures.as_ref())
        .and_then(|c| c.profile(actor))
    {
        return Ok(crate::tactical_creatures::source_for_profile(profile)
            .map_err(|e| invalid(&e.to_string()))?
            .statistics
            .creature_type);
    }
    if let Some(profile) = state.table.as_ref().and_then(|table| {
        table
            .character_profiles
            .values()
            .find(|p| p.entity_id == actor)
    }) {
        // This is the supported source species identity, never an entity name.
        if profile.species_id == "human" {
            return Ok(CreatureType::Humanoid);
        }
    }
    Err(unavailable(
        "target creature type needs its source-backed profile",
    ))
}

fn validate_armor_training(
    state: &CampaignState,
    actor: EntityId,
    loadout: &ActorEquipmentLoadout,
) -> Result<(), RulesError> {
    for (item, category) in [(loadout.worn_armor, "light"), (loadout.shield, "shield")] {
        let Some(item) = item else { continue };
        let physical = state
            .items
            .get(&item)
            .ok_or_else(|| invalid("worn item is absent"))?;
        // The current physical registry contains these two armor definitions only;
        // future additions need their source armor category, not an inferred label.
        if !matches!(
            (physical.definition_id.as_str(), category),
            ("leather-armor", "light") | ("shield", "shield")
        ) {
            return Err(unavailable(
                "worn armor requires a supported source training category",
            ));
        }
        let trained_pc = state
            .table
            .as_ref()
            .and_then(|t| t.character_profiles.values().find(|p| p.entity_id == actor))
            .is_some_and(|p| p.armor_training.iter().any(|g| g == category));
        let trained_creature = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|c| c.profile(actor))
            .map(|p| {
                crate::tactical_creatures::source_for_profile(p)
                    .map(|source| source.statistics.gear.contains(&physical.definition_id))
            })
            .transpose()
            .map_err(|e| invalid(&e.to_string()))?
            .unwrap_or(false);
        if !trained_pc && !trained_creature {
            return Err(unavailable(
                "casting requires training with the worn armor and shield",
            ));
        }
    }
    Ok(())
}

/// Bind legal entity choices against current source/geometry without making costs,
/// rolling dice, choosing a target for a player, or changing concentration. The
/// scheduler authenticates the plan and commits this proof with begin_cast atomically.
pub fn bind_spell(
    state: &CampaignState,
    plan: &SpellCastPlan,
    choice: &SpellTargetChoice,
) -> Result<BoundSpell, RulesError> {
    let kind = executable_spell_kind(plan)?;
    if plan.origin.campaign_id != state.campaign_id()
        || plan.origin.expected_event_sequence != state.applied_event_sequence
    {
        return Err(RulesError::Stale);
    }
    if matches!(plan.choice.grant, SpellGrantChoice::Prepared)
        && plan_spell_cast_at(state, &plan.origin, &plan.choice, plan.occurrence)? != *plan
    {
        return Err(invalid(
            "prepared spell plan differs from current authoritative source",
        ));
    }
    let encounter = state
        .encounter
        .as_ref()
        .ok_or_else(|| unavailable("casting requires an encounter"))?;
    crate::spatial::validate_encounter(encounter, state).map_err(spatial)?;
    let caster = encounter
        .participant(plan.choice.actor)
        .ok_or_else(|| unavailable("caster is outside the encounter"))?;
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    // Real current hands are mandatory in this path. Historical imported sheet
    // booleans remain valid historical records but cannot invent physical items.
    let loadout = rules
        .tactical_inventory
        .as_ref()
        .and_then(|i| i.loadout(plan.choice.actor))
        .ok_or_else(|| unavailable("materialized current caster equipment is required"))?;
    validate_armor_training(state, plan.choice.actor, loadout)?;
    let mut verbal_possible = crate::tactical_conditions::can_speak(rules, plan.choice.actor)?;
    if let Some(profile) = rules
        .tactical_creatures
        .as_ref()
        .and_then(|c| c.profile(plan.choice.actor))
    {
        verbal_possible &= crate::tactical_creatures::source_for_profile(profile)
            .map_err(|e| invalid(&e.to_string()))?
            .statistics
            .can_speak;
    }
    let material = material_fact(state, plan.choice.material)?;
    let consumed_material =
        validate_spell_components(state, plan, verbal_possible, material.as_ref())?;
    let SpellTargetChoice::Entities(actors) = choice else {
        return Err(unavailable(
            "this spell path requires explicit creature targets",
        ));
    };
    let (minimum, maximum, repeated, requires_sight, required_type) = match &plan.program.targets {
        SpellTargetRule::Creatures {
            maximum,
            requires_sight,
            creature_type,
        } => (
            1,
            usize::from(*maximum),
            false,
            *requires_sight,
            creature_type.as_deref(),
        ),
        SpellTargetRule::CreatureOrObject => (1, 1, false, false, None),
        SpellTargetRule::Darts {
            count,
            requires_sight,
        } => (
            usize::from(*count),
            usize::from(*count),
            true,
            *requires_sight,
            None,
        ),
        SpellTargetRule::Rays { count } => {
            (usize::from(*count), usize::from(*count), true, false, None)
        }
        _ => {
            return Err(unavailable(
                "this source target shape still requires its complete spatial execution path",
            ));
        }
    };
    if actors.len() < minimum
        || actors.len() > maximum
        || (!repeated && actors.iter().collect::<HashSet<_>>().len() != actors.len())
    {
        return Err(invalid(
            "chosen target occurrences disagree with source selector",
        ));
    }
    let mut targets = Vec::with_capacity(actors.len());
    for &actor in actors {
        // Use a single non-disclosing error for absent/unlocated choices. A raw ID
        // learned outside the actor's view does not permit querying its true position.
        let target = encounter
            .participant(actor)
            .ok_or_else(|| unavailable("target cannot be selected from current perception"))?;
        let perception = perceive(encounter, state, plan.choice.actor, actor).map_err(spatial)?;
        if !perception.precisely_located || (requires_sight && !perception.sees) {
            return Err(unavailable(
                "target cannot be selected from current perception",
            ));
        }
        let world = state
            .entities
            .get(&actor)
            .ok_or_else(|| invalid("participant lacks a world identity"))?;
        if !matches!(
            world.kind,
            EntityKind::Character | EntityKind::Npc | EntityKind::Creature
        ) || !rules.entities.contains_key(&actor)
        {
            return Err(unavailable(
                "object spell targets require their complete physical execution path",
            ));
        }
        let distance = participant_distance(caster, target).map_err(spatial)?;
        let range = match plan.program.range {
            SpellRangeLimit::Caster => 0,
            // Participant.reach can be a selected monster weapon's reach. It is
            // not a source grant to extend touching or an ordinary Unarmed Strike.
            // The supported source capability is the usual 5-foot touch; a feature
            // that specifically extends it must add its own source-derived grant.
            SpellRangeLimit::Touch => 5 * SPATIAL_UNITS_PER_FOOT,
            SpellRangeLimit::Feet(feet) => u32::from(feet) * SPATIAL_UNITS_PER_FOOT,
        };
        if distance > range {
            return Err(unavailable("selected target is outside the spell's range"));
        }
        if actor != plan.choice.actor {
            let cover = cover_from(
                encounter,
                caster.center().map_err(RulesError::Invalid)?,
                target.volume().map_err(RulesError::Invalid)?,
                &[plan.choice.actor, actor],
            )
            .map_err(spatial)?;
            if cover.requires_adjudication {
                return Err(unavailable(
                    "target path requires a recorded geometry ruling",
                ));
            }
            if cover.degree == CoverDegree::Total {
                return Err(unavailable("total cover prevents direct spell targeting"));
            }
        }
        if kind != ExecutableSpellKind::Healing
            && !crate::tactical_conditions::may_harm(rules, plan.choice.actor, actor)
        {
            return Err(unavailable("the caster cannot harm this target"));
        }
        let valid_type = required_type
            .map(|required| {
                creature_type(state, actor).map(|actual| format!("{actual:?}") == required)
            })
            .transpose()?
            .unwrap_or(true);
        targets.push(BoundSpellTarget { actor, valid_type });
    }
    Ok(BoundSpell {
        plan: Box::new(plan.clone()),
        kind,
        targets,
        consumed_material,
        completed: vec![],
    })
}

/// One canonical source attack occurrence for the shared attack adapter. It carries
/// no fake physical weapon identity and cannot be constructed by deserialization.
/// Live cover/AC/conditions and Exhaustion belong to the adapter at this occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellAttackOccurrence {
    origin: CommandMeta,
    cast_occurrence: u16,
    node_ordinal: u8,
    target_ordinal: u16,
    actor: EntityId,
    target: EntityId,
    source: SpellSourcePin,
    intrinsic_attack_bonus: i16,
    damage: SpellDamageDice,
}
impl SpellAttackOccurrence {
    pub fn origin(&self) -> &CommandMeta {
        &self.origin
    }
    pub fn cast_occurrence(&self) -> u16 {
        self.cast_occurrence
    }
    pub fn node_ordinal(&self) -> u8 {
        self.node_ordinal
    }
    pub fn target_ordinal(&self) -> u16 {
        self.target_ordinal
    }
    pub fn actor(&self) -> EntityId {
        self.actor
    }
    pub fn target(&self) -> EntityId {
        self.target
    }
    pub fn source(&self) -> &SpellSourcePin {
        &self.source
    }
    pub fn intrinsic_attack_bonus(&self) -> i16 {
        self.intrinsic_attack_bonus
    }
    pub fn damage(&self) -> &SpellDamageDice {
        &self.damage
    }
}

pub fn spell_attack_occurrence(
    cast: &SpellCast,
    bound: &BoundSpell,
    node_ordinal: u8,
    target_ordinal: u16,
) -> Result<SpellAttackOccurrence, RulesError> {
    validate_spell_cast(cast)?;
    if !matches!(
        cast.phase,
        SpellCastPhase::Committed | SpellCastPhase::Released
    ) || node_ordinal != 0
        || *bound.plan != cast.plan
        || bound.kind != ExecutableSpellKind::AttackDamage
        || bound.completed.contains(&SpellProgramOccurrence {
            node: node_ordinal,
            target: target_ordinal,
        })
    {
        return Err(invalid(
            "spell attack is not a committed occurrence of its bound source cast",
        ));
    }
    let target = bound
        .targets
        .get(usize::from(target_ordinal))
        .ok_or_else(|| invalid("spell attack occurrence is outside its selected targets"))?;
    let [
        SpellProgramNode::AttackDamage {
            damage,
            share: SpellDamageShare::PerAttack,
            ..
        },
    ] = cast.plan.program.nodes.as_slice()
    else {
        return Err(invalid(
            "spell attack occurrence has no complete source attack program",
        ));
    };
    Ok(SpellAttackOccurrence {
        origin: cast.plan.origin.clone(),
        cast_occurrence: cast.plan.occurrence,
        node_ordinal,
        target_ordinal,
        actor: cast.plan.choice.actor,
        target: target.actor,
        source: cast.plan.program.source.clone(),
        intrinsic_attack_bonus: cast
            .plan
            .program
            .attack_bonus
            .ok_or_else(|| invalid("spell attack modifier is absent"))?,
        damage: damage.clone(),
    })
}
