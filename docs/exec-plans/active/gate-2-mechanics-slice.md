# Gate 2 mechanics slice

Status: Active. Sole writer: mechanics agent on `codex/gate2-mechanics-kernel`, baseline `90b0ec1` refreshed from origin before implementation.

Objective: implement the typed domain mechanics and pure rules kernel consumed by the existing runnable-campaign application. Root integrates persistence and independently reviews this slice.

Scope: all Gate 2 resolution primitives, pinned SRD 5.2.1 kernel definitions, deterministic dice requests/results, issuer ownership, validation, semantic action replay and read-only queries. No app/persistence edits; the compatibility slice owns schema-version changes. Gate 4 spatial/complete tactical systems, Gate 5 complete noncombat systems and Gate 6 complete character/content catalogs remain explicitly deferred, not redefined as completed.

Contract: Gate 2 checkpoint; product rules fidelity, local authority, physical dice and exact suspension requirements; ADRs 002/004/005/009–012/014–016. Official source: SRD CC 5.2.1 PDF, SHA256 `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`; source inventory/attribution supplied by the separate ledger slice.

Acceptance: pure validated transitions never mutate on error; players supply selected action IDs/raw faces, never authoritative DC/modifier/after-state; domain snapshots retain pending/resolved rolls, effects/resources/timing. Replay re-resolves recorded typed actions with exact metadata and compares outcomes. Tests cover arithmetic, rule distinctions, privilege/stale/pending failure, expiry/recovery and malformed definitions.

Slices: (1) state/dice contracts, (2) data validation and resolver, (3) adversarial tests and full verification, (4) coherent commit for root review/cherry-pick.

Decisions: domain holds serializable dice records; rules exports the `ResolveRoll` trait. One semantic event per accepted action (`rules.action_resolved`@1), containing original trusted metadata/action and independently recomputed outcome. Caller supplies stable request/effect identifiers. Trusted adjudication supplies contextual facts/DC with explicit provenance until spatial/noncombat owners implement those derivations.

Validation: pending implementation. Toolchain: workspace-local Rust 1.98.1 GNU, separate target directory. Run `./scripts/verify-fast` during iteration and `./scripts/verify` on completed slice.

Risks: full SRD catalogs/class progression and spatial condition clauses require later gates; every unsupported action must reject or stay visibly deferred. Exact next action: add typed state contracts and send stable integration types to root.
