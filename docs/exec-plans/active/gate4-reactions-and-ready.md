# Gate 4 — Reaction and Ready source audit

Status: **Active; narrow Shield-training correction planned; reaction execution remains unimplemented.**
Branch: `codex/gate4-spell-shield-training`; base `ba028fe`.
Writer: rules_architecture. Root owns integration and gate acceptance.

## Objective and boundaries

Correct the source distinction between armor training and Shield training in the
existing spell binding path. Record an implementation-ready audit for Ready,
Shield, and Counterspell using the shared tactical resolution, without implementing
new reaction commands in this correction. This advances the product's authoritative
rules, physical dice, player agency, reaction timing and exact recovery requirements
(`docs/product-definition.md`, especially combat timing and save/resume), Gate 4,
ADR 026 and the active spell/casting plans. It does not change the product or gate
finish line. Full catalogs remain Gate 6 and rituals/long casting remain Gate 5.

Pinned SRD 5.2.1 sources: armor training p.177; physical spell components pp.105–106;
Reaction pp.10/186; Ready pp.186–187; Counterspell p.120; Shield pp.161–162;
concentration p.179; Attack action p.177 and simultaneous effects p.187.

## Acceptance criteria and work

1. Check in this plan before production edits. Inspect the integration at `e64c53f`
   for the audit and use parent-approved `ba028fe` for the isolated correction.
2. Untrained worn armor continues to prohibit casting; an untrained Shield alone
   does not. Current physical hands, speech and material access still decide whether
   the actual spell's components can be supplied. Do not alter armor class here.
3. Add paired spell-binding regressions for real Shield plus free/occupied hand and
   trained/untrained worn armor. Both accepted queries and rejections are immutable;
   serialization retains the same outcome. Use existing executable spells so tests
   do not claim an unavailable reaction spell executor.
4. Record the required retained state, source clauses, ordering, privacy, recovery
   boundaries and real test scenarios for the next Ready/reaction implementation.
5. Independent exact-diff review by environment_audit. Run focused spell tests and
   strict domain/rules Clippy only after the globally serialized compiler handoff.
   Root performs canonical/integrated verification before merge.

## Non-goals and risks

No public Ready/Shield/Counterspell execution, content-grant invention, new queue,
save format, app/UI change or Gate 4 completion claim. A private unsupported spell
program must remain rejected before costs. The main correction risk is accidentally
relaxing worn-armor or physical-hand validation while removing Shield training as
a casting prerequisite.

## Validation and next action

Plan checked in before code; no source changes or compiler runs yet. Next: narrow
`tactical_spells/binding.rs` training check, add meaningful paired regressions, finish
the audit below, request independent review, then wait for compiler ownership.
