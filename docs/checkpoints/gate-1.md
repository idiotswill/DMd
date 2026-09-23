# Gate 1 — Campaign persistence foundation

Status: **In progress**

Gate 1 turns the Gate 0 state model into a durable multi-session campaign foundation. Passing one persistence slice does not complete Gate 1 or make DMd a playable game.

## Product-definition traceability

Gate 1 advances the requirements that campaign truth survives save/exit/restart, failures do not require manual database repair, unrelated campaigns remain isolated, and later gameplay has one recoverable authoritative persistence path.

Voice/table UX, complete rules, living-world simulation, procedural materialization, Director behavior, desktop UX, and endurance play remain later gates.

## Slice A — atomic authoritative commit and journal

PR #4 is responsible for this slice.

Acceptance requires:

- a valid clean campaign can be initialized at journal sequence 0;
- material transitions commit resulting state, trusted command audit, immutable events, and causal edges atomically;
- persistence—not resolvers/providers—allocates contiguous per-campaign event sequences;
- stale `expected_event_sequence` values cannot overwrite newer state;
- campaign/session/issuer/actor references are validated before commit;
- causal parents must exist in the same campaign and precede their child;
- event provenance referenced by materialized state cannot dangle or cross campaigns;
- trusted command issuer is persisted separately from optional in-world actor;
- restart/reopen recovers the last committed state and rejects detectable state/journal corruption;
- the conversation/core/rules layers have no direct SQLite/persistence write dependency;
- existing session-ledger behavior remains green;
- full repository verification and CI pass on the exact reviewed head.

Completion of Slice A means the production path has a trustworthy atomic persistence primitive. It does **not** mean save-format evolution or replay is complete.

## Remaining Gate 1 blockers after Slice A

### Snapshot migration and replay

- define explicit codecs/migrations between future materialized-state schema versions;
- load a snapshot at sequence N and replay later events safely;
- verify recovery when snapshots are old, missing, or rejected;
- define compatibility/error behavior for unsupported future/legacy save versions.

### Query/materialized projections

- add normalized current-state/projection tables where gameplay query performance and constraints require them;
- update projections inside the same transaction as journal append/head advancement;
- prove projection rebuild/recovery behavior rather than treating JSON as the permanent query architecture.

### Campaign lifecycle

- durable create/open/list/archive/delete flows for multiple unrelated campaigns;
- deliberate aggregate deletion semantics that do not permit partial historical surgery;
- backup/export/restore strategy before destructive migrations or lifecycle operations.

### Versioned rules/content manifest

- resolve and validate the ruleset/content-pack IDs and versions stored by the campaign;
- define missing/incompatible pack behavior;
- keep licensing/content data separate from engine/save semantics.

## Gate 1 completion review

Gate 1 must not be marked complete until the remaining blockers above are implemented and verified through the real persistence path. The eventual Gate 1 review must include restart/recovery, migration/replay, campaign isolation/lifecycle, versioned manifest handling, and full diff/CI evidence.
