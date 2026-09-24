# Execution plan — Gate 2 source and coverage ledger

Status: **Slice implementation complete — final head requires CI and review before merge**

## Objective and branch

Pin the commercially reusable rules source and give every source rules/content family a primary implementation gate before dependent implementation ships.

- Branch: `codex/gate2-source-ledger`, one writer in a separate worktree.
- Baseline: freshly fetched `main` `414040b33d58701cec81e6d73b791347c6d43ca1`.
- PR: [#16](https://github.com/idiotswill/DMd/pull/16).

## Scope, non-goals and contract

This slice adds source provenance, exact attribution, the machine-readable Rules Coverage Ledger and CI checks for ownership/completeness. It does not implement gameplay, change saves, or claim rules completion. Gate 2 kernel/application integration remains a separate slice.

Relevant contract: product-definition commercial boundary, faithful selected rules, complete coverage accounting and local authority; Gate 2 checkpoint; ledger template; ADR 014 content identity/manifest validation; ADR 016 runnable campaign composition.

## Acceptance criteria

1. Verify the current source directly from official sources and pin version, publication date, PDF digest, commercial license, attribution and compatibility wording.
2. Account for the complete source's rules/content families, including the glossary and catalogs, with page citations and exactly one primary gate per row.
3. Clearly distinguish primitives from full tactical/noncombat/content completion. Every deferral names its receiving gate and later acceptance path.
4. Run an offline Rust test in existing CI that rejects missing/duplicate family ownership, invalid citations/status and completion claims without evidence.
5. Review the full diff and run `./scripts/verify-fast`, `./scripts/verify` or directly verified canonical equivalents, and exact-head CI before merge.

## Planned work

1. Record source decision and licensing/provenance.
2. Derive inventory from the official PDF contents, glossary and catalogs; map implementation owners.
3. Add mechanical ledger validation and negative cases.
4. Open/update PR, obtain independent review, fix findings, record exact evidence and merge only after expected-head protection and green checks.

## Decisions

- Ruleset ID `srd-5.2`, exact rules/content version `5.2.1`, agreed with the primary implementation agent.
- Select the official English SRD 5.2.1, published 2025-05-01, under CC BY 4.0. Do not mix 2014 SRD 5.1 mechanics or ingest D&D Beyond Basic Rules.
- Download/research scratch artifacts stay outside the repository. Checked-in provenance gives a reproducible URL/digest and offline inventory; CI does not fetch the web.
- Ledger accounting is not gameplay acceptance. No mechanics row is marked implemented in this slice.

## Validation and risks

Official source selection verified; source PDF legal/contents pages inspected visually. The ledger covers 55 families and every substantive source chapter, plus 155 glossary entries, 338 top-level spell headings, 258 magic-item headings, 330 creature stat blocks, all 12 classes/subclasses, four backgrounds, nine species and 17 feats. Subentries/variants remain obligations of their parent family; no gameplay completion is claimed.

Validation on implementation head `db2699ca4d6bdb894e6ace2f53e11c7d16555753` and the reviewed name correction:

- [CI #299](https://github.com/idiotswill/DMd/actions/runs/35997910803) passed on that exact implementation head: Rust format/check/Clippy/workspace tests, Rust 1.88 MSRV, genericity guard and architecture guard. Job steps were directly inspected.
- Local `cargo test --locked -p dmd-domain --test rules_coverage_ledger` passes all four positive/negative integration checks. Source-targeted Clippy with `-D warnings` and formatting pass.
- Activated Git Bash `./scripts/verify` passes formatting and workspace check, then exposes existing Windows-only unused imports/helpers in untouched `content_manifest_fail_closed.rs`. No lint was weakened. The main Gate 2 implementation slice owns that portability repair. A first invocation before sourcing the toolchain did not find Cargo and is not counted as verification.
- Local `cargo test --locked --workspace`, `./scripts/check-genericity` and `./scripts/check-boundaries` pass separately using the portable Windows GNU toolchain and isolated `target-source-ledger` build directory.
- Primary-agent full-diff/catalog review found one extraction spacing error: `GrayOoze` is corrected to `Gray Ooze` against source page 293 and its contents index. All 330 creature entries were independently cross-checked against the complete source stat-block index.
- Final bookkeeping also adds live-ledger navigation from the template and tests the complete attribution statement/official source URLs. It creates a new head; final exact-head CI/review evidence belongs in PR #16 before expected-head-protected merge, avoiding a self-referential commit hash.

No commercial-source blocker found; non-SRD protected content remains excluded. Future source updates require explicit version/provenance review, not silent replacement of pinned campaign content. This slice establishes accounting and provenance only. The primary Gate 2 plan still owns actual mechanics/application integration, final ledger statuses and gate acceptance.

## Exact next action

Verify the final PR #16 head and complete delta, merge only with expected-head protection after green required checks, and refresh `main`. The primary implementation branch then integrates this source identity/notice/ledger with the production kernel and records gameplay evidence. Do not mark Gate 2 accepted from this slice alone.
