# Gate 1 — Snapshot migration and replay

Status: in progress
Branch: `gate1/snapshot-replay`
Base: `main` @ `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`
PR: pending

## Objective

Build Gate 1 Slice B: version-aware campaign snapshots and a fail-closed replay path that can reconstruct authoritative state from an earlier snapshot plus ordered journal events without making persistence or providers responsible for gameplay semantics.

## Product-definition traceability

This slice advances the product requirements that campaign truth survives restart and save-format evolution, failures remain recoverable without manual database repair, and meaningful provenance can support replay/recovery. It does not complete player-facing save UX or the overall game.

Relevant durable sources:

- `AGENTS.md`
- `docs/product-definition.md`
- `docs/architecture/002-state-and-events.md`
- `docs/architecture/011-atomic-journal-persistence.md`
- `docs/checkpoints/gate-1.md`
- `crates/dmd-domain/src/state.rs`
- `crates/dmd-domain/src/record.rs`
- `crates/dmd-domain/src/event.rs`
- `crates/dmd-persistence/src/journal_store.rs`

## Scope

- add immutable per-campaign snapshot storage keyed by journal sequence;
- backfill one snapshot at the current materialized head for databases created before this migration;
- initialize new campaigns with a sequence-0 snapshot;
- create later snapshots automatically after a bounded journal interval, inside the same transaction as the accepted material transition;
- define an explicit `CampaignState` snapshot codec/migration chain that rejects unsupported future versions and missing legacy migration paths rather than guessing;
- load the latest snapshot at or before a replay target;
- load replay events from the same SQLite read snapshot and enforce contiguous same-campaign sequence coverage;
- replay through an injected typed event-applier contract so persistence orchestrates recovery but does not own gameplay semantics;
- fail closed on unknown event kind/version, invalid payload/application, identity/schema mutation, journal gaps, missing snapshots, target beyond head, or invalid final state/provenance;
- add tests for migration backfill, sequence-0 snapshot creation, periodic snapshots, old-snapshot replay, unsupported snapshot versions, missing snapshots, unsupported event versions, journal gaps/campaign mismatch, and snapshot immutability;
- document the replay/snapshot compatibility contract and update the Gate 1 checkpoint.

## Non-goals

- defining every future gameplay event kind or its semantic applier before those gameplay systems exist;
- reconstructing historical state earlier than the oldest retained/backfilled snapshot when no suitable snapshot exists;
- normalized query/projection tables and projection rebuilds;
- campaign create/list/archive/delete UX, backup/export/restore, or controlled whole-campaign purge;
- rules/content-pack manifest resolution;
- combat, simulation, Director, voice, or UI work;
- merging this save-format change without a separate explicit human approval.

## Acceptance criteria

- [ ] Existing databases gain an immutable snapshot at their current materialized head during migration.
- [ ] New campaigns persist a sequence-0 snapshot atomically with initialization.
- [ ] Material commits create a new immutable snapshot when the snapshot interval is crossed.
- [ ] Snapshot decode is version-aware and has an explicit sequential migration boundary; unsupported future versions and legacy versions without a registered path fail with distinct errors.
- [ ] Replay selects the latest snapshot at or before the requested target and reads snapshot/head/events consistently.
- [ ] Replay requires contiguous event sequences and same-campaign records through the target.
- [ ] Persistence delegates event semantics to a typed replay applier and fails closed on unsupported kind/version or invalid payload/application.
- [ ] Replay cannot change campaign identity or state schema unexpectedly and returns a domain-valid final state at the requested sequence.
- [ ] Final replayed state provenance is checked against the persisted journal.
- [ ] Missing/too-new/too-old snapshot situations produce explicit recoverable errors rather than silent fallback or invention.
- [ ] Snapshot rows reject direct update/delete; later controlled campaign purge remains the only intended deletion path.
- [ ] Existing Slice A atomicity, journal immutability, session ledger, genericity, architecture guard, and MSRV checks remain green.
- [ ] Full diff is reviewed against ADR 002, ADR 011, Gate 1, and product-definition recovery requirements.
- [ ] `./scripts/verify` equivalent and CI pass on the exact final reviewed head.
- [ ] Human explicitly approves this additional save-format/high-impact slice before merge.

## Planned slices

1. Check in this execution plan and close Slice A bookkeeping. — in progress
2. Add snapshot schema/backfill/immutability and initialization/periodic-write path. — pending
3. Add version-aware snapshot codec and replay orchestration with injected event applier. — pending
4. Add migration/replay failure-mode and recovery tests. — pending
5. Add ADR/checkpoint updates, inspect full diff, run exact-head CI, and hand off for explicit human approval. — pending

## Decisions

- 2026-09-23 — Start from accepted Slice A squash merge `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`; use one dedicated writable branch `gate1/snapshot-replay`.
- 2026-09-23 — Replay semantics will be injected through an application-defined typed applier contract. Persistence may order/read/validate replay inputs but must not infer gameplay meaning from arbitrary JSON.
- 2026-09-23 — Current state schema is version 1. The production codec will establish a sequential migration registry now; because no earlier supported production schema exists yet, legacy versions without a registered migration path will fail explicitly rather than introducing a fake production migration.
- 2026-09-23 — Existing databases will be backfilled with a snapshot at their current materialized head. This preserves forward recovery from the migration point but does not fabricate historical snapshots that never existed.
- 2026-09-23 — Snapshot deletion will be blocked like journal surgery. Complete campaign purge remains a separate controlled lifecycle operation.

## Validation

- Base `main` verified at accepted Slice A merge `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`.
- Implementation validation: pending.

## Risks / blockers

- This slice changes the save-format surface again and therefore requires explicit human approval before merge.
- Current journal events are infrastructure/test events; complete gameplay replay cannot exist until gameplay subsystems define their durable event kinds and typed appliers. This slice must not claim otherwise.
- A database upgraded from Slice A only has a backfilled snapshot at its then-current head, so replay to an earlier historical sequence is unavailable unless an older snapshot actually exists. The API must surface that limit explicitly.
- Snapshot frequency is a persistence performance policy. The initial interval should be bounded and mechanically tested, but may require later tuning from endurance benchmarks.

## Next action

Commit plan/bookkeeping, open a draft PR, then implement immutable snapshot storage/backfill and sequence-0/periodic snapshot writes before adding replay logic.
