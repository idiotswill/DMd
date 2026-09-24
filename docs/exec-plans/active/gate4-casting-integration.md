# Gate 4 — Casting in the shared tactical resolution

Status: **Active; source binding and program leaves drafted, shared queue wiring pending.**
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

The forty source casting tests pass on the current helper implementation, including
sixteen new binding/retention/execution regressions. Strict domain/rules all-target
Clippy, `cargo fmt --check` and `git diff --check` pass. Logs are retained outside the repository under
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

Next action: obtain the final independent delta review for this helper checkpoint,
then integrate the coherent movement/opportunity-attack checkpoint and wire actual
casting stages into the one tactical resolution. The Rust slot has been released to
root for its application checks; no further builds without the next explicit handoff.
