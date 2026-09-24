# Execution plan — post-Gate-1 product roadmap bootstrap

Status: **Implementation complete — final documentation head must pass CI before merge**

## Objective

On a dedicated docs/governance branch from current `main`, install the owner-approved finished-product expansion, Gate 2–14 roadmap, requirement traceability, gate execution protocol, Rules Coverage Ledger requirement, and mandatory human vertical-slice checkpoint. Reconcile repository agent governance with the owner's autonomous-within-gate merge authorization.

After exact-head validation, Codex may merge this bootstrap PR itself, refresh `main`, and begin Gate 2 on a fresh branch without pausing for routine approval.

## Branch / PR

- Branch: `codex/post-gate1-roadmap`
- PR: [#15](https://github.com/idiotswill/DMd/pull/15), docs/governance only, base `main`
- Verified baseline: `056b788364029f74afd15c9dcb592920308998fa`

## Verified baseline when pack was refreshed

- Gate 1 accepted/merged through PR #14.
- `docs/exec-plans/active/` no longer existed because completed Gate 1 plans were archived.
- no open PRs were present.

Verified live on 2026-09-24: a fresh clone of `main` is exactly the baseline above; GitHub reports PR #14 merged to that commit and no open PRs at entry. Gate 1 remains accepted. Bootstrap corrects its stale merge-pending wording using directly re-verified PR #14 metadata and exact-head CI #294.

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
- Independent full-diff review of `fb28aa6` plus protocol whitespace correction found no blockers. Its final suggestion to spell out every handoff gate-summary field is incorporated in the final documentation bookkeeping.
- Mechanical text comparison verifies the original product contract is preserved as an unchanged prefix and every substantive expansion requirement is integrated as an unchanged suffix.
- Local `bash -lc './scripts/check-genericity && ./scripts/check-boundaries'` passes. An earlier invocation without a login shell lacked `grep`/`dirname` and is not counted as verification; use the initialized Git Bash environment.
- `git diff --check` passes after removing a trailing blank line from the supplied protocol.
- [CI #296](https://github.com/idiotswill/DMd/actions/runs/35995172146) passed on implementation head `fb28aa64f242ee3f2accab8876a3f2179f68524c`: verify-fast (format/check), Clippy, workspace tests, Rust 1.88 MSRV, genericity guard and architecture guard. These directly cover the canonical `./scripts/verify` commands.
- Local Rust/C++ setup is being prepared outside the repository. Bootstrap does not claim local Rust verification.
- Final plan archival/whitespace bookkeeping creates a new head. That exact head must pass all repository CI checks and a final diff review before expected-head-protected merge; final SHA/run evidence is recorded in PR #15 to avoid a self-referential commit hash.

## Blockers/risks

- No unresolved bootstrap blocker. Repository movement must still be checked before merge.
- Licensing details must be verified from current official sources before becoming implementation assumptions.
- Future physical playtest/reference-hardware acceptance may require evidence outside Codex's cloud environment.

## Next action

Verify the final PR #15 head and complete diff, merge with expected-head protection, fetch merged `main`, then create a fresh Gate 2 branch and active plan. Gate 2 starts with current official rules-source/licensing verification and a complete source-derived ledger. Bootstrap is not a gate-end pause.
