//! Closed private temporal profile; ordinary family validators still prove work.
use super::*;

fn boundary_work(kind: &TacticalWorkKind) -> bool {
    matches!(
        kind,
        TacticalWorkKind::Effect { .. }
            | TacticalWorkKind::DeathSave { .. }
            | TacticalWorkKind::StableRecovery { .. }
            | TacticalWorkKind::RecoverStable { .. }
            | TacticalWorkKind::EndOccupiedSpace { .. }
            | TacticalWorkKind::CreatureRecharge { .. }
            | TacticalWorkKind::ConcentrationSave { .. }
    )
}
fn attack_work(kind: &TacticalWorkKind) -> bool {
    matches!(
        kind,
        TacticalWorkKind::AttackRoll
            | TacticalWorkKind::AttackDamage
            | TacticalWorkKind::FinishAttack
            | TacticalWorkKind::ResumeHit { .. }
            | TacticalWorkKind::CommitShield { .. }
            | TacticalWorkKind::SpellProgram { .. }
            | TacticalWorkKind::FinishSpell { .. }
            | TacticalWorkKind::Effect { .. }
            | TacticalWorkKind::ConcentrationSave { .. }
            | TacticalWorkKind::StableRecovery { .. }
            | TacticalWorkKind::RecoverStable { .. }
    )
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    let Some(r) = flow(state)?.resolution.as_ref() else {
        return Ok(());
    };
    if r.shove.is_some()
        || r.movement.is_some()
        || !r.areas.is_empty()
        || !r.missiles.is_empty()
        || !r.falls.is_empty()
        || r.legendary_window.is_some()
        || r.grapple
            .as_ref()
            .is_some_and(|g| g.activity.is_some() || !g.opportunity_refreshes.is_empty())
    {
        return Err(prerequisite("unsupported private attack/boundary consumer"));
    }
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("private temporal work trace absent"))?;
    let attack_context = r.attack.is_some()
        || trace
            .nodes
            .iter()
            .any(|n| n.work.kind == TacticalWorkKind::AttackRoll);
    if !attack_context {
        if r.grapple.is_some()
            || r.hit_review.is_some()
            || !r.casts.is_empty()
            || trace.nodes.iter().any(|n| !boundary_work(&n.work.kind))
        {
            return Err(prerequisite("unsupported private turn boundary"));
        }
        return Ok(());
    }
    let root = reads::attack_root(state)?;
    if r.boundary != TurnBoundary::Start
        || r.turn_actor != active(state)?
        || trace.nodes.iter().any(|n| !attack_work(&n.work.kind))
    {
        return Err(prerequisite("unsupported own-turn attack lineage"));
    }
    let mut descendants = std::collections::HashSet::new();
    for node in &trace.nodes {
        if node != root
            && node.parent.is_none_or(|parent| {
                parent >= node.work.occurrence || !descendants.contains(&parent)
            })
        {
            return Err(invalid("unrelated work is not an attack descendant"));
        }
        if !descendants.insert(node.work.occurrence) {
            return Err(invalid("duplicate attack work occurrence"));
        }
    }
    if let Some(a) = &r.attack {
        let own = match (&a.source, &a.admission) {
            (TacticalAttackSource::Weapon(w), TacticalAttackAdmission::OwnTurn) => {
                w.choice.purpose == WeaponAttackPurpose::Normal
                    && w.window.kind == WeaponActionKind::AttackAction
            }
            (
                TacticalAttackSource::CreatureWeapon { weapon: w, .. },
                TacticalAttackAdmission::CreatureAction { approach: None },
            ) => {
                w.choice.purpose == WeaponAttackPurpose::Normal
                    && w.window.kind == WeaponActionKind::AttackAction
            }
            (
                TacticalAttackSource::CreatureFeature { .. },
                TacticalAttackAdmission::CreatureAction { .. },
            ) => true,
            (
                TacticalAttackSource::Unarmed { .. },
                TacticalAttackAdmission::UnarmedAction { window },
            ) => window.kind == WeaponActionKind::AttackAction,
            _ => false,
        };
        if !own || a.origin != r.origin || a.actor != r.turn_actor {
            return Err(prerequisite("attack is outside the four own-turn families"));
        }
    }
    if let Some(hit) = &r.hit_review
        && (hit.attack_origin != r.origin.id || r.attack.is_none())
    {
        return Err(invalid("hit response belongs to another attack"));
    }
    for cast in &r.casts {
        let hit = r
            .hit_review
            .as_ref()
            .ok_or_else(|| invalid("private cast lacks its hit response"))?;
        if hit.selected_cast != Some(cast.cast.plan.occurrence)
            || cast.cast.plan.choice.spell_id != "shield"
            || cast.cast.plan.cost != SpellCastingCost::Reaction
            || cast.cast.plan.program.concentration
            || !matches!(cast.cast.plan.choice.mode, SpellCastMode::Immediate)
            || cast.targets.len() != 1
            || cast.targets[0].actor != cast.cast.plan.choice.actor
            || r.attack
                .as_ref()
                .is_none_or(|a| a.target != cast.cast.plan.choice.actor)
        {
            return Err(prerequisite(
                "private casting is restricted to the exact hit Shield",
            ));
        }
        crate::tactical_spells::validate_retained_spell(cast)?;
    }
    reads::validate_cuts(state)
}
