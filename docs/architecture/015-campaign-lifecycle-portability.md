# ADR 015: Campaign lifecycle and portability

Status: Proposed — pending human review and explicit approval.

## Context

DMd needs durable lifecycle operations for multiple unrelated campaigns without weakening the append-only journal, immutable snapshot, replay, or derivative-projection guarantees established by ADRs 011, 012, and 013.

A campaign must be creatable, discoverable, archivable, reopened after restart, exported for recovery/portability, restored safely, and intentionally destroyed without manual database surgery. Ordinary history deletion must remain forbidden. Restore must not admit persisted audit/event data that normal persistence could not safely replay.

This ADR covers lifecycle persistence. Content-manifest resolution remains a separate Gate 1 integration boundary governed by ADR 014.

## Decision

### Lifecycle metadata is derivative operational metadata

Each durable campaign has one `campaign_lifecycle` row keyed by `CampaignId`. It records display name, storage status (`active` or `archived`), current state schema version, and lifecycle timestamps.

Authoritative gameplay state remains accepted campaign state plus append-only command/event history and immutable snapshots. Lifecycle metadata is not gameplay authority. State writes mechanically keep lifecycle display/schema metadata synchronized.

### Archive is the safe default removal operation

Normal removal is archive. Archive changes only lifecycle availability metadata and does not rewrite domain `CampaignStatus`, journal, commands, causal links, snapshots, sessions, or derivative query projections.

### Purge is an explicit whole-aggregate administrative operation

Permanent deletion is named `purge` and requires an admin issuer plus a non-empty reason. The caller must first materialize a complete validated campaign export and supply that recovery artifact to purge.

Inside the purge transaction, a campaign-specific authorization row is written first. It authorizes deletion of the `campaign_state_current` aggregate root and acquires SQLite's write reservation; it does not authorize direct deletion of historical children. While that reservation is held, purge re-exports the complete durable aggregate and compares every exported payload field with the supplied backup except export-generation time. Any lifecycle, session, state, journal, causal, command, or snapshot difference makes the backup stale and aborts before deletion.

Immutable command/event/causal/snapshot delete triggers ignore purge authorization. They permit child deletion only once the matching root is absent during the root's FK cascade, so a manually inserted authorization row cannot unlock selective history surgery.

The root deletion statement is mechanically the complete-purge boundary. FK cascades remove lifecycle metadata, command/event/causal history, immutable snapshots, and ADR-013 derivative query projections. An `AFTER DELETE` root trigger then removes non-root-cascading `play_sessions`; their participants cascade from the session rows. The same trigger removes transient purge/restore authorization rows. Production Rust does not rely on follow-up cleanup statements to complete the aggregate.

Deleting the root without campaign-specific purge authorization is rejected. If validation, exact-backup comparison, root deletion, cascade, cleanup, or any other transaction step fails, the transaction rolls back and the committed campaign aggregate remains intact.

### Export is a versioned recovery artifact

Campaign export format version 1 contains:

- export format version and campaign-state schema version;
- campaign identity and lifecycle metadata;
- current serialized campaign state and event head;
- play sessions and participants;
- command audit records;
- event journal records and causal edges;
- immutable campaign snapshots, including stored creation metadata.

Derivative ADR-013 query projections are deliberately excluded. They are rebuilt from restored authoritative current state by the projection triggers and are removed by root cascade during purge.

Export validation fails closed on incompatible format/schema, campaign identity mismatches, invalid domain state, non-contiguous journal history, invalid causal ordering, cross-campaign rows, invalid sessions, snapshots, or state/snapshot provenance outside the accepted journal prefix.

Before an export is accepted for restore or purge, lifecycle validation re-establishes normal persistence invariants:

- exported sessions are reconstructed as domain `PlaySession` values and validated against decoded `CampaignState`;
- participant players and optional characters must exist;
- player command issuers must exist;
- command/event actors must resolve to existing entities/factions;
- referenced command/event sessions must exist in the exported campaign and event session correlation must match the originating command;
- command and event serialized records must have a non-blank kind, a positive schema version representable as `u32`, and syntactically valid JSON payloads, matching the storage invariants of `SerializedRecord::encode`;
- accepted command audit rows must have non-blank resolution explanation and advance the event sequence;
- journal events may reference only accepted commands;
- each accepted command's emitted event batch must be exactly the contiguous range `expected_event_sequence + 1 ..= resulting_event_sequence`;
- rejected audit rows remain portable only without emitted journal events.

These checks ensure a modified export cannot be restored successfully merely because SQLite columns accept it and then fail later during snapshot+journal replay.

Ruleset/content references are preserved as stored identifiers. Whether they resolve to installed compatible manifests is owned by the content-manifest subsystem under ADR 014.

### Restore is transactional and collision-safe

Restore validates the entire export before opening its write transaction and refuses to overwrite an existing campaign identity. Invalid session, authority, serialized-record, audit-batch, journal, snapshot, or provenance data therefore fails before any restore rows can commit.

A restore transaction inserts a campaign-specific import authorization only to suppress normal sequence-0 snapshot bootstrap. This permits the exact exported immutable snapshot set to be restored rather than synthesizing replacement recovery history.

All exported rows are inserted in dependency order in one transaction. ADR-013 projection triggers derive query projections from the inserted current state in that same transaction. Constraint or global identity collisions abort the restore, leaving neither a partial target campaign nor mutations to unrelated campaigns.

Restore does not rewrite issuer kinds, actor references, event sources, payloads, or other accepted provenance fields. It preserves them verbatim only after validating the same durable invariants required by normal persistence.

## Consequences

- Archive is non-destructive; purge is explicit and recoverable only through its supplied exact backup.
- Purge authorization cannot be reused for selective journal surgery.
- Authorized root deletion mechanically removes the complete campaign aggregate, including sessions and derivative projections.
- Export/restore is a versioned compatibility surface; incompatible future changes require an explicit format/version migration decision.
- Restore rejects malformed or internally inconsistent audit/event payloads before writes rather than deferring failure to replay.
- Snapshot timestamps and exact stored snapshot history survive round trips.
- A valid export can still fail restore because of global primary-key collisions with another campaign; such failure is atomic.
- Projection data remains derivative and is neither exported nor treated as a second source of truth.

## Rejected alternatives

### Disable immutable-history triggers globally during purge

Rejected because it creates a database-wide escape hatch and weakens unrelated campaign isolation.

### Treat purge authorization as permission to delete historical rows

Rejected because an ordinary SQLite row would become a reusable selective-history surgery capability. Authorization is checked only on aggregate-root deletion.

### Complete purge with follow-up application cleanup

Rejected because interruption or direct authorized SQL could leave non-cascading campaign-owned rows behind. Root deletion itself must mechanically complete the aggregate purge.

### Export only `CampaignState` JSON

Rejected because it cannot reconstruct command/event provenance, causal history, immutable snapshots, or play-session records.

### Restore raw exported rows after only relational-reference checks

Rejected because SQLite does not encode serialized-record JSON/codec invariants or accepted-command event-batch semantics. Such rows can commit yet make replay fail later.

### Serialize derivative query projections into export v1

Rejected because ADR 013 defines projections as rebuildable query indexes, not authoritative recovery input.

### Overwrite an existing campaign during restore

Rejected for Gate 1. Replacement remains an explicit export, authorized purge, then restore sequence.
