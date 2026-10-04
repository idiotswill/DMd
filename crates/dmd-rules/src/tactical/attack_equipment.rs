//! Private, rules-produced equipment continuation. Public and restore admission
//! remain closed: consistency of these records does not authenticate history.
use super::turns::*;
use super::*;

#[cfg(test)]
#[path = "attack_equipment_tests.rs"]
mod tests;

pub(crate) struct SelectedEquipment<'a> {
    state: &'a CampaignState,
    origin: CommandMeta,
    actor: EntityId,
    window: WeaponActionWindow,
}
impl<'a> SelectedEquipment<'a> {
    pub(crate) fn state(&self) -> &'a CampaignState {
        self.state
    }
    pub(crate) fn origin(&self) -> &CommandMeta {
        &self.origin
    }
    pub(crate) fn actor(&self) -> EntityId {
        self.actor
    }
    pub(crate) fn window(&self) -> WeaponActionWindow {
        self.window
    }
}

fn current_version(state: &CampaignState) -> Result<(), RulesError> {
    if flow(state)?.version != TacticalExecutionVersion::EncounterReleaseV1.flow_version()
        || flow(state)?.phase != TacticalPhase::Active
    {
        return Err(invalid("equipment continuation requires active flow 5"));
    }
    Ok(())
}
fn owner(state: &CampaignState, meta: &CommandMeta, actor: EntityId) -> Result<(), RulesError> {
    if controller(state, actor).is_some_and(|player| meta.issuer != CommandIssuer::Player(player)) {
        return Err(RulesError::Unauthorized);
    }
    authorize(state, meta, actor)
}
fn key(state: &CampaignState, occurrence: u16) -> Result<TacticalWorkKey, RulesError> {
    Ok(TacticalWorkKey {
        resolution: resolution(state)?.origin.id,
        occurrence,
    })
}
fn node(state: &CampaignState, key: TacticalWorkKey) -> Result<&TacticalWorkNode, RulesError> {
    let r = resolution(state)?;
    if key.resolution != r.origin.id || key.occurrence >= r.next_occurrence {
        return Err(invalid("equipment work belongs to another resolution"));
    }
    let trace = r
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("equipment work ancestry absent"))?;
    let mut nodes = trace
        .nodes
        .iter()
        .filter(|n| n.work.occurrence == key.occurrence);
    let node = nodes
        .next()
        .ok_or_else(|| invalid("equipment parent absent"))?;
    if nodes.next().is_some() {
        return Err(invalid("duplicate equipment parent"));
    }
    Ok(node)
}
fn entered(state: &CampaignState) -> Result<TacticalWorkKey, RulesError> {
    let occurrence = resolution(state)?
        .work_trace
        .as_ref()
        .and_then(|t| t.active)
        .ok_or_else(|| invalid("equipment completion has no entered work"))?;
    let key = key(state, occurrence)?;
    node(state, key)?;
    Ok(key)
}
fn declaration(state: &CampaignState, attack: &TacticalAttack) -> Result<(), RulesError> {
    current_version(state)?;
    let w = attack
        .weapon()
        .ok_or_else(|| invalid("equipment allowance lacks a physical attack"))?;
    if w.choice.after_equipment != Some(AfterAttackEquipmentIntent::Choose)
        || w.choice.equipment_change.is_some()
        || w.choice.purpose != WeaponAttackPurpose::Normal
        || w.window.kind != WeaponActionKind::AttackAction
        || !matches!(
            (&attack.source, &attack.admission),
            (
                TacticalAttackSource::Weapon(_),
                TacticalAttackAdmission::OwnTurn
            ) | (
                TacticalAttackSource::CreatureWeapon { .. },
                TacticalAttackAdmission::CreatureAction { approach: None }
            )
        )
        || attack.origin.id != resolution(state)?.origin.id
        || attack.actor != resolution(state)?.turn_actor
    {
        return Err(invalid("attack has no unused equipment allowance"));
    }
    Ok(())
}

pub(super) fn opted_in(attack: &TacticalAttack) -> bool {
    attack
        .weapon()
        .is_some_and(|w| w.choice.after_equipment.is_some())
}

/// Called only at the actual material pause while its original work is entered.
pub(super) fn pause(
    state: &mut CampaignState,
    meta: &CommandMeta,
    pause: AttackEquipmentPause,
) -> Result<(), RulesError> {
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("attack absent"))?;
    if !opted_in(attack) {
        return Ok(());
    }
    declaration(state, attack)?;
    let work = entered(state)?;
    let (kind, accepted_raw) = match pause {
        AttackEquipmentPause::Knockout => (TacticalWorkKind::AttackDamage, attack.damage_roll),
        AttackEquipmentPause::Graze => (TacticalWorkKind::FinishAttack, attack.attack_roll),
    };
    if node(state, work)?.work.kind != kind
        || attack.weapon().unwrap().after_equipment_parent.is_some()
        || pause == AttackEquipmentPause::Graze && (attack.automatic_miss || accepted_raw.is_none())
    {
        return Err(invalid("equipment pause lacks its actual parent"));
    }
    let parent = AttackEquipmentCompletionParent {
        work,
        pause,
        paused_by: meta.clone(),
        accepted_raw,
        suspended_outcome: attack
            .outcome
            .ok_or_else(|| invalid("paused outcome absent"))?,
    };
    resolution_mut(state)?
        .attack
        .as_mut()
        .unwrap()
        .weapon_mut()
        .unwrap()
        .after_equipment_parent = Some(parent);
    validate_pause(state)
}

pub(super) fn validate_pause(state: &CampaignState) -> Result<(), RulesError> {
    let Some(attack) = resolution(state)?.attack.as_ref() else {
        return Ok(());
    };
    let Some(w) = attack.weapon() else {
        return Ok(());
    };
    if !opted_in(attack) {
        return if w.after_equipment_parent.is_none() {
            Ok(())
        } else {
            Err(invalid("unrequested equipment parent"))
        };
    }
    declaration(state, attack)?;
    let expected = match attack.stage {
        TacticalAttackStage::KnockoutChoice => Some((
            AttackEquipmentPause::Knockout,
            TacticalWorkKind::AttackDamage,
            attack.damage_roll,
        )),
        TacticalAttackStage::MasteryChoice => Some((
            AttackEquipmentPause::Graze,
            TacticalWorkKind::FinishAttack,
            attack.attack_roll,
        )),
        _ => None,
    };
    let Some((pause, kind, raw)) = expected else {
        return if w.after_equipment_parent.is_none() {
            Ok(())
        } else {
            Err(invalid("premature equipment completion parent"))
        };
    };
    let parent = w
        .after_equipment_parent
        .as_ref()
        .ok_or_else(|| invalid("paused attack lost its completion parent"))?;
    validate_equipment_change_origin(state, &parent.paused_by, attack.actor)
        .map_err(|e| invalid(&e))?;
    if parent.pause != pause
        || node(state, parent.work)?.work.kind != kind
        || parent.accepted_raw != raw
        || Some(parent.suspended_outcome) != attack.outcome
        || parent.paused_by.session_id != attack.origin.session_id
        || parent.paused_by.expected_event_sequence < attack.origin.expected_event_sequence
        || attack.automatic_miss
        || pause == AttackEquipmentPause::Graze && raw.is_none()
    {
        return Err(invalid(
            "paused equipment parent differs from actual attack",
        ));
    }
    // Exact raw identity and its ancestor path bind this particular attack. The
    // ordinary attack validator independently rederives the dice/outcome.
    let rules = state.rules.as_ref().ok_or(RulesError::Uninitialized)?;
    let attack_raw = attack
        .attack_roll
        .ok_or_else(|| invalid("paused attack raw absent"))?;
    let record = rules
        .rolls
        .iter()
        .find(|r| r.request.id == raw.unwrap_or(attack_raw))
        .ok_or_else(|| invalid("paused equipment raw absent"))?;
    let PendingPurpose::TacticalResolution {
        encounter: e,
        key: raw_key,
    } = record.purpose
    else {
        return Err(invalid("paused equipment raw purpose differs"));
    };
    if e != encounter(state)?.id
        || raw_key.origin != attack.origin.id
        || raw_key.subject != attack.target
        || record.accepted_by.expected_event_sequence > parent.paused_by.expected_event_sequence
        || (pause == AttackEquipmentPause::Knockout
            && raw.is_some()
            && (raw_key.role != TacticalRollRole::AttackDamage
                || raw_key.occurrence != parent.work.occurrence
                || record.accepted_by != parent.paused_by))
        || ((pause == AttackEquipmentPause::Graze || raw.is_none())
            && raw_key.role != TacticalRollRole::Attack)
    {
        return Err(invalid("paused equipment raw cause differs"));
    }
    let mut cursor = Some(parent.work.occurrence);
    let mut found = false;
    for _ in 0..32_768 {
        let Some(at) = cursor else {
            break;
        };
        let ancestor = node(state, key(state, at)?)?;
        if at == raw_key.occurrence {
            found = true;
            break;
        }
        if ancestor.parent.is_some_and(|p| p >= at) {
            return Err(invalid("cyclic equipment ancestry"));
        }
        cursor = ancestor.parent;
    }
    if !found {
        return Err(invalid("equipment pause is not descended from its raw"));
    }
    Ok(())
}

/// Direct material choices re-enter the retained actual parent. No-intent calls
/// retain their old path, including its old ancestry behavior.
pub(super) fn resume<F>(
    state: &mut CampaignState,
    meta: &CommandMeta,
    action: F,
) -> Result<(), RulesError>
where
    F: FnOnce(&mut CampaignState) -> Result<(), RulesError>,
{
    let attack = resolution(state)?
        .attack
        .as_ref()
        .ok_or_else(|| invalid("attack absent"))?;
    if !opted_in(attack) {
        return action(state);
    }
    super::attacks::validate(state)?;
    validate_pause(state)?;
    let parent = attack
        .weapon()
        .unwrap()
        .after_equipment_parent
        .as_ref()
        .ok_or_else(|| invalid("no retained equipment completion pause is due"))?;
    let work = node(state, parent.work)?.work.clone();
    let previous = super::work_trace::enter(state, &work)?;
    let result = action(state).and_then(|()| super::falling::queue_losses(state, meta));
    let reset = super::work_trace::leave(state, previous);
    result?;
    reset
}

pub(super) fn completed(
    state: &mut CampaignState,
    meta: &CommandMeta,
    attack: &TacticalAttack,
    outcome: WeaponAttackOutcome,
) -> Result<(), RulesError> {
    if !opted_in(attack) {
        return Ok(());
    }
    declaration(state, attack)?;
    let w = attack.weapon().unwrap();
    let completed_work = entered(state)?;
    if !matches!(
        node(state, completed_work)?.work.kind,
        TacticalWorkKind::AttackDamage | TacticalWorkKind::FinishAttack
    ) || w
        .after_equipment_parent
        .as_ref()
        .is_some_and(|p| p.work != completed_work)
        || resolution(state)?.attack_after_equipment.is_some()
    {
        return Err(invalid("equipment completion parent differs"));
    }
    let source = match &attack.source {
        TacticalAttackSource::Weapon(_) => AttackEquipmentSource::Ordinary,
        TacticalAttackSource::CreatureWeapon {
            source, feature_id, ..
        } => AttackEquipmentSource::Creature {
            source: source.clone(),
            feature_id: feature_id.clone(),
        },
        _ => return Err(invalid("equipment completion is not physical")),
    };
    let cause = AttackEquipmentCause {
        origin: attack.origin.clone(),
        actor: attack.actor,
        turn_number: resolution(state)?.turn_number,
        window: w.window,
        choice: w.choice.clone(),
        source,
        outcome,
        completed_by: meta.clone(),
        completed_work,
    };
    if matching_receipt(state, &cause)?.after_equipment.is_some() {
        return Err(invalid("attack equipment was already decided"));
    }
    push_frame(state, vec![TacticalWorkKind::AttackAfterEquipment])?;
    let work = resolution(state)?
        .frames
        .last()
        .and_then(|f| f.first())
        .unwrap()
        .clone();
    resolution_mut(state)?.attack_after_equipment = Some(Box::new(TacticalAttackAfterEquipment {
        cause,
        work,
        selected_by: None,
    }));
    Ok(())
}

fn matching_receipt<'a>(
    state: &'a CampaignState,
    c: &AttackEquipmentCause,
) -> Result<&'a WeaponAttackReceipt, RulesError> {
    current_version(state)?;
    validate_equipment_origin(state, &c.origin, c.actor).map_err(|e| invalid(&e))?;
    validate_equipment_change_origin(state, &c.completed_by, c.actor).map_err(|e| invalid(&e))?;
    let mut matches = flow(state)?
        .budget
        .weapon_history
        .iter()
        .filter(|r| r.origin.id == c.origin.id);
    let r = matches
        .next()
        .ok_or_else(|| invalid("completed equipment attack absent"))?;
    if matches.next().is_some()
        || r.origin != c.origin
        || r.actor != c.actor
        || r.turn_number != c.turn_number
        || !r.on_actor_turn
        || r.window != c.window
        || c.window.kind != WeaponActionKind::AttackAction
        || r.weapon != c.choice.weapon
        || state
            .items
            .get(&r.weapon)
            .is_none_or(|item| item.definition_id != r.definition_id)
        || r.target != c.choice.target
        || r.delivery != c.choice.delivery
        || r.ability != c.choice.ability
        || r.grip != c.choice.grip
        || r.ammunition != c.choice.ammunition
        || r.purpose != c.choice.purpose
        || r.purpose != WeaponAttackPurpose::Normal
        || c.choice.after_equipment != Some(AfterAttackEquipmentIntent::Choose)
        || c.choice.equipment_change.is_some()
        || r.ground_pickup_before.is_some()
        || r.outcome != c.outcome
        || c.outcome == WeaponAttackOutcome::Pending
        || c.completed_by.session_id != c.origin.session_id
        || c.completed_by.expected_event_sequence < c.origin.expected_event_sequence
        || c.completed_work.resolution != c.origin.id
    {
        return Err(invalid("completed equipment cause differs from history"));
    }
    if let AttackEquipmentSource::Creature { source, feature_id } = &c.source {
        let profile = state
            .rules
            .as_ref()
            .and_then(|r| r.tactical_creatures.as_ref())
            .and_then(|r| r.profile(c.actor))
            .ok_or_else(|| invalid("completed equipment creature source absent"))?;
        if profile.source != *source
            || crate::tactical_creature_equipment::creature_attack_gear(profile, feature_id)?
                != Some(r.definition_id.as_str())
        {
            return Err(invalid("completed equipment source differs"));
        }
    }
    Ok(r)
}

pub(super) fn waiting(state: &CampaignState) -> bool {
    flow(state)
        .ok()
        .and_then(|f| f.resolution.as_ref())
        .and_then(|r| r.attack_after_equipment.as_ref())
        .is_some_and(|r| r.selected_by.is_some())
}
pub(super) fn validate_work(
    state: &CampaignState,
    work: &TacticalWorkItem,
) -> Result<EntityId, RulesError> {
    let record = resolution(state)?
        .attack_after_equipment
        .as_ref()
        .ok_or_else(|| invalid("equipment work lacks its cause"))?;
    if record.work != *work || work.kind != TacticalWorkKind::AttackAfterEquipment {
        return Err(invalid("equipment work identity differs"));
    }
    Ok(record.cause.actor)
}
pub(super) fn select(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: &TacticalWorkItem,
) -> Result<(), RulesError> {
    validate_work(state, work)?;
    let record = resolution(state)?.attack_after_equipment.as_ref().unwrap();
    matching_receipt(state, &record.cause)?;
    if record.selected_by.is_some()
        || entered(state)? != key(state, work.occurrence)?
        || resolution(state)?
            .frames
            .iter()
            .flatten()
            .any(|w| w.occurrence == work.occurrence)
        || meta.expected_event_sequence < record.cause.completed_by.expected_event_sequence
    {
        return Err(invalid("equipment work selection differs"));
    }
    resolution_mut(state)?
        .attack_after_equipment
        .as_mut()
        .unwrap()
        .selected_by = Some(meta.clone());
    Ok(())
}

pub(super) fn validate(state: &CampaignState) -> Result<(), RulesError> {
    for r in &flow(state)?.budget.weapon_history {
        if let Some(final_record) = &r.after_equipment {
            if matching_receipt(state, &final_record.cause)? != r {
                return Err(invalid("equipment decision history differs"));
            }
            decision_metadata(state, final_record)?;
        }
    }
    let Some(r) = flow(state)?.resolution.as_ref() else {
        return Ok(());
    };
    if r.attack.is_some() {
        validate_pause(state)?;
    }
    let live = r
        .frames
        .iter()
        .flatten()
        .chain(r.pending.iter().map(|p| &p.work))
        .chain(r.failed_save.iter().map(|p| &p.pending.work))
        .filter(|w| w.kind == TacticalWorkKind::AttackAfterEquipment)
        .collect::<Vec<_>>();
    let Some(record) = &r.attack_after_equipment else {
        return if live.is_empty() {
            Ok(())
        } else {
            Err(invalid("unowned equipment work"))
        };
    };
    let receipt = matching_receipt(state, &record.cause)?;
    let n = node(state, key(state, record.work.occurrence)?)?;
    let parent = node(state, record.cause.completed_work)?;
    if receipt.after_equipment.is_some()
        || r.attack.is_some()
        || record.work.kind != TacticalWorkKind::AttackAfterEquipment
        || n.work != record.work
        || n.parent != Some(parent.work.occurrence)
        || !matches!(
            parent.work.kind,
            TacticalWorkKind::AttackDamage | TacticalWorkKind::FinishAttack
        )
        || r.turn_actor != record.cause.actor
        || r.origin != record.cause.origin
        || active(state)? != record.cause.actor
        || state
            .rules
            .as_ref()
            .and_then(|rules| rules.timing.as_ref())
            .is_none_or(|timing| timing.turn_number != record.cause.turn_number)
        || r.turn_number != record.cause.turn_number
        || record.work.occurrence <= record.cause.completed_work.occurrence
        || r.work_trace
            .as_ref()
            .is_some_and(|trace| trace.active.is_some())
        || (record.selected_by.is_none()
            && (live.len() != 1
                || *live[0] != record.work
                || !r.frames.iter().any(|f| f.len() == 1 && f[0] == record.work)))
        || (record.selected_by.is_some() && !live.is_empty())
    {
        return Err(invalid("equipment continuation ownership differs"));
    }
    if let Some(meta) = &record.selected_by {
        validate_equipment_change_origin(state, meta, record.cause.actor)
            .map_err(|e| invalid(&e))?;
        if meta.session_id != record.cause.origin.session_id
            || meta.expected_event_sequence < record.cause.completed_by.expected_event_sequence
            || r.pending.is_some()
            || r.failed_save.is_some()
            || r.legendary_window.is_some()
            || !r.frames.is_empty()
            || super::shove::waiting(state)
            || super::hit_reactions::waiting(state)
            || super::missiles::waiting(state)
            || super::falling::selected(state)?.is_some()
            || state
                .rules
                .as_ref()
                .is_some_and(|rules| rules.pending.is_some())
        {
            return Err(invalid("equipment selection precedes completion"));
        }
    }
    Ok(())
}
fn decision_metadata(
    state: &CampaignState,
    r: &AttackAfterEquipmentReceipt,
) -> Result<(), RulesError> {
    validate_equipment_change_origin(state, &r.selected_by, r.cause.actor)
        .map_err(|e| invalid(&e))?;
    validate_equipment_origin(state, &r.chosen_by, r.cause.actor).map_err(|e| invalid(&e))?;
    if r.selected_by.session_id != r.cause.origin.session_id
        || r.chosen_by.session_id != r.cause.origin.session_id
        || r.work.resolution != r.cause.completed_work.resolution
        || r.work.occurrence <= r.cause.completed_work.occurrence
        || r.work.occurrence >= 32_768
        || r.selected_by.expected_event_sequence < r.cause.completed_by.expected_event_sequence
        || r.chosen_by.expected_event_sequence <= r.selected_by.expected_event_sequence
        || r.chosen_by.id == r.selected_by.id
        || r.chosen_by.id == r.cause.origin.id
        || r.applied.as_ref().is_some_and(|a| {
            a.equipment_before.actor != r.cause.actor
                || matches!(a.operation, AttackEquipmentOperation::Pickup { .. })
                    != a.ground_before.is_some()
        })
    {
        return Err(invalid("equipment decision provenance differs"));
    }
    if let Some(applied) = &r.applied {
        validate_equipment_change_origin(state, &applied.equipment_before.command, r.cause.actor)
            .map_err(|e| invalid(&e))?;
        if applied.equipment_before.command.expected_event_sequence
            >= r.chosen_by.expected_event_sequence
        {
            return Err(invalid(
                "equipment before image does not precede its decision",
            ));
        }
        if let Some(ground) = &applied.ground_before {
            validate_equipment_change_origin(state, &ground.ground.origin, r.cause.actor)
                .map_err(|e| invalid(&e))?;
            if ground.equipment != applied.equipment_before
                || ground.item.id != ground.ground.item
                || ground.ground.origin.expected_event_sequence
                    >= r.chosen_by.expected_event_sequence
                || !matches!(applied.operation, AttackEquipmentOperation::Pickup { item, .. } if item == ground.item.id)
            {
                return Err(invalid("equipment pickup before image differs"));
            }
        }
    }
    Ok(())
}

/// The inverse belongs to the exact selected decision cut; a retired final
/// receipt cannot reconstruct equipment after later play.
pub(crate) fn validate_inverse_context(
    state: &CampaignState,
    receipt: &AttackAfterEquipmentReceipt,
) -> Result<(), RulesError> {
    validate(state)?;
    decision_metadata(state, receipt)?;
    let selected = resolution(state)?
        .attack_after_equipment
        .as_ref()
        .ok_or_else(|| invalid("equipment inverse lacks its selected cut"))?;
    if selected.cause != receipt.cause
        || selected.selected_by.as_ref() != Some(&receipt.selected_by)
        || key(state, selected.work.occurrence)? != receipt.work
        || receipt.chosen_by.expected_event_sequence != state.applied_event_sequence
    {
        return Err(invalid("equipment inverse cause differs"));
    }
    owner(state, &receipt.chosen_by, receipt.cause.actor)
}

pub(super) fn choose(
    state: &mut CampaignState,
    meta: &CommandMeta,
    work: TacticalWorkKey,
    choice: AttackEquipmentChoice,
    pack: &RulesPack,
) -> Result<(), RulesError> {
    validate(state)?;
    let record = resolution(state)?
        .attack_after_equipment
        .as_ref()
        .ok_or_else(|| invalid("no equipment choice due"))?;
    let selected_by = record
        .selected_by
        .clone()
        .ok_or_else(|| invalid("equipment work is not selected"))?;
    if work != key(state, record.work.occurrence)?
        || meta.campaign_id != state.campaign_id()
        || meta.expected_event_sequence != state.applied_event_sequence
        || meta.id.0.is_nil()
    {
        return Err(invalid("stale equipment choice"));
    }
    owner(state, meta, record.cause.actor)?;
    let token = SelectedEquipment {
        state,
        origin: meta.clone(),
        actor: record.cause.actor,
        window: record.cause.window,
    };
    let (mut candidate, applied) = match choice {
        AttackEquipmentChoice::Decline => (state.clone(), None),
        AttackEquipmentChoice::Apply(operation) => {
            let prepared = crate::tactical_weapons::ground::prepare_after(&token, operation, pack)
                .map_err(|e| invalid(&e.to_string()))?;
            let (candidate, image) = prepared
                .consume(state)
                .map_err(|e| invalid(&e.to_string()))?;
            (candidate, Some(Box::new(image)))
        }
    };
    let receipt = AttackAfterEquipmentReceipt {
        cause: record.cause.clone(),
        work,
        selected_by,
        chosen_by: meta.clone(),
        applied,
    };
    decision_metadata(state, &receipt)?;
    let restored = crate::tactical_weapons::ground::restore_after_image(&candidate, &receipt, pack)
        .map_err(|e| invalid(&e.to_string()))?;
    if restored != *state {
        return Err(invalid("equipment inverse changed unrelated state"));
    }
    flow_mut(&mut candidate)?
        .budget
        .weapon_history
        .iter_mut()
        .find(|r| r.origin.id == receipt.cause.origin.id)
        .unwrap()
        .after_equipment = Some(Box::new(receipt));
    resolution_mut(&mut candidate)?.attack_after_equipment = None;
    pump(&mut candidate, meta)?;
    validate(&candidate)?;
    *state = candidate;
    Ok(())
}
