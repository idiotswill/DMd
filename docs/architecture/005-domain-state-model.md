# ADR 005 — Domain and state model

Status: **Accepted for Gate 0 foundation**

## Goal

DMd needs one generic state model that can support unrelated campaigns, replacement characters, split scenes, procedural materialization, mysteries, off-screen simulation, physical dice, and later save migrations without embedding assumptions from any current campaign.

The domain model is deliberately independent of voice, UI, LLM providers, and any specific setting.

## Core rules

1. **Names are labels, never identity.** Stable typed UUIDs identify campaigns, players, characters, entities, locations, scenes, items, facts, claims, events, and other persistent records.
2. **All mutable records are campaign-scoped.** Cross-campaign references are invalid.
3. **A PC has two identities.** `CharacterId` addresses the persistent character/build record; `EntityId` addresses that character as an actor in world/scene/spatial systems.
4. **Current state is materialized state, not the historical record.** The event journal explains how state changed; snapshots accelerate reconstruction.
5. **Truth, claims, beliefs, and knowledge are different records.** Hearing a claim never makes it objectively true. A successful social interaction does not promote testimony into world truth.
6. **World time is numeric and setting-agnostic.** Calendar names and date formatting come from content packs.
7. **Split scenes are normal.** A campaign may have multiple active/paused scenes at different locations.
8. **Standing directives are state.** Instructions such as “follow the trail until it ends or danger appears” survive across several low-level simulation steps without repeated prompts.
9. **Ownership and physical custody are separate.** Borrowed, stolen, hidden, dropped, faction-owned, and container-held items require no special-case semantics.
10. **Every durable mutation has provenance.** Events record campaign, sequence, world time, source, actor when relevant, command correlation, and causal parent when relevant.

## Aggregate overview

```text
Campaign
├── manifest
│   ├── ruleset version
│   ├── content-pack versions
│   └── deterministic world seed
├── WorldClock
├── Players
├── Characters ──> WorldEntities
├── Locations
├── Scenes
│   ├── presences
│   └── StandingDirectives
├── ItemInstances
├── Facts
├── Claims
├── Beliefs
├── KnowledgeRecords
└── applied_event_sequence
```

`CampaignState` is the serializable materialized snapshot root. It is **not** intended to become a god-object containing all gameplay logic. Rules, world simulation, director, conversation, persistence, and other subsystems operate through commands/events and their own services.

## Identity and campaign isolation

A display name may change and may collide with another display name. Runtime code must never use names as foreign keys.

Every campaign-scoped record includes `CampaignId`. `CampaignState::validate()` checks current structural invariants, including campaign mismatches, map-key/record-ID mismatches, and core dangling references.

Persistence will additionally enforce campaign isolation with database keys/constraints where practical.

## Player and character ownership

`Player` represents a human participant in a campaign.

`Character` represents the persistent PC record and has:

- `CharacterId`
- world-facing `EntityId`
- optional controlling `PlayerId`
- lifecycle state (`Active`, `Absent`, `Retired`, `Dead`)

This permits death and replacement without deleting history, guest/unassigned imports, temporary player absence, and later conversion of a former PC into an autonomous world actor if desired.

No engine invariant assumes exactly four players or exactly one lifetime character per player.

## World entities and locations

`WorldEntity` is the common world-facing identity for actors and significant objects that participate in scenes, observations, event causality, and spatial systems.

`Location` uses a parent hierarchy as the first topology primitive. Exact tactical geometry, routes, portals, and region graphs will be added by the spatial/world layers rather than overloaded into the base location record.

## World time

The engine stores `WorldInstant(i64)` and `WorldDuration(i64)` in a campaign-defined base tick.

The core does not know that a year has twelve months, that a round has six seconds, or what a weekday is. Rules/content layers define those conversions.

This keeps fantasy calendars and non-D&D settings possible and avoids coupling simulation arithmetic to presentation text.

## Scene state

A `Scene` has:

- stable `SceneId`
- campaign and location
- mode (`Exploration`, `Social`, `Combat`, `Travel`, `Downtime`, or custom)
- lifecycle (`Active`, `Paused`, `Closed`)
- world start time
- entity presences

Several scenes may coexist. This is required for a split party.

A future Scene Director may control attention/presentation between scenes, but it does not own the underlying scene facts.

## Standing directives

`StandingDirective` represents an ongoing player intention that should suppress repetitive micro-prompts.

Examples:

- continue along a trail until it disappears or danger appears
- keep moving until a meaningful decision is required
- map landmarks while travelling
- search for valuables while following the group

Stop conditions are explicit. The simulation may advance multiple ordinary steps, but it must stop before crossing a condition requiring player agency.

## Truth and knowledge model

### Fact

A `Fact` is accepted world truth for a validity interval. It has provenance and may point to the event that established it.

Example: `bridge.destroyed = true`.

### Claim

A `Claim` is what an entity asserted at a particular time.

Example: `witness says the baron ordered the bridge destroyed`.

The claim exists regardless of whether its proposition is true.

### Belief

A `Belief` belongs to an entity, carries confidence, and records its basis (facts, claims, observations, or inference).

NPC decision systems must reason from allowed beliefs/knowledge rather than querying hidden world truth directly.

### Knowledge

A `KnowledgeRecord` says a holder has access to a fact or claim.

The holder is either a specific entity or explicit shared-table knowledge. Out-of-character table discussion does not automatically create in-character shared knowledge.

### Proposition

Facts, claims, and beliefs share a typed `Proposition` structure whose subject may be a campaign, entity, location, faction, or item.

This prevents mystery/investigation state from degrading into unstructured prose while still allowing content-defined predicates.

## Inventory

`ItemInstance` separates:

- definition/content identity
- instance identity
- display name
- quantity
- owner
- physical custody/location/container
- current state

There is intentionally no `Party` owner/carrier variant in the base model. If a campaign later wants a shared legal entity, it must model that entity explicitly rather than using an ambiguous global loot bucket.

## Event provenance

Every persisted domain event is wrapped in `EventEnvelope<T>` and `EventMeta`.

Metadata includes:

- `EventId`
- `CampaignId`
- monotonic per-campaign sequence
- world time
- event source
- actor when relevant
- causal parent event when relevant
- originating command when relevant

This enables deterministic debugging questions such as “why did this settlement price change?” without forcing all current state to be recomputed from the beginning on every startup.

## Mutation lifecycle

```text
natural table speech
        │
        ▼
conversation interpretation
        │ structured proposal(s)
        ▼
command validation
        │
        ├── ambiguous material choice ──> ask player
        │
        └── valid command
              │
              ▼
        rules/world resolver
              │
              ▼
        domain event(s)
              │
              ▼
        SQLite transaction
        ├── append events
        ├── update materialized state
        └── update applied sequence
              │
              ▼
        structured outcome
              │
              ▼
        narrative/UI/TTS rendering
```

The language model does not write `CampaignState`.

## Generated world materialization

Procedural generation may create candidate detail from deterministic seed streams. Once detail becomes observed/interacted-with/canonical, it receives persistent IDs and enters normal campaign state/event history.

Revisiting an observed location must load persistent state rather than rerun generation and produce contradictory facts.

## Snapshot/version model

`CampaignState` contains `schema_version` and `applied_event_sequence`.

Save migrations operate on explicit schema versions. A snapshot states exactly which journal sequence it includes. Recovery can therefore load a snapshot and replay only later events.

## Explicitly deferred

Gate 0 does **not** yet freeze schemas for:

- 5e character mechanics, classes, spells, conditions, effects, resources
- tactical geometry and line of sight
- factions and NPC utility/goal models
- quests/opportunities
- economy quantities and market simulation
- combat timing/reaction stack
- procedural generation recipes
- detailed relationship dimensions
- director pacing state

Those systems must reference these base identities/provenance concepts rather than replace them.

## Gate 0 acceptance implications

Before this ADR is considered stable:

- an empty generic campaign snapshot validates;
- cross-campaign records fail validation;
- duplicate human-readable names remain legal and distinct;
- character death does not require deleting the world entity/history;
- split scenes can coexist;
- claims can exist without becoming facts;
- ownership can differ from custody;
- state schema version and applied event sequence are explicit;
- production domain code contains no current-campaign names or IDs.
