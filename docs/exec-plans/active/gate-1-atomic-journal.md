# Gate 1 — Atomic state and event journal

Status: in progress
Branch: `gate1/atomic-journal`
PR: pending
Base: `main` @ `a836ef3f9c001ea6bbec81e191209230bd2293f6`
Verified head: pending

## Objective

Build the first Gate 1 production persistence slice: every accepted material command transition can commit authoritative campaign state, command/resolution provenance, and an append-only event journal atomically in SQLite, with per-campaign sequence allocation and restart/recovery verification.

## Scope

- introduce the SQLite schema for current materialized campaign state, command audit records, immutable journal events, and causal event edges;
- add a typed persistence API that accepts trusted `GameCommand<C>` metadata and typed event payloads rather than provider-shaped JSON;
- validate campaign/state/issuer/session/event-causality boundaries before durable commit;
- allocate contiguous per-campaign event sequences in persistence;
- reject stale commands using `expected_event_sequence`;
- persist the next `CampaignState` plus all event/audit records in one database transaction;
- load the current materialized state and inspect journal records after restart;
- add tests for success, stale-state rejection, invalid-state rejection, causal ordering/campaign isolation, and transaction rollback/reopen behavior;
- document the scoped Gate 1 acceptance contract and any remaining persistence debt.

## Non-goals

- full snapshot-version migration/replay machinery;
- normalized projection tables for every domain aggregate;
- rules/content-pack schema loading;
- combat or gameplay rules;
- world simulation/Director logic;
- voice, local inference, or desktop UI;
- automatic merge to `main`.

The materialized-state row introduced in this slice is a durable recovery/snapshot representation, not a replacement for the normalized query/projection tables required by later Gate 1 work.

## Relevant durable context

- `AGENTS.md`
- `docs/product-definition.md` — save/restart/recovery and production-path requirements
- `docs/architecture/002-state-and-events.md`
- `docs/architecture/009-typed-intent-command-boundary.md`
- `docs/checkpoints/gate-0-review.md` — first Gate 1 blocker
- `crates/dmd-core/src/lib.rs`
- `crates/dmd-domain/src/event.rs`
- `crates/dmd-domain/src/state.rs`
- `crates/dmd-persistence/`

## Acceptance criteria

- [ ] A clean valid campaign state can be initialized durably at event sequence 0.
- [ ] An accepted typed command transition persists command audit metadata, one or more immutable journal events, causal links, and the resulting materialized state atomically.
- [ ] Event sequences are allocated contiguously by persistence and are unique per campaign.
- [ ] A command whose `expected_event_sequence` is stale is rejected without changing state or journal.
- [ ] Invalid next state, campaign mismatch, missing/cross-campaign session, duplicate event IDs, and missing/future/cross-campaign causal parents are rejected.
- [ ] Durable audit records preserve issuer separately from world actor and retain the validated typed payload serialization used for the accepted command.
- [ ] Reopening the SQLite database recovers the last committed materialized state and journal head.
- [ ] Dropping/rolling back an uncommitted transaction does not expose partial authoritative state.
- [ ] Existing session-ledger behavior remains green.
- [ ] `./scripts/verify` equivalent and CI pass on the exact final head.
- [ ] Complete diff is reviewed against ADR 002 and the product-definition persistence/recovery requirements.

## Planned slices

1. Define the persistence schema and typed pending-event/audit boundary.
2. Implement initialize/load/atomic-commit APIs and invariant checks.
3. Add restart, stale-write, causality, and rollback tests.
4. Add Gate 1 checkpoint/ADR clarification only where implementation exposes a durable decision.
5. Run full verification, inspect the complete diff, update this plan and the PR summary.

## Decision log

- 2026-09-23 — Begin from merged Gate 0 `main`; one writable branch for this execution chat.
- 2026-09-23 — The first slice will retain a serialized current-state representation as a durable recovery/snapshot artifact while leaving normalized per-domain projections to a later Gate 1 slice; this avoids a disposable persistence path while keeping the atomicity problem bounded.
- 2026-09-23 — Persistence, not the resolver/provider, owns journal sequence allocation.
- 2026-09-23 — Accepted material mutations must preserve trusted command issuer metadata separately from any in-world actor.

## Validation

- `./scripts/verify-fast` — pending
- `./scripts/verify` — pending
- CI — pending

## Risks / blockers

- Save/journal schema is a high-impact compatibility surface; this branch must not be merged without explicit human approval after review.
- General snapshot migration/replay and normalized current-state projections remain Gate 1 blockers after this slice and must stay explicit.
- SQLite concurrency behavior must not rely on an unchecked read-then-write race; stale-state protection needs a transactional compare-and-set or equivalent guard.

## Next action

Read the current core/domain/persistence code on this branch, then implement the journal schema and atomic commit API in one coherent multi-file change.