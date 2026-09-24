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

Builds are globally serialized. The resumable spatial evaluator passed all 30 focused
tests (four added regressions); strict domain/rules library Clippy passed. The tactical
movement adapter and its reducer scenarios are not yet linked or compiled.
Next: finish focused geometry verification, integrate the coordinated ordinary attack
checkpoint, then attach movement to its shared cursor and actual opportunity adapter.
No Gate 4 acceptance claim.

## Implementation checkpoint

Plan `1423e10`; early domain contracts `5a013ed` include source melee options agreed
with the attack author and optional/default/skip-None turn movement progress. No shared
resolution or dispatcher file has changed. The spatial follow-up draft resumes jump
origin/run-up, distinguishes transit occupancy from a chosen endpoint, clears completed
jump/displacement context and queries lifecycle Frightened sources. Its Dash capacity
matches the existing bounded twenty-grant turn record; this is a capacity limit, not a
source grant. Internal admission/segment/history helpers are drafted but not attached
until shared queue ownership transfers. Their reducer tests remain unrun.

Opportunity response provenance is retained by `5eadf61`; `7efda05` distinguishes an
automatic source cancellation from a controller's decline. Root's wire-transparent
boxed cursor `1237c3a` is integrated as `8f6631c`. The attack author has now handed
shared dispatcher/work/pump ownership to this branch after ordinary checkpoint
`334d82b`; that author retains `tactical/attacks/**` and its melee choice contract.
Agreed actions are `Move { path }`, `DeclineOpportunity` and `OpportunityAttack {
choice }`. The reactor and target derive from the live selected crossing.

Ten reducer scenarios are drafted, including source reach versus unarmed reach,
multi-controller ordering, raw attack/damage/concentration, incapacitation before a
crossing, stale/foreign commands and malformed saved decisions. They remain unlinked
and unverified until the actual adapter exists. The spatial delta received an
independent read-only review with no concrete blocker. The review's additional
cumulative-cap boundary fixture passes alongside the three initial regressions.

## Shared integration draft after the verified geometry checkpoint

The ordinary attack checkpoint is integrated as `4c96a42` and its melee choice
contract as `336b2fe`. The movement cursor is now attached to the same boxed
resolution, with source work dispatch, material-choice validation, automatic removal
of canceled opportunities and preserved parent frames. This source checkpoint
requires the attack author's real `opportunity_options` and
`begin_opportunity_attack` adapters; it is not compiled or merge-ready on its own.
No placeholder adapter or second queue has been added.

Parent approved retained straight travel for source Charge: optional `straight {
start,end }` records only contiguous collinear forward actual movement, and optional
`budget.movement_origin` retains the original accepted mover command. A direction
change, reversal or external displacement cannot add old traveled distance. Source
attack admission snapshots the proof before clearing it. The geometry convention for
"toward" requires decreasing occupied-space distance and the forward continuation of
the moving footprint to cross the target's occupied space. This is an explicit
integer geometry interpretation, not a verbatim source algorithm. Additional source
tests cover turn-back, direction change, parallel/away targets and displacement.

Root must collect these CommandMeta origins in restore audit comparison:
`budget.movement_origin`, `resolution.movement.origin`,
`resolution.movement.initial_progress_origin`, `traversed[].cause`,
`decisions[].origin` and `opportunity.origin`. These fields remain optional/default
where older saves can omit the new feature.

The casting reviewer found and this draft fixes a self-perception shortcut: aware
self-location no longer implies sight. Blinded, Invisible, darkness and special senses
use the same perception path as another target; self/touch effects can still use
precise self-location. Unaware observers still learn no fresh location. The new sight
and straight-travel regressions await the next combined compile, so the earlier
30-test result applies only to `b498865`, not this expanded draft.

Accepted movement will interrupt a retained knockout Short Rest only when a segment
actually commits. Merely querying, offering or declining a reaction does not invent
activity. The attack checkpoint owns the matching accepted strenuous-action hook.

Root approved a coherent falling follow-up: the first movement checkpoint rejects an
unsupported planned endpoint before expenditure. Full movement acceptance additionally
requires actual queued falling damage/vitality, including death/incapacitation during
airborne interruption; no production path may leave a dead flyer suspended. This is
incomplete Gate 4 work, not a narrowed product requirement or silent exception.

Falling will retain a source-derived landing and use the same vitality/continuation
path: 1d6 Bludgeoning per 10 feet, capped at 20d6; Prone depends on actual damage.
Liquid landing retains the explicit Reaction/check choice. A dead flyer physically
lands without attempting to revive or damage a corpse through a living vitality API.
Roll role tags 9 (`FallDamage`) and 10 (`LiquidLandingCheck`) are reserved with the
casting author, whose new tags begin at 11. No falling execution is claimed yet.
