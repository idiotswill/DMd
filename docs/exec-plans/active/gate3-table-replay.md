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

Implementation has not started. Refactor preflight around a typed composed recovery event,
extend the live replay adapter, add focused tamper/legacy/session regressions, run all app
tests and strict Clippy/formatting. Full integrated repository verification remains root's
responsibility after reviewing/cherry-picking this slice. No owner blocker is known.
