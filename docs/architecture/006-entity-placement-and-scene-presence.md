# ADR 006 — Entity placement and scene presence

Status: **Accepted for Gate 0 foundation**

## Problem

Scenes are presentation/adjudication contexts, not a complete world-position system. NPCs, creatures, and other relevant entities continue to exist when no active scene is focused on them. If location lived only inside `Scene`, off-screen simulation would have no authoritative answer to “where is this NPC now?”

## Decision

`WorldEntity` stores an optional coarse `LocationId` independently of scenes.

- `Some(location)` means the entity's current coarse world location is established.
- `None` means its position is deliberately unresolved, unknown, unplaced, or outside the currently modeled location graph.
- Tactical coordinates, exact distances, occupied grid cells, elevation, reach, and line-of-sight are **not** stored here; those belong to the later spatial/tactical subsystem.

A `Scene` still owns a location and a set of presences. Scene presence answers “who is participating in or observing this active adjudication context?”, while entity placement answers “where does the world currently say this entity is?”

## Active participant invariant

For an **active** scene, an entity with `PresenceRole::Participant` must have a coarse location equal to the scene location.

An entity may participate in at most one active scene at a time.

This prevents state such as:

```text
entity location = marketplace
active scene A = marketplace, entity participates
active scene B = castle, entity also participates
```

from silently becoming valid.

Observers are intentionally not subject to this physical-participation invariant. A later UI/director layer may use observation for non-physical attention, remote viewing, or presentation purposes.

Paused/closed scenes are historical or suspended contexts and do not assert present physical participation.

## Off-screen simulation

World/NPC simulation reads persistent entity placement even when no scene includes the entity. A future schedule/travel subsystem may temporarily represent movement with richer state, but it must still project to or supersede this coarse placement explicitly rather than deriving location from narration.

## Scene transitions

Moving a participant between locations is a state mutation. The eventual resolver should update placement and scene membership atomically in one transaction so no committed state can leave the entity physically in one location while actively participating in another.

## Explicitly deferred

This ADR does not define:

- tactical coordinates or grids;
- containment inside vehicles/creatures;
- route-progress representation;
- teleportation mechanics;
- planar coordinate systems;
- visibility or line of sight.

Those systems must preserve the distinction between persistent world placement and scene/presentation membership.
