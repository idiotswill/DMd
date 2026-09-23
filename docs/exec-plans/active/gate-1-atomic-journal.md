# Gate 1 — Atomic state and event journal

Status: in progress
Branch: `gate1/atomic-journal`
PR: #4
Base: `main` @ `a836ef3f9c001ea6bbec81e191209230bd2293f6`
Verified implementation head: `f55e13aaa01e243d33b831c0e53b7b82291f95ed` (CI run 137 — pass)

## Objective

Build the first Gate 1 production persistence slice: every accepted material command transition can commit authoritative campaign state, command/resolution provenance, and an append-only event journal atomically in SQLite, with per-campaign sequence allocation and restart/recovery verification.

## Scope

- SQLite schema for current materialized campaign state, command audit records, immutable journal events, and causal event edges;
- typed pending-event/audit boundary rather than provider-shaped persistence input;
- campaign/state/issuer/session/event-causality and state-held provenance validation;
- persistence-owned contiguous event sequence allocation;
- stale command rejection plus compare-and-set head advancement;
- next `CampaignState` plus audit/events/causal edges in one transaction;
- current-state and journal loading after restart, including head/provenance integrity checks;
- tests for success, stale-state rejection, initialization/recovery provenance, causal ordering/campaign isolation, authority metadata, and rollback/reopen behavior;
- mechanical dependency guard preventing interpretation/core/rules layers from acquiring a direct persistence/SQLx write path;
- scoped Gate 1 checkpoint and atomic-journal ADR.

## Non-goals

- full snapshot-version migration/replay machinery;
- normalized projection tables for every domain aggregate;
- complete campaign lifecycle/backup/export flows;
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
- [x] Existing session-ledger behavior remains green on the verified implementation head.
- [ ] Recovery revalidates state-held event references after restart/corruption simulation.
- [ ] Architecture dependency guard is checked by full CI.
- [ ] Proposed ADR/checkpoint wording is reviewed against the implementation.
- [ ] `./scripts/verify` equivalent and CI pass on the exact final head.
- [ ] Complete diff is reviewed against ADR 002, ADR 011, and product-definition persistence/recovery requirements.

## Planned slices

1. Define the persistence schema and typed pending-event/audit boundary. — complete
2. Implement initialize/load/atomic-commit APIs and invariant checks. — complete
3. Add restart, stale-write, causality, provenance, authority, and rollback tests. — implementation complete; final CI pending after review hardening
4. Add the scoped Gate 1 checkpoint/ADR and mechanical boundary guard. — in progress
5. Run final verification, inspect complete diff, update plan/PR, and hand off for human merge approval. — pending

## Decision log

- 2026-09-23 — Begin from merged Gate 0 `main`; one writable branch for this execution chat.
- 2026-09-23 — Retain a serialized current-state representation as a durable recovery/snapshot artifact while leaving normalized per-domain projections to later Gate 1 work.
- 2026-09-23 — Persistence, not the resolver/provider, owns journal sequence allocation.
- 2026-09-23 — Accepted material mutations preserve trusted command issuer metadata separately from any in-world actor.
- 2026-09-23 — Sequence-0 initialization rejects state records that cite nonexistent event provenance.
- 2026-09-23 — CI run 136 failed only at rustfmt; the requested formatting was applied without weakening checks.
- 2026-09-23 — CI run 137 passed the implementation/provenance head, including tests and Rust 1.88 MSRV.
- 2026-09-23 — Recovery should recheck state-held journal provenance, not merely JSON/domain shape and journal count/head.
- 2026-09-23 — Enforce direct persistence dependency boundaries in CI so conversation/core/rules cannot silently grow SQLite write authority.

## Validation

- CI run 136 on `c9f61d935135bd66d84f7bc8af05a050ae938348` — failed at rustfmt only; no compiler/test evidence claimed from that run.
- CI run 137 on `f55e13aaa01e243d33b831c0e53b7b82291f95ed` — pass: fast verification, Clippy, tests, Rust 1.88 MSRV, genericity guard.
- final head CI — pending after recovery/boundary/doc hardening.

## Risks / blockers

- Save/journal schema is a high-impact compatibility surface; this branch must not be merged without explicit human approval after final review.
- Snapshot migration/replay, normalized current-state projections, campaign lifecycle/backup, and versioned rules/content manifest handling remain Gate 1 blockers after this slice.
- SQLite true simultaneous writers may surface lock/contention errors in addition to stale/CAS rejection. Such failures are non-committed and require retry/re-resolution from current state; application-level write scheduling/retry policy remains later work.
- `SerializedRecord` preserves versioned typed-Rust serialization but serialization itself is not proof of authorization; dependency boundaries and application/core call paths remain part of the authority model.

## Next action

Commit recovery-time provenance validation, architecture boundary guard, ADR/checkpoint docs, run exact-head CI, inspect the complete PR diff/review state, then close this plan and update PR #4 for explicit human approval.