# Gate 4 — Encounter state compatibility

Status: **Implemented and independently reviewed; integrated in PR #23, exact-head merge checks pending.**

## Objective and branch

Add explicit schema-4 compatibility for optional typed tactical encounter state on
`codex/gate4-state-compatibility`, based on `14a94d6`. Root coordinates integration
with the independently owned encounter model and reviews the composed foundation.

## Scope and authority

Own `CampaignState` schema/optional field, migration 0010, snapshot codec, portable
export compatibility and focused persistence regressions. Relevant contracts are
the Gate 4 checkpoint, product-definition exact suspension and tactical truth,
ADRs 011/012/015/017/020/021/024 and the active Gate 4 plan.

No encounter model, rules, application, renderer, content or global plan edits.
No historical snapshot, event, audit, observation or session rewrite. Envelope 2
remains supported because encounter state adds no separate aggregate table.

## Acceptance criteria and slices

1. Add optional typed `encounter`, schema 4 and empty-state initialization.
2. Upgrade schema-3 current images transactionally, rejecting malformed JSON,
   duplicate fields, identity/head/lifecycle mismatch and future encounter data.
   Lifecycle and existing projection triggers must reflect schema 4.
3. Compose explicit snapshot migrations 1→2→3→4 in memory. Every legacy path
   rejects non-null encounter data. Unknown/malformed typed encounter data fails
   closed; older historical bytes and schema metadata stay exact.
4. Preserve envelope-2/schema-3 exports and mixed historical anchors; envelope-1
   restrictions remain intact. Corrupt migration/restore cannot partially write.
5. Verify targeted compatibility/persistence suites after integrating the model;
   root serializes build access and runs final full verification on integration.

## Decisions, risks and validation

- Existing nested state additions must not accidentally make legacy JSON undecodable.
  The independently owned `TacticalEncounter` model checkpoint `b28ec07` was
  integrated as `d69cabe`; its structural validator is called by the domain
  validation boundary. Runtime continuations remain a later Gate 4 slice; unknown
  pending fields currently fail typed decoding rather than accepting opaque JSON.
- SQLx migrations are already checksummed: existing migration files stay unchanged.
- SQLx 0.8.6 commits each migration separately by default. The public `migrate_sqlite`
  call now owns one outer transaction; nested migration transactions use savepoints.
  A late schema-4 preflight failure must roll back all pending earlier upgrades too.
- Current state gains only explicit `encounter: null` and schema 4; envelope 2 is
  unchanged. Default codec composes 1→2→3→4 without rewriting historical records.
- `cargo test --offline -p dmd-persistence` passed all **72 tests**, zero failed or
  ignored. This includes 11 schema-compatibility tests and 6 table/session tests.
  The new chain regression starts at database migrations 7 and 8, deliberately
  fails migration 10, and proves earlier pending migration records, current state,
  lifecycle, schema objects and old anchor bytes/timestamps all remain unchanged.
- `cargo clippy --offline -p dmd-persistence --all-targets -- -D warnings` passed.
  Both commands ran serially on the workspace GNU toolchain with one build job.
  Owned Rust files were formatted and the patch passed `git diff --check`.
- Round-trip tests preserve non-null typed geometry, remembered terrain, existing
  initiative/action/reaction budgets and exact snapshot/export bytes. Corrupt
  current and earliest-anchor geometry/provenance/unknown continuation fields are
  rejected before restore writes; legacy 1/2/3 cannot carry future encounter data.
- Root owns full integrated verification, independent review and ADR 025; this
  foundation does not claim complete combat or production Gate 4 acceptance.

## Exact next action

Independent review of 9dd6405 found no blockers in migration atomicity, legacy decoding,
immutable history or restore preflight. Root integrated it in PR #23 and added the public
Send-future regression after Windows CI exposed SQLx Acquire lifetime inference. The
already-acquired `run_direct` path retains the same migration/savepoint behavior. Final
integrated verification and CI are prerequisites for merging the exact PR head. Encounter
continuation semantics and desktop acceptance remain in the active gate plan.
