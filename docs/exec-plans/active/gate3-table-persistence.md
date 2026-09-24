# Gate 3 table persistence

Status: active; sole writer rules-architecture agent on
`codex/gate3-table-persistence`, based on `fff6900`.

## Objective and ownership

Preserve exact table decisions and session authority through schema-3 state,
atomic journal/session commits, and durable non-authoritative observations.
Own domain state/version/validation integration and observation contracts,
persistence migrations/code/tests and ADR 021. Root owns `TableState` and its
validation, table/rules composition, character creation and desktop integration.

Gate 3 and product Session Zero, player agency, question/action separation,
transcript, and exact suspension clauses apply. ADRs 008, 011, 012, 017 and 020
govern session/world boundaries, atomicity, immutable history and compatibility.

## Planned work and acceptance

1. Add domain observation identity/audience/record contracts, outside CampaignState.
2. Add optional table state and schema 3 after the root's domain contract lands.
   Migration 0009 upgrades current rows only; codec chains 1→2→3 preserve anchors.
3. Add export format 2 observations with explicit format-1 upgrade. Old exports
   cannot smuggle observation history or newer state fields; old rules remain exact.
4. Add atomic optional session start/replace side effects to the existing commit
   boundary, preserving its public wrapper. Compare locked session images; enforce
   active uniqueness, closed immutability, reference/identity/time invariants.
5. Add append/read observation APIs with stable-ID equality-based idempotency,
   trusted issuer/visibility/session-attendance checks, bounded records and atomic
   batches. Observations never advance authoritative state or journal sequences.
6. Test mixed snapshots, migration rollback, old exports, restore/purge observations,
   session transaction rollback/CAS, observation retries/privacy/references.

## Risks and non-goals

No full session history/transcript is embedded in world snapshots. Legacy table
settings, characters and observations are not invented by migration. Table action
replay and UI visibility filtering remain root application responsibilities.
The compatibility commit must land with the new domain table type, not alone.

## Verification and exact next action

Implementation has not started. Root will supply the table-domain commit; begin
independent observation contracts and transactional session infrastructure now.
Run focused persistence/domain tests, strict Clippy and required repository checks
on the integrated branch before handing the slice to root for final integration.
