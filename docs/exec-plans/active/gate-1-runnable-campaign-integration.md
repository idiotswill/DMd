# Gate 1 runnable campaign integration

Status: in progress
Branch: gate1/runnable-campaign-integration
PR: pending
Base: main/f3f18568be62b49458f8085d9ecc635ff87162d3
Verified head: pending

## Objective
Establish the production-intended application/runtime composition boundary that treats a persisted campaign as runnable only after its exact persisted ruleset and content-pack references resolve through a validated local `ContentCatalog`.

## Scope
- Add the missing top-level application/runtime composition crate if required to combine raw persistence with content resolution without reversing architecture dependencies.
- Provide application-facing create/open/resume/restore operations that fail closed on catalog-load or exact-content resolution failure before returning runnable campaign state.
- Preserve raw persistence create/open/export/restore APIs as content-agnostic recovery/diagnostic mechanisms.
- Add integration coverage using real SQLite persistence and real local manifest/catalog fixtures for success, restart/restore, missing or wrong-version content, incompatible packs, corrupt local content, unrelated-campaign isolation, and raw-vs-runnable access.
- Update the architecture dependency guard and durable architecture documentation for the new composition direction.

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

## Acceptance criteria
- [ ] `dmd-persistence` gains no dependency on rules/core/application/content-policy layers; `dmd-core` gains no persistence dependency.
- [ ] The application/runtime composition layer can load a validated `ContentCatalog` from configured local roots and exposes catalog-load errors explicitly.
- [ ] Application-facing create resolves exact content before persisting/returning a runnable campaign.
- [ ] Application-facing open/resume resolves the persisted campaign references before returning runnable campaign state.
- [ ] Application-facing restore preflights the exported campaign's exact content before committing raw restore and resolves again before returning runnable state.
- [ ] Missing manifests, wrong versions, incompatible packs, missing dependencies, corrupt declared files, and catalog-load failures fail explicitly with no substitution/degraded runnable mode.
- [ ] Raw persistence access can still inspect an unresolved campaign and is mechanically distinct from the runnable application API.
- [ ] Real SQLite + local manifest integration tests cover successful create/open/restart/restore, failure cases, and unrelated-campaign isolation.
- [ ] Architecture dependency guard covers the new crate direction.
- [ ] Full diff is reviewed against ADRs/checkpoint; `./scripts/verify`/repository CI passes on the exact final head.
- [ ] PR summary and this plan record validation and unresolved debt before control review.

## Planned slices
1. Introduce/document the application/runtime composition boundary and dependency direction.
2. Implement runnable campaign wrapper/service over raw persistence + `ContentCatalog`.
3. Add real SQLite/local-manifest integration tests, including failure/isolation/recovery distinction.
4. Run CI, inspect full diff, fix failures, update plan/PR, and stop for control review.

## Decision log
- 2026-09-24 — Use a top-level application/runtime crate rather than adding content-policy dependencies to persistence or persistence dependencies to core; ADR 014 already assigns runnable-campaign content resolution to this composition boundary.
- 2026-09-24 — Keep raw lifecycle APIs public and content-agnostic for recovery/diagnostics; the new application type is the only API that returns a value explicitly typed/named as runnable.

## Validation
- `./scripts/verify-fast` — pending (local execution unavailable in this environment; CI will be used on exact pushed heads)
- `./scripts/verify` — pending (repository CI equivalent required on exact final head)
- CI — pending

## Risks / blockers
- No local network-backed checkout/compiler is available in this execution environment, so compile/test feedback must come from GitHub Actions after coherent branch commits.
- Restore preflight must validate content before raw restore commits, without duplicating or weakening persistence's export-integrity validation.

## Next action
Open the draft PR, then add the application/runtime crate and integration tests as one coherent implementation commit.
