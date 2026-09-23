# Gate 1 — Snapshot migration and replay

Status: ready for human approval once the documentation-complete PR head passes exact-head CI
Branch: `gate1/snapshot-replay`
Base: `main` @ `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`
PR: #7

## Objective

Build Gate 1 Slice B: version-aware campaign snapshots and a fail-closed replay path that can reconstruct authoritative state from an earlier snapshot plus ordered journal events without making persistence or providers responsible for gameplay semantics.

## Product-definition traceability

This slice advances the product requirements that campaign truth survives restart and save-format evolution, failures remain recoverable without manual database repair, and meaningful provenance can support replay/recovery. It does not complete player-facing save UX or the overall game.

Relevant durable sources:

- `AGENTS.md`
- `docs/product-definition.md`
- `docs/architecture/002-state-and-events.md`
- `docs/architecture/011-atomic-journal-persistence.md`
- `docs/architecture/012-snapshot-migration-and-replay.md`
- `docs/checkpoints/gate-1.md`
- `crates/dmd-domain/src/state.rs`
- `crates/dmd-domain/src/record.rs`
- `crates/dmd-domain/src/event.rs`
- `crates/dmd-persistence/src/journal_store.rs`
- `crates/dmd-persistence/src/snapshot_replay.rs`

## Scope

- immutable per-campaign snapshot storage keyed by journal sequence;
- migration-time backfill at the materialized head that actually exists;
- sequence-0 snapshot initialization for new campaigns;
- bounded periodic snapshot creation inside the authoritative transaction path;
- explicit version-aware `CampaignState` snapshot codec/migration chaining;
- latest-snapshot-at-or-before-target selection;
- same-read-snapshot journal/provenance validation and contiguous replay;
- injected typed replay event-applier boundary so persistence orchestrates recovery without owning gameplay semantics;
- fail-closed unsupported versions, invalid applications, identity/schema mutation, journal corruption, unavailable snapshots, and invalid provenance;
- recovery/migration/replay regression coverage;
- durable compatibility ADR and Gate 1 checkpoint update.

## Non-goals

- defining every future gameplay event kind or semantic applier before those gameplay systems exist;
- reconstructing history earlier than the oldest real compatible snapshot;
- normalized query/projection tables and rebuilds;
- campaign create/list/archive/delete UX, backup/export/restore, or controlled whole-campaign purge;
- rules/content-pack manifest resolution;
- combat, simulation, Director, voice, or UI work;
- merging this save-format change without a separate explicit human approval.

## Acceptance criteria

- [x] Existing databases gain an immutable snapshot at their current materialized head during migration.
- [x] New campaigns persist a sequence-0 snapshot atomically with initialization.
- [x] Material commits create a new immutable snapshot when the snapshot interval is crossed.
- [x] Snapshot decode is version-aware and has an explicit sequential migration boundary; unsupported future versions and legacy versions without a registered path fail with distinct errors.
- [x] Replay selects the latest snapshot at or before the requested target and reads snapshot/head/events consistently.
- [x] Replay requires contiguous event sequences and same-campaign records through the target.
- [x] Persistence delegates event semantics to a typed replay applier and fails closed on unsupported kind/version or invalid payload/application.
- [x] Replay cannot change campaign identity or state schema unexpectedly and returns a domain-valid final state at the requested sequence.
- [x] Snapshot and final replay provenance are checked against persisted journal history.
- [x] Direct snapshot loads verify their selected journal prefix/provenance before returning authoritative-looking state.
- [x] Missing/too-new/too-old snapshot situations produce explicit errors rather than silent fallback or invention.
- [x] Snapshot rows reject direct update/delete; later controlled campaign purge remains the intended deletion path.
- [x] Existing Slice A atomicity, journal immutability, session ledger, genericity, architecture guard, and MSRV behavior remain green on the implementation head.
- [x] Full PR diff and review-thread state were inspected against ADR 002, ADR 011, Gate 1, and the recovery requirements; no review-thread blockers were present.
- [ ] Documentation-complete PR head passes full exact-head CI before approval handoff. This plan is intentionally not mutated solely to record that later CI result because doing so would move the reviewed head; the PR summary records the exact final head/run.
- [ ] Human explicitly approves this additional save-format/high-impact slice before merge.

## Planned slices

1. Check in this execution plan and close Slice A bookkeeping. — complete
2. Add snapshot schema/backfill/immutability and initialization/periodic-write path. — complete
3. Add version-aware snapshot codec and replay orchestration with injected event applier. — complete
4. Add migration/replay corruption, compatibility, and recovery tests. — complete
5. Add ADR/checkpoint updates, inspect full diff, run exact-head CI, and hand off for explicit human approval. — implementation/docs complete; exact-head closeout CI then human approval remain

## Decisions

- 2026-09-23 — Start from accepted Slice A squash merge `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`; use one dedicated writable branch `gate1/snapshot-replay`.
- 2026-09-23 — Replay semantics are injected through an application-defined typed applier contract. Persistence may order/read/validate replay inputs but must not infer gameplay meaning from arbitrary JSON.
- 2026-09-23 — Current state schema is version 1. The production codec establishes a sequential migration registry; because no earlier supported production schema exists yet, legacy versions without a registered migration path fail explicitly rather than introducing a fake production migration.
- 2026-09-23 — Existing databases are backfilled with a snapshot at their current materialized head. This preserves forward recovery from the migration point but does not fabricate historical snapshots that never existed.
- 2026-09-23 — Snapshot update/delete is blocked like journal surgery. Complete campaign purge remains a separate controlled lifecycle operation.
- 2026-09-23 — Replay-to-head derives its target from contiguous append-only journal history rather than `campaign_state_current.applied_event_sequence`, allowing recovery when the materialized current-state artifact is damaged.
- 2026-09-23 — Direct snapshot loading validates the selected journal prefix and state-held event provenance before returning `CampaignState`; structural deserialization alone is not authority.
- 2026-09-23 — Snapshot creation interval is initially 100 material events and is treated as a tunable persistence policy, not a gameplay invariant.

## Validation history

- Base `main` verified at accepted Slice A merge `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`.
- CI run 150 — rustfmt failed; MSRV also exposed a lockfile update caused by a newly-added test-only dependency. The dependency was removed rather than changing the production graph.
- CI run 151 — fast verification/MSRV/architecture/genericity passed; Clippy found one API naming warning and tests were skipped by the hard gate.
- CI run 152 — full suite passed after renaming the migration API to `source_version()`.
- Fresh review found replay-to-head trusted the mutable current-state head; this was changed to derive the target from the append-only journal and corruption tests were added.
- CI run 153 — failed rustfmt only on recovery-hardening changes.
- CI run 154 on `4d7f8144cc82e62dab07632b042f73bbc93acf1a` — full suite passed.
- Fresh full-diff review found direct snapshot loading did not prove journal prefix/provenance integrity; the loader was hardened and a forged-snapshot regression was added.
- CI run 155 — failed rustfmt only on the new direct-snapshot integrity guard.
- CI run 156 on `73d07f7af8730e6eba27aebeb18d43e6d978229d` — **passed** fast verification, Clippy, full tests including direct snapshot integrity, Rust 1.88 MSRV, genericity, and architecture boundaries.
- PR #7 review comments and inline review threads were checked after run 156; none were present.
- Documentation-complete exact-head CI: required next; final evidence will be recorded in the PR summary without moving the branch solely to restate CI metadata.

## Risks / unresolved debt

- This slice changes the save-format compatibility surface and therefore still requires explicit human approval before merge.
- Current durable journal events are infrastructure/test events; complete gameplay replay cannot exist until gameplay subsystems define their event kinds/versions and typed appliers.
- A database upgraded from Slice A has only the snapshot backfilled at its upgrade-time head; replay to earlier history is unavailable unless an older real snapshot exists.
- The 100-event snapshot interval may need tuning from endurance benchmarks.
- Normalized projections/rebuilds, campaign lifecycle/backup/export/purge, and versioned rules/content manifest resolution remain Gate 1 blockers after Slice B.
- Snapshot+journal replay diagnoses/reconstructs authoritative state; automatic repair/write-back of a damaged `campaign_state_current` row is intentionally not added in this slice.

## Next action

Run full CI on the documentation-complete PR head. If and only if that exact head is green, update the PR summary/ready-for-review state without changing the branch and request explicit human approval for PR #7. Immediately before any later merge, re-fetch the unchanged PR head and verify CI is still green on that exact SHA.
