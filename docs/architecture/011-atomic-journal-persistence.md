# ADR 011 — Atomic campaign-state and event-journal persistence

Status: **Proposed for Gate 1 — PR #4**

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

1. compare the command's `expected_event_sequence` with the authoritative current campaign head;
2. validate campaign/session/issuer/actor and event/provenance references;
3. allocate the next contiguous event sequences inside persistence;
4. compare-and-set the materialized campaign head;
5. append the immutable command audit record;
6. append immutable journal events and causal edges;
7. commit the transaction.

If any step fails, the transaction is not authoritative and the prior committed state remains the recovery point.

## Current-state representation

This slice stores a versioned serialized `CampaignState` in `campaign_state_current` together with its `applied_event_sequence`.

That row is a **recovery/materialized-state artifact**, not the final query architecture. ADR 002 still requires efficient normalized current-state/projection tables where gameplay systems need them. Later Gate 1 work may add those projections while keeping the same atomic transaction boundary.

The serialized state's embedded campaign ID, schema version, and applied sequence must match the row metadata when loaded.

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

Recovery rechecks state-held event references so a structurally valid JSON snapshot cannot silently point at missing or cross-campaign history.

## Append-only behavior

The production API exposes no operation that edits individual historical command/event/causal records. SQLite update triggers reject mutation of those rows. Campaign-level deletion remains a separate future lifecycle operation and must remove a campaign aggregate deliberately rather than performing journal surgery.

## Recovery behavior

Opening a campaign verifies:

- state JSON decodes;
- campaign/schema/sequence metadata agree;
- domain invariants hold;
- journal count/head agree with the state sequence;
- event provenance referenced by materialized state still exists in the same campaign.

An uncommitted transaction is not a recovery point. Restart/reopen tests must recover only the last committed state.

## Explicitly deferred

This ADR does **not** claim completion of:

- snapshot-version migration between future `CampaignState` schemas;
- replaying typed event payloads to reconstruct arbitrary historical state;
- normalized projection tables for every domain subsystem;
- backup/restore UX or campaign export/import;
- per-campaign application-level write scheduling/retry policy;
- rules/content-pack schema migration;
- gameplay resolvers that produce the transitions.

Those remain Gate 1/later production work. This slice establishes the durable transaction contract they must use.
