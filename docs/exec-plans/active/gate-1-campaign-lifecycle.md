# Gate 1 campaign lifecycle

Status: validation closeout — post-PR #11 integration complete; exact-head CI pending
Branch: gate1/campaign-lifecycle
PR: #9
Control review #5301125460 head: 8a9098a124c1f276f9fd5fd5a0af832acdbdeb92
Integrated base: main@88f813e3afc07d51d2d62416a7b11747acc956f7 (PR #11 content manifests merged)
Integrated lifecycle head before this plan closeout: cf986ea26487aedeec2181b7fd36fc718752cb4b

## Objective
Build production-intended durable lifecycle operations for multiple unrelated campaigns, including create/open/list/archive, safe whole-aggregate purge, and portable backup/export/restore without weakening append-only history, replay integrity, projections, content isolation, or authority boundaries.

## Scope
- Durable campaign create/open/list/archive operations.
- Explicit admin-only whole-campaign purge, distinct from archive/ordinary deletion.
- Mechanical preservation of command/event/causal/snapshot immutability outside aggregate-root cascade.
- Mechanical complete aggregate cleanup when an authorized campaign root is deleted, including non-root-cascading play sessions/participants, derivative projections, and transient authorization rows.
- Versioned export/backup with explicit compatibility metadata and durable recovery records.
- Transactional restore with state, journal, snapshot, session, authority, provenance, serialized-record codec, command-audit, and event-batch validation.
- Integration with merged query projections and current `main` after PR #11 without changing the content-manifest implementation.
- Restart/reopen, multi-campaign isolation, stale/corrupt backup, destructive failure, replay-safety, and partial-restore coverage.

## Non-goals
- Expanding normalized projection scope beyond integration with the merged Gate 1 projection layer.
- Wiring lifecycle/application runnable-campaign opening to `ContentCatalog`; control review #5301125460 leaves that composition step for a focused Gate 1 integration PR after #9 lands.
- Changing PR #11 content-manifest behavior, APIs, tests, or ADR 014.
- Gameplay mechanics, simulation/Director, voice/AI/UI, or product-scope changes.
- Merging this destructive-persistence/save-format work without explicit human approval.

## Relevant durable context
- `docs/product-definition.md` — restart/recovery, unrelated campaigns, no manual database surgery.
- `docs/checkpoints/gate-1.md` — lifecycle, projections, and content-manifest blockers plus their integration boundary.
- ADR 004 — campaign/content isolation.
- ADR 011 — atomic append-only journal and controlled complete aggregate purge boundary.
- ADR 012 — immutable snapshots and explicit migration/replay behavior.
- ADR 013 — merged derivative query-projection architecture; projections remain non-authoritative.
- ADR 014 — merged versioned content-manifest contract; raw persistence remains content-agnostic and runnable-campaign composition is a separate integration requirement.
- Proposed ADR 015 — lifecycle/portability contract; remains proposed pending human approval.
- Control review #5301125460 — lifecycle mechanics approved; required final work was current-main integration, ADR renumber, fresh exact-head CI, and combined-diff review.

## Acceptance criteria
- [x] Create/open/list/archive and exact-backup purge/export/restore baseline behavior is implemented with unrelated-campaign isolation and restart coverage.
- [x] Selective command/event/causal/snapshot deletion remains mechanically blocked even when a purge-authorization row is manually inserted.
- [x] Campaign-root deletion is rejected without campaign-specific purge authorization.
- [x] Authorized direct deletion of `campaign_state_current` mechanically removes the complete campaign aggregate, including sessions, participants, history/snapshots/lifecycle, derivative projections, and purge/restore authorization state.
- [x] Purge requires an admin issuer, non-empty reason, and caller-supplied validated exact backup; durable aggregate mismatch aborts before deletion.
- [x] Restore validates campaign/session/player/character/issuer/actor/provenance references before committing writes.
- [x] Restore validates command/event serialized-record invariants: non-blank kind, positive `u32` schema version, and valid JSON payload.
- [x] Accepted command audit rows require non-blank resolution explanation; events may reference only accepted commands.
- [x] Accepted command event metadata exactly describes its contiguous emitted sequence span; rejected audit rows remain portable only without emitted events.
- [x] Corruption regressions cover malformed command/event JSON, kind/version metadata, resolution explanation, rejected-command event references, and inconsistent event-batch metadata; invalid restores leave the target campaign absent.
- [x] Projection rows rebuild automatically from restored authoritative current state and cascade away during purge; export v1 remains projection-free.
- [x] Current `main@88f813e3afc07d51d2d62416a7b11747acc956f7` is incorporated as a merge parent.
- [x] PR #11 content-manifest code/tests/docs remain unchanged relative to current main; `main → #9` contains only the 10 lifecycle/dependency/test/documentation files.
- [x] Lifecycle migration remains `0007_campaign_lifecycle.sql`; PR #11 introduced no persistence migration.
- [x] Lifecycle ADR is renamed from 014 to 015; merged content-manifest ADR 014 is untouched.
- [x] Complete post-#11 combined diff was re-inspected against ADRs 004/011/012/013/014 and proposed ADR 015. Reviewed lifecycle production/test blobs are unchanged from the accepted lifecycle mechanics; new integration changes are the ADR renumber and execution-plan metadata.
- [ ] `./scripts/verify-fast`, clippy, full workspace tests, Rust 1.88 MSRV, genericity guard, and architecture guard pass on the exact final post-#11 head.
- [x] Irreversible decisions and remaining debt are recorded below.

## Progress
1. **Complete-root purge invariant — complete.** Root deletion is the mechanical complete-purge boundary; selective history surgery remains blocked.
2. **Replay-safe restore validation — complete.** Imported serialized/audit/event/session/provenance data is validated before writes.
3. **Projection integration — complete.** Migration 0006 and ADR 013 remain owned by projections; restore rebuilds derivative projections and purge cascades them.
4. **Post-content-manifest integration — complete.** Current main is a merge parent; PR #11 files are preserved from main; lifecycle migration stays 0007; lifecycle ADR is 015; combined diff is lifecycle-only.
5. **Validation closeout — pending exact-head CI.** No further code/schema change is planned unless CI exposes a real integration defect.

## Decisions
- Archive remains operational lifecycle metadata and does not rewrite domain `CampaignStatus` or accepted history.
- Export/restore is a versioned save compatibility surface and fails closed rather than inferring compatibility from JSON shape.
- Purge authorization applies only to deletion of `campaign_state_current`; historical child delete guards ignore authorization and allow deletion only during root cascade.
- The root deletion statement itself is the complete-purge boundary; production Rust does not finish the aggregate with follow-up cleanup.
- Restore validation reproduces `SerializedRecord::encode` storage invariants and accepted-command/event sequence relationships before opening the write transaction.
- Query projections remain derivative and excluded from export format v1.
- Merged content-manifest code and ADR 014 are preserved unchanged in this integration pass.
- Lifecycle/content runnable composition is intentionally deferred to a focused follow-up PR; #9 preserves stored `VersionedRef` values but does not add the application-level `ContentCatalog` runnable gate.

## Validation status
- Review #5296941307 blockers were resolved and accepted by control review #5301125460.
- CI 255 (`35968478504`) was fully green on lifecycle head `8a9098a124c1f276f9fd5fd5a0af832acdbdeb92`, but predates PR #11 and is historical evidence only.
- Current `main` was directly verified as `88f813e3afc07d51d2d62416a7b11747acc956f7`, merging PR #11.
- Comparison `f4473745... → 88f813e3...` shows PR #11 added only content-manifest domain code/tests/docs and no persistence migration.
- Comparison `88f813e3... → cf986ea2...` shows exactly 10 lifecycle/dependency/test/documentation files and no content-manifest files, proving the merged PR #11 implementation is unchanged in the integrated tree.
- The complete PR patch was re-opened after integration. Lifecycle migration/code/tests remain the reviewed implementation; docs now use ADR 015 and point to ADR 014 for the separate content-manifest contract.
- Fresh exact-head full CI after this plan-only closeout commit is pending.

## Irreversible / high-impact decisions
- Export format version 1 is a durable compatibility surface; incompatible future changes require an explicit format/version migration decision.
- Migration `0007_campaign_lifecycle.sql` changes immutable-history deletion semantics, campaign-root purge authorization, non-cascading aggregate cleanup, and exact-snapshot restore authorization.
- Purge remains permanently distinct from archive/ordinary deletion and requires an exact validated recovery artifact through the production API.
- Restore is additive, does not silently overwrite an existing campaign, and validates authority/provenance/serialized audit fields before preserving them.
- Proposed ADR 015 changes destructive persistence/save portability architecture and must not become accepted or merge without explicit human approval.

## Cross-workstream contracts
- Query projections are merged: campaign-owned projections are derivative; lifecycle restore rebuilds them from authoritative state, purge removes them by root cascade, and export v1 does not serialize them.
- Content manifests are merged as ADR 014: lifecycle preserves stored ruleset/content `VersionedRef` values. The application/lifecycle runnable-campaign `ContentCatalog` composition remains an explicit follow-up Gate 1 integration task rather than scope for this destructive-persistence PR.

## Remaining debt
- No CLI/UI/filesystem adapter is included for naming, storing, rotating, encrypting, or retaining exported JSON files.
- Export v1 has structural/relational/domain integrity validation but no cryptographic signature/authenticity guarantee.
- Runnable-campaign manifest resolution composition remains a focused post-#9 Gate 1 integration task.

## Blockers / risks
- Validation blocker: exact-head full CI must pass on the final post-#11 branch head.
- Governance blocker: this remains irreversible migration/save-format/destructive-persistence work and must stay draft/unmerged until explicit human approval.

## Next action
Run/verify exact-head full PR CI. If green, update PR #9 with the post-#11 integration evidence and stop for human review without merging or accepting ADR 015.
