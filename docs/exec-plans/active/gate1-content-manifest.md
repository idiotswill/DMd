# Gate 1 versioned rules/content manifest

Status: review blockers under repair
Branch: gate1/content-manifest
PR: #11 (draft)
Base at start: main @ d450766b1b834d739c586db5594de6dd23dd9722
Initial verified head: d450766b1b834d739c586db5594de6dd23dd9722
Reviewed head: 8542c93802b65df2af3dab74686d2ef2e43e7c9f

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

## Non-goals
- Campaign backup/archive/delete/purge/export/restore implementation.
- Normalized gameplay projection storage.
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
- ADR 013 `docs/architecture/013-versioned-content-manifests.md` — exact manifest identity, compatibility, local authority, lifecycle integration requirement, and future migration boundary introduced by this work.

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
- [ ] Unknown JSON fields at top-level and nested manifest schema boundaries fail closed, including typo regression coverage.
- [ ] Every component of a declared content path is checked without following symlinks; intermediate symlink escape regression passes.
- [ ] `./scripts/verify-fast` passes on the repaired implementation head.
- [ ] The constituent full verification checks pass on the repaired implementation head.
- [ ] PR CI is green on the exact repaired final head.
- [ ] Full diff is re-inspected against ADR/checkpoint invariants after blocker fixes.

## Implemented slices
1. Added typed manifest/catalog/resolution infrastructure in `dmd-domain` with exact versions, deterministic local discovery, explicit compatibility/dependency checks, and stable non-security file integrity metadata without introducing campaign content.
2. Added generic integration tests for unrelated rules/content combinations plus missing, wrong-version, incompatible, duplicate, malformed, unsafe-path, missing-file, and corrupt-content failure modes.
3. Added ADR 013 documenting manifest authority/compatibility, provider non-authority, explicit future upgrades, and the lifecycle create/open/resume/restore integration requirement without editing the shared Gate 1 checkpoint text.
4. Opened draft PR #11 early and used CI to fix rustfmt and Clippy failures without suppressing checks.
5. Control review #5296031702 on head `8542c93802b65df2af3dab74686d2ef2e43e7c9f` found two merge blockers: permissive unknown-field deserialization and intermediate symlink traversal in declared content paths. Repair is in progress; prior green CI run #201 is not evidence for the repaired head.

## Decision log
- 2026-09-23 — Keep `Campaign.ruleset`/`content_packs` as persisted exact refs; resolution must never rewrite them to an installed “closest” version.
- 2026-09-23 — Keep raw SQLite persistence generic. Content authority belongs to the manifest/domain compatibility layer; lifecycle/application callers must resolve after load and before a campaign is considered runnable.
- 2026-09-23 — Place the manifest identity/validation/catalog contract in `dmd-domain`, next to persisted `VersionedRef` semantics, because it contains no concrete rules data or campaign lore and introduces no dependency from domain state to `dmd-rules`. If a dedicated application/content-installation composition layer appears later, filesystem discovery can be extracted without changing the exact manifest contract.
- 2026-09-23 — Keep campaign-save `VersionedRef` serde behavior unchanged while making manifest parsing strict: manifest fields deserialize references through a private `deny_unknown_fields` wire type so this review fix does not silently change the broader save format.
- 2026-09-23 — Declared file verification must inspect every relative path component with `symlink_metadata` before reading the terminal file; direct manifest-file roots must be just as safe as directory discovery roots.
- 2026-09-23 — Do not encode permissive/restrictive licensing policy in this slice. Legal/licensing policy remains deferred until verified from authoritative current sources.
- 2026-09-23 — Pack file integrity is corruption detection, not DRM/authenticity. Manifest schema v1 names `fnv1a64` explicitly so stronger digest/signature mechanisms can evolve by schema/content-contract version without silently changing existing saves.
- 2026-09-23 — Duplicate installed manifest identities fail catalog construction rather than allowing search-root order to decide authority.

## Validation
- CI run #201 on reviewed head `8542c93802b65df2af3dab74686d2ef2e43e7c9f`: `verify-fast`, Clippy, workspace tests, Rust 1.88 MSRV, genericity guard, and architecture guard passed, but review found gaps not covered by those tests.
- Repaired-head validation — pending.
- `./scripts/verify` is not invoked as one wrapper by CI, but CI executes its constituent commands plus the MSRV check. Do not claim the wrapper itself was directly run unless it is run directly.

## Risks / blockers / deferred debt
- Merge blocked until both findings in control review #5296031702 are fixed and exact-head CI is green.
- No application/lifecycle composition crate currently owns “open runnable campaign”. ADR 013 requires the separate `gate1/campaign-lifecycle` work to resolve campaign content before create/open/resume/restore is considered runnable; this branch intentionally does not implement lifecycle behavior.
- A future manifest schema may need cryptographic digests, publisher signatures, or trust stores for authenticity. Gate 1 provides deterministic local authority and corruption detection only.
- Content installation/distribution UX and source provenance are not implemented here.
- Explicit rules/content upgrade and migration workflows remain future work; resolution never mutates stored refs.
- Licensing/proprietary-content policy remains unresolved. No third-party rules corpus or licensing entitlement assumption was added; any future policy must be verified from authoritative current sources first.

## Next action
Commit the strict manifest-schema and component-wise symlink repairs with regression tests, run exact-head CI, inspect the complete diff, update this plan and PR summary, and hand back to control review without merging.
