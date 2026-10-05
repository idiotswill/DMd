# Proposed corrections to frozen public Grapple 71673d7

Status: ROOT APPROVED; SOLE SOURCE ALLOCATED; implementation is uncompiled/unrun.
2026-10-05; author ground_next_oct5. No repository modification, formatter,
compiler, Cargo/npm, database/native execution, dependency intake or publication.

Exact inspected source: `gate4-grapple-public-completion`, clean branch
`codex/gate4-grapple-public-completion`, HEAD
`71673d70b2fb2b27da48a7e04ba8ee88fe554291`, tree
`2da352f146a5e2847ec2541c0febc1b0bddcf5b4`. Root retains source ownership.

Read both complete independent reviews:

- `grapple-public-716-mechanics-independent-review-2026-10-05.md`: five
  actionable semantic findings, not acceptance approval.
- `grapple-71673d7-owned-bridge-independent-review-2026-10-05.md`: concrete
  lint findings; no further authority defect found within its stated scope.

Also followed the named producers, occurrence source bindings, shared release
and aftermath paths, original save observer/delta checks, current and retained
hand readers, movement options/form, and exact transport/reducer lint sites.
This is static source reasoning, not a reproduced runtime failure on 716.

Root reports corrected private418 stopped after fmt/strict Clippy/opportunity1/
owner10 passed: attack15 had13 passes and2 failures (concentration request absent,
Knockout stable expectation); remaining56 were unrun. That head is not a cleared
receiving dependency. Root owns its diagnosis and eventual normal receipt of a
verified correction. No current or older result is evidence for public716.
Root/core subsequently diagnosed both failures as new private test assumptions:
paid Hold Person correctly has no effect on the pinned Fey Goblin, and Knock Out
produces HP1, default death state and owned Unconscious rather than stable=true.
Core owns the plan-first correction to a genuine second-Human/four-actor case
and the owned Knock Out assertions. A later reviewed normal receipt may change
only those two inherited new controls and the previously reviewed 27-line
withdrawal correction; all unrelated inherited controls remain protected.

## Scope and order

On approval, record this addendum in the repository execution plan before code.
Implement one coherent correction of the five findings and lint, with genuine
application tests and no new private checkpoint as a completion boundary.
Preserve the single typed reducer, original envelopes, fixed boxed candidate,
original-anchor replay, all snapshot equality, staged export verification,
audience history, exact retries and raw/legacy refusals.

Suggested sequence: mechanical lint; occurrence source identity; Finish boundary;
owned voluntary-decision continuity; separate opportunity-window hand read;
explicit self-only action/UI; integrated new test bodies and static audit.
No acceptance/publication or group3 dependency receipt is included. Root decides
whether the separately verified private correction is normally received before
authoring or before public exact-head verification. Never cherry-pick its tests
or silently attribute its result to this descendant.

## 1. Keep paid cast identity separate from advancing command identity

Affected files: rules `tactical/grapple/reads/modern.rs`, supporting private
source-read helper if needed in `tactical/attacks/spell.rs`, domain
`tactical_grapples/validation.rs`, new application test target.

Keep `TacticalAttack.origin`, `AttackAdmission.attack` and cut `issued_by` as the
actual advancing command. Keep `TacticalRollKey.origin` and its UUID derivation
as the actual paid casting origin. Neither identity is rewritten to make an
equality pass. Non-spell families retain their current attack-origin rule.

Introduce a private pure occurrence resolver in modern reads, conceptually:

```rust
struct AttackRawSource {
    origin: CommandId,
    actor: EntityId,
    target: EntityId,
}
fn raw_source_for_admission(
    state: &CampaignState,
    admission: &GrappleReadCut,
) -> Result<AttackRawSource, RulesError>;
```

The resolver requires the exact unique AttackRoll node of that admission and
walks its causal ancestry, rejecting absent, cyclic, future or ambiguous nodes.
For a spell, the enclosing actual `SpellProgram { cast, at }` selects exactly
one retained `TacticalCasting` by `cast.plan.occurrence`. Resolve its original
plan, actor, selected target and source through `retained_spell_binding` and
`spell_attack_occurrence`; check the subject and actual raw request's roller.
Do not look up every old ray through the one current `resolution.attack`.

Important historical detail: `spell_attack_occurrence` rejects an already
completed occurrence. Earlier ray cuts survive later rays. For that historical
source calculation, first validate the original retained record. Make a local
read-only clone with only the requested `at` removed from `completed`, then
reconstruct the binding and occurrence using the unchanged existing helpers.
Do not alter cast phase, targets, plan, paid origin, source or other completed
entries. The clone is never installed in state, exposed as execution authority,
or used to apply a spell; it derives immutable source identity only. Existing
retained spell validators already use local completion-free source images for
historical proof reconstruction. Original replay still proves when this ray
was actually produced and completed.

Require RequestIssue.source_attack to be the exact admission key reached from
its real AttackRoll ancestry, its grip set to match, role/work occurrence to
match, and its raw to occur exactly once. Replace both incorrect origin checks:
the issued-request enumeration and RequestIssue validation. Enumerate current
resolution attack raws by causal admission and derived raw source rather than
silently dropping raws whose paid origin differs from the advancing command.

For the activated domain validator, derive the corresponding structural origin
from exact AttackRoll -> SpellProgram ancestry and the unique retained cast's
`plan.origin.id`; retain target-index and work ancestry checks. Domain cannot
validate pinned source semantics, so rules performs the stronger reconstruction.
Keep legacy `ReadShape::Legacy` and its direct same-origin rule unchanged.

Coverage: strengthen the existing unrun three-ray case to assert the first
explicit SelectWork admission and later advancing commands differ from the paid
cast where appropriate; all three raw origins remain the actual cast identity,
all raw IDs/work/admission keys stay distinct, and earlier completed cuts validate.
Retain cold file/portable execution between each occurrence. Add hostile sibling
admission substitution, advancing-command raw origin substitution, wrong retained
cast/target and missing admission cases, comparing every destination typed cell.
Do not modify old spell tests or raw UUID/source fixtures.

## 2. A live grip must prevent Finished and replacement, not session pause

Affected files: `tactical/release.rs`, narrowly shared full-state Grapple
validation if needed, new application target. No automatic release or grip clear.

Add a shared read-only `require_no_live_grips(state)` boundary used by release
readiness/dependency validation and Finished-history validation before timing is
removed. Reject `rules.tactical_grapples` with any active grip. The Finish command,
Host readiness UI and replacement/Finished validation must reach the same guard.
Ensure an activated full-state image cannot retain live grips with phase Finished.

Keep `aftermath::require_quiescent` and `require_aftermath_session_boundary`
independent of this Finish prohibition. They intentionally permit quiescent live
relations on the retained initiative cadence. Existing
`table::reducer::require_aftermath_attendance` already requires all retained PC
owners and Player source owners; preserve it without Host substitution.

Add a real accepted sequence: PC/Goblin grip, actual turn progression to clear
the previous attack window, ConcludeHostilities, failed Finish with every row
unchanged, accepted EndSession with the live grip and timing preserved, rejected
resume missing a required owner, accepted new session with real attendance,
explicit owner release, accepted Finish, then genuine new battlefield/encounter
and physical initiative. Cold-reopen/portable continuation at pause, release,
Finished and replacement. Assert unchanged old roll/cancel/decision history and
completion provenance. A failed Finish while another blocker is present would
not isolate this defect; the test must establish the otherwise-ready state.

## 3. Authenticate a retained voluntary Grapple save through the owned chain

Affected files: `tactical/grapple/execution.rs`, `tactical/turn_validation.rs`,
new application target. Do not skip all voluntary decisions after a marker.

Add the narrow private method mirroring `validate_recorded_grapple`:

```rust
fn validate_retained_grapple_decision(
    &self,
    decision: &TacticalSaveDecision,
) -> Result<(), RulesError>;
```

It accepts only the new GrappleSave role under the activated owned route. For a
ClosedImage read, require that exact complete decision in that exact certified
flow. For a fixed candidate read, require byte/field equality with either the
certified predecessor's retained decision or this command's actual observed
`ProducedEvidence.decisions` suffix. Reject raw/ordinary reads, foreign role,
unobserved new records and modified inherited records. The existing observer
already ties newly produced decisions to the actual pending work and command;
the delta validator already requires exact unchanged prefix and actual suffix.

In shared turn validation, use that owned evidence for only an activated
GrappleSave voluntary decision, retaining all chronology/provenance, uniqueness,
no-fake-roll and exact cancellation checks. Every other role/private legacy path
retains current behavior. New VoluntarilyFailSave input still goes through its
unchanged current-owner authorization before observation; controller transfer
cannot authorize fresh input from the former owner.

New genuine case: Host-controlled Goblin grapples a separately created
Player-controlled Goblin target; that Player chooses and voluntarily fails the
save; Host finishes the actual equipment choice; then Host transfers the target
controller. Assert accepted exact retry of the original Player voluntary request
after transfer and cold/portable replay. On the target's real next turn, reject a
fresh former-owner action without writes, then accept that same otherwise-legal
action from its current Host owner. Keep the existing grappler-transfer test.
Hostile imports change decision owner/key/origin or cancellation while preserving
matching current/latest images; complete original replay and row equality refuse.

## 4. Separate the opportunity menu's hand read from the attack's components

Affected files: domain `tactical_grapples.rs` and its validator; rules
`tactical/grapple/reads.rs`, `reads/modern.rs`, `tactical_hands.rs`,
`tactical/attacks/opportunity.rs`; origin collector only if a new origin field
requires it; additive app tests. Keep AttackRead's physical=false semantics.

Add a dedicated reader variant (all fields Copy, keeping GrappleCutKey Copy):

```rust
OpportunityWindow {
    attack: CommandId,
    window: CommandId,
    reactor: EntityId,
    mover: EntityId,
    step: u16,
}
```

Use the existing `GrappleReadCut`/retained proof vectors; do not add a required
field to an old struct or rewrite protected fixture literals. The cut has the
actual selected attack's AttackRoll work key, selected-command issued_by, no
source_attack inheritance, and the complete sorted outgoing grip set of its
reactor, even for an unarmed/intrinsic attack. Empty is a genuine no-reservation
window read, permitted only by activated shape; the legacy validator rejects the
new reader variant. Existing AttackAdmission keeps its original complete attack
condition/physical relevance set. RequestIssue inherits only that attack cut.

At the actual OA producer, first validate the live offered window exactly using
current owned hands, before spending Reaction. After installing the actual
retained Opportunity attack and registering its AttackRoll node, capture both
cuts together, retain their union of immutable proofs, then validate once. The
window cut binds original window command, reactor/mover/step and its actual
MovementOpportunity ancestry; geometry/options remain the exact existing retained
`TacticalOpportunityWindow`. A free release before selection may legitimately
refresh that offered window; the cut seals the one actually selected. A release
after raw issuance cannot refresh or reconstruct it from today's hands.

Introduce a private nonserializable `OpportunityWindowRead<'a>` obtained only
from a guarded/ClosedImage ReadContext and the exact attached Opportunity attack.
It binds that attack, the exact retained window and unique window cut; it exposes
only state/actor/retained outgoing proofs to
`EffectiveHands::opportunity_window(&OpportunityWindowRead)`. No caller-supplied
reservation mask or arbitrary window constructor is added. This hand reader
always reconstructs the reactor's original reservations and source anatomy.
`validate_admission` uses it to rebuild the complete retained menu, retaining
exact equality. The attack's own hit/damage reader remains component-specific.

For cuts whose old attack is no longer attached, validate their trace ancestry,
identity, chronology, proof closure and causal endings; full original replay
proves the exact old menu/producer. Do not borrow the later attached attack's
identity. Retained cuts/proofs survive release and existing last-consumer rules.
Add malformed window identity, sibling OA/work, omitted extra reservation and
fake hand/source negatives; no semantic fallback to current hands is allowed.

Current-source positive coverage can add a real PC with an outgoing grip selecting
Unarmed OA against a third genuine actor, then releasing its held target during
the issued attack and retaining cut/raw/window/one Reaction. This proves capture
and lifetime, but does not reproduce the missing two-handed-menu exclusion.
That decisive regression remains REQUIRED after root normally receives genuine
Physical creation: purchase Greatsword through its new accepted action, hold it
in one hand and a live grip in the other, offer ordinary-reach OA with Unarmed
but without Greatsword, choose Unarmed, release during the pending raw, finish
without menu drift, duplicate Reaction, equipment pickup or raw rewriting.
No fabricated Item/definition/mastery/profile may substitute for that test.
Glaive long-reach refresh and genuine applicable intrinsic-source cases remain
the reviewed whole-dependency acceptance obligations, not current passes.

## 5. Explicit self-only movement, including in-range paths

Affected files: rules `tactical.rs`, `tactical/movement.rs`, Grapple guard/capture
and movement validation; app movement projection/DTO and tactical read caller;
desktop tactical-api, MovementForm and new tests. Original Move payload unchanged.

Add `TacticalAction::MoveSelfOnly { path: Vec<TacticalMoveStep> }`. Only validated
activation plus owned table execution can admit it. Extend raw/stateless/old-flow
guards explicitly for the new action; do not let an ordinary raw resolver acquire
permission because there happens to be no current grip. New action requires a
nonempty actual outgoing grip set and the current owned actor. Incoming Grappled
speed constraints remain unchanged.

Use one shared movement implementation with a private typed intent enum
`Ordinary`/`SelfOnly`, not a serialized boolean permission. Ordinary Move with an
outgoing live grip rejects before mutation and tells the user to choose self-only
movement; Ordinary with none preserves prior behavior. SelfOnly captures the
existing `GrappleSelfOnlyAdmission` from the actual declaration, retains exact
grips/path/prefix, and never moves the held target. Original replay sees the new
action and verifies that exact capture. Even a path staying in range requires
the explicit action. Releasing the last grip while suspended does not erase the
chosen movement admission or switch its meaning mid-route.

Add omitted-by-default `self_only_required: bool` to TableMovementOptions,
derived solely for the audience's own active actor under the owned modern route.
False is skipped in Rust serialization, preserving all old projection bytes and
digests; the TS property is optional. It exposes no grip ID/count/other actor.
MovementForm keeps original behavior when absent; when true, show the deliberate
button `Move only my creature along this route` and explain that held creatures
stay put and range may end the grip. That explicit submission sends MoveSelfOnly.
Do not silently translate a saved ordinary Move retry to the new action.

This correction makes the self-only choice explicit; it does not complete or
remove ordinary dragging/carrying. Those mechanical and application obligations
remain in the approved active Gate4 plan and must receive genuine producer,
ownership, movement-cost, replay, persistence and UI coverage before gate
acceptance. The narrower correction must not be presented as a scope waiver.

Inventory permitted new-test correction: the UNRUN public716 test
`self_only_move_retains_grip_until_actual_range_crossing_and_never_moves_the_target`
currently sends ordinary Move. Change only that newly authored test's choice to
MoveSelfOnly, preserve all existing assertions and add the preceding ordinary
Move rejection with complete-store equality. Add real in-range movement and
suspended OA/last-release continuation cases, stale/foreign/old-protocol/raw
negatives and exact uncertain-response retry. Add separate frontend tests for
explicit button/action and old absent-field behavior without editing original
Movement.test.ts bodies or existing received captures.

## 6. Source-only lint fixes, with original behavior retained

The complete core memo identifies `table_transport_runtime.rs`, not
`table_transport.rs`, for the transport needless-borrow sites:

- `table/reducer.rs::apply_operation`: next is already `&mut CampaignState`.
  Pass next to reference-taking callees instead of &next/&mut next; remove the
  binding's mut only after all such uses are corrected. Preserve owned-value
  borrowing in the separate ordinary wrapper at line53.
- `kernel/engine.rs::apply_table_with_context`: likewise fix apply and sync_deaths
  calls at185/189, preserving the outer resolve clone's borrow at15.
- `table_transport_runtime.rs`: read.state() already returns &CampaignState.
  Correct the four source options639-642, owns_source732, request_meta1196,
  derive_intent1197 and observation visibility1291; do not change receivers.
- `table_presentation_history.rs544-552`: combine the unchanged nested predicates
  guarding non-table presentation history into one if. No lint allow/suppression.

These are static lint findings, not observed public-head Clippy results. The
eventual exact-head strict all-target invocation must establish actual status.

## Preservation, required review and handback

Do not alter any of the original894 Rust test bodies,28 inline cfg(test) modules,
154 protected files or8 content files during this correction. Add new tests or
amend only the named newly authored UNRUN716 cases. Keep initial716 and both
independent finding memos immutable as evidence. Any later verified parent
receipt must list its own reviewed test migrations; do not falsely report that
its expected inherited differences are authored control preservation.

Re-run static exact-body/content/input-identity audits after an explicitly
allocated formatter window. Freeze one coherent commit with complete diff/test
inventory for separate review. Root must examine all five corrections, original
guards and genuine new test bodies before scheduling compiler/runtime work.
Maintain full original-anchor/snapshot/audit/session/presentation verification;
none of the new typed records supplies independent execution authority.

Root then schedules fresh exact-head strict lint and targeted public acceptance,
the complete original receiving suites, frontend check/test/build and canonical
verification. Keep every previous failure and unrun result. Full group3 normal
Physical/Ground/Ogre/Mage receipt, decisive Greatsword/Glaive/Graze, source/LR/
Inspiration/concentration intersections and packaged native physical-dice/reopen
evidence remain required. This addendum does not close Grapple or Gate4.

Allocation update: root read the complete plan and both reviews and approved the
five bounded corrections plus lint. Sole source ownership is transferred from
clean716 to this writer. Commit this plan first, then implement and hand back
coherent unformatted source for full review. No formatter, runtime, dependency
receipt, publication or acceptance is allocated. Private418 is not received.
