# ADR 021 — Current table state, atomic sessions and observation compatibility

Status: **Accepted — Gate 3; see the [integrated checkpoint](../checkpoints/gate-03-desktop-table-loop.md).**

## Context

Gate 3 must resume the exact pending player decision, current Session Zero agreement and
active player/character binding after restart. Queries and conversation must also survive
restart without becoming world mutations. ADR 008 keeps historical play sessions and
transcript records outside the world snapshot; ADRs 011, 012, 017 and 020 require atomic
provenance, explicit schema evolution and unchanged historical recovery anchors.

## Decision

Campaign-state schema 3 adds optional `table: TableState`. This is bounded current runtime
authority: the contract, supported character profiles, current session binding, current
situation and one unresolved decision or roll context. Session history and observations
remain separate. `CampaignState::validate` calls table validation; the current active
binding must equal the durable session projection when loaded, committed or exported.
This extends ADR 008's current-state boundary without embedding the historical ledger.

Migration 0009 changes schema-2 current images to schema 3 with `table: null`. Preflight
rejects malformed JSON, duplicate root keys, embedded schema/identity/head mismatch,
unexpected table data and mismatched lifecycle metadata. Current projection triggers
follow the update. Existing snapshot bytes, schema metadata, timestamps, journal and
audit rows are unchanged. The default snapshot codec composes 1→2→3 in memory. It does
not infer table settings, PC profiles or decisions for an older campaign.

The portable envelope is version 2 and adds ordered `observations`. `CampaignExport::upgraded`
validates supported envelope/state metadata, upgrades only the current JSON and metadata,
then validates the complete result before restore opens a write transaction. Format 1
has no observations and supports its historical state schemas 1 and 2; nonempty observations
or schema-3 data under that envelope are rejected. Missing observations decode to an empty
list. Old immutable snapshots remain exact, including mixed schema-1/schema-2 anchors.

`commit_campaign_transition_with_session` extends the existing atomic commit with optional
`SessionChange::Start` or `Replace { expected, next }`. The original commit API delegates
with no session change. Admin/system authority, identity, campaign, session time, references,
active uniqueness and exact previous-session comparison are required. Closed sessions
cannot be rewritten or reopened through this path. The session projection is written in
the same transaction before session-linked audit/event rows, and any later failure rolls
back all effects. The legacy raw session API remains a trusted storage/recovery utility;
production table lifecycle uses the atomic command path. Opening a table detects raw
projection drift rather than silently accepting it.

`session_observations` is an append-only conversation ledger. Each record has a stable ID,
campaign and optional session, trusted issuer, explicit Party/Player/Host audience, observed
authoritative head, typed-payload kind/version and bounded JSON. Its positive per-campaign
ordinal orders conversation independently of the authoritative event sequence. Appending
does not alter current state, audit records, events or snapshots. New player observations
require an existing player present in the active session; private player audiences must
exist and be present, and player issuers cannot target another player's private audience.
Import issuers and future observed heads are rejected. The app must provide trusted identity,
typed payload validation and visibility filtering; raw persistence reads are host-trusted.

Observation batches acquire a SQLite write reservation before reading authority and assigning
ordinals. Reusing an ID with exactly equal content returns its original row, including after
session closure; unequal content fails. A failed batch leaves no partial records. History
reads support session or campaign pagination and campaign-scoped stable-ID lookup so app
retries can return the original answer without regenerating it against a newer state.

Export validation checks observation ordering, identity, campaign, head, payload shape,
issuer/audience references and session existence. Historical attendance and active status
are not reinterpreted from the latest session projection: valid queries remain history
after attendance changes or closure. Observations are restored after referenced sessions
inside the aggregate restore transaction, participate in exact purge-backup comparison,
and can only be removed by authorized whole-campaign purge.

## Verification and boundaries

`table_persistence.rs` exercises session rollback after provisional insertion, authority,
CAS and closed-history rejection, current table/projection agreement, observation atomicity,
idempotency after closure, immutable history, reference/privacy rejection, export/restore,
purge freshness, schema-2 mechanics preservation and corrupt migration rollback.
`state_schema_compatibility.rs` retains the Gate 1 migration/export/replay regressions.
Application-level composed table/rules replay and semantic observation display are integrated
under ADR 024 and covered by the Gate 3 checkpoint; storage validation alone is not the
basis of desktop acceptance.
