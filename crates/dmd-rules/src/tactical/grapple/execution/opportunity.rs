//! Historical permission comes from an actual producer or the owned predecessor,
//! never from re-running a stopped route against a fabricated before-scene.
use super::*;

#[derive(Clone, PartialEq, Eq)]
struct Crossing {
    encounter: EncounterId,
    setup: CommandMeta,
    resolution: CommandMeta,
    turn: u64,
    movement: TacticalMovement,
    admission: GrappleTransportAdmission,
    steps: Vec<GrappleTransportStepReceipt>,
    grip: TacticalGrip,
}

impl Crossing {
    fn capture(state: &CampaignState) -> Result<Self, RulesError> {
        let r = resolution(state)?;
        let c = r
            .grapple
            .as_ref()
            .ok_or_else(|| invalid("crossing proof absent"))?;
        let history = c
            .transport
            .as_ref()
            .ok_or_else(|| invalid("crossing transport absent"))?;
        let mut movement = r
            .movement
            .as_deref()
            .ok_or_else(|| invalid("crossing movement absent"))?
            .clone();
        let grip = unique(
            c.proofs
                .iter()
                .filter(|proof| proof.declaration.id == history.admission.grip),
        )?
        .clone();
        if !crate::table::grapple_enabled(state)
            || movement.origin != r.origin
            || history.admission.origin != r.origin
            || movement.actor != r.turn_actor
            || history.admission.holder != movement.actor
            || history.admission.path != movement.path
            || history.admission.holder_from != movement.initial_position
            || history.admission.spent_before != movement.initial_spent
            || grip.declaration.grappler != movement.actor
            || grip.declaration.target != history.admission.target
            || usize::from(movement.next_step) != history.steps.len()
            || movement.traversed.len() != history.steps.len()
        {
            return Err(invalid("accepted crossing differs from its paired prefix"));
        }
        // Answers from other reactors and a later stop are mutable consequences.
        // The complete admitted path and already committed prefix are not.
        movement.offered.clear();
        movement.decisions.clear();
        movement.opportunity = None;
        Ok(Self {
            encounter: encounter(state)?.id,
            setup: encounter(state)?.origin.clone(),
            resolution: r.origin.clone(),
            turn: r.turn_number,
            movement,
            admission: history.admission.clone(),
            steps: history.steps.clone(),
            grip,
        })
    }
}

/// No serialized capability and no constructor outside this module. The sole
/// minting method runs the unchanged live validator against the actual candidate.
pub(in crate::tactical) struct LiveOpportunity {
    candidate: *const CampaignState,
    command: CommandMeta,
    window: TacticalOpportunityWindow,
    choice: TacticalMeleeChoice,
    crossing: Option<Crossing>,
    trigger: Vec<TacticalWorkNode>,
}
impl LiveOpportunity {
    pub(in crate::tactical) fn window(&self) -> &TacticalOpportunityWindow {
        &self.window
    }
}

/// A bounded projection of records already persisted by the real shared producer.
/// Stage/dice/outcome advance; the original declaration and crossing stay fixed.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct GroundOpportunity {
    crossing: Crossing,
    attack: TacticalAttack,
    response: TacticalOpportunityDecision,
    ancestry: Vec<TacticalWorkNode>,
    cuts: Vec<GrappleReadCut>,
    proofs: Vec<TacticalGrip>,
}

fn unique<'a, T: 'a>(mut items: impl Iterator<Item = &'a T>) -> Result<&'a T, RulesError> {
    let item = items
        .next()
        .ok_or_else(|| invalid("accepted opportunity binding absent"))?;
    if items.next().is_some() {
        return Err(invalid("accepted opportunity binding is ambiguous"));
    }
    Ok(item)
}

fn node(state: &CampaignState, occurrence: u16) -> Result<&TacticalWorkNode, RulesError> {
    let trace = resolution(state)?
        .work_trace
        .as_ref()
        .ok_or_else(|| invalid("crossing ancestry absent"))?;
    unique(
        trace
            .nodes
            .iter()
            .filter(|node| node.work.occurrence == occurrence),
    )
}

fn trigger(
    state: &CampaignState,
    work: &TacticalWorkItem,
    reactor: EntityId,
) -> Result<Vec<TacticalWorkNode>, RulesError> {
    let selected = node(state, work.occurrence)?;
    let parent = selected
        .parent
        .ok_or_else(|| invalid("crossing has no entered segment"))?;
    let segment = node(state, parent)?;
    if selected.work != *work
        || selected.work.kind != (TacticalWorkKind::MovementOpportunity { reactor })
        || parent >= selected.work.occurrence
        || segment.work.kind != TacticalWorkKind::MoveSegment
    {
        return Err(invalid(
            "accepted opportunity differs from its entered crossing",
        ));
    }
    Ok(vec![selected.clone(), segment.clone()])
}

fn attached(state: &CampaignState) -> Result<Option<GroundOpportunity>, RulesError> {
    let Some(r) = state
        .encounter
        .as_ref()
        .and_then(|e| e.flow.as_ref())
        .and_then(|f| f.resolution.as_ref())
    else {
        return Ok(None);
    };
    let Some(attack) = &r.attack else {
        return Ok(None);
    };
    let TacticalAttackAdmission::Opportunity(window) = &attack.admission else {
        return Ok(None);
    };
    if r.grapple
        .as_ref()
        .and_then(|c| c.transport.as_ref())
        .is_none()
    {
        return Ok(None);
    }
    let crossing = Crossing::capture(state)?;
    let movement = r
        .movement
        .as_ref()
        .ok_or_else(|| invalid("accepted crossing movement absent"))?;
    let response = unique(
        movement
            .decisions
            .iter()
            .filter(|decision| decision.reactor == attack.actor),
    )?;
    let from = crossing
        .steps
        .last()
        .map_or(crossing.admission.holder_from, |step| step.holder.to);
    let to = movement
        .path
        .get(usize::from(movement.next_step))
        .ok_or_else(|| invalid("accepted crossing is beyond its path"))?
        .destination;
    if window.reactor != attack.actor
        || window.mover != attack.target
        || window.mover != movement.actor
        || window.step_index != movement.next_step
        || window.from != from
        || window.to != to
        || response.origin != attack.origin
        || response.kind != TacticalOpportunityDecisionKind::Attack
        || !movement.offered.contains(&attack.actor)
        || !state
            .rules
            .as_ref()
            .and_then(|r| r.timing.as_ref())
            .is_some_and(|t| t.reactions_spent.contains(&attack.actor))
    {
        return Err(invalid(
            "accepted opportunity differs from its response and prefix",
        ));
    }
    let root = reads::attack_root(state)?;
    let parent = root
        .parent
        .ok_or_else(|| invalid("accepted attack has no crossing parent"))?;
    if root.work.kind != TacticalWorkKind::AttackRoll || parent >= root.work.occurrence {
        return Err(invalid("accepted attack has invalid crossing ancestry"));
    }
    let mut ancestry = vec![root.clone()];
    ancestry.extend(trigger(state, &node(state, parent)?.work, attack.actor)?);
    let work = TacticalWorkKey {
        resolution: r.origin.id,
        occurrence: root.work.occurrence,
    };
    let c = r
        .grapple
        .as_ref()
        .ok_or_else(|| invalid("accepted opportunity cuts absent"))?;
    let mut cuts = Vec::new();
    let mut proofs = Vec::new();
    for reader in [
        GrappleReader::AttackAdmission {
            attack: attack.origin.id,
        },
        GrappleReader::OpportunityWindow {
            attack: attack.origin.id,
            window: window.origin.id,
            reactor: attack.actor,
            mover: attack.target,
            step: window.step_index,
        },
    ] {
        let cut = unique(
            c.cuts
                .iter()
                .filter(|cut| cut.key == (GrappleCutKey { work, reader })),
        )?;
        if cut.issued_by != attack.origin || cut.source_attack.is_some() {
            return Err(invalid(
                "accepted opportunity cut differs from its producer",
            ));
        }
        for id in &cut.grips {
            let proof = unique(c.proofs.iter().filter(|proof| proof.declaration.id == *id))?;
            if !proofs.contains(proof) {
                proofs.push(proof.clone());
            }
        }
        cuts.push(cut.clone());
    }
    let mut attack = attack.clone();
    attack.stage = TacticalAttackStage::AttackRoll;
    attack.attack_roll = None;
    attack.damage_roll = None;
    attack.outcome = None;
    Ok(Some(GroundOpportunity {
        crossing,
        attack,
        response: response.clone(),
        ancestry,
        cuts,
        proofs,
    }))
}

impl ExecutionContext<'_> {
    pub(in crate::tactical) fn admit_opportunity(
        &self,
        state: &CampaignState,
        meta: &CommandMeta,
        actor: EntityId,
        target: EntityId,
        choice: &TacticalMeleeChoice,
    ) -> Result<LiveOpportunity, RulesError> {
        let window = crate::tactical::movement::validate_opportunity_with_read(
            &self.read(state)?,
            actor,
            target,
        )?
        .clone();
        let (crossing, trigger) = if super::super::transport::history(state).is_some() {
            let owned = self
                .guarded
                .as_ref()
                .ok_or_else(|| invalid("ground crossing lacks owned producer"))?;
            if owned.command != meta {
                return Err(invalid("ground crossing belongs to another command"));
            }
            let work = crate::tactical::work_trace::prior_work(
                state,
                &TacticalWorkKind::MovementOpportunity { reactor: actor },
            )?
            .ok_or_else(|| invalid("selected crossing work absent"))?;
            (
                Some(Crossing::capture(state)?),
                trigger(state, &work, actor)?,
            )
        } else {
            (None, vec![])
        };
        Ok(LiveOpportunity {
            candidate: std::ptr::from_ref(state),
            command: meta.clone(),
            window,
            choice: choice.clone(),
            crossing,
            trigger,
        })
    }

    pub(in crate::tactical) fn observe_opportunity(
        &mut self,
        state: &CampaignState,
        live: LiveOpportunity,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        if !std::ptr::eq(live.candidate, state) {
            return Err(invalid(
                "live opportunity token belongs to another candidate",
            ));
        }
        let Some(crossing) = live.crossing else {
            return Ok(());
        };
        let accepted =
            attached(state)?.ok_or_else(|| invalid("live crossing did not produce its attack"))?;
        let owned = self
            .guarded
            .as_mut()
            .ok_or_else(|| invalid("crossing producer absent"))?;
        let prior = resolution(owned.predecessor)?
            .movement
            .as_ref()
            .ok_or_else(|| invalid("original selected movement absent"))?;
        let choice_matches = match (&live.choice, &accepted.attack.source) {
            (TacticalMeleeChoice::Weapon(choice), TacticalAttackSource::Weapon(source)) => {
                &source.choice == choice
            }
            (
                TacticalMeleeChoice::UnarmedDamage { ability },
                TacticalAttackSource::Unarmed { ability: source },
            ) => source == ability,
            (
                TacticalMeleeChoice::CreatureFeature { feature_id, weapon },
                TacticalAttackSource::CreatureFeature {
                    feature_id: source,
                    weapon: item,
                    ..
                },
            ) => source == feature_id && item == weapon,
            _ => false,
        };
        if owned.command != &live.command
            || !choice_matches
            || accepted.attack.origin != live.command
            || accepted.attack.admission
                != TacticalAttackAdmission::Opportunity(Box::new(live.window.clone()))
            || accepted.crossing != crossing
            || accepted.ancestry[1..] != live.trigger
            || prior.opportunity.as_ref() != Some(&live.window)
            || Crossing::capture(owned.predecessor)? != crossing
            || prior
                .decisions
                .iter()
                .any(|d| d.reactor == accepted.attack.actor)
            || resolution(owned.predecessor)?.attack.is_some()
            || !owned.produced.opportunities.is_empty()
        {
            return Err(invalid(
                "accepted crossing differs from its actual live producer",
            ));
        }
        owned.produced.opportunities.push(accepted);
        Ok(())
    }

    pub(in crate::tactical) fn complete_opportunity(
        &mut self,
        state: &CampaignState,
        meta: &CommandMeta,
    ) -> Result<(), RulesError> {
        self.check_state(state)?;
        let Some(accepted) = attached(state)? else {
            return Ok(());
        };
        self.read(state)?.certify_opportunity(&accepted)?;
        let r = resolution(state)?;
        let current = r
            .work_trace
            .as_ref()
            .and_then(|trace| trace.active)
            .ok_or_else(|| invalid("crossing completion lacks entered work"))?;
        let work = node(state, current)?;
        if self
            .guarded
            .as_ref()
            .is_none_or(|owned| owned.command != meta)
            || !matches!(
                work.work.kind,
                TacticalWorkKind::AttackDamage | TacticalWorkKind::FinishAttack
            )
            || !reads::in_lineage(state, current, accepted.ancestry[0].work.occurrence)?
            || r.attack
                .as_ref()
                .is_none_or(|attack| attack.outcome.is_none())
        {
            return Err(invalid(
                "crossing retirement differs from its actual completion",
            ));
        }
        self.guarded
            .as_mut()
            .ok_or_else(|| invalid("crossing completion lacks owned producer"))?
            .produced
            .completed_opportunities
            .push(accepted);
        Ok(())
    }

    pub(super) fn validate_opportunity_delta(
        &self,
        state: &CampaignState,
    ) -> Result<(), RulesError> {
        let Some(owned) = &self.guarded else {
            return Ok(());
        };
        let mut owed: Vec<_> = attached(owned.predecessor)?
            .into_iter()
            .chain(owned.produced.opportunities.iter().cloned())
            .collect();
        for completed in &owned.produced.completed_opportunities {
            let matches: Vec<_> = owed
                .iter()
                .enumerate()
                .filter(|(_, old)| *old == completed)
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err(invalid("crossing completion lacks one actual admission"));
            }
            owed.remove(matches[0]);
        }
        if owed != attached(state)?.into_iter().collect::<Vec<_>>() {
            return Err(invalid(
                "owned execution rewrote or retired an uncompleted crossing",
            ));
        }
        Ok(())
    }
}

impl ReadContext<'_> {
    fn certify_opportunity(&self, accepted: &GroundOpportunity) -> Result<(), RulesError> {
        if self.closed.is_some() {
            return Ok(());
        }
        let owned = self
            .guarded
            .ok_or_else(|| invalid("accepted crossing lacks original execution"))?;
        if !owned.produced.completed_opportunities.contains(accepted)
            && (attached(owned.predecessor)?.as_ref() == Some(accepted)
                || owned.produced.opportunities.contains(accepted))
        {
            Ok(())
        } else {
            Err(invalid(
                "accepted crossing lacks its predecessor or observed producer",
            ))
        }
    }

    pub(in crate::tactical) fn accepted_ground_opportunity(
        &self,
        attack: &TacticalAttack,
    ) -> Result<bool, RulesError> {
        let Some(accepted) = attached(self.state)? else {
            return Ok(false);
        };
        if resolution(self.state)?.attack.as_ref() != Some(attack) {
            return Err(invalid("crossing reader differs from attached attack"));
        }
        self.certify_opportunity(&accepted)?;
        Ok(true)
    }
}
