# Gate 1 runnable campaign integration

Status: implementation and control review complete; merge gates remain
Branch: gate1/runnable-campaign-integration
PR: #12 (draft)
Base: main/f3f18568be62b49458f8085d9ecc635ff87162d3
Verified implementation/control-review head: 975c67e2747f9769ecb8b5cf52eebea44ddcd8db — CI #273 green

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
- ADR 016 — runnable-campaign application composition introduced by this work; still Proposed pending explicit human approval.

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
- [x] Final implementation/control-review repository CI is green on exact head `975c67e2747f9769ecb8b5cf52eebea44ddcd8db` in CI run #273.
- [x] PR summary and this plan record validation status and unresolved debt before merge consideration.

## Completed slices
1. Introduced `dmd-app` as the outer application/runtime composition boundary and documented the dependency direction in ADR 016.
2. Implemented `CampaignRuntime` and `RunnableCampaign` over raw persistence + `ContentCatalog` with explicit catalog/content/lifecycle error separation.
3. Added real file-backed SQLite + on-disk manifest integration coverage for create/open/resume/restart/restore, exact-version failure, missing content/dependencies, incompatibility, declared-file corruption, catalog failure, campaign isolation, and raw recovery access.
4. Extended the mechanical architecture dependency guard and performed a complete PR diff review.
5. Closed control-review finding #5302630835 by making `RunnableCampaign` fields private and updating external integration tests to consume the capability only through read-only accessors.
6. Reconciled this execution plan after control review so durable repository memory records CI #273 and completed implementation validation accurately.

## Decision log
- 2026-09-24 — Use a top-level application/runtime crate rather than adding content-policy dependencies to persistence or persistence dependencies to core; ADR 014 already assigns runnable-campaign content resolution to this composition boundary.
- 2026-09-24 — Keep raw lifecycle APIs public and content-agnostic for recovery/diagnostics; the new application type is the only API that returns a value explicitly typed/named as runnable.
- 2026-09-24 — Reload/validate the configured local `ContentCatalog` at every runnable boundary for Gate 1. This makes content removal/change/corruption visible on the next create/open/resume/restore and avoids an indefinitely trusted process-local catalog snapshot.
- 2026-09-24 — Runnable restore resolves the exported current state's exact references before invoking raw persistence restore, then resolves the persistence-returned state again. Persistence remains responsible for complete export integrity validation.
- 2026-09-24 — Raw recovery remains deliberately available when content is unresolved; callers that intend gameplay must cross the distinct `RunnableCampaign` application boundary.
- 2026-09-24 — `RunnableCampaign` fields are private and only immutable references are exposed. The only constructor remains `CampaignRuntime::make_runnable`, so Rust visibility mechanically prevents downstream fabrication of runnable capability state.
- 2026-09-24 — CI #273 on `975c67e2747f9769ecb8b5cf52eebea44ddcd8db` completed the implementation/control-review validation. This documentation-only reconciliation necessarily creates a later PR head; GitHub CI status on that immutable later head is the authoritative merge gate rather than another plan mutation that would recursively create yet another head.

## Validation
- Full repository CI run #265 on implementation head `a1e1b83fcde1bda60de8f9f2319229179f7391a0` — green: `verify-fast`, Clippy, workspace tests, Rust 1.88 MSRV, genericity guard, architecture guard.
- Full repository CI run #269 on pre-review head `0ad38bdb81f9fab241dbaf7fb2d592ef8e6eb60c` — green across the same repository checks.
- Targeted `cargo test --locked -p dmd-app --test runnable_campaign` executed successfully while applying the capability-sealing fix before commit `8bf4d1d9c0c9f3185e56426ab94099bd8908abfa` was pushed.
- Full repository CI run #273 on exact implementation/control-review head `975c67e2747f9769ecb8b5cf52eebea44ddcd8db` — green: `verify-fast`, Clippy, workspace tests, Rust 1.88 MSRV, genericity guard, architecture guard.
- `scripts/verify` inspection — its checks are the same `verify-fast` + Clippy + workspace tests + genericity + architecture sequence exercised by CI; CI additionally exercises the declared Rust 1.88 MSRV.
- Full diff review — complete PR file set inspected; temporary formatting/lock/capability helper workflows removed; `Cargo.lock` minimized to only the new `dmd-app` workspace package entry.
- This reconciliation commit changes only the active execution plan. Before merge, GitHub CI must be green on the PR's exact current head; that external status is authoritative and should not be echoed by another plan-only commit solely to record it.

## Unresolved debt / deferred work
- `CampaignRuntime` reloads and verifies all configured local manifests on every runnable operation. Catalog caching/change watching is intentionally deferred until there is measured need; any future optimization must preserve equivalent change detection and fail-closed validation.
- `dmd-app` is currently the library-level composition boundary, not a user-facing executable shell. Wiring future UI/voice/gameplay surfaces through it is later work and must not bypass `RunnableCampaign`.
- Content installation UX, version migration/remediation flows, licensing policy, and publisher trust remain intentionally outside Gate 1 scope.
- Raw persistence lifecycle APIs remain public by design for diagnostics/recovery. As additional production entrypoints appear, architecture guards should be extended mechanically so gameplay-facing code cannot accidentally adopt raw `OpenCampaign` as runnable state.

## Merge gates
- GitHub CI must be green on the exact current PR head after this plan-only reconciliation commit.
- ADR 016 remains **Proposed** and requires explicit human approval before merge under `AGENTS.md` human-approval boundaries.
- PR #12 must remain unmerged until both gates are satisfied.

## Blockers
- No implementation or control-review blocker remains.
- Merge eligibility is governed by the exact-head CI gate and explicit human approval of ADR 016.
- Local Rust execution is unavailable in this chat environment, so GitHub Actions is the directly verified compiler/test authority for pushed heads.

## Next action
Check GitHub CI on the current PR head. If it is green and a human explicitly approves ADR 016, PR #12 may be considered for merge; otherwise keep it draft and unmerged. Do not create another plan-only commit solely to echo the external CI result, because that would create a new head and recreate the same governance loop.
