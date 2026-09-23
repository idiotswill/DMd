# Gate 1 — Query/materialized projections

Status: in progress
Branch: `gate1/projections`
PR: pending
Base: `main` @ `d450766b1b834d739c586db5594de6dd23dd9722`
Verified head: pending

## Objective

Build the production-intended normalized/materialized query projection layer required by Gate 1 so gameplay query paths do not depend permanently on deserializing `campaign_state_current.state_json`, while projections remain strictly derivative of accepted authoritative state/journal history and are recoverable from the existing snapshot+journal path.

## Scope

- identify the currently materialized Gate 0 domain records that need queryable relational projections now;
- add campaign-scoped projection schema and typed persistence query APIs for those records without inventing schemas for deferred gameplay systems;
- update projections inside the same SQLite transaction that accepts an authoritative state/journal transition;
- initialize/backfill projections for existing campaigns from their accepted current state during migration/startup as appropriate;
- provide deterministic projection rebuild from state reconstructed through the accepted snapshot+journal replay boundary;
- detect projection/head mismatch or malformed/cross-campaign projection data and fail closed rather than silently treating it as authority;
- mechanically constrain campaign isolation and projection metadata where SQLite can enforce it;
- add failure, corruption, rebuild, atomicity, and multi-campaign isolation tests through the real persistence path;
- document the durable projection contract in an ADR or focused update without changing unrelated Gate 1 blocker scope.

## Non-goals

- campaign create/archive/delete/backup/export/purge flows owned by `gate1/campaign-lifecycle`;
- rules/content-pack manifest resolution owned by `gate1/content-manifest`;
- gameplay rules or durable future combat/simulation/economy/world-generation models;
- voice, AI, UI, or product-scope changes;
- weakening append-only journal, replay, authority, provenance, snapshot, or migration guarantees;
- making projection state authoritative or independently mutable gameplay state.

## Relevant durable context

- `AGENTS.md`
- `docs/product-definition.md`
- `docs/checkpoints/gate-1.md`
- ADR 002 — state, transactions, and event journal
- ADR 004 — engine/content/campaign isolation
- ADR 005 — domain and state model
- ADR 007 — information provenance
- ADR 011 — atomic campaign-state and event-journal persistence
- ADR 012 — snapshot migration and replay

## Acceptance criteria

- [ ] The plan records which existing `CampaignState` record families are projected now and why; deferred future subsystem data remains unmodeled.
- [ ] Projection rows are campaign-scoped and queryable without decoding the whole `CampaignState` JSON document.
- [ ] Accepted material transitions update authoritative state, immutable journal history, and projections in one SQLite transaction; injected projection failure rolls back the whole transition.
- [ ] Projection state has an explicit per-campaign sequence/version marker tied to the authoritative materialized head and cannot be mistaken for a second source of truth.
- [ ] Existing/new campaigns receive a deterministic projection image derived from accepted authoritative state.
- [ ] A public recovery/rebuild path reconstructs authoritative state through the accepted snapshot+journal replay path and replaces projections deterministically.
- [ ] Corrupt/stale projection data is detectable and repairable by rebuild without rewriting journal/snapshot history.
- [ ] Campaign A projection queries/rebuilds cannot read, alter, or satisfy references using Campaign B rows.
- [ ] Existing append-only history, replay, state-held provenance, stale-command, and authority invariants remain intact.
- [ ] Real persistence tests cover rollback on projection write failure, stale/corrupt projection detection, deterministic rebuild, restart behavior, and campaign isolation.
- [ ] `./scripts/verify-fast` passes on the implementation head.
- [ ] `./scripts/verify` passes on the final reviewed head.
- [ ] Repository CI passes on the exact PR head declared ready.
- [ ] Full PR diff is reviewed against this plan and relevant ADRs before ready-for-review.

## Planned slices

1. Inspect the exact current domain record shapes, authoritative transaction path, replay APIs, migrations, and persistence tests; freeze the minimum projection contract for records that exist today.
2. Add projection schema/migration and projection encoding/query/replacement primitives with campaign isolation and head metadata.
3. Integrate projection replacement atomically into campaign initialization and accepted material transition commits.
4. Add deterministic replay-backed rebuild/recovery and stale/corruption detection without trusting projection state as authority.
5. Add failure/isolation/rebuild tests and mechanical drift guards; run fast verification.
6. Open/update the draft PR continuously, perform full-diff review, run full verification/CI on the exact final head, update this plan/PR summary, and leave merge to human approval.

## Decision log

- 2026-09-23 — Branch and `main` both verified at `d450766b1b834d739c586db5594de6dd23dd9722`; no pre-existing active plan exists on this branch.
- 2026-09-23 — Projection data is derivative only. Rebuild authority must remain snapshot+journal replay, not projection rows.
- 2026-09-23 — Scope will mirror only domain records already present in `CampaignState`; schemas for deferred combat, simulation, economy, generation, opportunity, and UI systems are explicitly excluded.
- 2026-09-23 — Coordination boundary: do not implement lifecycle or content-manifest behavior here. Any required incompatible cross-branch contract will be recorded and surfaced rather than privately worked around.

## Validation

- `./scripts/verify-fast` — pending
- `./scripts/verify` — pending
- CI — pending

## Risks / blockers

- Projection normalization must be useful for gameplay queries without duplicating future subsystem schema prematurely; exact column choices must follow the current domain records rather than anticipated features.
- A migration/backfill path that trusts already-corrupt `campaign_state_current` would undermine recovery semantics; rebuild logic must preserve the accepted snapshot+journal authority boundary.
- Whole-campaign projection replacement may be acceptable at current Gate 1 scale but should keep a clear path to incremental projection writers when later gameplay volume warrants it; no performance claim will be made without measurement.
- No known cross-branch blocker yet.

## Next action

Read the current `CampaignState` record definitions plus `journal_store.rs`, `snapshot_replay.rs`, migrations, and focused persistence tests; then record the exact projection table/query surface before implementation.
