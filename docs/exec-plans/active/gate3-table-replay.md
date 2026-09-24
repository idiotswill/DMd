# Gate 3 composed table recovery

Status: active, sole writer rules-architecture agent on `codex/gate3-table-replay`,
based on root integration head `48be314`.

## Objective and scope

Extend the production application replay and portable restore preflight to compose
`table.action_resolved@1` with existing rules-only history. Preserve player agency,
exact unresolved decisions, audit identity, deterministic outcomes and session history.
Own `rules_restore.rs`, `RulesReplayApplier` and the final replay validation in
`rules_runtime.rs`, focused recovery tests and this plan. Root owns all table-engine,
protocol, runtime and catalog-loading changes.

Product Session Zero, player agency, suspension/recovery and transcript clauses;
Gate 3 acceptance; ADRs 008, 011, 012, 020, 021 and 024 apply.

## Acceptance and implementation

- Validate every current/anchor table and mechanical image with installed source data.
- Match typed event metadata, action and outcome with accepted command audits, including
  exact nested rules metadata/outcome and permitted table-to-rules action mapping.
- Audit table pending origins and nested rules rulings/requests/results/permissions.
- Replay from the earliest immutable anchor through every later event and snapshot.
  Raw rules events cannot bypass a table-enabled state; rules-only saves remain supported.
- Reconcile canonical post-anchor session changes with final exported session projections.
- Reject unsupported history and tampered restores before any database writes.
- Retain the earliest anchor trust boundary: validate available earlier typed envelopes,
  without fabricating unavailable historical state or prior closed-session projections.

## Verification and next action

Implemented the typed composed event preflight, exact action/outcome audit parity, stateless
pre-anchor table authority/nested-action checks, table pending-origin checks, all-snapshot
canonical replay and session-ledger reconciliation. The live replay adapter uses the same
table engine and rejects raw mechanical events once table state exists. Final replay checks
validate table profiles even when a latest snapshot requires no subsequent events.
Root follow-up `2354c9f` is present as dependency copy `4b068c2`; its observation helper now
validates all table observation bodies during restore. No root-owned table/catalog files changed.

All 36 application tests passed: 6 helper unit, 15 rules runtime, 8 runnable boundary,
4 table loop and 3 composed recovery tests. Strict app/all-targets Clippy passed. The genuine
format-1 legacy restore fixture now expects the current schema and confirms absent table state.
The focused recovery test was extended to explicitly reject an altered intermediate table
snapshot; its final focused rerun, strict Clippy and formatting check all passed.

The earliest available anchor remains the trust base for unavailable earlier state and closed
session projections. Available earlier typed records still require matching authority, actions,
outcomes and origins. No invented replay is claimed for that unavailable prefix.
Review identified a root-owned integration follow-up: `has_rules_history` must recognize table
audit/event/observation lineage as well as rules lineage so stripping current/historical table
images cannot bypass preflight. Root was notified and owns that `lib.rs` guard/regression.
Next action: root cherry-picks the owned commit, reviews it, completes the lineage guard and
performs integrated repository verification. No owner decision is required.
