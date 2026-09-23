# Gate 1 — Snapshot migration and replay

Status: completed and merged
Branch: `gate1/snapshot-replay`
PR: #7
Base: `main` @ `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`
Verified PR head: `2e0b2eb931a7586b573275de20e9077c53676633` (CI run 157 — pass)
Merged to `main`: `40a7cf001f51028a98195ada28e2e1f7dbd5af84`

## Objective

Build Gate 1 Slice B: version-aware campaign snapshots and a fail-closed replay path that can reconstruct authoritative state from an earlier snapshot plus ordered journal events without making persistence or providers responsible for gameplay semantics.

## Scope completed

- immutable per-campaign snapshot storage keyed by journal sequence;
- migration-time backfill at the materialized head that actually exists;
- sequence-0 snapshot initialization for new campaigns;
- bounded periodic snapshot creation inside the authoritative transaction path;
- explicit version-aware `CampaignState` snapshot codec/migration chaining;
- latest-snapshot-at-or-before-target selection;
- same-read-snapshot journal/provenance validation and contiguous replay;
- injected typed replay event-applier boundary;
- fail-closed unsupported versions, invalid applications, identity/schema mutation, journal corruption, unavailable snapshots, and invalid provenance;
- recovery/migration/replay regression coverage;
- ADR 012 and Gate 1 checkpoint updates.

## Acceptance results

- [x] Existing databases gain an immutable snapshot at their current materialized head during migration.
- [x] New campaigns persist a sequence-0 snapshot atomically with initialization.
- [x] Material commits create a new immutable snapshot when the snapshot interval is crossed.
- [x] Snapshot decode is version-aware with explicit sequential migration boundaries.
- [x] Replay selects the latest snapshot at or before the requested target and requires contiguous same-campaign history.
- [x] Persistence delegates event semantics to a typed replay applier and fails closed on unsupported/invalid events.
- [x] Replay rejects campaign identity/schema mutation and invalid final state/provenance.
- [x] Direct snapshot loads verify journal prefix/provenance before returning state.
- [x] Snapshot rows reject direct update/delete.
- [x] Exact reviewed head passed CI run 157 across fast verification, Clippy, tests, Rust 1.88 MSRV, genericity, and architecture boundaries.
- [x] Human explicitly approved the save-format/high-impact slice.
- [x] PR #7 merged to `main` as `40a7cf001f51028a98195ada28e2e1f7dbd5af84`.

## Important decisions

- Persistence orchestrates replay but does not infer gameplay meaning from arbitrary JSON.
- Replay-to-head derives its target from append-only journal history rather than mutable current-state metadata.
- Structural snapshot deserialization is not sufficient authority; journal-prefix and provenance checks are required.
- Upgrade-time backfill preserves only state that actually exists; older history is not fabricated.
- Snapshot rows are immutable; controlled whole-campaign purge remains later lifecycle work.
- The initial 100-event snapshot interval is a tunable persistence policy, not a gameplay rule.

## Validation history

- CI run 150 — rustfmt/dependency-lock issue; test-only dependency removed.
- CI run 151 — Clippy API naming issue; corrected without weakening checks.
- CI run 152 — full suite passed.
- Fresh review found replay-to-head trusted mutable current-state head; corrected with corruption regressions.
- CI run 153 — rustfmt only.
- CI run 154 — full suite passed.
- Fresh full-diff review found direct snapshot loading did not prove journal prefix/provenance integrity; corrected with forged-snapshot regression.
- CI run 155 — rustfmt only.
- CI run 156 — full suite passed on implementation head.
- CI run 157 on `2e0b2eb931a7586b573275de20e9077c53676633` — full suite passed on the documentation-complete approved head.

## Remaining Gate 1 debt handed forward

- normalized/query projections and rebuild behavior;
- campaign lifecycle/backup/export/restore/controlled purge;
- versioned rules/content manifest resolution;
- concrete durable gameplay event kinds/versions and typed replay appliers as gameplay subsystems arrive;
- endurance tuning of snapshot interval/replay performance.
