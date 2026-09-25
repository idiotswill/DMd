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

## Reviewed and verified source checkpoint

[PR28](https://github.com/idiotswill/DMd/pull/28) source head
`32970a11a3ae6e889723901d8d82cc6b771fc826` passed canonical `./scripts/verify`:
329 Rust tests, formatting, workspace/all-target checks, strict workspace/all-target
Clippy, genericity guard and architecture guard (eight tests, one platform-specific
skip). Durable local log: `tooling/gate4-weapons-canonical.log` outside the repository.
All six jobs passed in [Linux CI](https://github.com/idiotswill/DMd/actions/runs/36102146446)
and [Windows desktop](https://github.com/idiotswill/DMd/actions/runs/36102146444), including
MSRV compatibility and stable release packaging.

Fresh independent full-diff review checked pinned SRD16,48,89–91,177 against all
planner/equipment/history/mastery code,17 weapon tests and the character-grant delta.
No concrete source, identity, choice or history defect remains. The explicit source
armor exclusion, unchanged starter shop and source-plan-only scope remain documented.
Root separately inspected the complete dependency and relevant source/test boundaries.

This final evidence update is documentation only. Require its exact delta review,
code equality with the verified source head and its own final-head CI before protected
merge. Fetch main, compare the resulting tree and check post-merge CI before dependent
integration. Actual source attacks, mastery execution and complete Gate4 acceptance
remain active work; this verification does not substitute for those production paths.
