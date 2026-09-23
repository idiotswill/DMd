# Gate 1 — Query/materialized projections

Status: implementation complete; exact-head validation pending
Branch: `gate1/projections`
PR: `#10`
Base: `main` @ `d450766b1b834d739c586db5594de6dd23dd9722`
Implementation head before this closeout update: `9bf6194ef6368079d6cf2ba4851309ce4eefaede`

## Objective

Build the production-intended normalized/materialized query projection layer required by Gate 1 so gameplay query paths do not depend permanently on deserializing `campaign_state_current.state_json`, while projections remain strictly derivative of accepted authoritative state/journal history and are recoverable from the existing snapshot+journal path.

## Scope

- project the `CampaignState` record families that exist now: campaign/clock metadata, players, characters, entities, factions, locations, scenes/presences, items, facts, claims, beliefs, knowledge, and standing directives;
- add campaign-scoped relational projection schema and typed query APIs without inventing future subsystem persistence;
- maintain projections inside the same SQLite transaction/statement boundary that accepts authoritative state/journal transitions;
- backfill valid existing materialized states and invalidate projections when the materialized JSON itself is corrupt so replay recovery remains available;
- provide deterministic projection rebuild from state reconstructed through the accepted snapshot+journal replay boundary;
- detect missing/stale head metadata and record-family count corruption and fail closed rather than treating projections as authority;
- mechanically constrain campaign isolation and projection metadata where SQLite can enforce it;
- add failure, corruption, rebuild, restart, atomicity, and multi-campaign isolation tests through the real persistence path;
- document the durable projection contract in ADR 013.

## Non-goals

- campaign create/archive/delete/backup/export/purge flows owned by `gate1/campaign-lifecycle`;
- rules/content-pack manifest resolution owned by `gate1/content-manifest`;
- gameplay rules or durable future combat/simulation/economy/world-generation/opportunity models;
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
- ADR 013 — derivative query projections

## Acceptance criteria

- [x] The plan records which existing `CampaignState` record families are projected now and why; deferred future subsystem data remains unmodeled.
- [x] Projection rows are campaign-scoped and queryable without decoding the whole `CampaignState` JSON document.
- [x] Accepted material transitions update authoritative state, immutable journal history, and projections in one SQLite transaction; injected projection failure rolls back the whole transition.
- [x] Projection state has an explicit per-campaign sequence/version marker tied to the authoritative materialized head and cannot be mistaken for a second source of truth.
- [x] Existing/new campaigns receive a deterministic projection image derived from accepted materialized state when that recovery artifact is valid.
- [x] A public recovery/rebuild path reconstructs authoritative state through the accepted snapshot+journal replay path and replaces projections deterministically.
- [x] Corrupt/stale projection data and malformed/stale materialized recovery artifacts covered by the projection recovery contract are repairable by replay-backed rebuild without rewriting journal/snapshot history.
- [x] Campaign A projection queries/rebuilds cannot read, alter, or satisfy references using Campaign B rows.
- [x] Existing append-only history, replay, state-held provenance, stale-command, and authority invariants remain intact in the implementation design and regression suite.
- [x] Real persistence tests cover rollback on projection write failure, stale/corrupt projection recovery, malformed materialized JSON recovery, stale materialized sequence recovery, restart behavior, and campaign isolation.
- [x] `./scripts/verify-fast` passed in repository CI during implementation after formatting fixes; final exact-head rerun remains required below.
- [ ] `./scripts/verify` equivalent CI passes on the final reviewed head.
- [ ] Repository CI passes on the exact PR head declared ready.
- [x] Full PR diff was reviewed against this plan and relevant ADRs; review findings below were corrected before final validation.

## Implemented design

1. `0006_query_projections.sql` adds a per-campaign projection head plus normalized tables for the existing state record families only.
2. SQLite insert/update triggers replace the derivative image atomically with `campaign_state_current`; a forced projection write failure aborts the authoritative transition and journal append.
3. Malformed materialized JSON deletes the derivative projection image instead of blocking snapshot+journal recovery. Valid replay repair regenerates projections.
4. `load_campaign_projection_summary` verifies projection schema/sequence against the materialized recovery head and mechanically checks per-family row counts before query use.
5. `list_projected_entities_at_location` proves a gameplay-oriented relational query can execute without decoding the whole `CampaignState` document.
6. `rebuild_campaign_projections` records the materialized sequence observed before replay, reconstructs state from snapshots+journal, and CAS-replaces the recovery row only if it did not move concurrently. This permits repair of a stale materialized sequence without allowing replay to overwrite a concurrent accepted transition.
7. Campaign-scoped keys/FKs and query predicates enforce isolation. Record-local JSON remains only for substructure without a demonstrated relational query requirement.

## Decision log

- 2026-09-23 — Branch and `main` both verified at `d450766b1b834d739c586db5594de6dd23dd9722`; no pre-existing active plan existed on this branch.
- 2026-09-23 — Projection data is derivative only. Rebuild authority remains snapshot+journal replay, never projection rows.
- 2026-09-23 — Scope mirrors only domain records already present in `CampaignState`; schemas for deferred combat, simulation, economy, generation, opportunity, and UI systems are excluded.
- 2026-09-23 — Whole-image projection replacement is accepted for Gate 1 because it makes atomicity/deletion/drift behavior simple; incremental projection writers are deferred until measured scale warrants them.
- 2026-09-23 — Direct malformed `campaign_state_current.state_json` must not make authoritative snapshot/journal recovery impossible. Corrupt materialized JSON therefore invalidates projections without being parsed.
- 2026-09-23 — Rebuild CAS compares against the materialized sequence observed before replay, not the replayed sequence, so stale materialized metadata is repairable while concurrent accepted movement still fails safely.
- 2026-09-23 — A normalized `world_seed` column was removed during full-diff review because SQLite JSON numeric extraction cannot safely represent the complete Rust `u64` range; the exact seed remains in record JSON until there is a proven exact relational query requirement/storage contract.
- 2026-09-23 — Coordination boundary held: no lifecycle or content-manifest implementation was introduced, and no cross-branch contract blocker was discovered.

## Validation

- Historical implementation CI after the initial formatting fix proved `cargo fmt --check`, `cargo check`, Clippy with warnings denied, architecture guard, genericity guard, and Rust 1.88 MSRV. That run exposed an existing snapshot-recovery corruption fixture that the first projection trigger blocked; the trigger was corrected rather than weakening the recovery test.
- Projection regression coverage now includes: normalized query + campaign isolation; same-transaction rollback on forced projection failure; row-count corruption detection and replay repair; malformed materialized JSON invalidation/rebuild; stale materialized sequence replay repair; and close/reopen persistence.
- Final exact-head `./scripts/verify` equivalent repository CI: pending. The successful exact-head run will be recorded in PR #10's summary rather than committed back into this file, because committing the result would itself create a new unvalidated head.

## Remaining debt / explicit tradeoffs

- Whole-image replacement is deliberately simple and unbenchmarked at Gate 1 scale. Later high-volume simulation may require incremental projectors, but only with equivalent atomic/rebuild guarantees.
- The typed query surface is intentionally small; additional projection queries should be added when actual gameplay systems establish concrete access patterns rather than pre-modeling speculative UI/simulation needs.
- Projection head/count validation catches missing rows, extra rows, stale heads, and malformed recovery invalidation. It does not checksum every record payload against replay history, so arbitrary manual same-count in-place SQLite tampering is not independently detected by query reads. Replay-backed rebuild remains the repair path; production application writes are constrained to the trigger-maintained path.
- Exact unsigned `world_seed` remains record-local JSON rather than a lossy SQLite numeric projection until a concrete exact-query/storage requirement exists.
- No unresolved cross-branch blocker is known.

## Next action

Run and verify the repository CI on the exact closeout head, update PR #10 with the exact SHA/results and remaining debt, then leave the draft PR for human review/approval. Do not merge.
