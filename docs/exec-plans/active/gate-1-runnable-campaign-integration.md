# Gate 1 runnable campaign integration

Status: implementation complete; control re-review pending
Branch: gate1/runnable-campaign-integration
PR: #12 (draft)
Base: main/f3f18568be62b49458f8085d9ecc635ff87162d3
Verified head: final PR CI is authoritative; this plan update intentionally precedes the final exact-head run

## Objective
Establish the production-intended application/runtime composition boundary that treats a persisted campaign as runnable only after its exact persisted ruleset and content-pack references resolve through a validated local `ContentCatalog`.

## Scope
- Add the missing top-level application/runtime composition crate if required to combine raw persistence with content resolution without reversing architecture dependencies.
- Provide application-facing create/open/resume/restore operations that fail closed on catalog-load or exact-content resolution failure before returning runnable campaign state.
- Preserve raw persistence create/open/export/restore APIs as content-agnostic recovery/diagnostic mechanisms.
- Add integration coverage using real SQLite persistence and real local manifest/catalog fixtures for success, restart/restore, missing or wrong-version content, incompatible packs, corrupt local content, unrelated-campaign isolation, and raw-vs-runnable access.
- Update the architecture dependency guard and durable architecture documentation for the new composition direction.
- Make `RunnableCampaign` an actual capability boundary: downstream crates may inspect it through read-only accessors but cannot construct one directly.

## Non-goals
- Gameplay rules or Gate 2 mechanics.
- Content installation UX or explicit content-version migration workflows.
- Licensing/redistribution policy or cryptographic publisher trust.
- Voice/UI work.
- Changing persistence save semantics or making persistence interpret content policy.

## Relevant durable context
- `docs/product-definition.md` — real application path, restart/recovery, offline operation, campaign isolation.
- `docs/checkpoints/gate-1.md` — Gate 1 persistence/content/lifecycle integration acceptance.
- ADR 004 — engine/content/campaign isolation.
- ADR 013 — derivative query projections remain non-authoritative.
- ADR 014 — exact versioned content resolution and raw-persistence/content-policy separation.
- ADR 015 — lifecycle/portability and raw restore semantics.
- ADR 016 — runnable-campaign application composition introduced by this work.

## Acceptance criteria
- [x] `dmd-persistence` gains no dependency on rules/core/application/content-policy layers; `dmd-core` gains no persistence dependency.
- [x] The application/runtime composition layer can load a validated `ContentCatalog` from configured local roots and exposes catalog-load errors explicitly.
- [x] Application-facing create resolves exact content before persisting/returning a runnable campaign.
- [x] Application-facing open/resume resolves the persisted campaign references before returning runnable campaign state.
- [x] Application-facing restore preflights the exported campaign's exact content before committing raw restore and resolves again before returning runnable state.
- [x] Missing manifests, wrong versions, incompatible packs, missing dependencies, corrupt declared files, and catalog-load failures fail explicitly with no substitution/degraded runnable mode.
- [x] Raw persistence access can still inspect an unresolved campaign and is mechanically distinct from the runnable application API.
- [x] `RunnableCampaign` cannot be fabricated by downstream crates: its fields are private, construction remains private to `dmd-app`, and callers have borrow-only `lifecycle()`, `state()`, and `content()` accessors.
- [x] Real SQLite + local manifest integration tests cover successful create/open/restart/restore, failure cases, and unrelated-campaign isolation.
- [x] Architecture dependency guard covers the new crate direction.
- [x] Full diff reviewed against ADRs/checkpoint; unrelated lockfile churn removed.
- [ ] Final repository CI is green on the exact control-review head. This remains a PR-check gate rather than being self-certified by the commit that updates this plan.
- [x] PR summary and this plan record validation status and unresolved debt before control review. The PR summary is finalized after exact-head CI so it can record the immutable SHA/run.

## Completed slices
1. Introduced `dmd-app` as the outer application/runtime composition boundary and documented the dependency direction in ADR 016.
2. Implemented `CampaignRuntime` and `RunnableCampaign` over raw persistence + `ContentCatalog` with explicit catalog/content/lifecycle error separation.
3. Added real file-backed SQLite + on-disk manifest integration coverage for create/open/resume/restart/restore, exact-version failure, missing content/dependencies, incompatibility, declared-file corruption, catalog failure, campaign isolation, and raw recovery access.
4. Extended the mechanical architecture dependency guard and performed a complete PR diff review.
5. Closed control-review finding #5302630835 by making `RunnableCampaign` fields private and updating external integration tests to consume the capability only through read-only accessors.

## Decision log
- 2026-09-24 — Use a top-level application/runtime crate rather than adding content-policy dependencies to persistence or persistence dependencies to core; ADR 014 already assigns runnable-campaign content resolution to this composition boundary.
- 2026-09-24 — Keep raw lifecycle APIs public and content-agnostic for recovery/diagnostics; the new application type is the only API that returns a value explicitly typed/named as runnable.
- 2026-09-24 — Reload/validate the configured local `ContentCatalog` at every runnable boundary for Gate 1. This makes content removal/change/corruption visible on the next create/open/resume/restore and avoids an indefinitely trusted process-local catalog snapshot.
- 2026-09-24 — Runnable restore resolves the exported current state's exact references before invoking raw persistence restore, then resolves the persistence-returned state again. Persistence remains responsible for complete export integrity validation.
- 2026-09-24 — Raw recovery remains deliberately available when content is unresolved; callers that intend gameplay must cross the distinct `RunnableCampaign` application boundary.
- 2026-09-24 — `RunnableCampaign` fields are private and only immutable references are exposed. The only constructor remains `CampaignRuntime::make_runnable`, so Rust visibility mechanically prevents downstream fabrication of runnable capability state.

## Validation
- Full repository CI run #265 on implementation head `a1e1b83fcde1bda60de8f9f2319229179f7391a0` — green: `verify-fast`, Clippy, workspace tests, Rust 1.88 MSRV, genericity guard, architecture guard.
- Full repository CI run #269 on pre-review head `0ad38bdb81f9fab241dbaf7fb2d592ef8e6eb60c` — green across the same repository checks.
- Targeted `cargo test --locked -p dmd-app --test runnable_campaign` executed successfully while applying the capability-sealing fix before commit `8bf4d1d9c0c9f3185e56426ab94099bd8908abfa` was pushed.
- `scripts/verify` inspection — its checks are the same `verify-fast` + Clippy + workspace tests + genericity + architecture sequence exercised by CI; CI additionally exercises the declared Rust 1.88 MSRV.
- Full diff review — complete PR file set inspected; temporary formatting/lock/capability helper workflows removed; `Cargo.lock` minimized to only the new `dmd-app` workspace package entry.
- Final exact-head CI after this plan update — pending and required before declaring control re-review readiness externally.

## Unresolved debt / deferred work
- `CampaignRuntime` reloads and verifies all configured local manifests on every runnable operation. Catalog caching/change watching is intentionally deferred until there is measured need; any future optimization must preserve equivalent change detection and fail-closed validation.
- `dmd-app` is currently the library-level composition boundary, not a user-facing executable shell. Wiring future UI/voice/gameplay surfaces through it is later work and must not bypass `RunnableCampaign`.
- Content installation UX, version migration/remediation flows, licensing policy, and publisher trust remain intentionally outside Gate 1 scope.
- Raw persistence lifecycle APIs remain public by design for diagnostics/recovery. As additional production entrypoints appear, architecture guards should be extended mechanically so gameplay-facing code cannot accidentally adopt raw `OpenCampaign` as runnable state.

## Blockers
- No implementation blocker is known after closing control-review finding #5302630835. Final exact-head repository CI remains the mandatory re-review gate.
- Local Rust execution is unavailable in this chat environment, so GitHub Actions is the directly verified compiler/test authority for pushed heads.

## Next action
Allow the final exact-head CI run to complete, record its SHA/run in PR #12, and stop for human control re-review without merging.
