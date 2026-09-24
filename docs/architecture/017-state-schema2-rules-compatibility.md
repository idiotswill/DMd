# ADR 017 — State schema 2 and Gate 1 compatibility

Status: proposed for the integrated Gate 2 rules foundation PR.

## Context

Gate 2 adds typed optional mechanical state to `CampaignState`. Leaving the state
schema at version 1 would permit an old binary to ignore that field and erase it
on its next material write. Gate 1 campaigns, recovery anchors, and portable
exports must remain usable without inventing rules state or rewriting history.

## Decision

State schema 2 introduces optional rules state. An explicit schema-1 migration
adds `rules: null` and changes only the embedded schema version. It does not
enable a ruleset, infer characters, or fill mechanical defaults.

SQL migration 0008 upgrades schema-1 materialized current rows in one SQLite
transaction. A preflight rejects malformed JSON, embedded/row identity, sequence
or schema disagreement, missing/mismatched lifecycle metadata, and unexpected
non-null mechanical data. Duplicate top-level fields and invalid numeric schema
or sequence types are rejected rather than normalized. A rejected migration leaves its prior rows unchanged;
corrupt saves require the existing recovery/diagnostic path rather than guessed
repairs. Other domain corruption is still rejected by normal state validation.

The existing lifecycle and projection update triggers carry the new schema
metadata and rebuild derivative projections in the same transaction. Unchanged
journal heads do not trigger new periodic snapshots.

Immutable snapshots retain their original JSON, schema metadata, sequence and
timestamp. The default `CampaignStateSnapshotCodec` registers the explicit
schema-1-to-2 migration and applies it only in memory. Future/unknown versions,
mismatched embedded legacy schema and unexpected legacy mechanical data fail
closed. Registration cannot replace the built-in migration.

`CampaignExport::upgraded()` returns a fully validated, upgraded clone. For a
schema-1 export, only its current JSON and current/lifecycle/export state-version
metadata are upgraded. Historical snapshots, commands, events, causal links,
sessions and timestamps remain exact. A legacy export cannot smuggle a snapshot
from a newer schema. Restore upgrades and validates before opening its write
transaction. App restore preflight must use the same helper before content
resolution. The portable envelope format remains version 1 because its topology
is unchanged and its explicit state-schema field carries this compatibility
decision.

Export validation decodes every snapshot through the version-aware codec, then
checks its campaign, sequence and provenance. It no longer assumes every
historical snapshot has the current schema. New exports may legitimately contain
schema-1 recovery anchors and schema-2 current state.

## Consequences

- Older binaries reject schema-2 authoritative state rather than silently losing
  its mechanics.
- Existing valid Gate 1 databases and exports remain usable.
- Replay recovers old anchors without altering accepted history or rerolling.
- Persistence remains content-agnostic and does not implement rules semantics.
- Raw malformed/current recovery artifacts can block automatic migration; this
  is deliberate fail-closed behavior, and automatic corruption repair is outside
  this gate.
- The schema bump and compatibility slice must land atomically with the typed
  domain rules field in the Gate 2 PR.

## Backup and recovery

Before upgrading an existing installation, retain validated campaign exports
from its old binary, or a consistent SQLite backup made after clean shutdown or
through SQLite's backup API. Never copy only a live database file while excluding
uncheckpointed WAL data. Keep the original backup unchanged during migration.

A failed migration rolls its own transaction back. Diagnose or replay damaged
schema-1 materialized state using the compatible old recovery tooling on a copy,
then retry the upgrade of that recovered copy. Do not edit immutable snapshots to
get migration to pass. A schema-1 portable backup can be restored directly into a
fresh current database; restore upgrades its current state and retains its anchors.

There is no down-migration that discards schema-2 mechanics. To return to an old
binary, restore the original pre-upgrade backup into a separate database and
explicitly accept that later play is absent from that older recovery point.
