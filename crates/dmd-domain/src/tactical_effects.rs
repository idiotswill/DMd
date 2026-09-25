//! Durable lifecycle records. Source authorization and geometry belong to the encounter
//! resolver; these records are never a player-supplied state patch or executable script.
use crate::{
    Ability, AgentRef, CampaignState, CommandId, CommandIssuer, CommandMeta, Condition, DamageType,
    DieSpec, EffectId, EntityId, TurnBoundary, WorldInstant,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_TACTICAL_EFFECTS: usize = 512;
pub const MAX_PENDING_EFFECT_TRIGGERS: usize = MAX_TACTICAL_EFFECTS * 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectSource {
    /// Immutable pack definition ID; validated against content by the encounter resolver.
    pub definition_id: String,
    pub actor: EntityId,
    pub command: CommandMeta,
    /// Source resolution order within the command, used for equally potent overlaps.
    pub ordinal: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalEffectExpiry {
    Never,
    AtTime(WorldInstant),
    /// Count matching boundaries, not global turn numbers. One means the next observed
    /// matching boundary; the caller establishes effects after any current boundary.
    AfterOwnerBoundaries {
        owner: EntityId,
        boundary: TurnBoundary,
        remaining: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcentrationStage {
    Casting,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConcentrationGroup {
    pub id: EffectId,
    pub source: EffectSource,
    pub expires: TacticalEffectExpiry,
    pub stage: ConcentrationStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TacticalEffectTarget {
    Creature(EntityId),
    /// Membership is derived by authoritative geometry. Enter/leave observations update
    /// it; turn triggers and overlap queries use the durable current membership.
    Zone {
        occupants: Vec<EntityId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectCondition {
    /// Stable view identity, distinct from the lifecycle effect/group IDs.
    pub id: EffectId,
    pub condition: Condition,
}

/// Source-derived defense clauses. Installation is internal to the authenticated
/// spell resolver; these are not an equipment edit or a player modifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectDefense {
    BaseArmorClass {
        base: u8,
        ability: Ability,
        ends_when_wearing_armor: bool,
    },
    ArmorClassBonus {
        bonus: u8,
    },
    PreventSpellDamage {
        spell_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectOverlap {
    /// Same-spell/effect identity, shared across casters. Not the caster's entity ID.
    pub key: String,
    /// Source-derived comparison, not a player modifier. Larger means more potent.
    pub potency: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSubject {
    Source,
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneContact {
    CreatureEnters,
    ZoneEntersCreature,
    CreatureLeaves,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectTriggerEvent {
    Turn {
        subject: EffectSubject,
        boundary: TurnBoundary,
    },
    Damage {
        subject: EffectSubject,
    },
    ZoneContact(ZoneContact),
    /// The actual donning operation, not simply having armor in one's custody.
    ArmorWorn {
        subject: EffectSubject,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectTriggerFrequency {
    EveryOccurrence,
    /// Rules with the same key share a limit (for example entry and end-turn damage).
    OncePerTargetPerTurn {
        key: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSaveEnd {
    None,
    TargetEffect,
    ConcentrationGroup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectTriggerPayload {
    /// Lifecycle cleanup when selected, without a roll or a damage callback.
    EndTargetEffect,
    EndConcentrationGroup,
    ExpireTargetEffect,
    ExpireConcentrationGroup,
    SavingThrow {
        ability: Ability,
        dc: u16,
        on_success: EffectSaveEnd,
        on_failure: EffectSaveEnd,
    },
    /// Scheduling only: the continuation rolls/applies damage through ordinary rules.
    Damage {
        dice: Vec<DieSpec>,
        modifier: i16,
        damage_type: DamageType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTriggerRule {
    pub event: EffectTriggerEvent,
    pub frequency: EffectTriggerFrequency,
    pub payload: EffectTriggerPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalEffect {
    pub id: EffectId,
    pub source: EffectSource,
    /// None only in an internal Install proposal. The reducer stamps installation;
    /// persisted effects require Some. A held spell's casting origin can be older.
    pub established_at: Option<EffectOperationStamp>,
    pub target: TacticalEffectTarget,
    pub concentration_group: Option<EffectId>,
    pub expires: TacticalEffectExpiry,
    pub overlap: Option<EffectOverlap>,
    pub conditions: Vec<EffectCondition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defenses: Vec<EffectDefense>,
    /// Stable source clause indexes. Simultaneous consequence order is chosen explicitly.
    pub triggers: Vec<EffectTriggerRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTurn {
    pub actor: EntityId,
    /// Global authoritative combat turn number; never the round number.
    pub number: u64,
    pub boundary: TurnBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectOperationStamp {
    pub command: CommandMeta,
    /// Strictly increasing calls within one command; caller supplies deterministic order.
    pub step: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectObservation {
    /// Observe End before advancing initiative, Start after advancing initiative.
    Turn(EffectTurn),
    Time,
    ArmorWorn {
        target: EntityId,
    },
    /// Actual damage after immunity/resistance, including damage absorbed by temp HP.
    Damage {
        source: Option<EntityId>,
        target: EntityId,
        amount: u32,
    },
    Zone {
        effect: EffectId,
        target: EntityId,
        contact: ZoneContact,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTicketId {
    pub command: CommandId,
    pub step: u16,
    pub ordinal: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledEffectTrigger {
    pub id: EffectTicketId,
    /// A group expiry references the group ID; all other tickets reference an effect.
    pub effect: EffectId,
    /// None is reserved for an expiry, Some indexes the immutable trigger clause.
    pub rule_index: Option<u16>,
    pub source: EffectSource,
    pub target: EntityId,
    pub payload: EffectTriggerPayload,
    pub cause: EffectObservation,
    pub origin: EffectOperationStamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTriggerUse {
    pub effect: EffectId,
    pub key: String,
    pub target: EntityId,
    pub turn_actor: EntityId,
    pub turn_number: u64,
    pub origin: EffectOperationStamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticalEffects {
    pub groups: Vec<ConcentrationGroup>,
    pub effects: Vec<TacticalEffect>,
    pub turn: Option<EffectTurn>,
    pub trigger_uses: Vec<EffectTriggerUse>,
    pub last_operation: Option<EffectOperationStamp>,
    /// Observing a boundary never chooses the order of simultaneous consequences.
    pub pending: Vec<ScheduledEffectTrigger>,
}

fn text_valid(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 160 && !value.chars().any(char::is_control)
}

impl TacticalEffects {
    pub fn group_for_owner(&self, owner: EntityId) -> Option<&ConcentrationGroup> {
        self.groups.iter().find(|group| group.source.actor == owner)
    }

    pub fn concentration_bindings(&self) -> Vec<(EntityId, EffectId)> {
        self.groups.iter().map(|g| (g.source.actor, g.id)).collect()
    }

    /// Structural/reference validation does not authenticate historical state. Application
    /// audit + semantic replay must still bind retained origins to accepted journal events.
    pub fn validate(&self, campaign: &CampaignState) -> Result<(), String> {
        if self.groups.len() > MAX_TACTICAL_EFFECTS
            || self.effects.len() > MAX_TACTICAL_EFFECTS
            || self.trigger_uses.len() > MAX_TACTICAL_EFFECTS * 128
            || self.pending.len() > MAX_PENDING_EFFECT_TRIGGERS
        {
            return Err("effect lifecycle capacity exceeded".into());
        }
        let entity = |id: EntityId| {
            campaign.entities.contains_key(&id)
                && campaign
                    .rules
                    .as_ref()
                    .is_some_and(|r| r.entities.contains_key(&id))
        };
        let origin = |meta: &CommandMeta| -> Result<(), String> {
            if meta.id.0.is_nil()
                || meta.campaign_id != campaign.campaign_id()
                || meta.expected_event_sequence > campaign.applied_event_sequence
                || matches!(meta.issuer, CommandIssuer::Import)
                || matches!(meta.issuer, CommandIssuer::Player(id) if !campaign.players.contains_key(&id))
                || matches!(meta.actor, Some(AgentRef::Entity(id)) if !entity(id))
                || matches!(meta.actor, Some(AgentRef::Faction(_)))
            {
                return Err("invalid tactical effect command provenance".into());
            }
            Ok(())
        };
        let source = |s: &EffectSource| -> Result<(), String> {
            origin(&s.command)?;
            if !entity(s.actor)
                || !text_valid(&s.definition_id)
                || s.command
                    .actor
                    .is_some_and(|actor| actor != AgentRef::Entity(s.actor))
                || (matches!(s.command.issuer, CommandIssuer::Player(_))
                    && s.command.actor.is_none())
            {
                return Err("invalid tactical effect source".into());
            }
            Ok(())
        };
        let expiry = |e: &TacticalEffectExpiry| -> Result<(), String> {
            if let TacticalEffectExpiry::AfterOwnerBoundaries {
                owner, remaining, ..
            } = e
                && (!entity(*owner) || *remaining == 0 || *remaining > 1_000_000)
            {
                return Err("invalid owner-relative effect expiry".into());
            }
            Ok(())
        };
        let mut ids = HashSet::new();
        let mut owners = HashSet::new();
        for group in &self.groups {
            if group.id.0.is_nil() || !ids.insert(group.id) || !owners.insert(group.source.actor) {
                return Err("duplicate concentration identity or owner".into());
            }
            source(&group.source)?;
            expiry(&group.expires)?;
            if group.stage == ConcentrationStage::Active
                && !self
                    .effects
                    .iter()
                    .any(|e| e.concentration_group == Some(group.id))
            {
                return Err("active concentration group has no effects".into());
            }
        }
        for effect in &self.effects {
            if effect.id.0.is_nil() || !ids.insert(effect.id) {
                return Err("duplicate tactical effect identity".into());
            }
            source(&effect.source)?;
            let installed = effect
                .established_at
                .as_ref()
                .ok_or("effect has no installation provenance")?;
            origin(&installed.command)?;
            if effect.source.command.expected_event_sequence
                > installed.command.expected_event_sequence
                || (effect.source.command.expected_event_sequence
                    == installed.command.expected_event_sequence
                    && effect.source.command != installed.command)
                || self.last_operation.as_ref().is_none_or(|last| {
                    installed.command.expected_event_sequence > last.command.expected_event_sequence
                        || (installed.command.expected_event_sequence
                            == last.command.expected_event_sequence
                            && (installed.command != last.command || installed.step > last.step))
                })
            {
                return Err("invalid effect installation provenance".into());
            }
            expiry(&effect.expires)?;
            if let Some(group_id) = effect.concentration_group {
                let group = self
                    .groups
                    .iter()
                    .find(|g| g.id == group_id)
                    .ok_or("orphaned tactical concentration effect")?;
                if group.source != effect.source || group.stage != ConcentrationStage::Active {
                    return Err("concentration effect source or stage mismatch".into());
                }
            }
            match &effect.target {
                TacticalEffectTarget::Creature(id) if !entity(*id) => {
                    return Err("unknown effect target".into());
                }
                TacticalEffectTarget::Zone { occupants } => {
                    let mut seen = HashSet::new();
                    if occupants.len() > 128
                        || !effect.conditions.is_empty()
                        || !effect.defenses.is_empty()
                        || occupants.iter().any(|id| !entity(*id) || !seen.insert(*id))
                    {
                        return Err("invalid zone occupants or direct zone condition".into());
                    }
                }
                _ => {}
            }
            if effect.overlap.as_ref().is_some_and(|o| !text_valid(&o.key)) {
                return Err("invalid overlap identity".into());
            }
            let mut conditions = HashSet::new();
            for view in &effect.conditions {
                if view.id.0.is_nil() || !ids.insert(view.id) || !conditions.insert(view.condition)
                {
                    return Err("duplicate condition view identity or condition".into());
                }
            }
            if effect.conditions.len() > 16 || effect.triggers.len() > 16 {
                return Err("effect contains too many clauses".into());
            }
            if effect.defenses.len() > 8 {
                return Err("effect contains too many defense clauses".into());
            }
            for (index, defense) in effect.defenses.iter().enumerate() {
                if effect.defenses[..index].contains(defense)
                    || match defense {
                        EffectDefense::BaseArmorClass { base, .. } => *base == 0 || *base > 30,
                        EffectDefense::ArmorClassBonus { bonus } => *bonus == 0 || *bonus > 20,
                        EffectDefense::PreventSpellDamage { spell_id } => !text_valid(spell_id),
                    }
                {
                    return Err("invalid or duplicate source defense clause".into());
                }
            }
            for trigger in &effect.triggers {
                if matches!(trigger.event, EffectTriggerEvent::ZoneContact(_))
                    && !matches!(effect.target, TacticalEffectTarget::Zone { .. })
                {
                    return Err("zone trigger attached to a creature effect".into());
                }
                if let EffectTriggerFrequency::OncePerTargetPerTurn { key } = &trigger.frequency
                    && !text_valid(key)
                {
                    return Err("invalid trigger limit key".into());
                }
                match &trigger.payload {
                    EffectTriggerPayload::SavingThrow {
                        dc,
                        on_success,
                        on_failure,
                        ..
                    } => {
                        if *dc == 0
                            || *dc > 100
                            || (effect.concentration_group.is_none()
                                && [on_success, on_failure]
                                    .contains(&&EffectSaveEnd::ConcentrationGroup))
                        {
                            return Err("invalid effect saving throw".into());
                        }
                    }
                    EffectTriggerPayload::Damage { dice, modifier, .. } => {
                        if dice.len() > 16
                            || (*modifier < 0 && dice.is_empty())
                            || dice.iter().any(|d| {
                                d.count == 0 || d.count > 100 || d.sides < 2 || d.sides > 100
                            })
                            || (dice.is_empty() && *modifier == 0)
                        {
                            return Err("invalid trigger damage expression".into());
                        }
                    }
                    EffectTriggerPayload::EndConcentrationGroup
                        if effect.concentration_group.is_none() =>
                    {
                        return Err("group-ending trigger lacks a group".into());
                    }
                    EffectTriggerPayload::ExpireTargetEffect
                    | EffectTriggerPayload::ExpireConcentrationGroup => {
                        return Err("expiry is not an authored trigger payload".into());
                    }
                    _ => {}
                }
            }
            if self.effects.iter().any(|other| {
                other.id != effect.id
                    && effect
                        .overlap
                        .as_ref()
                        .zip(other.overlap.as_ref())
                        .is_some_and(|(a, b)| a.key == b.key && a.potency == b.potency)
                    && effect.established_at == other.established_at
                    && effect.source.ordinal == other.source.ordinal
                    && match &effect.target {
                        TacticalEffectTarget::Creature(id) => effect_has_target(other, *id),
                        TacticalEffectTarget::Zone { occupants } => {
                            occupants.iter().any(|id| effect_has_target(other, *id))
                        }
                    }
            }) {
                return Err("overlapping effects have ambiguous equal-source precedence".into());
            }
        }
        if let Some(turn) = self.turn
            && (!entity(turn.actor)
                || turn.number == 0
                || campaign
                    .rules
                    .as_ref()
                    .and_then(|r| r.timing.as_ref())
                    .is_some_and(|t| turn.number > t.turn_number))
        {
            return Err("invalid effect turn cursor".into());
        }
        if let Some(stamp) = &self.last_operation {
            origin(&stamp.command)?;
        }
        if (!self.effects.is_empty() || !self.groups.is_empty() || self.turn.is_some())
            && self.last_operation.is_none()
        {
            return Err("effect lifecycle has no operation provenance".into());
        }
        if let Some(last) = &self.last_operation
            && self
                .effects
                .iter()
                .map(|e| &e.source)
                .chain(self.groups.iter().map(|g| &g.source))
                .any(|source| {
                    source.command.expected_event_sequence > last.command.expected_event_sequence
                        || (source.command.expected_event_sequence
                            == last.command.expected_event_sequence
                            && source.command != last.command)
                })
        {
            return Err("effect source does not precede or match lifecycle provenance".into());
        }
        let mut uses = HashSet::new();
        for usage in &self.trigger_uses {
            origin(&usage.origin.command)?;
            let effect = self
                .effects
                .iter()
                .find(|e| e.id == usage.effect)
                .ok_or("trigger history references an ended effect")?;
            if !entity(usage.target)
                || !uses.insert((usage.effect, usage.key.clone(), usage.target))
                || !self
                    .turn
                    .is_some_and(|t| t.actor == usage.turn_actor && t.number == usage.turn_number)
                || !effect.triggers.iter().any(|t| {
                    matches!(&t.frequency,
                    EffectTriggerFrequency::OncePerTargetPerTurn { key } if key == &usage.key)
                })
                || self.last_operation.as_ref().is_none_or(|last| {
                    usage.origin.command.expected_event_sequence
                        > last.command.expected_event_sequence
                        || (usage.origin.command.expected_event_sequence
                            == last.command.expected_event_sequence
                            && (usage.origin.command != last.command
                                || usage.origin.step > last.step))
                })
            {
                return Err("invalid per-turn trigger history".into());
            }
        }
        let mut tickets = HashSet::new();
        for ticket in &self.pending {
            origin(&ticket.origin.command)?;
            source(&ticket.source)?;
            if !entity(ticket.target)
                || !tickets.insert(ticket.id)
                || ticket.id.command != ticket.origin.command.id
                || ticket.id.step != ticket.origin.step
                || self.last_operation.as_ref().is_none_or(|last| {
                    ticket.origin.command.expected_event_sequence
                        > last.command.expected_event_sequence
                        || (ticket.origin.command.expected_event_sequence
                            == last.command.expected_event_sequence
                            && (ticket.origin.command != last.command
                                || ticket.origin.step > last.step))
                })
            {
                return Err("invalid pending effect ticket provenance".into());
            }
            match &ticket.cause {
                EffectObservation::Turn(turn) if self.turn != Some(*turn) => {
                    return Err("pending ticket has a foreign turn boundary".into());
                }
                EffectObservation::Damage {
                    source,
                    target,
                    amount,
                } if (*amount == 0 && ticket.rule_index.is_some())
                    || !entity(*target)
                    || source.is_some_and(|id| !entity(id)) =>
                {
                    return Err("invalid pending damage-trigger cause".into());
                }
                EffectObservation::Zone { effect, target, .. }
                    if !entity(*target)
                        || (ticket.rule_index.is_some()
                            && (*effect != ticket.effect || *target != ticket.target)) =>
                {
                    return Err("invalid pending zone-trigger cause".into());
                }
                EffectObservation::ArmorWorn { target } if !entity(*target) => {
                    return Err("invalid pending armor-trigger target".into());
                }
                _ => {}
            }
            match ticket.payload {
                EffectTriggerPayload::ExpireConcentrationGroup => {
                    let group = self
                        .groups
                        .iter()
                        .find(|g| g.id == ticket.effect)
                        .ok_or("expiry ticket references an unknown group")?;
                    if ticket.rule_index.is_some()
                        || ticket.source != group.source
                        || ticket.target != group.source.actor
                        || !expiry_matches(&group.expires, &ticket.cause, campaign.clock.now)
                    {
                        return Err("invalid group expiry ticket".into());
                    }
                }
                _ => {
                    let effect = self
                        .effects
                        .iter()
                        .find(|e| e.id == ticket.effect)
                        .ok_or("trigger ticket references an unknown effect")?;
                    if ticket.source != effect.source {
                        return Err("ticket source differs from effect".into());
                    }
                    if ticket.payload == EffectTriggerPayload::ExpireTargetEffect {
                        let expiry_target = match effect.target {
                            TacticalEffectTarget::Creature(id) => id,
                            TacticalEffectTarget::Zone { .. } => effect.source.actor,
                        };
                        if ticket.rule_index.is_some()
                            || ticket.target != expiry_target
                            || !expiry_matches(&effect.expires, &ticket.cause, campaign.clock.now)
                        {
                            return Err("invalid effect expiry ticket".into());
                        }
                    } else {
                        let rule = ticket
                            .rule_index
                            .and_then(|i| effect.triggers.get(usize::from(i)))
                            .ok_or("ticket has no matching trigger rule")?;
                        if matches!(
                            rule.frequency,
                            EffectTriggerFrequency::OncePerTargetPerTurn { .. }
                        ) && self.turn.is_none()
                        {
                            return Err("per-turn pending trigger has no turn cursor".into());
                        }
                        if rule.payload != ticket.payload
                            || !effect_trigger_targets(effect, rule, &ticket.cause)
                                .contains(&ticket.target)
                        {
                            return Err("ticket differs from its triggering source clause".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn expiry_matches(
    expiry: &TacticalEffectExpiry,
    cause: &EffectObservation,
    now: WorldInstant,
) -> bool {
    match expiry {
        TacticalEffectExpiry::Never => false,
        TacticalEffectExpiry::AtTime(at) => *at <= now,
        TacticalEffectExpiry::AfterOwnerBoundaries {
            owner,
            boundary,
            remaining,
        } => {
            *remaining == 1
                && matches!(cause, EffectObservation::Turn(turn) if turn.actor == *owner && turn.boundary == *boundary)
        }
    }
}

pub fn effect_has_target(effect: &TacticalEffect, target: EntityId) -> bool {
    match &effect.target {
        TacticalEffectTarget::Creature(id) => *id == target,
        TacticalEffectTarget::Zone { occupants } => occupants.contains(&target),
    }
}

/// The same mapping is used by scheduling and restored-ticket validation.
pub fn effect_trigger_targets(
    effect: &TacticalEffect,
    rule: &EffectTriggerRule,
    observation: &EffectObservation,
) -> Vec<EntityId> {
    let matches_subject = |subject, id| match subject {
        EffectSubject::Source => effect.source.actor == id,
        EffectSubject::Target => effect_has_target(effect, id),
    };
    let actor = match (&rule.event, observation) {
        (EffectTriggerEvent::Turn { subject, boundary }, EffectObservation::Turn(turn))
            if *boundary == turn.boundary && matches_subject(*subject, turn.actor) =>
        {
            (turn.actor, *subject)
        }
        (
            EffectTriggerEvent::Damage { subject },
            EffectObservation::Damage { target, amount, .. },
        ) if *amount > 0 && matches_subject(*subject, *target) => (*target, *subject),
        (EffectTriggerEvent::ArmorWorn { subject }, EffectObservation::ArmorWorn { target })
            if matches_subject(*subject, *target) =>
        {
            (*target, *subject)
        }
        (
            EffectTriggerEvent::ZoneContact(kind),
            EffectObservation::Zone {
                effect: id,
                target,
                contact,
            },
        ) if effect.id == *id && kind == contact => (*target, EffectSubject::Target),
        _ => return Vec::new(),
    };
    match &effect.target {
        TacticalEffectTarget::Creature(target) => vec![*target],
        TacticalEffectTarget::Zone { occupants } if actor.1 == EffectSubject::Source => {
            occupants.clone()
        }
        TacticalEffectTarget::Zone { .. } => vec![actor.0],
    }
}
