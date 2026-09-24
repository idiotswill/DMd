# ADR 016 — Runnable campaign application composition

Status: **Accepted — human-approved 2026-09-24 as the final architecture merge gate for PR #12.**

## Context

ADR 014 defines exact versioned rules/content resolution and deliberately keeps raw persistence content-agnostic. ADR 015 defines lifecycle create/open/export/restore as durable storage and recovery operations. After both landed, no production application boundary combined those responsibilities, so a caller could load persisted campaign state without any type or API distinction proving that its stored ruleset/content references still resolved.

Putting content policy into `dmd-persistence` would reverse the intended separation: persistence owns durable state/history integrity, not gameplay-content compatibility. Making `dmd-core` depend on persistence would likewise make the inner engine layer an infrastructure composition root.

## Decision

### `dmd-app` is the outer runtime composition boundary

Gate 1 introduces `dmd-app` as the production-intended outer application/runtime layer.

Its dependency direction is:

```text
dmd-app
  ├── dmd-domain        (campaign state + ContentCatalog/content contracts)
  └── dmd-persistence   (durable lifecycle/storage)

dmd-persistence ──> dmd-domain
```

No lower layer may depend on `dmd-app`. `dmd-persistence` remains forbidden from depending on `dmd-core`, `dmd-rules`, `dmd-conversation`, or `dmd-app`, and `dmd-core` remains forbidden from depending on persistence. The architecture boundary guard enforces these reverse-dependency prohibitions mechanically.

This ADR does not require `dmd-app` to own all future application concerns. It establishes the composition root where infrastructure and validated domain/content policy may be combined without contaminating inner layers.

### Runnable state is an application-layer capability

`dmd-persistence::OpenCampaign` remains a raw durable/recovery representation. It is not evidence that gameplay may safely run.

`dmd-app::RunnableCampaign` is returned only after the campaign's exact persisted `ruleset` and `content_packs` have resolved successfully through a freshly loaded, validated local `ContentCatalog`.

Application-facing create/open/resume/restore therefore fail closed on either:

- catalog construction/integrity failure (`CatalogLoadError`), or
- exact campaign content-resolution failure (`ContentResolutionError`).

No operation substitutes another version, ignores a missing dependency, or degrades into gameplay with unresolved content.

### Content is revalidated at every runnable boundary

`CampaignRuntime` stores configured local content roots, not an indefinitely trusted catalog snapshot. Each create/open/resume/restore operation reloads `ContentCatalog` from those roots before returning runnable state.

This catches content that disappeared, changed version, became corrupt, or otherwise stopped satisfying the stored campaign references after an earlier successful open. Gate 1 accepts the repeated local validation cost in exchange for a simple fail-closed boundary. Future caching is permitted only if it preserves equivalent change detection and validation guarantees.

### Create resolves before persistence mutation

Runnable create loads the catalog and resolves the proposed campaign first. If resolution fails, no campaign rows are written. Raw persistence create remains available separately for recovery-oriented/internal tooling that intentionally does not assert runnability.

### Open and resume resolve persisted references

Runnable open first reads the stored campaign through raw persistence, then resolves the exact references found in that persisted state. Only the application wrapper returns `RunnableCampaign`.

Resume is the same safety boundary as open rather than a separate content path.

### Restore preflights content before raw restore commits

Runnable restore decodes the export's persisted current `CampaignState`, loads the catalog, and resolves its exact references **before** invoking raw persistence restore. Missing/incompatible/corrupt content therefore cannot create a restored durable aggregate and only fail afterward.

Raw persistence restore still performs the complete ADR-015 export validation and transactional write. After it succeeds, the application boundary resolves the exact state returned from persistence again before producing `RunnableCampaign`.

Invalid exports may therefore fail independently at the persistence boundary even when their referenced content exists. Content resolution never replaces persistence integrity validation.

### Raw recovery and diagnostics remain intentionally available

Recovery/diagnostic/admin tooling may continue to call `dmd-persistence` directly to inspect, export, restore, archive, or purge unresolved campaign data where ADR 015 permits it.

That path is intentionally named and typed as persistence lifecycle data rather than `RunnableCampaign`. Application gameplay code must use the `dmd-app` runnable boundary.

## Consequences

- A persisted campaign is no longer equivalent to a runnable campaign at the production application boundary.
- Exact content pinning from ADR 014 is enforced end-to-end across create/open/resume/restore.
- Local content removal, wrong-version replacement, compatibility failure, missing dependencies, manifest/catalog corruption, and declared-file corruption fail explicitly before gameplay state is returned.
- Persistence and core dependency directions remain unchanged.
- Raw recovery remains possible when content is unavailable, without weakening gameplay safety.
- The current implementation reloads all configured local manifests on every runnable operation; catalog caching/change watching is deferred until measured need justifies it.

## Rejected alternatives

### Put `ContentCatalog` checks in `dmd-persistence`

Rejected. Persistence would become responsible for gameplay/content policy and raw recovery of unresolved campaigns would be coupled to local content availability.

### Make `dmd-core` the composition root

Rejected. Core would need a persistence dependency, violating the accepted inner-layer direction and mixing reusable engine logic with infrastructure composition.

### Resolve content only once at process startup

Rejected for Gate 1. Files can disappear or become corrupt while the process remains alive; an indefinitely cached success would let a later open cross the runnable boundary without revalidation.

### Restore first and resolve afterward

Rejected for the runnable API. It would leave a newly restored aggregate committed even though the operation promised a runnable campaign and content resolution failed.

### Remove raw persistence lifecycle APIs

Rejected. ADR 014 explicitly permits unresolved recovery/diagnostic access, and ADR 015 owns content-agnostic portability/integrity semantics.
