# ADR 004 — Engine, Content, and Campaign Isolation

**Status:** Accepted for Gate 0

## Decision

DMd separates reusable engine behavior, rules/content definitions, world templates, and mutable campaign state.

## Layers

### Engine
Reusable systems only: rules resolution, simulation, conversation, persistence, Director, audio, and UI/application services.

### Rules/content packs
Versioned definitions such as classes, spells, creatures, items, cultures, calendars, procedural tables, and setting parameters. Content is validated against schemas before play.

### World template
Authored macro geography, states/cultures, generation constraints, and optional prepared facts. A template is reusable across campaigns.

### Campaign state
One playthrough: generated/observed locations, PCs/NPCs, inventories, relationships, knowledge, events, quests, clocks/processes, and world changes.

## Invariants

- Production engine code must not refer to campaign-specific names or IDs.
- Human-readable names are labels, never primary identity keys.
- A campaign may be deleted without modifying engine or content-pack code.
- Multiple campaigns may use the same rules/content pack without sharing mutable state.
- Generated facts become canonical campaign state once materialized/observed; revisiting does not regenerate contradictory facts.
- Content packs and schemas are versioned. Save migrations must know which content/rules version produced a campaign state.
- Current Asterra materials belong only in private research, sanitized regression fixtures, or a future optional import/content pack.

## Genericity acceptance test

Delete all campaign data, create a fresh campaign with an unrelated seed and unrelated character names, play/save/reload, then create a second campaign using the same engine. No production code or configuration should need editing, and the two campaigns must remain isolated.
