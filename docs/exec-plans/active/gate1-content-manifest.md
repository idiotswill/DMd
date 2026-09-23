# Gate 1 versioned rules/content manifest

Status: in progress
Branch: gate1/content-manifest
PR: pending
Base: main @ d450766b1b834d739c586db5594de6dd23dd9722
Verified head: d450766b1b834d739c586db5594de6dd23dd9722 (pre-plan)

## Objective
Make campaign `ruleset` and `content_packs` references resolve against explicit, versioned, validated, local content manifests so existing saves cannot silently change meaning or run against missing/incompatible content.

## Scope
- Define a production manifest identity/version/schema contract for rulesets and content packs.
- Add deterministic local-first manifest discovery, structure validation, integrity checks, duplicate detection, and exact `VersionedRef` resolution.
- Mechanically validate engine/content compatibility and ruleset/content-pack compatibility.
- Return explicit failures for missing IDs, wrong versions, incompatible manifests, duplicates, malformed manifests, and corrupted listed content files.
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

## Acceptance criteria
- [ ] Manifest identity is `(kind, id, version)` with an explicit manifest schema version and engine content-contract version.
- [ ] Campaign `VersionedRef` values resolve exactly; installed newer/older versions do not silently substitute.
- [ ] Missing ID, wrong version, kind mismatch, incompatible ruleset, duplicate identity, malformed manifest, unsafe file path, missing file, and file-integrity mismatch fail explicitly.
- [ ] Discovery order is deterministic and network-independent.
- [ ] Content-pack compatibility with the campaign ruleset and exact pack dependencies is mechanically checked.
- [ ] Campaign refs remain unchanged during resolution; upgrades require an explicit future migration/change path.
- [ ] Engine/rules code contains no current campaign/setting/PC names or four-character assumptions.
- [ ] Tests prove multiple unrelated ruleset/content-pack combinations resolve independently and incompatibilities do not leak across campaigns.
- [ ] Missing/incompatible/corrupt cases are exercised through tests.
- [ ] `./scripts/verify-fast` passes on the implementation head.
- [ ] `./scripts/verify` passes on the implementation head.
- [ ] PR CI is green on the exact final head.
- [ ] Full diff is inspected against ADR/checkpoint invariants before ready-for-review.

## Planned slices
1. Add typed manifest/catalog/resolution infrastructure in `dmd-rules`, using exact versions and stable non-security file integrity metadata without introducing campaign content.
2. Add generic fixtures/tests for deterministic discovery, exact resolution, compatibility, duplicate/malformed/corrupt failures, and unrelated campaign combinations.
3. Add a focused architecture contract for manifest authority/compatibility and lifecycle integration requirements; avoid editing shared Gate 1 checkpoint text unless needed at closeout.
4. Run fast/full verification, inspect the full PR diff, update plan/PR summary, record licensing/content and future migration debt, and verify CI on the exact head.

## Decision log
- 2026-09-23 — Keep `Campaign.ruleset`/`content_packs` as persisted exact refs; resolution must never rewrite them to an installed “closest” version.
- 2026-09-23 — Keep raw SQLite persistence generic. Content authority belongs to the rules/content layer; lifecycle/application callers must resolve after load and before a campaign is considered runnable.
- 2026-09-23 — Do not encode permissive/restrictive licensing policy in this slice. Legal/licensing metadata, if represented, is descriptive only until policy assumptions are verified from authoritative current sources.
- 2026-09-23 — Pack file integrity is corruption detection, not DRM/authenticity. The manifest contract must name the integrity algorithm explicitly so it can evolve by manifest schema version.

## Validation
- `./scripts/verify-fast` — pending
- `./scripts/verify` — pending
- CI — pending

## Risks / blockers
- No application/lifecycle composition crate currently owns “open runnable campaign”; this branch will provide the resolver contract and document that lifecycle create/open/restore must require it rather than implementing lifecycle behavior here.
- A future manifest schema may need cryptographic signatures/trust stores for publisher authenticity; Gate 1 only needs deterministic local authority and corruption/compatibility checks.
- Licensing/proprietary-content policy remains unresolved and must be verified from authoritative current sources before repository policy is added.

## Next action
Create the draft PR, then implement the manifest/catalog resolver and generic failure-mode tests in `dmd-rules` without adding campaign lore or lifecycle behavior.
