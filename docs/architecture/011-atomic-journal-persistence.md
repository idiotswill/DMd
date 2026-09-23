# ADR 011 — Atomic campaign-state and event-journal persistence

Status: **Accepted for Gate 1 Slice A — PR #6, merged as `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`**

## Context

Gate 0 defined the authority rule: language/provider output is never authoritative state, material mutations must be transactional and auditable, and persistence owns event sequencing. The repository did not yet have the durable commit path that makes those promises true after a crash or restart.

A material transition has several pieces that must agree:

- the campaign state version the command was resolved against;
- trusted command issuer metadata and optional in-world actor;
- the validated command payload used for the ruling;
- one or more resulting domain events;
- causal links to earlier events;
- the resulting materialized campaign state and its journal head.

Writing any of these independently would create failure windows where the save and its explanation disagree.

## Decision

For accepted material transitions, SQLite commits the following in one transaction:

1. read and validate the current materialized state and journal head;
2. revalidate event provenance already referenced by that current state;
3. compare the command's `expected_event_sequence` with the authoritative current campaign head;
4. validate campaign/session/issuer/actor and pending event/provenance references;
5. allocate the next contiguous event sequences inside persistence;
6. compare-and-set the materialized campaign head;
7. append the immutable command audit record;
8. append immutable journal events and causal edges;
9. commit the transaction.

If any step fails, the transaction is not authoritative and the prior committed state remains the recovery point.

## Current-state representation

This slice stores a versioned serialized `CampaignState` in `campaign_state_current` together with its `applied_event_sequence`.

That row is a **recovery/materialized-state artifact**, not the final query architecture. ADR 002 still requires efficient normalized current-state/projection tables where gameplay systems need them. Later Gate 1 work may add those projections while keeping the same atomic transaction boundary.

The serialized state's embedded campaign ID, schema version, and applied sequence must match the row metadata when loaded.

## Recovery snapshot consistency

Recovery integrity checks read the serialized state, journal head, and referenced event provenance inside one SQLite read transaction. This prevents a healthy campaign from being reported as corrupt merely because a concurrent writer committed between independent recovery queries.

Recovery fails closed if state JSON cannot be decoded, row/state metadata disagree, domain invariants fail, the journal head/count disagrees with the state sequence, or state-held event provenance is missing/cross-campaign.

## Journal head invariant

For a campaign at sequence `N`:

- materialized state records `applied_event_sequence = N`;
- the journal has exactly `N` events for that campaign;
- the maximum persisted sequence is `N`;
- event sequences are unique and positive within the campaign.

This makes gaps, partial journal loss, and state/head disagreement detectable on recovery.

## Sequence ownership and stale commands

Resolvers do not choose journal sequence numbers. They emit `PendingEvent<T>` values without campaign sequence.

Persistence allocates contiguous sequences from the currently committed head. `CommandMeta.expected_event_sequence` is optimistic concurrency metadata. A command resolved against an older head is rejected before it can become authoritative.

The materialized-state update also uses a compare-and-set on the old sequence. This is a second mechanical guard against another writer advancing the same campaign while the transition is being committed. SQLite lock/contention errors remain possible under true simultaneous writers; callers must treat those as non-committed failures and retry/re-resolve from current state rather than assuming success.

## Command audit and typed payload boundary

Trusted `CommandIssuer`/`CommandMeta` are shared domain metadata because persistence must preserve who authorized a command separately from which entity/faction acted in-world.

`GameCommand<C>` remains the typed core request. A typed payload may be encoded into a versioned `SerializedRecord` for durable audit only after application/core validation. Serialization itself is **not** authorization; it merely preserves the validated representation used by the accepted path.

The dependency guard prevents domain, conversation, core, and rules crates from importing the persistence crate or SQLx directly. Future provider adapters must remain on the interpretation side of the same boundary rather than acquiring a SQLite write path.

## Event and provenance integrity

Each persisted event records:

- stable event ID;
- campaign and persistence-allocated sequence;
- optional play-session correlation;
- in-world time;
- event source;
- optional world actor;
- originating command ID;
- versioned typed payload representation.

Direct causal parents are stored in `event_causes`. A parent must already exist in the same campaign and have a lower sequence, or appear earlier in the same pending batch.

Current-state records that contain event provenance—Fact source events, Claim source events, direct-observation Belief bases, and Knowledge source events—must point to an existing same-campaign journal event or an event in the transaction's pending batch. Sequence-0 initialization cannot contain dangling event provenance.

Both recovery and the beginning of each material commit recheck existing state-held provenance. A later valid write therefore cannot silently "heal" a corrupt snapshot by replacing state that referenced missing history.

## Append-only behavior

The production API exposes no operation that edits individual historical command/event/causal records. SQLite triggers reject both `UPDATE` and direct `DELETE` on `command_audit`, `event_journal`, and `event_causes`.

Those delete guards intentionally also prevent an ordinary cascading delete of `campaign_state_current` once journal history exists. Campaign deletion is a separate future lifecycle operation. It must deliberately purge the complete campaign aggregate through a controlled migration/lifecycle path rather than weakening append-only history or permitting partial journal surgery.

## Explicitly deferred

This ADR does **not** claim completion of:

- snapshot-version migration between future `CampaignState` schemas;
- replaying typed event payloads to reconstruct arbitrary historical state;
- normalized projection tables for every domain subsystem;
- backup/restore UX or campaign export/import;
- controlled whole-campaign purge/delete semantics;
- per-campaign application-level write scheduling/retry policy;
- rules/content-pack schema migration;
- gameplay resolvers that produce the transitions.

Those remain Gate 1/later production work. Slice B owns snapshot migration/replay without changing the atomic authority contract established here.
