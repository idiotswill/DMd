# Execution plan — Gate 2 source and coverage ledger

Status: **Active — source verified 2026-09-24**

## Objective and branch

Pin the commercially reusable rules source and give every source rules/content family a primary implementation gate before dependent implementation ships.

- Branch: `codex/gate2-source-ledger`, one writer in a separate worktree.
- Baseline: freshly fetched `main` `414040b33d58701cec81e6d73b791347c6d43ca1`.
- PR: pending creation after the first coherent source/inventory change.

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

Official source selection verified; source PDF legal/contents pages inspected visually. Required tests and final review pending. No commercial-source blocker found; non-SRD protected content remains excluded. Future source updates require explicit version/provenance review, not silent replacement of pinned campaign content.

## Exact next action

Create provenance/attribution and the source-derived ledger, then validate the complete inventory and open the PR for exact-head CI/review.
