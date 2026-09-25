# Gate 4 movement and opportunity attacks

Status: active implementation; spatial/falling leaves verified, queued falling and
movement privacy correction outstanding. No production or Gate 4 completion claim.
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

Builds are globally serialized. The integrated source at `03cac39` passed all 41
spatial tests, including the nine falling leaf cases. After an equivalent iterator
lint correction, all nine falling tests and strict domain/rules library Clippy passed;
formatting and diff checks passed after ordering one export. The opportunity author's
reviewed `f69dee6` supplies separate 28 attack, 10 movement, 32 spatial and 20 turn
test evidence; that run did not include these falling leaves.
Next: consume casting's coherent shared-queue handoff, attach actual falling and
stopped-movement outcomes, then correct whole-path hidden-truth admission and verify
nested airborne interruptions and real application recovery. No Gate 4 acceptance claim.

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

Independent OA adapter review found that its copied crossing origin outlives the live
`movement.opportunity` field. The common `validate_opportunity` now checks provenance
and rejects a crossing cause predating its movement command, so both live decisions
and reconstructed accepted attack admissions receive those checks. Two additional
malformed-anchor cases cover foreign campaign and pre-movement origin. They remain
unrun pending the same serialized combined build; the attack adapter retains its
separate upper bound against the actual accepted reaction command.

Independent falling leaf review of `dd8730c` found one concrete source-query issue:
Frightened can reference a real source outside the current encounter. Landing checks
now test encounter participation before asking spatial sight, matching attack
admission. A regression checks off-map Normal versus visible-source Disadvantage.
The reviewer found no other blocker in the bounded geometry/source leaf; eight new
falling tests remain unrun and runtime attachment remains incomplete.

Parent approved a bounded optional `last_movement` receipt when falling is attached:
retain the original accepted Move command, actual resolving cause, endpoint, completed
steps and source-derived completion/interruption/fall reason. Both commands must join
restore audit comparison. Cancel only this movement's remaining segment/opportunity
frames on actual falling displacement; preserve unrelated attack, concentration,
casting and other consequence work. An OA causing a fall is required regression
evidence. The explicit stop outcome is not yet implemented by this leaf checkpoint.

Landing outcome now preserves the campaign's existing opted-in
`ability_test_natural_extremes` policy. Default RAW still compares the actual total
with DC15; the explicit house rule changes natural1/20 outcomes without rewriting
faces. A ninth leaf regression covers both policies at totals that distinguish them.
Parent assigned the matching shared tactical-save/failed-save/LR consistency fix to
casting integration; that broader change is outside this leaf's ownership.

## Required movement privacy correction before acceptance

Parent review identified a remaining oracle: `admit` currently evaluates the complete
path against hidden authoritative occupancy/solids/difficult terrain before any move
is accepted. Normalizing error text is insufficient because success/failure itself
can reveal distant truth. The current movement checkpoint is not gate-accepted.

At the stopped-movement/falling handoff, admission must validate only bounded proposal
syntax, grid steps, mover-owned source capabilities and known budget lower bounds.
Malformed or unavailable capabilities still reject atomically. A valid travel attempt
is accepted, then the existing true per-segment evaluator commits each legal prefix
and stops only when the actual next crossing meets obstruction or exhausts movement.
The durable stop receipt discloses the reached place/coarse interruption only, never
a hidden actor or obstacle ID. This intentionally replaces the draft whole-path
hidden-truth rejection contract without waiving collision or budget correctness.

Regression pairs must compare near/far/absent hidden walls and remote invisible
creatures: distant hidden placement cannot change admission; different stops must
correspond to actual attempted crossings and retained prefix position/cost. Reaction-
caused displacement invalidates the unfinished remainder with a derived receipt and
must preserve unrelated consequence frames. Root approved this correction; it follows
casting's shared queue handoff rather than competing with that writer.

## Falling leaf verification checkpoint

Reviewed opportunity adapter `f69dee6` is integrated by dependency-only merge `03cac39`.
The merge retained both writers' plans; no shared-queue behavior was changed here.
Domain/rules library mtimes were refreshed before compilation to avoid reuse of newer
artifacts from another worktree's shared target. On this source tree, all 41 spatial
unit tests passed, including the complete nine-case falling leaf suite.

Strict Clippy identified `filter_map(bool::then)` in the new solid-surface collector.
Replacing it with equivalent `filter` plus `map` preserved predicate, order and landing
data. The nine falling tests then passed again, and strict library Clippy for both
`dmd-domain` and `dmd-rules` passed. `cargo fmt --check` required one export-order fix;
the subsequent formatting and diff checks passed. No falling gameplay execution is
implied: retained falling work, raw-roll queue attachment, actual landing/vitality,
movement stop receipts and the privacy correction above remain required integration.

The falling source/geometry received independent read-only review with the off-map
Frightened query correction closed at `8c6f271`. A separate reviewer cleared the exact
natural-extremes policy delta `3bc7e36`. Compiler ownership is released to root for its
application integration checks; casting still owns the shared dispatcher/pump until
its coherent checkpoint is handed off.

## Prepared privacy and stop regression matrix

These are the next implementation scenarios, not passing tests or an attached new
validator. Reuse the existing serialized `tactical_movement` reducer fixture and its
exact `replay_tactical` comparison; do not create a parallel movement engine.

- Clone one valid initial state and one command ID. Keep mover `(10,10,0)` and the
  adjacent route through x=20,30,40,50 unchanged; place other participants off-path.
  Compare an unobservable solid at
  x=25, at x=45, and no solid. Each structurally valid attempt must be accepted.
  Actual segment collision should leave prefixes at x=10/cost0, x=30/cost20, and
  x=50/cost40 respectively. Use physically blocking but non-observable geometry so
  the actor cannot learn the remote obstacle through preview. Stops contain the
  original Move command, actual causing command, reached endpoint and coarse reason;
  no obstacle ID, name or unvisited position enters the public result.
- Move the same invisible, otherwise unchanged participant between remote path and
  off-path locations, with no hostile reaction relationship. Admission cannot reveal
  its placement. Only an attempted occupied crossing can stop the accepted prefix,
  without identifying the occupant or adding a remembered contact as a side effect.
- Compare identical own Speed/Dash budgets with hidden difficult terrain present or
  absent. Spend only committed segment cost; exhaustion retains the legal prefix.
  Include a source-legal route longer than the remaining budget so the old whole-route
  over-budget rejection fixture is replaced explicitly, not silently deleted.
- Retain atomic rejection for malformed/empty/over-capacity paths, non-adjacent or
  misaligned points, Teleport without a source, unavailable Fly/Burrow capability,
  and voluntary movement forbidden by the actor's own state. These do not need remote
  geometry to decide. A later interruption rechecks changed source state per segment.
- At a genuine pre-crossing OA pause, serialize/restore and accept the same reaction
  metadata on both images. Damage or source displacement that invalidates the next
  segment must preserve the already-paid reaction and completed movement prefix.
  Falling must land the actor before any invalid remainder can advance. Prune only
  this movement's future segment/opportunity work and preserve independent attack,
  concentration, casting and effect children in the single shared queue.
- Corrupt retained stop cause, original Move ID, completed-step count, endpoint,
  spent cost and unfinished-work association separately. Structural validation and
  post-anchor semantic replay must reject invented receipts; the application must
  collect both command origins and exercise a fresh SQLite export/restore continuation.

Implementation should separate proposal-shape/source-capability checks from the
existing true one-segment evaluator. Do not sanitize an entire encounter into a fake
map to obtain a successful preview. A visible stop is evidence of an attempted local
crossing; generic rejection text alone cannot repair a whole-path success oracle.

## Independent opportunity-grip projection regression

The parent application now filters reaction grips by current physical hands. The
standalone `crates/dmd-app/src/table_opportunity_tests.rs` contribution exercises
that private projection against `prepare_weapon_attack` in a Reaction window. It
uses runtime-created legal PCs and materialized starting equipment, then an explicitly
isolated read-only acquisition snapshot for a real Greatsword and Quarterstaff. It
does not claim an acquisition event or expand the starter shop.

Both required Two-Handed and Versatile two-handed use are checked with the weapon
in either hand plus a free hand, already in both hands, blocked by a held shield or
other weapon, and absent from both hands. Positive plans check the actual source
damage dice; the projection retains the sole witnessed mover target, preserves legal
one-handed Versatile use, and leaves the complete input state unchanged. No equipment
change permission is supplied to the Reaction planner.

This branch does not contain the parent's new `table_movement` module, so the file is
intentionally unattached here. Parent integration must include it as a `#[cfg(test)]`
child module using `#[path = "table_opportunity_tests.rs"] mod tests;` in
`table_movement.rs`. The parent subsequently reported the attached test passed all
12 grip comparisons, alongside 59 application tests and strict workspace Clippy.
That is parent integration evidence, not an application build from this branch.
Direct Rust formatting and `git diff --check` passed here.

## Active privacy and accepted-prefix slice

The parent released `tactical/movement.rs`, pure `tactical_movement.rs`, domain
movement/flow receipt types and movement tests. Shared casting/continuations/turns/
turn-validation/dispatch remain owned by the casting author. Dependency merge includes
shared cast contract `0aae8c0` and reviewed spell attack adapter `5402050`; this is not
a fresh build claim.

Implement proposal-shape and actor-local capability admission without reading remote
collision, occupancy, terrain or costs. Preserve the existing one-segment spatial
evaluator as the sole geometry/cost authority. A valid accepted attempt commits its
legal prefix and stores a coarse result when it completes or reaches an obstruction,
budget limit or source interruption. Do not expose a hidden blocker ID or unvisited
position. Source capability errors and malformed proposals remain atomic rejections.

Add optional/default `TacticalFlow.last_movement` containing original Move and actual
cause metadata, actor, global turn number, start/actual endpoint, requested/completed
step counts and movement expenditure before/after. Historical receipts are not compared
with a later turn's current position/budget. Structural checks reject incoherent anchor
images; exact semantic event replay remains authority for the full original path and
outcome. Parent owns application origin collection and fresh SQLite recovery coverage.
Queued falling and interruption-induced landing remain the subsequent shared-queue
slice; this work must not silently waive that outstanding Gate4 acceptance.

Implemented bounded admission/result validation and replaced the prior all-or-nothing
blocked/over-budget fixture with the explicit privacy/reached-prefix matrix above.

Independent review identified a required shared-turn interaction: a valid transit
segment may enter an ally's space before an unseen later obstruction or unexpected
cost stops movement there. The accepted prefix remains committed. SRD14 makes the
mover Prone if it ends its turn sharing another creature's space, unless it is Tiny
or larger than that creature. The movement regression verifies the involuntary stop
and ability to leave later without premature Prone. The casting author's verified
shared checkpoint `81c1a4e` was merged before testing and includes the actual
source-ordered end-turn consequence. The movement test also takes the alternative
of ending the turn in the occupied space and verifies Prone while retaining the
original reached-place receipt.

The same review found a relevant opportunity-cover error outside the segment
evaluator. A source `Prerequisite` for uncertain cover must stop departure at the
reached prefix, not roll back the accepted Move or silently cross an unresolved OA.
The attack author supplies a source-options query that filters actual departure
(`before <= reach < after`) before cover adjudication. Movement handles that bounded
error as `Stopped`; identity/corruption errors remain failures. Pruning a queued
unavailable opportunity retains the original MoveSegment under any already active
attack/concentration children, so its later recheck stops safely without orphaning
their retained movement admission. The new regression first moves inside reach,
then attempts departure through the ambiguous cover.

### Source checkpoint and verification

Dependency-only merges include shared casting checkpoint `81c1a4e`. The reviewed
departure helper `5435743` was cherry-picked as `3e066ed`; its new test now passes
with this branch's movement implementation. No new parallel queue was introduced.
The receipt validator runs before the shared turn validator's inactive early return:
Initiative/InitiativeTies cannot contain movement results, and Finished snapshots
still validate historical metadata, counts and expenditure.

Focused local verification on the final production source tree:

- 42 attack/casting/opportunity tests passed, including the newly integrated
  within-reach/uncertain-cover regression.
- 18 movement tests and 24 turn tests passed. Movement fixtures round-trip both
  state and events through JSON, re-run semantic replay, and compare complete states.
  Coverage includes hidden near/far/absent solids, unseen creature occupancy, hidden
  difficult terrain, actual-prefix stops, independent reaction ownership, knockout
  and concentration interruptions, source-invalid intent, and malformed receipts.
- Initial test compilation exposed an ambiguous test-module import and UUID-display
  typo; both were corrected. The next run passed 42 attack tests and 17 movement
  tests; the remaining fixture incorrectly supplied one initiative die for Invisible
  Advantage. Supplying two dice for either non-Normal mode fixed the fixture without
  changing source rules. The final 18 movement and 24 turn tests passed afterward.
- Strict domain/rules all-target Clippy passed with `-D warnings` on the final tree.
  Final formatting and whitespace checks also passed.

Durable local logs outside the repository are
`../tooling/logs/gate4-movement-privacy-tests-20260925-r2.log` (42 attack passes and
the diagnosed fixture failure), `...-r3.log` (18 movement and 24 turn passes), and
`../tooling/logs/gate4-movement-privacy-clippy-20260925.log`.

Independent source review by the attack author covered the complete privacy/prefix
implementation and the final inactive/occupied-space additions. No remaining finding
was reported; exact committed-head confirmation follows the final checks.

### Integration boundary and next action

Parent application integration must collect
`encounter.flow.last_movement.original` and `.cause` as command origins. Where the
journal exists, bind `original` to its actual accepted Move action, actor and requested
step count; metadata equality alone does not prove movement occurred. Replay from the
earliest supported anchor compares every reached snapshot and the final state. The
existing rejection of initial anchors containing live tactical flow remains required.
The optional/default, omitted-when-empty field is within the unreleased tactical flow
extension; no separate SQL projection or schema migration is introduced here.

After focused checks and commit, release the compiler to the parent for application
restore regressions and canonical integration verification. Then implement queued
falling using the already verified source/geometry leaf and this same shared resolution.
Final unsupported endpoints currently stop before departure; interruption-induced
landing and fall damage are still outstanding Gate 4 acceptance. They must preserve
unrelated accepted attack/casting/concentration work and both original movement and
actual causing commands. This checkpoint does not claim complete movement, tactical
combat, desktop acceptance, or the end of Gate 4.

### Historical falling draft (superseded by later verified checkpoints)

## Falling leaf draft and integration handoff

Shared movement source checkpoint `2e745b2` is handed to the attack author for the
real opportunity adapters. Reviewed privacy admission fix `cfbdff0` is incorporated
as `de6929b`. Those adapters, the new shared movement tests and source Charge proof
have not yet compiled together. Casting receives shared dispatcher/pump ownership
after that coherent checkpoint; falling is currently confined to new leaf modules
and additive exports/tests. Its shared queue attachment follows casting's checkpoint.

New source leaf primitives derive the SRD182 falling dice and DC15 liquid landing
check from actual skills, exhaustion, conditions and source NPC modifiers. No
zero-die request is fabricated for falls shorter than ten feet. A private outcome
proof binds the actual landing check to the actor and fall; half damage is an ordered
adjustment before target defenses. The eventual vitality adapter must use actual
`damage_taken`, including temporary-HP absorption, when deciding Prone. A dead body
lands physically without using a living damage reducer to alter corpse HP.

The authored geometry convention is first vertical contact with the battlefield
floor, movement-blocking solid obstacle, load-bearing terrain or liquid surface.
Any positive footprint overlap with a narrow ledge counts; a touching edge does not.
This is an explicit GM map convention, not quoted SRD collision math. Terrain marked
`supports_top` or `burrowable` supports contact: the latter is already physically
solid in swept movement, so falling must not tunnel through it. Abstract difficult,
climbable or obscuring regions alone do not acquire a physical top. At equal heights
solid contact wins over liquid, with canonical source-ID ordering for ties. A forced
landing may share a creature's space and creates no invented optional collision
damage. A creature already in liquid does not suffer another air impact.

Seven new leaf regressions cover small ledges, obstruction, water heights, solid
ties, involuntary occupancy, burrowable ground without a top flag, actual flight
loss/Hover/death, integer bounds/dice cap, Reaction capacity and source skills,
raw-result identity, liquid halving before resistance and temporary HP, and short
falls without invented dice. These tests and the earlier unverified straight/sight
regressions are still UNRUN. Rust formatting and diff checks alone are not execution
evidence. Next: consume the tested real OA adapter; serialize focused movement and
falling compilation with the parent; then attach falling to the shared queue after
casting's handoff and verify actual airborne interruption/resumption end to end.

Independent OA adapter review found that its copied crossing origin outlives the live
`movement.opportunity` field. The common `validate_opportunity` now checks provenance
and rejects a crossing cause predating its movement command, so both live decisions
and reconstructed accepted attack admissions receive those checks. Two additional
malformed-anchor cases cover foreign campaign and pre-movement origin. They remain
unrun pending the same serialized combined build; the attack adapter retains its
separate upper bound against the actual accepted reaction command.

Independent falling leaf review of `dd8730c` found one concrete source-query issue:
Frightened can reference a real source outside the current encounter. Landing checks
now test encounter participation before asking spatial sight, matching attack
admission. A regression checks off-map Normal versus visible-source Disadvantage.
The reviewer found no other blocker in the bounded geometry/source leaf; eight new
falling tests remain unrun and runtime attachment remains incomplete.

Parent approved a bounded optional `last_movement` receipt when falling is attached:
retain the original accepted Move command, actual resolving cause, endpoint, completed
steps and source-derived completion/interruption/fall reason. Both commands must join
restore audit comparison. Cancel only this movement's remaining segment/opportunity
frames on actual falling displacement; preserve unrelated attack, concentration,
casting and other consequence work. An OA causing a fall is required regression
evidence. The explicit stop outcome is not yet implemented by this leaf checkpoint.

Landing outcome now preserves the campaign's existing opted-in
`ability_test_natural_extremes` policy. Default RAW still compares the actual total
with DC15; the explicit house rule changes natural1/20 outcomes without rewriting
faces. A ninth leaf regression covers both policies at totals that distinguish them.
Parent assigned the matching shared tactical-save/failed-save/LR consistency fix to
casting integration; that broader change is outside this leaf's ownership.

## Required movement privacy correction before acceptance

Parent review identified a remaining oracle: `admit` currently evaluates the complete
path against hidden authoritative occupancy/solids/difficult terrain before any move
is accepted. Normalizing error text is insufficient because success/failure itself
can reveal distant truth. The current movement checkpoint is not gate-accepted.

At the stopped-movement/falling handoff, admission must validate only bounded proposal
syntax, grid steps, mover-owned source capabilities and known budget lower bounds.
Malformed or unavailable capabilities still reject atomically. A valid travel attempt
is accepted, then the existing true per-segment evaluator commits each legal prefix
and stops only when the actual next crossing meets obstruction or exhausts movement.
The durable stop receipt discloses the reached place/coarse interruption only, never
a hidden actor or obstacle ID. This intentionally replaces the draft whole-path
hidden-truth rejection contract without waiving collision or budget correctness.

Regression pairs must compare near/far/absent hidden walls and remote invisible
creatures: distant hidden placement cannot change admission; different stops must
correspond to actual attempted crossings and retained prefix position/cost. Reaction-
caused displacement invalidates the unfinished remainder with a derived receipt and
must preserve unrelated consequence frames. Root approved this correction; it follows
casting's shared queue handoff rather than competing with that writer.

## Falling leaf verification checkpoint

Reviewed opportunity adapter `f69dee6` is integrated by dependency-only merge `03cac39`.
The merge retained both writers' plans; no shared-queue behavior was changed here.
Domain/rules library mtimes were refreshed before compilation to avoid reuse of newer
artifacts from another worktree's shared target. On this source tree, all 41 spatial
unit tests passed, including the complete nine-case falling leaf suite.

Strict Clippy identified `filter_map(bool::then)` in the new solid-surface collector.
Replacing it with equivalent `filter` plus `map` preserved predicate, order and landing
data. The nine falling tests then passed again, and strict library Clippy for both
`dmd-domain` and `dmd-rules` passed. `cargo fmt --check` required one export-order fix;
the subsequent formatting and diff checks passed. No falling gameplay execution is
implied: retained falling work, raw-roll queue attachment, actual landing/vitality,
movement stop receipts and the privacy correction above remain required integration.

The falling source/geometry received independent read-only review with the off-map
Frightened query correction closed at `8c6f271`. A separate reviewer cleared the exact
natural-extremes policy delta `3bc7e36`. Compiler ownership is released to root for its
application integration checks; casting still owns the shared dispatcher/pump until
its coherent checkpoint is handed off.

## Active queued-falling integration

The verified privacy checkpoint is `4a6f879`. Shared casting/continuation/turn/work
ownership was released by the casting author after `81c1a4e`; the attack author
retains attack-source modules and a coordinated CreatureAttack enum/dispatch addition.
The parent owns the compiler during application/extraction verification. Do not run
another Rust build here until the slot is explicitly granted.

Extend the same `TacticalResolution` with bounded optional retained fall records and
typed frame work, using the existing pure `tactical_falling` and `spatial::fall_destination`
authorities. FallDamage uses deterministic role tag 9 and LiquidLandingCheck tag 10;
existing cast tags 11/12 remain unchanged. Retain the original causing command and
actual source-derived geometry, never client-provided collision or damage claims.

Required source/queue behavior:

- A legally committed last movement segment that lacks support schedules its actual
  fall. Loss of non-Hover flight through Prone, Incapacitated, Speed 0 or death must
  schedule landing before later movement or independent work resumes. Hover still
  fails on death. No accepted state may silently complete with an unsupported dead
  flyer suspended.
- A conscious eligible creature falling into liquid may explicitly spend its own
  Reaction and select Athletics or Acrobatics, or decline. Persist that exact decision
  and raw request before pausing. Use source DC15 and the shared opt-in natural-extreme
  check policy; this is a check, so Legendary Resistance/voluntary failed save are not
  permissions to change it. Inspiration retains original/replacement raw faces.
- Derive actual capped d6 damage from vertical half-foot distance; apply successful
  liquid reduction before source defenses. Land the actor before vitality/drop-held
  consequences so ground-item positions match physical truth. Dead bodies land without
  fabricated living damage requests; immunity and damage below one full ten-foot band
  do not create fake dice. Source Prone depends on actual damage taken and immunity.
- Preserve the single authoritative raw-roll/history and vitality paths. Nested fall
  damage, knockout/rest interruption, death/stabilization and concentration effects
  finish before interrupted movement can resume. Prune only affected movement work;
  retain unrelated accepted attack, casting and concentration frames.
- A movement result may record `Fell`, carrying original Move plus the actual landing
  cause, reached endpoint, original completed-step counts and paid movement cost. Fall
  displacement consumes no additional voluntary movement. Historical receipt validation
  and semantic replay must distinguish this source outcome from an invented teleport.

Acceptance evidence must include ordinary ledge landing, water choice/decline and
exact serialized restart, half-before-defense, short fall/no fabricated dice, solid and
small supporting ledges, burrowable physical support, an already dead Hover body,
reaction-caused airborne knockout/death before path continuation, correct dropped-item
ground height, independent source siblings, foreign/stale/raw identity rejection, and
forged fall geometry/source/work/Reaction receipts. Parent adds real SQLite export/
restore and player-safe owned-choice projection before application acceptance.

Next: finalize the bounded retained-work contract, implement shared source admission/
request/landing validation and focused integration tests, then request the serialized
compiler slot and independent review. This plan is checked in before source edits.

Initial domain-only contract adds `TacticalResolution.falls`, the three fall work
variants and deterministic roles described above, `TacticalFallCause` (committed
MovementEnd or source FlightLost), Queued/Complete stages, and the coarse Fell movement
result. Complete retains accepted landing-check evidence, an optional actual damage
roll key and its resolving command. This contract checkpoint has formatting/whitespace
checks only: exhaustive rules/application matches and initializers deliberately follow
with the coherent implementation. It is not a compiling or runnable release checkpoint.
