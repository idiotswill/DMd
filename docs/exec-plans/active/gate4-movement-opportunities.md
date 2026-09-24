# Gate 4 movement and opportunity attacks

Status: active implementation; no verification or production completion claimed.
Branch: `codex/gate4-movement-opportunities`, base `5114159`.

## Objective and boundaries

Resolve accepted movement through the existing spatial evaluator and the single
TacticalResolution queue. Spend actual movement only for committed segments. Before
leaving a hostile creature's usable melee reach, retain the pre-crossing position and
offer the eligible controller its real reaction choice. Accepted attacks, damage,
knockout, concentration and other resulting work must finish before movement resumes.

This advances Gate 4 movement, source timing, visibility and exact suspension, and
product-definition player authority, tactical timing/knowledge and save/resume clauses.
Read AGENTS, Gate 4 checkpoint, ADR025 and ADR026. Source: pinned SRD5.2.1 pp14–15
(movement, terrain, occupancy and opportunities), p90 (weapon reach), pp180/186/188
(Disengage, Prone, Speed) and p190 (teleportation). Source-independent discretization
or timing choices must be explicitly recorded, never described as source mandates.

Own new domain/rules movement modules, narrow existing spatial resumability changes,
focused tests and this plan. The attack author owns tactical.rs/TacticalResolution
until its coherent checkpoint, then coordinates the small shared integration edits.
Root owns app/table/UI, production evidence and final GitHub integration. Ready is a
separate subsequent slice; no raw force/teleport permission is a player command.

## Contracts and decisions

- RulesState timing and TacticalTurnBudget remain the only movement/reaction budgets.
  Movement state retains intent, progress, source cause and unresolved decisions only.
- Ordinary input contains path positions/modes, never hidden obstacles, targets,
  reaction eligibility, movement costs, speeds or authorization flags.
- A source-derived internal forced/teleport handoff must identify its accepted effect;
  these cannot spend a player's movement or accidentally provoke opportunities. Do not
  equate every compelled move with forced displacement: movement using the creature's
  action/Bonus Action/Reaction still provokes under SRD15 unless a feature says otherwise.
- Reuse spatial geometry, including bounded paths, swept solids, occupancy, selected
  Dash speed, elevation, difficult terrain and source conditions. Preserve run-up and
  jump origin across pauses; evaluating one intermediate segment must retain the actual
  final destination occupancy rule.
- Threat options come from currently usable held weapons, unarmed and source attacks.
  The participant's single generic reach cannot silently replace per-weapon/source reach.
  The attack author supplies a typed immutable query and a sealed live-window adapter.
- Nested OA attack uses the reacting controller's real command metadata and Reaction
  opportunity; it preserves the parent movement origin and work frames. Declining does
  not spend a reaction. Invalid, stale and foreign choices leave the state unchanged.
- Re-evaluate remaining legal movement after every interrupt. Death, unconsciousness,
  new obstacles or exhausted movement cannot teleport the mover to its old destination.
  A stopped path must leave a playable state with an explicit durable outcome.
- Player projections must not expose the list of unseen potential reactors, source
  identities or unknown geometry. Rule admission may use truth; planning/presentation
  must use actor knowledge. Root supplies the filtered production view.

## Work sequence

1. Check in plan; agree attack/queue ownership and concrete types.
2. Add durable movement intent/progress contracts and spatial single-segment evaluation.
3. Implement movement admission, source budget derivation, pre-crossing windows and
   continuation resumption in the shared queue once the attack checkpoint is available.
4. Compose the sealed reaction attack adapter, decline and source displacement handoffs.
5. Test source boundaries, serialization/semantic replay and malformed restored state;
   independent review, focused tests/Clippy, then root application/SQLite checks.

## Required evidence

- Actual mixed-mode/Dash budgets; difficult terrain and allied/enemy occupancy; swept
  walls and invalid destinations; no cost or position mutation on invalid admission.
- Visible eligible reach crossing pauses before position/cost commits; holding a Reach
  weapon differs from unarmed reach; no opportunity without sight or remaining Reaction.
- Disengage and source forced displacement/teleport do not provoke. Compelled movement
  using the mover's action is not mislabeled as free forced motion.
- Multiple independent opportunities retain correct ordering/ownership; decline does
  not spend Reaction; one accepted reaction cannot be repeated after restart or retry.
- Raw attack/damage and any concentration followup suspend above the same movement
  frame. Incapacitating/killing or immobilizing the mover prevents the unfinished move.
- Serialize and replay every pause; reject forged cursor, cost, pre-crossing position,
  source option, reactor, command provenance and stale/foreign continuation.
- Player-safe view evidence and a real SQLite movement/OA/restart scenario remain root
  integration requirements, not evidence supplied by pure reducer tests alone.

## Verification and next action

No Rust build has run for this slice. Builds are globally serialized; root NPC checks
and the attack author's focused checks precede this branch. Formatting is permitted.
Next: commit this plan, add the new movement contracts and resumable evaluator, then
integrate only the coordinated attack checkpoint. No Gate 4 acceptance claim.
