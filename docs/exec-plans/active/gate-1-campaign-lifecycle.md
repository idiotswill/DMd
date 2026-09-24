# Gate 1 campaign lifecycle

Status: in progress — review #5296941307 blockers plus projection integration
Branch: gate1/campaign-lifecycle
PR: #9
Current branch head at review: a5fcf96a2b3b6114310ce42fb01248edf70d11ea
Integration base: main@f4473745bb44e6fa112ae87833c9336553304c20 (PR #10 projections merged)

## Objective
Build production-intended durable lifecycle operations for multiple unrelated campaigns, including create/open/list/archive, safe whole-aggregate purge, and portable backup/export/restore without weakening append-only history, replay integrity, projections, or authority boundaries.

## Scope
- Durable campaign create/open/list/archive operations.
- Explicit admin-only whole-campaign purge, distinct from archive/ordinary deletion.
- Mechanical preservation of command/event/causal/snapshot immutability outside aggregate-root cascade.
- Mechanical complete aggregate cleanup when an authorized campaign root is deleted, including non-root-cascading play sessions/participants and transient authorization rows.
- Versioned export/backup with explicit compatibility metadata and the durable records needed for recovery.
- Transactional restore with state, journal, snapshot, session, authority, provenance, serialized-record codec, command-audit, and event-batch validation.
- Integration with the now-merged derivative projection schema without making projections authoritative export state.
- Restart/reopen, multi-campaign isolation, stale/corrupt backup, destructive failure, replay-safety, and partial-restore coverage.

## Non-goals
- Expanding normalized projection scope beyond integration with the merged Gate 1 projection layer.
- Rules/content-pack manifest resolution beyond preserving stored `VersionedRef` values; that remains owned by PR #11 until integrated later.
- Gameplay mechanics, simulation/Director, voice/AI/UI, or product-scope changes.
- Merging this destructive-persistence/save-format work without explicit human approval.

## Relevant durable context
- `docs/product-definition.md` — restart/recovery, unrelated campaigns, no manual database surgery.
- `docs/checkpoints/gate-1.md` — Gate 1 lifecycle blocker and parallel-worker boundaries.
- ADR 004 — campaign/content isolation.
- ADR 011 — atomic append-only journal and controlled complete aggregate purge boundary.
- ADR 012 — immutable snapshots and explicit migration/replay behavior.
- ADR 013 on `main` — accepted/proposed query-projection architecture from merged PR #10; projections are derivative and cascade from the campaign root.
- Lifecycle ADR must be renumbered to ADR 014 during integration and remain proposed pending human approval.

## Acceptance criteria
- [x] Create/open/list/archive and exact-backup purge/export/restore baseline behavior is implemented with unrelated-campaign isolation and restart coverage.
- [x] Selective command/event/causal/snapshot deletion remains mechanically blocked even when a purge-authorization row is manually inserted.
- [x] Campaign-root deletion is rejected without campaign-specific purge authorization.
- [ ] Authorized direct deletion of `campaign_state_current` mechanically removes the entire current aggregate, including `play_sessions`, participants, history/snapshots/lifecycle, derivative projections, and purge/restore authorization state, without relying on follow-up Rust statements.
- [ ] Regression manually establishes purge authorization, deletes the root directly, and proves no campaign-owned rows survive.
- [x] Purge requires an admin issuer, non-empty reason, and caller-supplied validated exact backup; any durable aggregate mismatch aborts before deletion.
- [x] Restore validates campaign/session/player/character/issuer/actor/provenance references before committing writes.
- [ ] Restore validates command and event serialized-record invariants equivalent to `SerializedRecord::encode`: non-blank kind, positive `u32` schema version, and valid JSON payload.
- [ ] Accepted command audit rows require non-blank resolution explanation; events may reference only accepted commands.
- [ ] Event-producing command audit metadata must exactly describe its emitted contiguous sequence batch (`expected + 1 ..= resulting`), while rejected audit rows remain portable without emitted events.
- [ ] Corruption regressions cover malformed command/event JSON, kind/version metadata, resolution explanation, rejected-command event references, and inconsistent event-batch sequence metadata; all invalid restores leave the target campaign absent.
- [ ] Rebase/squash integration onto `main@f4473745...` preserves projection migration `0006`, renames lifecycle migration to `0007`, renumbers lifecycle ADR to `014`, and merges `dmd-persistence/src/lib.rs` exports.
- [ ] Projection rows are rebuilt automatically from restored authoritative current state and are removed by root cascade during purge; export v1 remains projection-free and authoritative-state-only.
- [ ] Complete integrated diff is reviewed against ADRs 004/011/012/013 and proposed lifecycle ADR 014.
- [ ] `./scripts/verify-fast`, clippy, full workspace tests, Rust 1.88 MSRV, genericity guard, and architecture guard pass on the exact final integrated head.
- [ ] Lifecycle ADR remains proposed/pending human approval.
- [x] Irreversible decisions and remaining debt are recorded below.

## Planned slices
1. **Complete-root purge invariant** — extend the lifecycle migration so authorized root deletion itself cleans all current non-cascading campaign-owned rows and transient authorization state; remove Rust reliance on post-root cleanup; add direct-SQL regression.
2. **Replay-safe restore validation** — validate serialized command/event record shape, accepted-command resolution provenance, event→accepted-command relation, and exact command event-batch sequence metadata; add mutation regressions.
3. **Projection integration** — rebase/squash onto merged projection `main`, renumber migration/ADR, merge module exports, and prove projection restore/purge behavior.
4. **Validation/closeout** — inspect full diff, run exact-head CI, update this plan and PR #9, and stop for human review without merging.

## Decisions
- Archive remains operational lifecycle metadata and does not rewrite domain `CampaignStatus` or accepted history.
- Export/restore remains a versioned save compatibility surface and fails closed rather than inferring compatibility from JSON shape.
- Purge authorization applies only to deletion of `campaign_state_current`; historical child delete triggers ignore authorization and allow deletion only during root cascade.
- The root deletion statement itself must now be the complete-purge boundary: an `AFTER DELETE` lifecycle trigger will clean non-cascading session rows and transient authorization rows after root-cascading history/projection rows are gone. Production Rust will not be the mechanism that completes the aggregate afterward.
- Restore validation will reproduce `SerializedRecord::encode` storage invariants for imported raw rows and validate accepted-command/event sequence relationships before opening the restore write transaction.
- Accepted audit rows must correspond to one or more journal events spanning exactly `expected_event_sequence + 1` through `resulting_event_sequence`; rejected audit rows may remain eventless and portable.
- Merged query projections are derivative: restoring `campaign_state_current` rebuilds them through projection triggers; deleting the root cascades them. They remain excluded from export format v1.
- Because PR #10 has merged and changed migration/ADR/module surfaces, this branch must be reconstructed on current `main` before final review rather than claiming old-base CI as integration evidence.

## Historical validation evidence
- Review head `a5fcf96a2b3b6114310ce42fb01248edf70d11ea` passed PR CI 238, but that evidence is superseded by review #5296941307 and the merge of PR #10.
- Earlier focused integrity tests covered selective-history authorization and session/authority restore references; both original review blockers remain fixed.

## Irreversible / high-impact decisions
- Export format version 1 is a durable compatibility surface; incompatible future changes require an explicit format/version migration decision.
- The lifecycle migration changes immutable-history deletion semantics, campaign-root purge authorization, non-cascading aggregate cleanup, and exact-snapshot restore authorization; final migration number after integration is `0007`.
- Purge remains permanently distinct from archive/ordinary deletion and requires an exact validated recovery artifact through the production API.
- Restore is additive, does not silently overwrite an existing campaign, and treats authority/provenance/serialized audit fields as data to validate before preserving.

## Cross-workstream contracts
- `gate1/projections` is now merged into `main`: campaign-owned query projections are derivative. Lifecycle restore relies on projection triggers to rebuild them from authoritative state; purge removes them by root cascade; export v1 does not serialize them.
- `gate1/content-manifest` remains open as PR #11: lifecycle preserves stored ruleset/content `VersionedRef` values. Manifest availability/compatibility gating remains a separate integration requirement after that work lands.

## Remaining debt
- No CLI/UI/filesystem adapter is included here for naming, storing, rotating, encrypting, or retaining exported JSON files.
- Export v1 has structural/relational/domain integrity validation but no cryptographic signature/authenticity guarantee.
- Content-manifest availability/compatibility remains deferred to the manifest integration boundary.

## Blockers / risks
- SQLite trigger ordering must continue to preserve immutable-history guards: root cascades must complete before session cleanup encounters journal/session `RESTRICT` references.
- Restore validator changes must not reintroduce the prior mistake of requiring rejected audit rows to emit events.
- PR #9 is currently non-mergeable against `main` because PR #10 claimed migration `0006`, ADR 013, and `dmd-persistence/src/lib.rs`; integration must resolve all three mechanically before final CI.
- This remains irreversible migration/save-format work and must stay unmerged until explicit human approval.

## Next action
Implement the complete-root purge trigger and replay-safe serialized audit/event validation with corruption regressions, then reconstruct the branch on current `main` as lifecycle migration `0007` / ADR 014, run exact-head full CI, inspect the integrated diff, update PR #9, and stop for human review.
