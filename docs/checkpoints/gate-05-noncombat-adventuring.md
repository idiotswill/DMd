# Gate 5 — Noncombat adventuring

Status: **Planned**

## Product requirements advanced

- exploration/investigation;
- evidence-conservative checks;
- social interaction;
- travel/camp/rest;
- inventory/economy/services;
- downtime;
- split scenes;
- player-created goals;
- PvP policy;
- player narrative ownership;
- scene compression.

## Objective

Make ordinary D&D play work outside combat so a session can consist primarily of exploration, conversation, preparation, travel and improvised goals.

## Entry conditions

Gates 2–4 accepted; noncombat ledger rows assigned to this gate are known.

## Acceptance criteria

Support:
- exploration and focused investigation;
- when-not-to-roll and repeat-check prevention;
- changed method/vantage/evidence;
- social influence without mind control;
- trade/shopping/services/lodging/debts;
- inventory, ownership/custody, equipment, loot;
- rests, camp, watches, travel/resources/weather as required;
- downtime and player-created projects at useful initial fidelity;
- split scenes and standing directives;
- table-contract PvP/inter-PC contests;
- player cosmetic narration after resolved outcomes;
- time/scene compression that preserves resource/time/world consequences;
- PC death/retirement/replacement integration.

## Architecture invariants

Facts/claims/beliefs/knowledge remain distinct. Skill rolls cannot invent inaccessible facts. Player-created goals become authoritative only through validated state transitions, not narrator fiat.

## Explicit non-goals

Full autonomous living world, procedural unauthored world creation, voice.

## Required failure/recovery behavior

Interrupted travel/rest/downtime recovers without double-spending/duplicate time. Failed checks do not corrupt knowledge. Unavailable market goods are not conjured by UI catalog.

## Merge/pause boundaries

Codex may merge in-scope Gate 5 work autonomously after exact-head verification. High-impact rules/product behavior must satisfy the existing product contract rather than redefine it. Pause at the end of Gate 5 with the complete noncombat integration summary and Gate 6 handoff.
## Candidate workstreams

Investigation/evidence, social resolution, travel/rest, economy/inventory, downtime/projects, split scenes, PvP policy enforcement, compression.

## Deferred requirements

World templates/adventure content Gate 6; off-screen autonomous systems Gate 7; procedural materialization Gate 8; autonomous conversation Gate 9.

## Open questions

Initial downtime/project fidelity should be enough for real play without attempting an economy simulator before Gate 7.

## Production integration acceptance

Run a session through the desktop app containing exploration, social negotiation, travel, inventory/economy change, rest/downtime and split party play. Include a substantial PC-to-PC planning/roleplay stretch requiring no roll and no forced DM interruption. Save/restart during noncombat state and preserve exact consequences.
