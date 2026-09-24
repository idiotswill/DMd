# Gate 1 campaign lifecycle

Status: validation closeout — implementation/review blockers resolved; exact-head full CI pending
Branch: gate1/campaign-lifecycle
PR: #9
Integrated base: main@f4473745bb44e6fa112ae87833c9336553304c20 (PR #10 projections merged)
Reviewed implementation head: 74256b11c523cb7f5bf47a73bf1473a5cc089604

## Objective
Build production-intended durable lifecycle operations for multiple unrelated campaigns, including create/open/list/archive, safe whole-aggregate purge, and portable backup/export/restore without weakening append-only history, replay integrity, projections, or authority boundaries.

## Scope
- Durable campaign create/open/list/archive operations.
- Explicit admin-only whole-campaign purge, distinct from archive/ordinary deletion.
- Mechanical preservation of command/event/causal/snapshot immutability outside aggregate-root cascade.
- Mechanical complete aggregate cleanup when an authorized campaign root is deleted, including non-root-cascading play sessions/participants and transient authorization rows.
- Versioned export/backup with explicit compatibility metadata and the durable records needed for recovery.
- Transactional restore with state, journal, snapshot, session, authority, provenance, serialized-record codec, command-audit, and event-batch validation.
- Integration with the merged derivative projection schema without making projections authoritative export state.
- Restart/reopen, multi-campaign isolation, stale/corrupt backup, destructive failure, replay-safety, and partial-restore coverage.

## Non-goals
- Expanding normalized projection scope beyond integration with the merged Gate 1 projection layer.
- Rules/content-pack manifest resolution beyond preserving stored `VersionedRef` values; that remains owned by the content-manifest workstream.
- Gameplay mechanics, simulation/Director, voice/AI/UI, or product-scope changes.
- Merging this destructive-persistence/save-format work without explicit human approval.

## Relevant durable context
- `docs/product-definition.md` — restart/recovery, unrelated campaigns, no manual database surgery.
- `docs/checkpoints/gate-1.md` — Gate 1 lifecycle blocker and parallel-worker boundaries.
- ADR 004 — campaign/content isolation.
- ADR 011 — atomic append-only journal and controlled complete aggregate purge boundary.
- ADR 012 — immutable snapshots and explicit migration/replay behavior.
- ADR 013 — merged derivative query-projection architecture; projections remain non-authoritative.
- Proposed ADR 014 — lifecycle/portability contract; must remain proposed pending human approval.

## Acceptance criteria
- [x] Create/open/list/archive and exact-backup purge/export/restore baseline behavior is implemented with unrelated-campaign isolation and restart coverage.
- [x] Selective command/event/causal/snapshot deletion remains mechanically blocked even when a purge-authorization row is manually inserted.
- [x] Campaign-root deletion is rejected without campaign-specific purge authorization.
- [x] Authorized direct deletion of `campaign_state_current` mechanically removes the entire current aggregate, including `play_sessions`, participants, history/snapshots/lifecycle, derivative projections, and purge/restore authorization state, without relying on follow-up Rust statements.
- [x] Regression manually establishes purge authorization, deletes the root directly, and proves no campaign-owned rows survive.
- [x] Purge requires an admin issuer, non-empty reason, and caller-supplied validated exact backup; any durable aggregate mismatch aborts before deletion.
- [x] Restore validates campaign/session/player/character/issuer/actor/provenance references before committing writes.
- [x] Restore validates command and event serialized-record invariants equivalent to `SerializedRecord::encode`: non-blank kind, positive `u32` schema version, and valid JSON payload.
- [x] Accepted command audit rows require non-blank resolution explanation; events may reference only accepted commands.
- [x] Event-producing command audit metadata exactly describes its emitted contiguous sequence batch (`expected + 1 ..= resulting`), while rejected audit rows remain portable only without emitted events.
- [x] Corruption regressions cover malformed command/event JSON, blank kind, zero/overflow versions, resolution explanation, rejected-command event references, and inconsistent event-batch sequence metadata; invalid restores leave the target campaign absent.
- [x] Integration onto `main@f4473745...` preserves projection migration `0006`, uses lifecycle migration `0007`, uses lifecycle ADR `014`, and preserves `dmd-persistence/src/lib.rs` projection/lifecycle exports and migration-on-open behavior.
- [x] Projection rows are rebuilt automatically from restored authoritative current state and are removed by root cascade during purge; export v1 remains projection-free and authoritative-state-only.
- [x] Complete integrated diff was reviewed against ADRs 004/011/012/013 and proposed lifecycle ADR 014; changed-file set contains only the 10 intended lifecycle/dependency/test/documentation files and no temporary helper workflow.
- [ ] `./scripts/verify-fast`, clippy, full workspace tests, Rust 1.88 MSRV, genericity guard, and architecture guard pass on the exact final integrated head.
- [x] Lifecycle ADR remains proposed/pending human approval.
- [x] Irreversible decisions and remaining debt are recorded below.

## Completed slices
1. **Complete-root purge invariant** — migration `0007` authorizes only aggregate-root deletion; child immutable-history guards ignore purge authorization; the root `AFTER DELETE` trigger mechanically removes non-cascading sessions plus transient authorization rows. Production Rust has no post-root session cleanup dependency. Direct-SQL regression proves sessions/participants/history/snapshots/lifecycle/projections/auth rows are all gone after one authorized root delete.
2. **Replay-safe restore validation** — import validates normal serialized-record shape, accepted resolution provenance, event→accepted-command relation, exact accepted-command event spans, session/authority/actor references, journal/causal/snapshot/state provenance, and rejects corrupt artifacts before writes.
3. **Projection integration** — lifecycle is rebased onto merged projection `main`; projection migration remains `0006`, lifecycle is `0007`, projection API/open/migration behavior in `lib.rs` is preserved, restore rebuilds projections from current state, and purge cascades them away.
4. **Validation/closeout** — focused integrated persistence suite is green; full diff reviewed; exact-head repository CI remains the final machine-validation step before human review.

## Decisions
- Archive remains operational lifecycle metadata and does not rewrite domain `CampaignStatus` or accepted history.
- Export/restore remains a versioned save compatibility surface and fails closed rather than inferring compatibility from JSON shape.
- Purge authorization applies only to deletion of `campaign_state_current`; historical child delete triggers ignore authorization and allow deletion only during root cascade.
- The root deletion statement itself is the complete-purge boundary: an `AFTER DELETE` lifecycle trigger cleans non-cascading session rows and transient authorization rows after root-cascading history/projection rows are gone. Production Rust does not complete the aggregate afterward.
- Restore validation reproduces `SerializedRecord::encode` storage invariants for imported raw rows and validates accepted-command/event sequence relationships before opening the restore write transaction.
- Accepted audit rows correspond to one or more journal events spanning exactly `expected_event_sequence + 1` through `resulting_event_sequence`; rejected audit rows may remain eventless and portable.
- Merged query projections are derivative: restoring `campaign_state_current` rebuilds them through projection triggers; deleting the root cascades them. They remain excluded from export format v1.
- The pre-projection corruption test that directly wrote `{}` into `campaign_state_current` was replaced with a corrupt-export artifact test because ADR-013 projection constraints now correctly prevent that invalid current-state update from committing.

## Validation status
- Review #5296941307 was re-read and used as the blocker contract.
- Focused integrated helper run `35967468476` passed on Rust 1.88 after projection integration: `campaign_lifecycle`, `campaign_lifecycle_integrity`, and the full `dmd-persistence` test suite all passed. The helper removed itself after committing only the intended test changes.
- PR CI 253 on bot-authored cleanup head `74256b11...` was `action_required` before jobs executed; it is not counted as validation evidence.
- `scripts/verify` was inspected: its checks are exactly `verify-fast`, clippy, full workspace tests, genericity guard, and architecture guard. Normal PR CI runs those same checks plus a separate Rust 1.88 workspace check.
- Final exact-head PR CI is pending on the user-authored closeout commit containing this plan update.

## Irreversible / high-impact decisions
- Export format version 1 is a durable compatibility surface; incompatible future changes require an explicit format/version migration decision.
- Migration `0007_campaign_lifecycle.sql` changes immutable-history deletion semantics, campaign-root purge authorization, non-cascading aggregate cleanup, and exact-snapshot restore authorization.
- Purge remains permanently distinct from archive/ordinary deletion and requires an exact validated recovery artifact through the production API.
- Restore is additive, does not silently overwrite an existing campaign, and treats authority/provenance/serialized audit fields as data to validate before preserving.
- Proposed ADR 014 changes destructive persistence and save portability architecture and must not become accepted or merge without explicit human approval.

## Cross-workstream contracts
- Query projections are merged in `main`: campaign-owned projections are derivative. Lifecycle restore relies on projection triggers to rebuild them from authoritative state; purge removes them by root cascade; export v1 does not serialize them.
- Content-manifest work remains separate: lifecycle preserves stored ruleset/content `VersionedRef` values. Manifest availability/compatibility gating remains an integration requirement owned by that subsystem.

## Remaining debt
- No CLI/UI/filesystem adapter is included for naming, storing, rotating, encrypting, or retaining exported JSON files.
- Export v1 has structural/relational/domain integrity validation but no cryptographic signature/authenticity guarantee.
- Content-manifest availability/compatibility remains outside this PR.

## Blockers / risks
- Machine blocker: exact-head full PR CI must pass after this closeout commit.
- Governance blocker: this remains irreversible migration/save-format/destructive-persistence work and must stay draft/unmerged until explicit human approval.

## Next action
Verify exact-head full PR CI, then update PR #9 with the integrated review/validation summary. If CI is green, stop for human review without merging or accepting ADR 014.
