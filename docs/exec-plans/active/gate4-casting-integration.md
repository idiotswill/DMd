# Gate 4 — Casting in the shared tactical resolution

Status: **Active; source binding and program leaves verified, shared queue wiring pending.**
Branch: `codex/gate4-casting-integration`; base `37d80442c0f8463040427d483df714e937ee9176`.
Writer: bootstrap_audit. Root owns application, UI, recovery integration and gate acceptance.

## Objective and boundaries

Turn the reviewed source spell plans into real, interruptible encounter actions using
the existing `TacticalResolution`. Advance the product's authoritative rules, physical
dice, player agency, visibility and exact save/resume requirements and Gate 4's casting,
effects, concentration and timing criteria. ADR 026 and pinned SRD 5.2.1 pp.16, 105–106,
179 and 186–187 govern this work; individual source programs retain their own pages.

Source grants, real component ItemIds, current positions and senses determine admission.
Input selects legitimate targets and source choices; it supplies no damage, DCs, actor
capabilities, prepaid flags or arbitrary effects. Invalid input is unchanged. Reject a
program that this slice cannot execute completely before costs or concentration changes.
An invalid *source target type*, in contrast, retains source expenditure and gives the
player-safe apparent result required by SRD106.

No second queue, RNG, invented controller choice, public state patch, template coupling,
or full spell/Gate 4 claim. The eighteen-family checklist in the source spell plan and
effect lifecycle plan remains active Gate 4 work. Full catalogs remain Gate 6; long
casting and rituals remain Gate 5. A closed first executable subset is an incremental
implementation boundary, never a change to gate acceptance.

## Work and coordination

1. Check source programs and geometry APIs; implement source-bound target/material
   validation and bounded closed-program admission in new files.
2. Coordinate cursor/frame/roll roles with the attack author before touching shared
   files. Install a typed cast cursor in the one resolution and reuse attack, vitality,
   effect, concentration and raw-roll reducers. The root's pending boxed-resolution
   memory change must be integrated without changing serialized representation.
3. Commit resources atomically, accounting for outstanding slot reservations. Only
   Counterspell receives its explicit slot exception. Ready uses the same cursor,
   concentration, authentic trigger and reaction economy; interrupted casting terminates.
4. Execute per-ray/dart occurrences and simultaneous save damage with correct sharing,
   current ability/Exhaustion/visibility and individual consequences. Source ongoing
   zones need durable geometric anchors before movement can derive membership; clients
   must never submit occupant lists as authority.
5. Meaningful invalid/unchanged, raw-dice, per-target, resource, serialization/replay and
   suspended-resume tests. Independent review; serialized focused tests and strict lint.
   Root performs full verification and real application recovery/desktop evidence.

## Validation and risks

The helper checkpoint `accb95eca478999b80fc4f35707302164f77c708` passed forty source
casting tests, including sixteen new binding/retention/execution regressions, strict
domain/rules all-target Clippy, formatting and diff checks. The duration follow-up below
records its separate results. Logs are retained outside the repository under
`research/gate4-casting/`. All Rust build use requires the shared build-slot handoff.
Main risks are double payment, retained reservation
drift, hidden-target leaks, mismatched shared roll semantics, and pausing outside the
single durable resolution. The ordinary attack checkpoint is `334d82b`; movement now
owns shared tactical files until its verified checkpoint. This writer changes source
spell child modules, new records and their exports/tests, not the shared scheduler.

## Current implementation and review

- The root's wire-transparent boxed continuation fix is integrated (`1237c3a`, local
  cherry-pick `e6a0db8`). No additional save layout has been attached by this slice yet.
- Read-only binding validates actual physical components/hands, source armor training,
  range, precise location, sight when required, cover, type, and source target counts.
  Unknown/unlocated IDs share an error; known invalid creature types retain private
  no-effect facts for the source's paid apparent-success behavior (SRD106).
- Root review caught attack reach being mistaken for touching reach. Corrected to the
  supported ordinary 5-foot touch capability, with a long-weapon-reach regression.
  Specifically extended touching still needs a source capability; no name-based grant.
- The closed first set is healing, saving-throw condition, automatic individual damage,
  and creature-target spell attacks. Canonical programs requiring objects, zones,
  behavior commands, protections or other absent leaves reject before any expenditure.
- Source spell attack proofs retain command, cast occurrence, node ordinal, target
  ordinal, intrinsic modifier and damage. The attack adapter must add current Exhaustion
  once and derive live visibility/cover/conditions per occurrence. No invented ItemIds.
- Internal `TacticalCasting` records retain ordered bindings and completed occurrences;
  exact source reconstruction complements, never replaces, accepted-event replay.
- Program leaves derive raw healing/dart dice and ordinary vitality operations. Condition
  proposals derive duration, stable IDs, concentration binding, same-spell overlap and
  target-end repeat saves. No leaf creates a second queue or makes a player's choice.
- All sixteen new regression cases and twenty-four prior source tests pass. The first
  compile required ordinary String/&str fixes and moving deterministic effect IDs to
  the domain crate's existing UUID authority, with no new dependency. The first test
  run passed 37/40: two fixtures incorrectly treated the source Goblin Warrior (Fey)
  as Humanoid and now use Cultist Fanatic; the third caught a real paid-dead-target
  issue. Spell amount leaves now return typed no effect for dead/invalid targets after
  validating dice, preserving expenditure instead of calling forbidden revival or
  failing mid-commit. No runnable cast/application/Ready/Counterspell claim is made.
- Independent read-only leaf/retained-record review by environment_audit found no
  additional concrete blocker. It explicitly excludes complete casting integration;
  shape reconstruction cannot authenticate historical grants, targets or provenance.
- Source binding also exposed the existing `perceive(self)` sight shortcut: it bypasses
  Blinded, darkness and Invisible. The spatial/movement writer owns its bounded fix.
  Self-location must remain known while sight-dependent self-targets use real senses.
- Reserved roll tags: physical attacks 7/8; movement falling/landing 9/10; casting 11+.

## Casting-duration follow-up

Root authorized a bounded lifecycle `SetCastingDuration` operation while the shared
movement/attack checkpoint is completed. It updates only the exact immutable source's
existing Casting-stage group; active groups, missing groups, a different source, pending
expiry, infinite expiry and already-due times are rejected unchanged. The canonical
spell program derives the operation after Commit or Ready release. Installation still
activates the group, and a cast with no affected targets still needs shared-driver cleanup.
Two lifecycle regression cases, canonical helper assertions, and a genuine
Ready/Commit/Release regression now pass: all 41 source spell tests and all 21 lifecycle
tests passed on this follow-up. Strict domain/rules all-target Clippy, `cargo fmt --check`
and `git diff --check` passed as well. Logs: `duration-spell-tests.txt`,
`duration-effect-tests.txt` and `duration-clippy.txt` under the external research directory.
The earlier helper checkpoint's forty-test evidence is separate from this delta.
Independent read-only review by environment_audit found no source blocker. The
shared driver must invoke the duration update exactly once at its journaled
Commit/Release transition; this internal leaf is not itself an idempotent command.

Nested source-cast provenance remains an explicit integration concern: the existing
effect source validator equates accepted command actor and actual caster. A truthful
Player A cause that internally triggers NPC B cannot be rewritten as an invented System
command. Direct casts are supported by existing provenance; a sealed distinction between
invocation and actual source caster is required before enabling that nested path.

Root also assigned the inherited Check/Save house-rule inconsistency to shared
integration: effect and concentration completion and Legendary Resistance failure
admission currently compare totals directly, ignoring the table's explicit
`ability_test_natural_extremes` choice. A shared kept-d20 outcome helper must preserve
default source behavior, honor that opt-in consistently, and leave the distinct
death-save natural-1/20 rules intact. Add explicit opt-in and default regressions;
the falling writer will use the same policy for landing checks. No scheduler file
is changed before the coordinated movement/attack handoff.

Next action: commit this verified duration leaf and release the compiler to root's
source-catalog verification. Then integrate the coherent movement/opportunity-
attack checkpoint and wire actual casting stages into the one tactical resolution.

## Shared casting checkpoint in progress

The verified duration leaf is `07c624765e6dfa9ae9bb7824d09742cc179b70c3`; exact
independent delta review found no blocker. Movement/opportunity attacks
`f69dee626dcb9fd1407fab99c4212fd21c04cb51` are merged at `03f93cf`; only domain exports
conflicted, and both casting and attack exports were retained. This writer now owns
shared dispatch/continuation files. environment_audit owns the attack source enum
and the separate sealed spell-attack adapter. No parallel scheduler is created.

Current unverified implementation adds `CastSpell { choice, targets }`, bounded
`resolution.casts`, `SpellProgram`/`FinishSpell`, and stable roll tags SpellSave11 and
SpellAmount12 (falling retains9/10). Source feature receipts and the ordinary budget
own NPC activation/uses; prepared slots remain in MechanicalEntity. Exact source
validation, physical components and closed-program checks precede costs. Individual
target work marks completion before nested vitality/effect consequences, and a
partition validator rejects omitted/duplicated targets or missing completion work.
The shared Check/Save outcome policy honors the explicit natural-extremes option
and labels that ruling as a house rule; default RAW and death-save rules remain distinct.

This is an implementation checkpoint for coordinated development, not verified or
accepted behavior. The first driver enables canonical healing, individual damage,
and saving-throw conditions. Attack programs remain rejected before costs until the
owned source adapter is connected. Ready triggers, Counterspell interruption, areas,
objects, additional effect families, and meaningful late-target decisions remain
active Gate4 work. Existing pure Ready/interruption contracts retain their source
semantics. No source grant is invented to make a sample reachable: Cure Wounds uses
the existing prepared catalog, Hold Person can use the real Cultist feature, and the
current NPC sample does not grant Magic Missile. App role/projection handling and
all accepted-event restore auditing remain root integration work.

Next: finish the public resolver regressions and strict retained-work checks, connect
the sealed attack adapter, and acquire the global compiler slot for focused tests.
No test or build evidence from the earlier leaves covers this new shared code.

The source attack handoff adds a sealed `begin_spell_attack` hook and counts its live
source occurrence in the same complete-work partition. Attack admission is enabled
only with that owned adapter dependency; this development checkpoint requires the
matching attack source files before compilation. Its keys preserve original casting
identity while the attack records the actual accepted continuation cause. Pending
late concealment/target choices remain part of full interruption support.

The owned attack adapter `5402050` is now merged with its six public spell cases.
Six additional public casting cases cover atomic healing payment and caster-owned
amount dice, range/Ready/forged-grant rejection, exact retained target partition,
source Cultist material and limited-use ownership with target-end repeat save,
the explicit natural-extremes save policy, and rejecting a canonical source spell
reassigned to an actor lacking that source profile. The immutable profile binding
is checked separately from canonical program reconstruction; mutable prepared
caster statistics are not frozen during restore. Missing/dead paid targets close
their occurrence instead of producing an unfinishable amount request.

An effect-save/Legendary Resistance regression checks both natural-1 total success
and natural-20 total failure under RAW versus the explicit option, without changing
raw faces. The shared driver and these cases are still uncompiled at this checkpoint.
Movement domain receipt `616ffa8` is integrated with the two required initializers;
its actual stop/receipt writer retains ownership of movement files. Next verification
is the coordinated casting/attack, movement, turn, source-spell and effect batch,
then strict domain/rules lint. Earlier leaf evidence does not cover this integration.

## Verified bounded shared integration

The actual public resolver now runs Cure Wounds, Fire Bolt, source Cultist Hold
Person and source Dragon Scorching Ray through the existing tactical resolution.
Prepared casting in these fixtures is imported mechanical state, not a new starter
character class option. No Magic Missile grant is invented: its amount leaf remains
tested separately until a source-supported public grant is installed. Seven casting
cases and six source-attack cases exercise accepted payment, physical material IDs,
source use counters, separate ray keys, actual continuation cause after another
player's save, paid dead-target completion, individual controller choices and replay
after every accepted action. Ready and unsupported programs reject before costs.

The restored attachment binds canonical creature spells to the actual immutable
actor profile and rejects differing accepted metadata at the same event head. Its
exact target-work partition covers pending save/amount work, live spell attacks,
completed occurrences and one finish frame. Application replay must still prove
historical source admission and resource payment; structural validation is not a
replacement for that accepted-event audit.

Source SRD14 shared-space behavior is one `EndOccupiedSpace` work item alongside
other simultaneous End effects. It derives positive 3D volume overlap, honors Tiny,
strictly larger size and Prone immunity, rechecks geometry at actual application,
and preserves the controller's End ordering. It introduces no roll or other
creature identity in its durable work. Source SRD187 rest coverage proves accepted
non-cantrip casting interrupts the real knockout-rest record while a cantrip or
definite rejected declaration preserves it. Explicit natural-extremes regressions
cover spell saves, effect saves, concentration and source Legendary Resistance;
ordinary RAW and the separate death-save rules retain their outcomes and raw faces.

Verification completed on the final implementation tree:

- `cargo test -p dmd-rules --test tactical_attacks --test tactical_movement
  --test tactical_turns --test tactical_effects`: **96 passed** (41,10,24,21).
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets -- -D warnings`: passed.
- Formatting and diff whitespace checks passed.
- The broader rules unit suite passed **131 tests** during this batch, including
  41 source-spell and 33 vitality cases. The subsequent shared-driver metadata
  tightening and lint-only changes were covered by the final integration/lint run.

External logs under `research/gate4-casting/` are
`shared-final-second-tests.txt`, `shared-final-second-clippy.txt` and
`shared-unit-tests.txt`. Earlier logs retain the actual fixture failures: a dragon
incorrectly requested ammunition despite lacking a ranged weapon, and the generic
borrowed-club fixture accidentally made the newly created Cultist an existing item
owner. Fixtures were corrected to real source allocation preconditions. Three
strict-lint issues were fixed without suppressions. No larger stack setting or
native acceptance claim is part of this evidence.

Independent reviews by environment_audit and rules_architecture cover the source
leaves, sealed spell attack adapter, restore guards, rest/policy changes and complete
shared-space consequence. The final full shared-driver review and application
integration remain pending root coordination. App work must handle SpellSave11,
SpellAmount12 (amount roller is the caster, not key.subject), CastSpell action
metadata and a generic shared-space End label, then run accepted-record restore
auditing and the packaged flow. Movement owns its subsequent privacy/stop delta and
receives shared queue ownership for falling after this checkpoint. No second queue
is permitted. Ready triggers, Counterspell, area/zone binding, further source effect
families, source multiattack/legendary nested casting and late target decisions
remain active Gate4 obligations. This checkpoint does not accept Gate4.
