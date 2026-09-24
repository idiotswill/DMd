# Gate 3 table persistence

Status: **Completed — Gate 3 technical implementation and native acceptance.**

## Completion and handoff

Foundation PR #20 merged at `ac900fb5c5603f26bd3f3108aecf81bf597eadae`; desktop PR #21
merged at `998eedea92da8aab288e1c586eb4a3e406c74b4c`, matching reviewed/tested head
`05ff272d16ab7d893e0c39950f07bd5176c82e3e`. Full verification passes 207 local Windows/GNU
tests and 208 Linux tests. Windows run `36025008694` passes MSRV/stable checks, native
host/frontend/notice tests and fresh packaging. All 1,068 final artifact checksums and
the dependency/SRD notice audit pass. Independent source review found no remaining blockers.

The packaged native scene covered creation, agreement, character/attendance, questions,
correction, raw dice, ambiguity, graceful/crash restart, duplicate launch, campaign isolation,
session end and new-session continuation. Final build reopened the pending Second Wind roll,
resolved raw 4 + 1 once with 1/2 uses retained, closed/saved/reopened, and started Beyond the
ridge with Alex/Mira, the original agreement, two accepted outcomes and unchanged resources.
The checkpoint distinguishes intermediate-build scene work from final-build continuation.
No manual save edits or developer gameplay bypass was used.

See [Gate 3 acceptance](../../checkpoints/gate-03-desktop-table-loop.md) for the complete
matrix, exact artifact identities, limits and debt. PR #22 records its final reviewed head,
checks and post-merge main verification. Gate 4 is not started. After final main verification,
pause for owner review; the next action is owner-authorized Gate 4 planning, not another
Gate 3 implementation slice. Human playability and reference-hardware/endurance acceptance
remain assigned to their existing later checkpoints.

## Historical execution notes

The implementation notes below preserve intermediate findings and next actions as history.
Their pending integration/build work is superseded by the completion evidence above.


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

The root's character/table/session identity contracts are present via cherry-picks
of `ae60a83`, `8a16c6e` and `983e020`; those are dependencies, not slice-owned work.
Schema 3, validation hook, migration 0009, chained codec, format-2 observations,
session side-effect transaction and audience-aware observation append APIs are implemented.
Campaign/session paginated reads and campaign-scoped stable-ID retry lookup are available.
ADR 021 records the compatibility and operational/world-state boundary decisions.

Full domain/persistence tests passed after the final session identity/time guards.
The six focused table persistence tests passed again after the last metadata regression.
Strict domain/persistence Clippy and formatting checks passed.

The application `CampaignExport` fixture requires its new empty observation field;
root owns that application adjustment and the composed table/rules replay integration.
Repository-wide `verify-fast`/`verify` and exact-head review must run on that integrated
branch before PR completion. Next action: root integrates the owned persistence/domain
integration and ADR commit, reviews the complete diff, and verifies the combined application.
