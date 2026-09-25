# Gate 4 - Primary attack execution

Status: Active; implementation and production integration pending.

## Objective and ownership

Branch `codex/gate4-primary-attacks`, worktree `gate4-attacks`, based on root
`5e410c6` plus reviewed turn bridge `7f6db28` (local cherry-pick `0fa40d4`).
Connect canonical physical-weapon and source-creature attacks to the one authoritative
TacticalResolution, raw attack/damage dice, vitality, and concentration continuations.
This slice owns new domain/rules attack modules, tactical dispatch/work integration and
focused tests. Root owns application/table/UI, NPC equipment construction and live AC.
No source NPC equipment WIP from the root checkout is included in this branch.

## Contracts and scope

Product clauses: source-faithful rules authority, physical player dice, player/actor
identity separation, durable invalid-input atomicity, and complete production encounters.
Gate 4 tactical encounters and ADR025/026 govern spatial/state and continuation boundaries.
Pinned SRD5.2.1 pp15-18,89-90,176-191,255-257 and represented stat-block source pages.

- Accept only typed actor choices, physical ItemIds, source feature IDs and raw faces.
  Source definitions, geometry, effects, equipment and grants derive all numeric rules.
- Reuse existing weapon preparation and accepted weapon history for chosen ability/grip,
  Loading, Light/Nick/Cleave, ammunition and equip-before/after behavior.
- Retain one typed attack intent/resource receipt with exact original command provenance;
  restored pending requests must be reconstructed from canonical source and retained
  inputs, never arbitrary after-state images or player-authored modifiers.
- All attack, damage, knockout/mastery decisions and concentration follow-ups use the
  existing TacticalResolution. Creating action work must not re-observe a turn boundary.
  After-End source actions nest in the existing continuation and preserve its remaining
  source windows; they must not advance the turn before the attack finishes.
- Source feature activation pays once. Ordinary loot weapons use ordinary equipment
  proficiency; stat-block attacks use their explicit bonuses/formulas and source riders.
- No network, random generation, direct persistence, or player-control automation here.

## Acceptance and verification

1. Complete accepted ordinary primary attacks through raw hit/miss/critical and damage,
   fixed damage, temporary HP/death, controller-owned knockout and concentration work.
2. Preserve physical identities, quantity/custody, equip timing and accepted history;
   no duplicate ammo spending or action/refund on continuation, replay or restart.
3. Derived range/cover/perception/conditions, natural1/20, underwater automatic misses,
   and explicit source attack bonuses remain authoritative.
4. Unsupported source riders or mastery consequences reject before action/attack/ammo
   costs. An accepted attack never silently discards a mandatory or chosen consequence.
5. Focused tests serialize/replay each accepted boundary, corrupt retained requests,
   change issuer/actor/head, and verify unchanged originals after invalid operations.
6. Run focused Rust tests and strict Clippy in the globally serialized build slot.
   Root owns canonical full verification and real application/native acceptance.

## Planned slices and remaining Gate 4 work

- Define retained attack work and request reconstruction; connect ordinary weapon path.
- Connect source feature receipt path with canonical damage and supported hit clauses.
- Add explicit post-hit decisions and reuse existing damage/concentration source work.
- Validate authority, source/resource provenance and each restored stage; regression tests.
- Independent review, focused verification and coherent checkpoint for root integration.

Sap/Vex/Slow require typed non-condition effect modifiers, not magic labels or a second
TacticalFlow effect authority. Unsupported modifiers, Charge movement predicates,
reaction triggers/interrupts, improvised/environmental attacks and broader source feature
programs remain active Gate4 obligations until connected. They are not moved to Gate6.
The first closed pipeline will explicitly reject cases it cannot finish before spending
resources. This source slice alone does not claim an end-user feature or completed gate.

## Evidence and next action

Existing weapon/equipment/history/mastery, spatial cover/perception, tactical conditions,
turn bridge, vitality and grouped concentration APIs inspected. Turn bridge exact-head
read-only signoff complete; author reports20turn+14creature+31damage tests and strictClippy.
The first ordinary-weapon draft now retains one source-reconstructed attack in the shared
resolution. Action/attack/bonus budgets and ammo are reserved once; after-attack equipment
and history settle atomically with damage before concentration/effect children execute.
That ordering prevents later unconsciousness from being followed by an accidental re-equip.
Thrown weapons retain their physical ItemId and move to the target's recorded position
in the encounter scene. This is a coarse landing convention under the encounter's
explicit geometry adjudication, not an SRD rule prescribing an exact landing square.
Precise trajectories, interception and hazardous landing surfaces require the later
geometry/adjudication continuation; no item destruction or recovery is inferred here.
Knockout and Graze pause before consequences and retain controller-owned decisions.
All persisted stages rederive range/cover/perception/hit facts; a review-found malformed
Graze anchor could otherwise reinterpret a successful attack as a miss, now covered by
a dedicated corruption regression. Every test helper serializes and replays each accepted
command, then runs domain, rules and tactical validation.

The first checkpoint supports ordinary weapons, Light/Nick and Graze. Other masteries,
source creature attack features, guessed-location attacks, partial-submersion adjudication,
mounted weapons, Savage Attacker choice and attack-trigger reactions remain explicit next
Gate4 work. They are not represented as complete by this checkpoint. Unsupported derived
masteries reject before any expenditure; no second effect authority is introduced.

The first integration run exposed a real knockout/rest conflict: a source-mandated Short
Rest can begin during combat, whereas the legacy kernel prohibited all such rests. The
new optional `TacticalRecovery.knockout_rest` authorization retains the original knockout
and actual rest-start command plus its start instant. It must match the existing sole
`RestProgress`; it is not another resource or rest clock. Healing/first aid can remove
Unconscious without cancelling that rest. Damage, initiative, or accepted strenuous actions
clear the authorization and rest; waking, rejected proposals and EndTurn do not. Finishing
early rejects even after waking. Legacy rests without this source evidence keep their prior
restrictions. Application restore audit must authenticate both nested command origins.

Focused verification: 19 attack scenarios and 20 existing turn scenarios pass, including
raw/replay/corruption boundaries and a woken rest interrupted only by accepted activity.
All 33 pure vitality tests pass, including new recovery provenance, restart, waking and
early-completion cases. The two incorrect initial expectations were corrected against source:
fixed Blowgun damage remains 1 even on a critical hit, and an adjacent thrown attack under
Disadvantage supplies two d20 faces. The rest-validation defect was fixed in production code.
Strict domain/rules all-target Clippy, formatting and diff whitespace checks pass.
Independent movement/turn review closed the forged Graze anchor, automatic-miss pause,
rest activity and awake early-completion findings; final committed-head review follows.
No full-workspace,
SQLite, desktop or gate-completion claim is made for this isolated checkpoint.

Next: obtain exact source review, then hand the coherent ordinary attack
checkpoint to root. Root's independent Box<TacticalResolution> change needs Box::new at
the new attack allocation. Movement then owns shared dispatcher/pump/work additions while
this slice owns new source/OA adapters under attacks and tactical_attacks. Consume verified
NPC equipment/live-AC and movement contracts before compiling those next adapters; no
competing queues, fake item identities or discarded mandatory source riders are permitted.

## Next adapter contract

Ordinary checkpoint `334d82b` received exact independent source review with no remaining
bounded findings. Root integrated it and supplied `624c642`; this branch's dependency
merge `d8f5e3f` has an identical tree, including the heap-backed cursor and NPC equipment.
The movement author now owns shared dispatch/work/pump edits. The attack author retains
attack modules and source work types; adapters consume that movement checkpoint next.

`TacticalMeleeChoice` carries a physical `WeaponUseChoice`, damage-only Unarmed Strike
ability choice, or a canonical creature feature plus its optional real Gear ItemId.
Live movement determines the reactor, mover, crossing and source options. The source
query must sort options canonically, use actual held implements and source reach, and
include real unarmed capability. Reaction acceptance must validate that live window,
spend once, retain the movement decision and resume its existing lower frames after
attack/concentration/effect children. No public fake adapter is provided by this contract.

Next retained attack refactor separates common actor/target/melee-or-ranged semantics
from Weapon, Unarmed, CreatureFeature and sealed Spell source data. Physical delivery,
equipment and ammo stay weapon-specific. Source profiles, current raw inputs and
accepted invocation receipts rederive bonuses/damage/riders. Spell occurrence proof
retains cast/node/target ordinals; no invented physical item stands in for a spell.
Source Charge predicates, nested counterattacks and additional mastery decisions remain
explicit same-gate work until their full retained continuations execute and are tested.

## Target-knowledge follow-up

Root's integration review found a privacy ordering defect after the ordinary checkpoint:
weapon range planning ran before the unlocated-target rejection. A retained hidden entity
ID could therefore distinguish in-range and out-of-range positions by rejection text.
The isolated fix checks current actor knowledge before source/geometry planning and gives
one non-disclosing error for missing and unlocated targets. A regression submits the same
hidden ID near and far, then an absent ID, and requires identical errors without action or
ammo changes. Formatting/diff checks pass; this follow-up's test and Clippy are pending in
root's queued integration batch, not represented by the prior 19-test evidence. New source,
opportunity and spell admission must preserve the same knowledge-before-geometry boundary.

## Opportunity/source continuation checkpoint in progress

The next bounded checkpoint consumes the movement author's shared cursor contracts and
adds one accepted reaction above that cursor. `TacticalAttack` now retains common target
and melee/ranged semantics separately from a typed physical Weapon, Unarmed, or pinned
CreatureFeature source. Only physical sources carry original equipment and ammunition.
An Opportunity admission retains the original crossing and must match its exact accepted
movement decision, original metadata, current reach/knowledge, and spent reaction. The
parent movement command and lower frames survive every attack/damage/material-choice
pause; movement resumes only after vitality children. Ordinary attacks clear their own
continuous movement proof, while reactions preserve the mover's proof.

This checkpoint implements the damage form of Strength Unarmed Strike (SRD190), ordinary
held melee weapon opportunities (SRD15), and canonical creature melee opportunities with
printed attack bonuses/damage and represented on-hit clauses: size-gated Prone and Chimera
Advantage Bite. A reaction cannot borrow the mover's Charge distance. Source Gear requires
the actual held ItemId; intrinsic features have no item. Creature feature use within an
own-turn routine, its straight-approach Charge, spells/rays, legendary attacks, grapple/shove,
and the remaining weapon masteries still require subsequent same-Gate4 execution slices.
They are not claimed by this checkpoint or deferred to catalog Gate6.

Read-only review added Charmed/Total-cover eligibility filtering before an opportunity is
offered and strict nested crossing-origin validation. Partial cover that needs geometry
adjudication fails closed rather than silently suppressing a possible reaction. Draft
regressions include fixed Unarmed critical damage, exact reaction/parent provenance,
forged source/crossing/cost rejection, source Wolf Prone, Chimera advantage cancellation,
and no borrowed Warhorse Charge. At this edit these new tests are unrun; root owns the
compiler until its integration batch completes. Existing source evidence above remains
scoped to the earlier commits, not this working delta.

### Verified opportunity checkpoint

The combined implementation now passes 28 attack integration tests, 10 movement
integration tests, all 32 spatial unit tests and 20 turn-continuation regressions.
Strict all-target Clippy for `dmd-domain` and `dmd-rules`, formatting and diff checks
pass. Each attack/movement fixture serializes accepted state and events, replays the
actual reducer, and validates the resumed state. This is 90 focused tests, not a full
workspace/desktop acceptance claim. The shared Cargo target initially reused newer
artifacts from another worktree despite older local source mtimes; explicitly refreshing
the two affected library roots forced the actual current sources to compile before
these results were recorded.

Review corrections verified here include grouping Chimera's same-type source dice pools
before resistance (including the odd-sum rounding case and critical doubling), preserving
explicit movement choices when a Wolf's Prone rider invalidates a pending Walk, and checking
real source reach before uncertain cover so a distant enemy cannot block unrelated movement.
The source Charge helper has an intentional `expect(dead_code)` until the real own-turn
feature adapter consumes it; this records unfinished same-gate work, not an acceptance waiver.

Root integration must update restore origin collection atomically: retain `attack.origin`,
`attack.source.Weapon.equipment_before.command`, and
`attack.admission.Opportunity.origin`. Source pins in intrinsic feature records are data,
not substitute commands. Movement separately retains its original command, initial progress
origin, traversed-segment causes, decision origins, selected crossing origin and current
budget movement origin. App projections continue using the common actor/target/stage;
source-specific fields and hidden target facts do not become player-facing details.

## Sealed spell attack continuation

The independently reviewed opportunity checkpoint is `f69dee626dcb9fd1407fab99c4212fd21c04cb51`.
The next bounded task consumes casting helper/duration merge `03f93cf` and adds a
sealed spell attack adapter in the owned attack modules and source enum. The casting
author owns shared resolution/work/dispatch and cast completion. Both paths use the
same existing attack and vitality queue; no resource payment, fake ItemId, synthesized
System issuer, or second pending queue is introduced.

The source record retains canonical cast occurrence, node and target ordinal and pin.
Its immutable casting origin remains distinct from the actual accepted command that
caused this attack occurrence. Reconstruct every pending source through the retained
cast and sealed proof. Live AC, cover, visibility, conditions and Exhaustion apply at
each ray; critical hits double dice only. Complete that exact cast occurrence before
pumping damage/concentration children. Paid dead/invalid target occurrences finish
with no effect rather than creating a request that cannot complete.

Acceptance: source forgery and duplicate occurrence rejection; raw attack/damage
validation; serialized/replayed pending stages; critical dice and repeated rays with
distinct request identities; dead/invalid later ray completion; truthful cross-actor
causal provenance; unchanged inventory and no repeated slot/action costs. Run focused
attack/casting/turn tests and strict lint only after the shared compiler handoff. At
this planning edit the new adapter is not implemented or verified. Root owns app
origin collection, full workspace verification and production acceptance. Own-turn
creature features/Charge, remaining masteries and broader source spell programs remain
explicit unfinished Gate4 work after this bounded task.

The source adapter and five public casting regressions are now drafted against shared
driver checkpoints `cf72285` and `0aae8c0`. The driver consumes the sealed callback;
the adapter checks the exact retained occurrence and actual causal head, derives live
hit facts, and stages the existing attack work without recursively running the pump.
Raw request identity uses the original casting command and unique shared occurrence;
the request's issuer metadata remains the truthful command that reached this ray.
The driver partitions every target occurrence among queued work, the active attack and
completed records, preventing a restored attack from stealing another ray's index.

Draft regressions exercise source level scaling/critical dice, bad raw faces and replay
metadata, explicit printed dragon spell attacks, three distinct rays separated by a
target player's concentration save, first-ray death, and eleven retained source/cause
forgeries. Formatting and diff checks pass. These tests and the new shared driver have
not yet compiled; the next action is the single coordinated casting/attack test batch
after root releases the compiler. This intermediate source checkpoint is not green
verification or a production acceptance claim. Root also needs to collect the new
`attack.admission.Spell.casting_origin` alongside `attack.origin` and retained cast origins.
