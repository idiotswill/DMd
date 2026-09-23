# ADR 013 — Derivative query projections

Status: proposed for Gate 1

## Context

`CampaignState` JSON is a durable materialized recovery artifact, not the permanent gameplay query architecture. Gate 1 already has an append-only command/event journal, immutable recovery snapshots, and replay through typed event appliers. The remaining persistence blocker is a normalized current-state query layer that can answer gameplay-oriented lookups without decoding the whole aggregate while preserving the existing authority and recovery model.

The current domain model already contains campaign/clock metadata, players, characters, world entities, factions, locations, scenes and presences, standing directives, item instances, facts, claims, beliefs, and current knowledge relations. Combat state, simulation scheduling, economy, procedural world-generation indexes, opportunity systems, and UI-specific views do not yet have stable domain contracts and must not be guessed into persistence.

## Decision

### Authority

Projection rows are derivative indexes only. They never authorize gameplay transitions and they are never replay input. Authoritative recovery remains the accepted snapshot plus contiguous append-only journal path defined by ADR 012, with `campaign_state_current` retained as the current materialized recovery artifact.

Every campaign projection has a `projection_heads` row containing the campaign id, state schema version, applied event sequence, and expected row counts for each projected record family, including exactly one campaign/clock metadata row. Query entrypoints verify that the projection head matches the authoritative materialized head and that the record-family counts still match before returning projected results. Missing, stale, or count-corrupt projection state fails closed.

### Projected record families

Gate 1 projects only records that exist in `CampaignState` today:

- campaign and world-clock metadata;
- players and characters;
- world entities, factions, and locations;
- scenes and scene presences;
- item instances;
- facts, claims, beliefs, and knowledge relations;
- standing directives.

The projection schema exposes stable relational identity, campaign ownership, lifecycle/status, location/relationship, time, predicate, and other currently queryable fields. Each record row also keeps that record's JSON representation for fields whose internal enum/value structure is not yet a proven relational query requirement. This does not make serialized `CampaignState` the query architecture: records are independently addressable and indexed, and additional columns can be promoted when real query workloads require them.

Future combat, simulation, economy, world-generation, opportunity, and UI schemas remain deferred until their domain contracts exist.

### Atomic maintenance

SQLite triggers derive the complete projection image whenever valid `campaign_state_current` state is inserted or updated. The triggers execute in the same SQLite transaction/statement context as authoritative campaign initialization or an accepted journal transition. A projection constraint or write failure therefore aborts the authoritative change rather than committing projection drift.

Malformed materialized JSON is treated differently because that row is itself a recoverable artifact, not authoritative history: the projection image is invalidated without parsing the malformed JSON so snapshot+journal replay remains able to repair it. Existing databases are backfilled by touching the already-accepted `campaign_state_current.state_json` during the projection migration after the triggers exist. Valid rows derive projections; corrupt rows remain projection-less until replay recovery. Neither path appends, deletes, or rewrites command/event history. Snapshot creation remains governed by the existing sequence-advance policy.

Whole-image replacement is deliberate for the current Gate 1 scale because it makes drift prevention and deletion semantics mechanically simple. Incremental projectors may replace it later only after measured gameplay volume justifies the additional complexity and equivalent atomic/rebuild guarantees are preserved.

### Isolation

Every projection primary key is campaign-scoped. Projection tables reference the campaign's projection head, and query APIs always constrain by campaign id. Cross-campaign identifiers cannot satisfy a query merely because the same record kind exists elsewhere.

### Rebuild and corruption recovery

`rebuild_campaign_projections` records the current materialized sequence, reconstructs state through the accepted snapshot+journal replay boundary, and then replaces the materialized recovery row only if its sequence is unchanged from the value observed before replay. That compare-and-swap condition prevents a concurrent accepted transition from being overwritten while still allowing replay to repair a stale or corrupted materialized sequence. The SQLite projection trigger regenerates the derivative image from the replayed state.

Rebuild never edits command audit rows, event journal rows, causal edges, or immutable snapshots. If the materialized row moved while rebuild was prepared, rebuild fails and must be retried against the newer state.

## Consequences

- Gameplay query code has a normalized/indexed path that does not require whole-aggregate JSON decoding.
- Projection corruption can be detected and repaired from authoritative history rather than becoming a second source of truth.
- Projection maintenance participates in the same failure boundary as accepted state/journal transitions.
- Campaign isolation is represented directly in schema and query keys.
- The initial implementation pays a whole-image replacement cost on material transitions; this is an explicit Gate 1 tradeoff, not a claim that the strategy is optimal at future simulation scale.
- Enum/value substructure that has no demonstrated query need remains record-local JSON rather than premature schema.

## Rejected alternatives

- **Keep querying `campaign_state_current.state_json`.** This preserves a monolithic serialized query architecture and weakens relational constraints/indexing.
- **Treat projections as authoritative state.** This creates competing truth and undermines replay/recovery invariants.
- **Update projections asynchronously after commit.** This introduces accepted states with stale query views and requires a second consistency protocol.
- **Model future gameplay subsystems now.** Their contracts do not exist yet; persistence would encode assumptions the domain has not accepted.
