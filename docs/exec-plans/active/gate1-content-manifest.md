# Gate 1 versioned rules/content manifest

Status: human-approved for merge; final exact-head CI required after this approval bookkeeping update
Branch: gate1/content-manifest
PR: #11 (draft until final merge step)
Base at start: main @ d450766b1b834d739c586db5594de6dd23dd9722
Integration target: main @ f4473745bb44e6fa112ae87833c9336553304c20 (merged PR #10 projections)
Reviewed head: 8542c93802b65df2af3dab74686d2ef2e43e7c9f
Pre-integration validated head: cb26ef1996390e27f6f71a37500f10a38425f59e
Post-integration implementation head: 3c531482e45539363b948d1cb0a1bd40b709e4c4
Final reviewed/bookkeeping head before approval: bce1b7b215850f15d9666b2611599b594bf0bc4f

## Objective
Make campaign `ruleset` and `content_packs` references resolve against explicit, versioned, validated, local content manifests so existing saves cannot silently change meaning or run against missing/incompatible content.

## Scope
- Define a production manifest identity/version/schema contract for rulesets and content packs.
- Add deterministic local-first manifest discovery, structure validation, integrity checks, duplicate detection, and exact `VersionedRef` resolution.
- Mechanically validate engine/content compatibility and ruleset/content-pack compatibility.
- Return explicit failures for missing IDs, wrong versions, incompatible manifests, duplicates, malformed manifests, corrupted listed content files, unknown manifest fields, and symlink path escapes.
- Keep provider/LLM output outside content authority; runtime authority comes only from configured local manifests plus campaign-persisted refs.
- Add generic tests covering unrelated rulesets/content packs and campaign combinations.
- Document the lifecycle/save interface requirement that campaign create/open/restore must resolve content before treating a save as runnable.
- Integrate current `main` after PR #10 without changing the reviewed fail-closed manifest behavior.
- Renumber the content-manifest ADR to ADR 014 because merged projections own ADR 013.

## Non-goals
- Campaign backup/archive/delete/purge/export/restore implementation.
- Normalized gameplay projection storage beyond accepting the already-merged PR #10 implementation from `main`.
- Complete tabletop/5e rules data or implementation.
- Procedural world generation, simulation/Director, voice/AI/UI.
- Importing current campaign lore, PCs, settings, or proprietary rules corpora.
- Establishing licensing policy without authoritative current-source verification.

## Relevant durable context
- `docs/product-definition.md` — unrelated campaigns, restart/recovery, local/offline operation, no campaign-specific assumptions.
- `docs/checkpoints/gate-1.md` — versioned rules/content manifest blocker.
- ADR 001 `docs/architecture/001-tech-stack.md` — modular Rust/local-first architecture and provider non-authority.
- ADR 004 `docs/architecture/004-content-and-campaign-isolation.md` — engine/content/world/campaign separation and versioned content.
- ADR 005 `docs/architecture/005-domain-state-model.md` — campaign manifest refs live in authoritative state.
- ADR 012 `docs/architecture/012-snapshot-migration-and-replay.md` — save compatibility must fail closed rather than guess.
- ADR 013 `docs/architecture/013-query-projections.md` — derivative query projection architecture merged through PR #10.
- ADR 014 `docs/architecture/014-versioned-content-manifests.md` — exact manifest identity, closed schema, compatibility, local authority, path safety, lifecycle integration requirement, and future migration boundary introduced by this work.

## Acceptance criteria
- [x] Manifest identity is `(kind, id, version)` with an explicit manifest schema version and engine content-contract version.
- [x] Campaign `VersionedRef` values resolve exactly; installed newer/older versions do not silently substitute.
- [x] Missing ID, wrong version, kind mismatch, incompatible ruleset, duplicate identity, malformed manifest, unsafe file path, missing file, and file-integrity mismatch fail explicitly.
- [x] Discovery order is deterministic and network-independent.
- [x] Content-pack compatibility with the campaign ruleset and exact pack dependencies is mechanically checked.
- [x] Campaign refs remain unchanged during resolution; upgrades require an explicit future migration/change path.
- [x] Engine/rules code contains no current campaign/setting/PC names or four-character assumptions.
- [x] Tests prove multiple unrelated ruleset/content-pack combinations resolve independently and incompatibilities do not leak across campaigns.
- [x] Missing/wrong-version/incompatible/duplicate/malformed/corrupt pack cases are exercised through tests.
- [x] Unknown JSON fields at top-level and nested manifest schema boundaries fail closed, including typo regression coverage.
- [x] Every component of a declared content path is checked without following symlinks; intermediate symlink escape regression passes.
- [x] Current `main` at `f4473745bb44e6fa112ae87833c9336553304c20` is incorporated while retaining PR #10 projection changes.
- [x] Content-manifest architecture document is renumbered from ADR 013 to ADR 014; projections retain ADR 013.
- [x] Post-integration implementation CI run #240 is green on `3c531482e45539363b948d1cb0a1bd40b709e4c4`.
- [x] Complete post-integration seven-file diff was inspected against task acceptance criteria and ADRs 013/014 on implementation head `3c531482e45539363b948d1cb0a1bd40b709e4c4`.
- [x] Final pre-approval CI run #241 is green on `bce1b7b215850f15d9666b2611599b594bf0bc4f`, and the final seven-file diff was re-inspected.
- [x] Human approval to merge PR #11 was given on 2026-09-24.
- [ ] CI is green on the exact approval-bookkeeping head created by this plan update.

## Implemented slices
1. Added typed manifest/catalog/resolution infrastructure in `dmd-domain` with exact versions, deterministic local discovery, explicit compatibility/dependency checks, and stable non-security file integrity metadata without introducing campaign content.
2. Added generic integration tests for unrelated rules/content combinations plus missing, wrong-version, incompatible, duplicate, malformed, unsafe-path, missing-file, and corrupt-content failure modes.
3. Added ADR 014 documenting manifest authority/compatibility, provider non-authority, explicit future upgrades, and the lifecycle create/open/resume/restore integration requirement without editing the shared Gate 1 checkpoint text.
4. Control review #5296031702 found two merge blockers: permissive unknown-field deserialization and intermediate symlink traversal in declared content paths. Both are fixed with regression coverage.
5. Control handoff comment #5808959536 identified post-PR-#10 integration bookkeeping: merge current `main`, reserve ADR 013 for projections, renumber manifests to ADR 014, and rerun exact-head CI.
6. Post-#10 integration preserves `main` as the tree baseline, overlays only the already-reviewed manifest module/export/tests, and carries both parents in a merge commit so projection work is not replayed or rewritten.
7. Final control/human review accepted the integrated branch and explicitly approved merging PR #11.

## Decision log
- 2026-09-23 — Keep `Campaign.ruleset`/`content_packs` as persisted exact refs; resolution must never rewrite them to an installed “closest” version.
- 2026-09-23 — Keep raw SQLite persistence generic. Content authority belongs to the manifest/domain compatibility layer; lifecycle/application callers must resolve after load and before a campaign is considered runnable.
- 2026-09-23 — Keep campaign-save `VersionedRef` serde behavior unchanged while making manifest parsing strict through a private `deny_unknown_fields` wire type.
- 2026-09-23 — Manifest schema v1 is closed at every currently nested manifest-owned JSON object; unknown fields fail parsing rather than being ignored.
- 2026-09-23 — Declared file verification inspects every relative path component with `symlink_metadata`; intermediate and terminal symlinks fail closed.
- 2026-09-23 — Pack file integrity is corruption detection, not DRM/authenticity. Manifest schema v1 names `fnv1a64` explicitly; stronger digest/signature mechanisms remain future work.
- 2026-09-24 — PR #10 merged projections to `main` and owns ADR 013. Content manifests move to ADR 014; no manifest behavior changes as part of the renumber.
- 2026-09-24 — Integrate `main` with a merge commit rather than force-rebasing published branch history. The merge tree uses current `main` as its baseline and adds only #11-owned content-manifest files plus the domain export, ADR 014, and this plan.
- 2026-09-24 — Human approval supersedes the earlier “do not merge yet” hold; merge remains gated only on exact-current-head CI and unchanged repository state.

## Validation
- CI run #220 on pre-integration head `cb26ef1996390e27f6f71a37500f10a38425f59e` was fully green, including both fail-closed blocker regressions, but is historical evidence only and is **not** used as post-#10 integration evidence.
- CI run #240 on post-integration implementation head `3c531482e45539363b948d1cb0a1bd40b709e4c4` is fully green: `verify-fast`, Clippy with warnings denied, workspace tests, Rust 1.88 MSRV, genericity guard, and architecture guard all passed.
- CI run #241 on final pre-approval head `bce1b7b215850f15d9666b2611599b594bf0bc4f` is fully green across the same jobs.
- The complete final seven-file PR diff against `main` was re-inspected on `bce1b7b215850f15d9666b2611599b594bf0bc4f`. `main` was rechecked and remained at `f4473745bb44e6fa112ae87833c9336553304c20` immediately before approval bookkeeping.
- `./scripts/verify` is not invoked as one wrapper by CI; CI executes its constituent commands plus the Rust 1.88 MSRV check. Do not claim the wrapper itself was directly run unless it is run directly.
- This approval bookkeeping commit moves the branch head, so exact-head CI must pass again before merge. No implementation files are changed by this update.

## Risks / blockers / deferred debt
- No known implementation or integration blocker remains. Merge is explicitly human-approved but must wait for exact-current-head CI after this plan-only update.
- No application/lifecycle composition crate currently owns “open runnable campaign”. ADR 014 requires the separate campaign-lifecycle work to resolve campaign content before create/open/resume/restore is considered runnable.
- A future manifest schema may need cryptographic digests, publisher signatures, or trust stores for authenticity. Gate 1 provides deterministic local authority and corruption detection only.
- Content installation/distribution UX and source provenance are not implemented here.
- Explicit rules/content upgrade and migration workflows remain future work; resolution never mutates stored refs.
- Licensing/proprietary-content policy remains unresolved. No third-party rules corpus or licensing entitlement assumption was added; any future policy must be verified from authoritative current sources first.

## Next action
Once CI is green on this approval-bookkeeping head and `main`/PR head remain unchanged, mark PR #11 ready for review and merge it to `main` using the exact expected head SHA. Then verify the resulting merge commit and `main` head.
