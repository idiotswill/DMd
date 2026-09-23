# ADR 002 — State, Transactions, and Event Journal

**Status:** Accepted for Gate 0

## Decision

Authoritative campaign state is stored locally in SQLite. A material game action is resolved and committed as one database transaction that updates normalized current state and appends immutable event records.

## Required properties

- Atomic state mutation
- Foreign-key and domain-invariant validation
- Append-only event journal for material changes
- Periodic snapshots for fast recovery/replay
- Versioned schema migrations
- Crash-safe recovery to the last committed transaction
- Human-readable resolution/provenance records for debugging

## Event role

Events answer “what changed, why, and from which inputs?” They do not replace efficient current-state tables.

An event may record zero, one, or several direct causal parent events. Consequence history is therefore a causal graph rather than a forced single-parent chain. This matters when an outcome is jointly caused by several prior developments—for example, disrupted trade plus military pressure producing a shortage.

The event journal remains append-only. Causal parents must refer to earlier material events in the same campaign; persistence/replay will enforce sequence and campaign constraints.

Physical dice are external inputs recorded in the resolution event. Procedural RNG must use explicit deterministic streams/seeds when reproducibility matters.

## Explicitly rejected

- Git commits as live save transactions
- Google Drive documents as runtime state authority
- Pure event sourcing with no materialized current state
- Silent direct writes from language/AI components
