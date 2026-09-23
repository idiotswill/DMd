# Gate 1 — Atomic state and event journal

Status: ready for human review
Branch: `gate1/atomic-journal-finalize`
PR: #6
Base: `main` @ `a836ef3f9c001ea6bbec81e191209230bd2293f6`
Verified implementation head: `596675f87931aa539e7c2ae128b470180eff21ce` (CI run 145 — pass)

## Objective

Build the first Gate 1 production persistence slice: every accepted material command transition can commit authoritative campaign state, command/resolution provenance, and an append-only event journal atomically in SQLite, with per-campaign sequence allocation and restart/recovery verification.

## Scope

- SQLite schema for current materialized campaign state, command audit records, immutable journal events, and causal edges;
- typed pending-event/audit boundary rather than provider-shaped persistence input;
- campaign/state/issuer/session/event-causality and state-held provenance validation;
- persistence-owned contiguous event sequence allocation;
- stale command rejection plus compare-and-set head advancement;
- next `CampaignState` plus audit/events/causal edges in one transaction;
- recovery loading from one SQLite read snapshot, including head/provenance integrity checks;
- tests for success, stale-state rejection, initialization/recovery/current-state provenance corruption, causal ordering/campaign isolation, authority metadata, rollback/reopen behavior, and direct-history-delete rejection;
- mechanical dependency guard preventing interpretation/core/rules layers from acquiring a direct persistence/SQLx write path;
- scoped Gate 1 checkpoint and atomic-journal ADR.

## Non-goals

- full snapshot-version migration/replay machinery;
- normalized projection tables for every domain aggregate;
- complete campaign lifecycle/backup/export/purge flows;
- rules/content-pack schema loading;
- combat or gameplay rules;
- world simulation/Director logic;
- voice, local inference, or desktop UI;
- merge without explicit human approval.

The materialized-state row introduced in this slice is a durable recovery/snapshot representation, not a replacement for the normalized query/projection tables required by later Gate 1 work.

## Relevant durable context

- `AGENTS.md`
- `docs/product-definition.md`
- `docs/architecture/002-state-and-events.md`
- `docs/architecture/009-typed-intent-command-boundary.md`
- `docs/architecture/011-atomic-journal-persistence.md`
- `docs/checkpoints/gate-0-review.md`
- `docs/checkpoints/gate-1.md`

## Acceptance criteria

- [x] A clean valid campaign state can be initialized durably at event sequence 0.
- [x] An accepted typed command transition persists command audit metadata, one or more immutable journal events, causal links, and the resulting materialized state atomically.
- [x] Event sequences are allocated contiguously by persistence and are unique per campaign.
- [x] A command whose `expected_event_sequence` is stale is rejected without changing state or journal.
- [x] Invalid/cross-campaign session and causal references are rejected; state-held event provenance cannot dangle.
- [x] Durable audit records preserve issuer separately from world actor and retain the versioned payload representation used by the accepted command.
- [x] Reopening SQLite recovers the last committed materialized state; uncommitted transaction changes are not visible after recovery.
- [x] Recovery reads state/head/provenance from one SQLite snapshot and rejects dangling provenance.
- [x] Every commit revalidates provenance already present in current state, preventing a later write from silently healing corrupted history.
- [x] Command/event/causal history rejects direct `UPDATE` and `DELETE`; whole-campaign purge is explicitly deferred to the lifecycle slice.
- [x] Direct dependency from domain/conversation/core/rules to SQLite persistence is mechanically blocked.
- [x] Existing session-ledger behavior remains green.
- [x] Proposed ADR/checkpoint wording has been reconciled with the implementation and deferred work.
- [x] Implementation CI run 145 passed on exact head `596675f87931aa539e7c2ae128b470180eff21ce`.
- [x] Complete PR #6 diff and review-thread state were inspected; no review comments/blockers were present before closeout docs.
- [ ] Final docs-only closeout head CI passes.
- [ ] Human explicitly approves this save-format/high-impact slice for merge.

## Planned slices

1. Define the persistence schema and typed pending-event/audit boundary. — complete
2. Implement initialize/load/atomic-commit APIs and invariant checks. — complete
3. Add restart, stale-write, causality, provenance, authority, rollback, corruption, and immutability tests. — complete
4. Add the scoped Gate 1 checkpoint/ADR and mechanical boundary guard. — complete
5. Run final verification, inspect complete diff, update plan/PR, and hand off for human merge approval. — implementation verified; docs-only final CI pending

## Decision log

- 2026-09-23 — Begin from merged Gate 0 `main`; one writable branch intended for this execution chat.
- 2026-09-23 — Retain a serialized current-state representation as a durable recovery/snapshot artifact while leaving normalized per-domain projections to later Gate 1 work.
- 2026-09-23 — Persistence, not the resolver/provider, owns journal sequence allocation.
- 2026-09-23 — Accepted material mutations preserve trusted command issuer metadata separately from any in-world actor.
- 2026-09-23 — `CommandMeta`/`CommandIssuer` live in the shared domain contract; `GameCommand<C>` remains the typed core request, avoiding a core↔persistence dependency cycle.
- 2026-09-23 — Sequence-0 initialization rejects state records that cite nonexistent event provenance.
- 2026-09-23 — Recovery rechecks state-held journal provenance and performs its state/head/provenance reads inside one SQLite transaction so concurrent commits do not create false corruption reports.
- 2026-09-23 — Material commit also revalidates provenance already present in current state before accepting a later transition, so corruption cannot be silently healed.
- 2026-09-23 — Event/provenance metadata lookup was batched to avoid N+1 database reads while retaining the same fail-closed checks.
- 2026-09-23 — SQLite triggers reject direct update/delete of command audit, event journal, and causal-edge history. Controlled complete campaign purge remains a later lifecycle operation.
- 2026-09-23 — CI mechanically blocks domain/conversation/core/rules crates from taking a direct `dmd-persistence` or `sqlx` dependency.
- 2026-09-23 — The original `gate1/atomic-journal` branch moved repeatedly while this chat was reviewing it, proving another writer was active. Per `AGENTS.md`, writes stopped each time and intervening commits were inspected. Finalization moved to chat-owned branch `gate1/atomic-journal-finalize` from verified head `e3e201d74430aa5aabbb3d9ac583abef39ea5cd2`; PR #6 supersedes PR #4 for review.
- 2026-09-23 — A temporary reconciliation PR (#5) was used only to obtain GitHub's three-way merged tree for non-overlapping recovery/hardening edits; it was closed without merge. The resulting tree was committed as a normal single-parent commit on the finalize branch.

## Validation history

- CI run 136 on `c9f61d935135bd66d84f7bc8af05a050ae938348` — failed rustfmt only.
- CI run 137 on `f55e13aaa01e243d33b831c0e53b7b82291f95ed` — passed implementation/provenance tests and Rust 1.88 MSRV.
- CI run 138 on `419e05955f02433677ce54e2695403159aab4627` — passed fast verification, Clippy, tests, MSRV, genericity, and architecture guard.
- CI run 139 on `561caebd1feb62baf7835c033c1756f7cd6d8a4f` — batching optimization failed rustfmt only.
- CI run 140 on `59c36c69deed4479833371e2d449d5cdc22acff5` — fast verification/MSRV/genericity/architecture passed; Clippy found four explicit-auto-deref warnings; tests were skipped by the hard gate.
- `bb68fdd598da83dd8635b256f5ce61b91f721c86` — fixed exactly those Clippy warnings.
- `e3e201d74430aa5aabbb3d9ac583abef39ea5cd2` — single-snapshot recovery read; verified green before branch isolation.
- CI run 144 on `6cd806a185c9961725ecc7cc8a6beac95500a507` — failed rustfmt only in the new integrity regression file.
- CI run 145 on `596675f87931aa539e7c2ae128b470180eff21ce` — **passed**: fast verification, Clippy, all tests, Rust 1.88 MSRV, genericity, architecture guard.
- final docs-only head CI — pending.

## Risks / remaining debt

- Save/journal schema is a high-impact compatibility surface; do not merge without explicit human approval.
- Snapshot migration/replay, normalized current-state projections, campaign lifecycle/backup/export/purge, and versioned rules/content manifest handling remain Gate 1 blockers after this slice.
- Delete guards intentionally block ordinary cascading campaign deletion once history exists. The later lifecycle implementation must define a controlled whole-campaign purge rather than weakening append-only history.
- SQLite true simultaneous writers may surface lock/contention errors in addition to stale/CAS rejection. Such failures are non-committed and require retry/re-resolution from current state; application-level write scheduling/retry policy remains later work.
- `SerializedRecord` preserves versioned typed-Rust serialization but serialization itself is not proof of authorization; dependency boundaries and application/core call paths remain part of the authority model.

## Next action

Run CI on the docs-only closeout commit. If green and PR #6 head is unchanged, update the PR summary with exact validation evidence and mark it ready for human review. Do not merge until the user explicitly approves this save-format/high-impact slice.
