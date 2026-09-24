# Gate 4 — Casting in the shared tactical resolution

Status: **Active; source binding and execution implementation pending.**
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

No tests or builds run yet. All Rust build use requires the shared build-slot handoff.
Root holds that slot at task start. Main risks are double payment, retained reservation
drift, hidden-target leaks, mismatched shared roll semantics, and pausing outside the
single durable resolution. Attack author owns existing shared tactical files until its
first verified checkpoint. This writer initially changes new modules/tests only.

Next action: agree the cast cursor integration contract, implement binding/admission
against the current source and spatial APIs, then obtain review before queue integration.
