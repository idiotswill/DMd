# Gate 4 physical weapon plans

Writer: root on `codex/gate4-physical-weapon-plans`, based on main
`0ff676dd0fceefa826230bdcafdfcb608fc571d7` after PR27. The encounter integration
branch supplies the reviewed implementation; this branch extracts one bounded dependency.

## Objective and authority

Provide source-derived physical weapon plans from actual ItemIds, custody, hands,
ammunition and accepted attack history. Preserve player choices for weapon mastery
consequences and derive Fighter mastery selections from the complete pinned catalog.
Root AGENTS, the product's source-fidelity and durable-authority requirements, Gate4,
ADR024 and the pinned SRD5.2.1 govern this work.

This is a pure planning dependency, not authorization to execute an attack. Context
must be supplied by the authoritative encounter resolver and each accepted command
must revalidate current source, geometry, timing and resources. No new scheduler,
RulesState attachment, save interpretation or completed player-facing combat is claimed.

## Scope and acceptance

1. Extract tactical_weapons and its equipment/history/mastery modules and tests;
   include character intrinsic validation and all-source Fighter mastery choices.
2. Keep the existing starting shop and historical character validation intact.
   Intrinsic validation deliberately excludes live armor, which its caller must verify
   independently against real equipment. Do not substitute starting equipment for
   current custody, or ordinary weapon statistics for explicit creature features.
3. Review source properties, physical equip/grip/ammunition transitions, attack-window
   history, Light/Nick/Cleave limits, critical damage and mastery choice preservation.
4. Run canonical ./scripts/verify, independent full-diff review and exact-head CI.
   Merge with expected-head protection, compare the merged tree, check post-merge CI
   and reconcile main into the encounter integration branch.

## Status, risks and next action

Plan created before extraction. Source catalog PR27 is merged and its exact tree was
verified; post-merge CI is running. No implementation or check success is claimed on
this branch yet. The pure helpers are already used in the separate encounter branch,
but real command authority, atomic resource updates, persistence/restore, complete
mastery execution and packaged Gate4 encounter acceptance remain active integration
work. Next: extract only the listed dependency files and inspect the complete diff.
