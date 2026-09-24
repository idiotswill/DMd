# ADR 012 — Snapshot migration and replay

Status: **Accepted — human-approved 2026-09-24 after implementation in PR #7 and reconfirmed during Gate 1 closeout PR #14.**

## Context

ADR 011 established the atomic authority boundary: accepted material transitions append immutable journal history and advance `campaign_state_current` in one SQLite transaction. That is necessary but not sufficient for long-lived saves.

DMd also needs to survive schema evolution, detect corrupted recovery inputs, reconstruct historical/current state without trusting a damaged materialized row, and keep persistence from becoming a second gameplay rules engine.

The replay path therefore needs explicit compatibility rules. A language model, provider response, arbitrary JSON payload, or damaged current-state row must never be treated as authoritative merely because it can be deserialized.

## Decision

### Immutable sequence-keyed snapshots

Campaign snapshots are stored by `(campaign_id, event_sequence)` and contain the state schema version plus serialized `CampaignState`.

- New campaigns receive a sequence-0 snapshot atomically with state initialization.
- Databases upgraded from the pre-snapshot Slice A schema receive one backfilled snapshot at the materialized head that actually exists at migration time.
- Later snapshots are created in the same authoritative SQLite transaction after at least 100 additional journal events since the latest snapshot.
- Snapshot rows reject direct update/delete. Controlled whole-campaign purge remains a separate lifecycle operation.

The 100-event interval is an initial persistence policy, not a gameplay rule. Endurance measurements may justify changing it later.

### Do not fabricate unavailable history

Migration backfill preserves the state that exists at upgrade time. It does not invent snapshots for earlier event sequences.

If no snapshot exists at or before a requested replay target, replay fails explicitly. Historical reconstruction earlier than the oldest available snapshot is unavailable unless a real compatible snapshot exists.

### Version-aware snapshot decoding

`CampaignStateSnapshotCodec` owns explicit sequential state-schema migration registration.

- A snapshot newer than the engine-supported state schema fails as an unsupported future version.
- An older snapshot without every required migration step fails as a missing migration path.
- Migration failure, decode failure, schema mismatch, and domain-invalid state are distinct failure classes.
- Duplicate migration registration is rejected without replacing the already-registered migration.

No compatibility path is guessed from JSON shape.

### Replay head comes from the append-only journal

Replay-to-head derives the target from the campaign journal's contiguous `COUNT == MAX(sequence)` relationship, not from `campaign_state_current.applied_event_sequence`.

`campaign_state_current` is used to establish that the campaign exists, but replay does not trust its serialized state or recorded head as recovery truth. This allows snapshot+journal recovery when the materialized current-state artifact itself is damaged.

### One consistent SQLite read snapshot

Snapshot selection, journal-head/prefix checks, event loading, causal metadata, and state-held provenance checks occur through one SQLite read transaction.

Replay requires:

1. a contiguous journal prefix through the requested target;
2. the latest available snapshot at or before that target;
3. snapshot sequence and campaign identity to match their storage metadata;
4. snapshot-held event provenance to resolve to the same campaign at or before the snapshot sequence;
5. replay events to be contiguous and ordered by per-campaign sequence;
6. causal parents to precede their children;
7. final state-held event provenance to resolve to the same campaign at or before the target.

The public direct snapshot loader applies the same journal-prefix and provenance checks before returning an authoritative-looking `CampaignState`.

### Persistence orchestrates replay; gameplay code owns event semantics

Persistence reconstructs durable `StoredJournalEvent` records and delegates semantic application to `ReplayEventApplier`.

Persistence does not infer gameplay meaning from event kind strings or payload JSON. A gameplay subsystem that defines a durable event kind/version must also provide the typed replay behavior for that event family.

The applier must fail closed on unsupported event kinds/versions, malformed payloads, or invalid transitions. Replay additionally rejects any applier that changes campaign identity or the state schema version unexpectedly.

### Final state must still satisfy domain invariants

After each applied event, replay advances only the persisted journal sequence. At the target, the resulting state must:

- identify the requested campaign;
- use the current supported state schema;
- report the requested applied event sequence;
- satisfy `CampaignState` domain validation;
- contain only valid same-campaign event provenance at or before the target.

A replay result is not authoritative merely because an applier returned `Ok(())`.

## Compatibility guarantees

This slice guarantees a mechanical recovery/migration boundary, not universal historical compatibility.

- Slice B databases have immutable recovery anchors and explicit schema-version handling.
- Slice A databases upgraded to Slice B can replay forward from the backfilled upgrade-time snapshot, but cannot reconstruct earlier history without an older real snapshot.
- Unsupported future snapshot versions and missing legacy migration paths fail explicitly rather than silently downgrading or guessing.
- Unknown gameplay event versions fail until the owning gameplay subsystem supplies a compatible typed applier.
- The current-state JSON row remains a durable materialized artifact, not the permanent query architecture.

## Rejected alternatives

### Trust `campaign_state_current` as replay head

Rejected. A damaged materialized row would then prevent the replay system from recovering the artifact it is intended to repair or diagnose.

### Let persistence interpret arbitrary event JSON

Rejected. That would duplicate gameplay semantics inside persistence and make save compatibility depend on stringly typed inference.

### Silently accept structurally valid snapshots

Rejected. A snapshot must also be backed by a contiguous journal prefix and valid provenance before it can be returned as authoritative state.

### Fabricate pre-migration snapshots

Rejected. Historical state that was never durably captured must remain unavailable rather than invented.

### Make snapshots mutable for convenience

Rejected. Recovery anchors must not become an alternate path for rewriting accepted campaign history.

## Consequences and remaining work

- Every durable gameplay event family must eventually define stable kind/version semantics and a typed replay applier.
- Normalized gameplay projections and projection rebuilds remain a later Gate 1 slice.
- Campaign backup/export/restore and controlled whole-campaign purge remain lifecycle work.
- Rules/content-pack manifest compatibility remains a separate Gate 1 concern.
- Snapshot interval and replay performance require endurance measurement before final tuning.
- This ADR extends the save-format compatibility surface and was accepted only after explicit human approval of the implemented PR #7 behavior, reconfirmed during final Gate 1 closeout.
