# Gate 2 state-schema compatibility slice

Status: slice implemented and tested; integration-only branch
`codex/gate2-state-compatibility`, based on `414040b`.

## Objective and scope

Preserve existing Gate 1 campaigns while introducing the schema-2 storage contract
needed for typed rules state. The owning Gate 2 branch supplies the typed optional
rules field; this slice owns the schema constant, persistence migration, snapshot
codec, export upgrade/restore behavior, tests, and compatibility decision record.
The slice will be cherry-picked into the owning Gate 2 PR and must not be merged
independently of the domain extension.

Relevant requirements: product-definition exact suspension, durable campaign
continuity and local-first play; Gate 2 deterministic durable mechanics and
failure without mutation; ADRs 011, 012, 014 and 016.

Non-goals: rules mechanics, content selection/upgrades, UI, Gate 3, historical
event rewriting, and changes to snapshot cadence.

## Acceptance

- Existing valid schema-1 current rows upgrade atomically to schema 2 with an
  explicit null rules field, retaining identity, sequence, and domain contents.
- Lifecycle and projection schema metadata track that upgrade.
- Immutable historical snapshots, event journal and command audit remain exact.
- The default codec explicitly upgrades schema-1 snapshots in memory.
- Valid schema-1 exports can be upgraded without mutating the caller, restored,
  reopened and replayed; original recovery anchors remain unchanged.
- Malformed/mismatched/future data fails before authoritative mutation.
- Targeted persistence regression tests and repository verification pass.

## Planned slices

1. Add ADR and the explicit v1-to-v2 codec/export migration.
2. Add SQL current-row/projection migration with corruption preflight.
3. Add old database/export/snapshot and failure regressions; update historical
   tests that assumed schema 1 was forever the current version.
4. Verify and hand coherent commit to the Gate 2 owner for integrated review.

## Decisions and risks

- Retaining schema 1 for a new optional rules field would let an older binary
  silently ignore and erase mechanical state. Schema 2 prevents that path.
- Persistence stays content-agnostic; rules state is not interpreted here.
- Schema-1 snapshots and exports must not acquire fabricated mechanical state.
- Current-row migration must not conceal metadata mismatches or overwrite
  unexpected rules data.
- Full production integration remains the parent branch's acceptance obligation.
- ADR 017 records schema compatibility, backup/rollback policy and the requirement
  that this slice land together with the typed domain extension.
- `CampaignExport::upgraded()` is the validated, nonmutating app preflight API.
- Snapshot migration typed-decodes the original shape before JSON conversion so
  duplicate fields cannot be silently collapsed.
- Old tests retain their original failure assertions, using the current schema
  constant for fixtures that describe the current/future version rather than
  hardcoding the old current version 1.

## Validation

- `./scripts/verify-fast`: passed (rustfmt and workspace/all-targets check).
- `cargo clippy --locked -p dmd-persistence --all-targets -- -D warnings`: passed.
- `cargo test --locked -p dmd-persistence`: 60 tests passed.
- `cargo test --locked --workspace`: all 119 tests passed on Windows GNU Rust.
- `./scripts/check-genericity` and `./scripts/check-boundaries`: passed.
- `git diff --check`: passed.
- `./scripts/verify` was attempted and stopped at inherited Windows-only unused
  imports/helpers in `dmd-domain/tests/content_manifest_fail_closed.rs`. The
  owning Gate 2 branch already has those platform guards and will run complete
  verification after integration; they were not duplicated in this slice.
- New regression evidence covers pre-0008 database upgrade; lifecycle/projection
  consistency; exact retained snapshot JSON/timestamps/immutability; legacy
  export nonmutation, history retention, restore/reopen/replay equality; eight
  corrupt database preflight cases with rollback of every campaign; six invalid
  export classes rejected before restore writes; protected built-in migration;
  malformed, duplicate-field and unexpected-mechanics rejection.

## Exact next action

Owning Gate 2 agent: cherry-pick this slice into the rules foundation branch,
ensure the typed optional rules field lands in the same PR, call
`export.upgraded()` before app restore content preflight, and run full integrated
verification/review. No PR or independent merge was created for this slice.
