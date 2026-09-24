# Execution plan — post-Gate-1 product roadmap bootstrap

Status: **Active — baseline verified 2026-09-24**

## Objective

On a dedicated docs/governance branch from current `main`, install the owner-approved finished-product expansion, Gate 2–14 roadmap, requirement traceability, gate execution protocol, Rules Coverage Ledger requirement, and mandatory human vertical-slice checkpoint. Reconcile repository agent governance with the owner's autonomous-within-gate merge authorization.

After exact-head validation, Codex may merge this bootstrap PR itself, refresh `main`, and begin Gate 2 on a fresh branch without pausing for routine approval.

## Suggested branch / PR

- Branch: `codex/post-gate1-roadmap`
- PR: docs/governance only, base `main`
- Pack refresh baseline: `056b788364029f74afd15c9dcb592920308998fa` (must be refreshed)

## Verified baseline when pack was refreshed

- Gate 1 accepted/merged through PR #14.
- `docs/exec-plans/active/` no longer existed because completed Gate 1 plans were archived.
- no open PRs were present.

Verified live on 2026-09-24: a fresh clone of `main` is exactly the baseline above; GitHub reports PR #14 merged to that commit and no open PRs. Gate 1 remains accepted. Its checkpoint still describes the completed merge as pending; bootstrap will correct only that stale wording.

## Scope

- verify accepted Gate 1 remains current;
- integrate the owner-approved finished-game requirements into `docs/product-definition.md`;
- add future-gate roadmap;
- add Gate 2–14 checkpoint docs;
- add requirement traceability;
- add gate execution protocol;
- add Rules Coverage Ledger requirement/template;
- add mandatory post-Gate-6 human vertical-slice checkpoint;
- update `AGENTS.md` governance to match owner authorization: Codex may merge within a gate after verification, owner pause occurs at gate end;
- update PR summary and exact validation evidence.

## Non-goals

- gameplay implementation in the bootstrap PR;
- new Rust production code;
- save schema changes;
- migrations;
- provider/model integration;
- Asterra import;
- detailed implementation design for distant gates.

## Relevant sources

- current root `AGENTS.md`;
- current `docs/product-definition.md`;
- accepted Gate 1 checkpoint/review;
- current `docs/exec-plans/` state;
- accepted ADRs needed to preserve existing invariants;
- exact current `main`;
- owner-approved roadmap/product inputs from this pack.

## Acceptance criteria

1. No existing product requirement is silently deleted or weakened.
2. Gate 1 accepted status is preserved unless direct evidence shows a regression/contradiction.
3. Roadmap has a clear Gate 2–14 sequence.
4. Every gate defines production integration acceptance.
5. Requirements traceability contains no known orphan product areas.
6. Rules completeness/provenance is a mechanical Gate 2 requirement.
7. Post-Gate-6 human vertical slice is mandatory.
8. Gate 14 expresses the complete sustained-play finish line.
9. Distant gates are behaviorally precise but not over-designed.
10. Agent governance reflects autonomous in-gate PR merge authority and end-of-gate owner pause.
11. Full diff is reviewed.
12. Applicable repository verification passes on the exact PR head.
13. Bootstrap PR is merged by Codex only after exact-head validation.
14. Gate 2 implementation begins only after the bootstrap is merged and `main` is refreshed, on a fresh branch.

## Planned slices

### Slice A — repository verification/governance reconciliation

Refresh `main`, verify Gate 1 accepted state, inspect current governance, and adapt only what remains stale.

### Slice B — authoritative product definition

Integrate owner-approved product requirements without weakening current contract.

### Slice C — roadmap/checkpoints/traceability/protocol

Add roadmap, Gate 2–14 files, traceability, gate execution protocol, rules ledger requirement, and vertical-slice checkpoint.

### Slice D — review/validation/merge

Inspect full diff, run verification, resolve contradictions, update plan/PR, merge with expected-head protection after exact-head checks are green, refresh `main`, then start Gate 2 on a new branch.

## Decisions

- Product definition remains the authoritative finished-game contract.
- Checkpoints map product requirements into bounded implementation stages.
- Execution plans contain gate-specific implementation design.
- Only the active gate receives deep implementation design.
- Codex is autonomous to merge in-scope work inside an active gate after rigorous verification.
- Owner pause/review is at the end of each gate.
- Human playtesting is a hard product checkpoint, not optional UX polish.
- Legal/commercial content provenance is enforced from rules/content work onward.
- Autonomous merge authority never permits silently narrowing product scope or waiving failed acceptance criteria.

## Validation status

- Fresh clone and GitHub PR metadata verify baseline `056b788364029f74afd15c9dcb592920308998fa`, merged PR #14, and no open PRs.
- Product expansion is appended in full after the existing contract; no existing requirement is removed.
- Read-only independent input audit identified stale runbook governance, a circular Gate 6 playtest prerequisite, and missing explicit traceability rows; all are reconciled in bootstrap.
- `git diff --check` passes during preparation. Full exact-head review and CI are pending.
- This Windows environment has Git Bash but no installed Rust/C++ toolchain. Local toolchain setup is in progress outside the repository; CI must directly verify the canonical equivalent checks before merge.

## Blockers/risks

- Repository may have moved after the pack refresh; fetch/reconcile before writing.
- Licensing details must be verified from current official sources before becoming implementation assumptions.
- Future physical playtest/reference-hardware acceptance may require evidence outside Codex's cloud environment.

## Next action

Install the approved product/checkpoint inputs, reconcile current governance, review the complete diff, and verify the exact PR head before merge. Gate 2 follows on a fresh branch after refreshed `main`.
