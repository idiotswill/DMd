# Gate 1 — Campaign persistence foundation

Status: **In progress**

Gate 1 turns the Gate 0 state model into a durable multi-session campaign foundation. Passing one persistence slice does not complete Gate 1 or make DMd a playable game.

## Product-definition traceability

Gate 1 advances the requirements that campaign truth survives save/exit/restart, failures do not require manual database repair, unrelated campaigns remain isolated, and later gameplay has one recoverable authoritative persistence path.

Voice/table UX, complete rules, living-world simulation, procedural materialization, Director behavior, desktop UX, and endurance play remain later gates.

## Slice A — atomic authoritative commit and journal

Status: **Accepted and merged**

PR #6 was explicitly approved and merged to `main` as squash commit `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`. PR #4 was the original draft path and was closed as superseded.

Accepted properties:

- valid clean campaign initialization at journal sequence 0;
- atomic material transition commit of resulting state, trusted command audit, immutable events, and causal edges;
- persistence-owned contiguous per-campaign event sequences;
- stale-state rejection;
- campaign/session/issuer/actor validation;
- same-campaign earlier-event causal constraints;
- fail-closed state-held event provenance validation;
- recovery reads state/head/provenance from one SQLite snapshot;
- later transitions cannot silently heal corrupt current-state provenance;
- trusted command issuer persists separately from optional in-world actor;
- direct update/delete of command/event/causal history is rejected at the database layer;
- conversation/core/rules layers have no direct SQLite/persistence write dependency;
- full repository verification passed on the exact approved PR head.

## Slice B — snapshot migration and replay

Status: **Accepted and merged**

PR #7 was explicitly approved and squash-merged to `main` as `40a7cf001f51028a98195ada28e2e1f7dbd5af84`. Exact reviewed PR head `2e0b2eb931a7586b573275de20e9077c53676633` passed CI run 157.

Accepted properties:

- immutable per-campaign snapshots keyed by journal sequence;
- migration-time backfill at the current materialized head without fabricating older history;
- sequence-0 snapshot creation for new campaigns;
- bounded periodic snapshot creation inside the authoritative transaction path;
- explicit version-aware `CampaignState` snapshot codec/migration chaining;
- replay from the latest snapshot at or before a target using contiguous same-campaign journal events;
- replay-to-head derives its target from append-only journal history rather than trusting mutable current-state metadata;
- gameplay event semantics are delegated to typed appliers rather than inferred by persistence from arbitrary JSON;
- fail-closed handling for unsupported versions, invalid transitions, journal gaps, missing snapshots, identity/schema mutation, and invalid provenance;
- direct snapshot loads validate their journal prefix and state-held provenance before returning authoritative-looking state;
- snapshot rows reject direct update/delete;
- restart/recovery and corruption/failure-mode tests exercise the real persistence path;
- ADR 012 records the compatibility and replay contract.

Slice B does not claim complete gameplay replay until gameplay subsystems define their durable event kinds/versions and typed appliers.

## Remaining Gate 1 blockers after Slice B

### Query/materialized projections

- add normalized current-state/projection tables where gameplay query performance and constraints require them;
- update projections inside the same transaction as journal append/head advancement;
- prove projection rebuild/recovery behavior rather than treating JSON as the permanent query architecture.

### Campaign lifecycle

- durable create/open/list/archive/delete flows for multiple unrelated campaigns;
- deliberate aggregate deletion semantics that temporarily/explicitly bypass history delete guards only for a complete authorized campaign purge;
- backup/export/restore strategy before destructive migrations or lifecycle operations.

### Versioned rules/content manifest

- resolve and validate the ruleset/content-pack IDs and versions stored by the campaign;
- define missing/incompatible pack behavior;
- keep licensing/content data separate from engine/save semantics.

## Parallel execution boundary

The three remaining Gate 1 blockers may proceed in parallel only on separate branches with one writable chat/agent per branch. Each worker must create and maintain its own execution plan and must not merge without control/review coordination.

## Gate 1 completion review

Gate 1 must not be marked complete until all three remaining blockers are implemented and verified through the real persistence path. The eventual Gate 1 review must include restart/recovery, migration/replay, projection rebuild behavior, campaign isolation/lifecycle, versioned manifest handling, and full diff/CI evidence.
