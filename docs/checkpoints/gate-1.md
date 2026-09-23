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

Slice A establishes the trustworthy atomic persistence primitive. It does **not** complete save-format evolution, replay, projections, or campaign lifecycle management.

## Slice B — snapshot migration and replay

Status: **Implementation complete on PR #7; exact-head closeout CI and explicit human merge approval remain required**

Implemented properties:

- immutable per-campaign snapshots keyed by journal sequence;
- migration-time backfill of a snapshot at the current materialized head for pre-Slice-B databases without fabricating older history;
- sequence-0 snapshot creation for new campaigns;
- bounded periodic snapshot creation inside the authoritative transaction path after at least 100 events since the latest snapshot;
- explicit version-aware `CampaignState` snapshot codec/migration chaining;
- distinct failure behavior for unsupported future snapshot versions and legacy versions lacking a migration path;
- replay from the latest snapshot at or before a target sequence using contiguous same-campaign journal events;
- replay-to-head derives its target from append-only journal history rather than trusting the mutable current-state head;
- event semantics are delegated to an injected typed applier contract rather than inferred by persistence from arbitrary JSON;
- fail-closed handling of unsupported event kinds/versions, invalid payloads/transitions, sequence gaps, target/head mismatch, campaign identity changes, schema changes, missing snapshots, and invalid provenance;
- direct snapshot loads validate their selected journal prefix and state-held provenance before returning authoritative-looking state;
- snapshot rows reject direct update/delete;
- restart/recovery, migration, corrupted-current-state, journal-gap, missing/future-snapshot, malicious-applier, and forged-snapshot tests exercise the real persistence path;
- implementation head `73d07f7af8730e6eba27aebeb18d43e6d978229d` passed CI run 156 across fast verification, Clippy, tests, Rust 1.88 MSRV, genericity, and architecture boundaries;
- the compatibility/recovery decision is documented in `docs/architecture/012-snapshot-migration-and-replay.md`.

Slice B does not claim complete gameplay replay until gameplay subsystems have defined their durable event kinds/versions and typed appliers. It also does not provide automatic write-back repair of a damaged current-state row; this slice reconstructs/validates state without silently mutating recovery evidence.

Before Slice B may merge, the documentation-complete PR head must pass exact-head CI and a human must explicitly approve this additional save-format/high-impact change.

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

## Gate 1 completion review

Gate 1 must not be marked complete until the remaining blockers above are implemented and verified through the real persistence path. The eventual Gate 1 review must include restart/recovery, migration/replay, campaign isolation/lifecycle, versioned manifest handling, and full diff/CI evidence.
