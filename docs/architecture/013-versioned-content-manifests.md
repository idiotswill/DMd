# ADR 013 — Versioned rules/content manifests

Status: **Proposed for Gate 1 content-manifest blocker — PR #11**

## Context

`Campaign` already persists exact `VersionedRef` values for one ruleset and zero or more content packs. Before this decision those strings had no production resolution contract: a structurally valid save could be loaded even when the referenced rules/content was absent, a different version was installed, or local content files had changed.

That is not a safe save boundary. Existing campaign state must not silently change meaning because local content was upgraded, replaced, damaged, or inferred by a provider.

ADR 004 also requires engine code, rules/content definitions, world templates, and mutable campaign state to remain separate. This slice therefore establishes content identity and compatibility without importing a complete tabletop corpus or campaign lore into the engine.

## Decision

### Manifest identity is exact and versioned

An installed manifest is identified by the tuple:

```text
(kind, id, version)
```

where `kind` is either `ruleset` or `content_pack`.

`id` and `version` are stable opaque identity tokens. Gate 1 does not assign ordering semantics to version strings and does not perform semver range selection. A campaign reference resolves only to the exact persisted `id` + `version` for the required kind.

Installing a newer or older version never substitutes for the version named by an existing save.

### Manifest schema and engine content contract are independently versioned

Every manifest declares:

- `manifest_schema_version` — the shape/meaning of the manifest document;
- `content_contract_version` — the engine/content interface the manifest targets.

Gate 1 supports manifest schema version 1 and content-contract version 1. Unknown schema versions or incompatible content-contract versions fail explicitly. Future support must be added deliberately; it is not inferred from JSON shape.

Manifest schema version 1 is a closed schema: unknown fields are rejected at the manifest, declared-file, checksum, and manifest-reference boundaries. Optional lists may default only when the declared field is genuinely absent; a misspelled field is an invalid manifest rather than an empty constraint set.

### Content-pack compatibility is explicit

A ruleset manifest cannot declare ruleset compatibility or pack dependencies.

A content-pack manifest must declare at least one exact compatible ruleset reference and may declare exact content-pack dependencies. When a campaign is resolved:

1. its ruleset must exist at the exact persisted version;
2. each content pack must exist at the exact persisted version;
3. each content pack must list the exact campaign ruleset as compatible;
4. each content-pack dependency must be explicitly present in the campaign at the exact required version;
5. one campaign may not select multiple versions of the same content-pack ID.

There is no implicit compatibility based on similar names, latest versions, provider suggestions, or load order.

### Local discovery is deterministic and network-independent

`ContentCatalog::load_from_roots` discovers files named `manifest.json` under explicitly configured local roots. Directory entries and discovered manifest paths are sorted before loading, and symlinks are rejected rather than followed.

Campaign play therefore does not require GitHub, Drive, a package registry, an LLM provider, or any other network service to establish content authority.

Duplicate installed manifests with the same exact identity fail catalog construction rather than making root/search order authoritative.

### Declared content files are integrity-checked

A manifest may list relative regular files with exact byte length and a named checksum algorithm. Manifest paths that are absolute, contain parent traversal, or otherwise escape the pack directory are invalid.

Every path component between the manifest directory and a declared content file is inspected without following symlinks. A symlink at either an intermediate directory component or the terminal file fails closed, so a declared relative path cannot escape the pack through symlink traversal.

Manifest schema version 1 supports `fnv1a64` as a stable, dependency-free corruption checksum. It detects ordinary missing/changed/corrupt content but is **not** a cryptographic authenticity mechanism, signature scheme, DRM system, or licensing control. A future manifest schema/content contract may add cryptographic digests and publisher signatures without changing the meaning of version-1 manifests.

### Provider/LLM output is never content authority

Only the campaign's persisted `VersionedRef` values plus the configured local catalog participate in resolution. Provider text, model output, narration, prompts, or arbitrary JSON cannot install, select, upgrade, or authorize a ruleset/content pack.

If AI-assisted tooling later proposes content changes, those proposals must cross an application-defined validation/installation boundary before they can become locally installed content.

### Raw persistence remains content-agnostic

SQLite persistence continues to store/recover authoritative campaign state and journal history without interpreting rules/content semantics.

The application/lifecycle boundary that creates, opens, resumes, restores, or otherwise declares a campaign **runnable** must resolve `state.campaign.ruleset` and `state.campaign.content_packs` through a validated `ContentCatalog` first. Failure to resolve is a runnable-campaign failure, not permission to continue with degraded/substituted content.

This is the interface requirement for the separate `gate1/campaign-lifecycle` branch. That branch owns create/open/export/restore behavior; this branch does not implement lifecycle operations.

Replay/recovery tooling may still inspect raw historical state for diagnosis, but must not present unresolved state as safe gameplay state.

### Upgrades are explicit future operations

Resolution never rewrites campaign references. Changing a campaign from one ruleset/content version to another is a future explicit migration/upgrade operation that must define compatibility, state transformation where required, provenance, failure/rollback behavior, and save-format consequences.

A content update on disk therefore cannot silently alter the interpretation of an existing save.

## Failure behavior

The manifest/catalog boundary fails explicitly on representative cases including:

- malformed manifest JSON;
- unknown manifest or nested schema fields;
- unsupported manifest schema;
- incompatible engine content contract;
- invalid identity/reference tokens;
- duplicate manifest identity;
- missing exact ID;
- installed ID with the wrong version;
- manifest kind mismatch;
- ruleset/content incompatibility;
- missing exact pack dependency;
- duplicate pack ID in one campaign;
- unsafe declared file path;
- missing/non-regular declared file;
- declared file length or checksum mismatch;
- symlinked discovery/content paths, including intermediate declared-file path components.

These failures do not mutate campaign state or choose substitutes.

## Separation and licensing

This ADR defines content identity, compatibility, local resolution, and corruption detection only.

It does not authorize copying, redistributing, bundling, converting, or ingesting any third-party tabletop corpus. No current campaign lore, setting names, PCs, proprietary rules text, or assumption about exactly four characters belongs in the manifest infrastructure.

Any future repository policy about third-party licensing, redistribution, SRD/open-content terms, proprietary imports, or bundled content must be verified against authoritative current sources before adoption. Descriptive manifest metadata must not be treated as proof that DMd has legal rights to distribute content.

## Consequences

- Campaign saves now have a mechanical exact-content resolution contract instead of unvalidated strings.
- Multiple unrelated rulesets/content packs can coexist without engine changes or campaign-specific names.
- Local content damage and incompatible versions fail closed before a campaign is considered runnable.
- Save meaning is stable across incidental local upgrades because no version substitution occurs.
- Lifecycle create/open/restore must integrate this resolver before Gate 1 can claim end-to-end runnable campaign lifecycle behavior.
- Strong publisher authenticity/signature trust, content installation UX, explicit content-version migration workflows, and licensing policy remain future work.

## Rejected alternatives

### Resolve the newest installed version

Rejected. That silently changes the meaning of an existing save and makes rollback/recovery nondeterministic.

### Let persistence interpret or repair content references

Rejected. Persistence owns durable state/history integrity, not gameplay/content compatibility policy.

### Let providers select missing content

Rejected. Provider/LLM output is never authoritative game state or content authority.

### Embed one rules corpus in core code

Rejected. It violates campaign/content genericity and would turn this Gate 1 foundation into a campaign/system-specific implementation.

### Treat checksum metadata as licensing/authenticity

Rejected. Corruption detection and legal/publisher trust are separate concerns and require different evidence/mechanisms.
