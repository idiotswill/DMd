# DMd product definition

This document defines the end-state product DMd is being built to become. It is a product contract, not a description of current implementation status.

Code and executable tests are authoritative for what the current build actually does. This document is authoritative for what a finished DMd must ultimately do. An incomplete implementation does not redefine the finish line.

## Product statement

DMd is a production-intended, fully functioning, local-first tabletop RPG application that can act as the game master for sustained open-ended campaigns.

The target is **not** a proof of concept, prototype, technology demonstration, SDK, framework, prompt system, rules-library demo, scripted campaign, or collection of partially connected subsystems.

Intermediate gates may intentionally expose incomplete functionality, but accepted work must use production-intended architecture and advance toward the complete player-facing game. A gate may simplify what is available; it must not satisfy real-product acceptance with a disposable parallel implementation.

Artifact size is not a success criterion. Product completeness, reliability, usability, replayability, and sustained play are.

## What “finished game” means

A finished DMd must allow ordinary players to use the actual application rather than a developer harness.

At minimum, the product must support the following end-to-end behavior:

- a normal user can install/start the application without a development environment;
- a user can create a new campaign without editing source code, prompts, JSON, SQLite, or Git repositories;
- arbitrary player characters and party compositions can be created and maintained;
- four people can sit around a table and play naturally as the primary target configuration, without making four players a core engine limit;
- natural voice interaction can sustain normal multi-hour tabletop sessions;
- physical dice are first-class inputs for visible player rolls;
- combat, exploration, social play, travel, downtime, inventory, rests, leveling/progression, death, retirement/replacement characters, NPC interaction, and persistent world consequences work through the real application;
- save, exit, restart, and continue preserve authoritative campaign state accurately;
- campaigns survive many sessions and unscripted divergence rather than depending on a pre-authored route;
- failures are recoverable without requiring players to repair a database, Git repository, prompt, or internal state file;
- an explicit admin/debug escape hatch can correct genuine mistakes while preserving provenance;
- a second unrelated campaign can run on the same executable without inheriting campaign-specific assumptions;
- the actual gameplay loop works locally/offline; cloud services may be optional enhancements but are not core gameplay dependencies;
- players do not need to understand Rust, SQLite, schemas, prompts, models, GitHub, or the internal architecture.

A successful session should feel like playing a tabletop RPG with a DM, not operating an engineering system.

## Feature-completion rule

Architecture, schemas, traits/interfaces, APIs, mocks, unit tests, benchmarks, or scripted demonstrations are not by themselves feature completion.

A feature is ultimately complete only when it participates correctly in the real product path, with realistic state, persistence, failure behavior, and player interaction appropriate to its gate.

Examples:

- a combat engine is not complete because an isolated test can resolve an attack; the production application must eventually run a complete encounter through the normal interaction path;
- a voice subsystem is not complete because speech-to-text transcribes a sample; it must participate in sustained table conversation, corrections, interruptions, roll association, and authoritative validation;
- persistence is not complete because rows are written; campaign state must survive restart, recovery, migration, and replay expectations;
- world simulation is not complete because a clock increments; changes must produce coherent, causally grounded consequences that can reach the players.

## Production path, experiments, and prototypes

Research prototypes and experiments are allowed when they are clearly isolated under research/experiment/benchmark tooling and are not mistaken for product implementation.

Production gate acceptance must not be satisfied by a throwaway path that bypasses the intended architecture.

A temporary simplified interface is acceptable when it exercises the same production-intended core path. For example, a text-only input surface may precede voice if it still produces the same typed declarations/commands used by the eventual voice application.

A separate toy implementation that bypasses persistence, authority, rules, simulation, or application boundaries does not count toward production acceptance merely because it demonstrates the concept.

## Living-world invariant

DMd campaigns must continue to develop through simulated time and autonomous world actors/processes rather than requiring scripted player triggers.

The world must not freeze simply because the players are looking elsewhere.

Where circumstances justify it, NPCs, factions, settlements, institutions, economies, projects, conflicts, relationships, travel, environmental processes, and other world systems may advance, fail, react, compete, cooperate, transform, or end.

World change must be grounded in established state, capabilities, knowledge, resources, geography, time, and prior events. The system must not manufacture arbitrary consequences merely to create drama.

Player action, player inaction, off-screen actors, and elapsed time can all contribute to future state when causally justified.

## Emergent play requirement

A campaign must remain capable of producing meaningful new play after authored starting material has been exhausted.

Changing world conditions should be able to produce new player-facing:

- rumors and gossip;
- leads and clues;
- requests and offers;
- conflicts and disputes;
- discoveries and observable consequences;
- opportunities and threats;
- alliances, rivalries, and relationship changes;
- jobs or quest-like situations;
- travel complications or openings;
- political, economic, social, environmental, magical, or military developments.

These should emerge from the state of the world rather than requiring a hidden loop that periodically calls a random `generate_quest()` equivalent.

A “quest” is primarily a player-facing framing of an underlying situation. Internally, the important truth may instead be that actors have goals, resources, shortages, losses, obligations, fears, plans, relationships, and conflicting interests. A merchant offering payment to clear a road should be explainable by the actual road danger, losses, trade pressure, available alternatives, and the merchant’s incentives.

Players may accept, reject, redirect, exploit, misunderstand, or completely ignore such situations. The world continues appropriately either way.

## Information must carry world motion to the table

A changing world that players can never perceive does not create a living campaign.

World developments should produce observable traces where appropriate: travelers, prices, absences, damage, celebrations, refugees, shortages, patrols, letters, notices, changed behavior, visible construction, rumors, testimony, news, environmental signs, and other evidence.

Information may be accurate, incomplete, biased, stale, mistaken, manipulated, or deliberately false, but where possible it should arise from actual observations, beliefs, incentives, or misinformation rather than unconstrained narration.

The Fact / Claim / Belief / Knowledge separation exists in part to support this behavior.

## Simulator, opportunity layer, and Director

The product must preserve a hard distinction between world truth and presentation/pacing.

Conceptually:

```text
WORLD SIMULATOR
What actually changes and why?
        ↓
INFORMATION / OPPORTUNITY LAYER
What traces, rumors, requests, leads, conflicts, discoveries, or openings result?
        ↓
DIRECTOR
Which meaningful developments should be surfaced now, and how should pacing/spotlight be managed?
```

The Director may select, prioritize, delay, combine, or frame developments that the world can support.

The Director must not invent unsupported authoritative world truth, rewrite dice, retroactively create convenient causes, or force outcomes merely because the current session needs excitement.

If the Director surfaces an assassination attempt, the world must be able to explain who wanted it, who could arrange it, what they knew, why now, and how the attempt became possible.

Quiet periods are valid. Pacing is not permission to cheat causality.

## Explainability and provenance

Meaningful world changes must be inspectable enough that an admin/developer can answer “why did this happen?” without receiving “because the AI decided it was interesting” as the real cause.

For example, an expensive food market might trace through a chain such as:

```text
poor harvest
→ regional supply decline
→ merchant stockpiling
→ road disruption
→ current local price pressure
```

The precise simulation fidelity may vary by distance/relevance, but important changes need causal/provenance records sufficient for debugging, trust, replay/recovery where applicable, and narrative consistency.

## Long-horizon endurance acceptance

Before DMd can be considered a finished product, the same production build must survive realistic open-ended play rather than only scripted acceptance scenes.

A final endurance campaign must include, at minimum:

1. Start from a clean installation with no existing campaign state.
2. Create a brand-new world/campaign and arbitrary PCs through supported product flows.
3. Play multiple real sessions through the normal application interface.
4. Exercise combat, exploration, social interaction, travel, downtime, inventory/resources, rests, advancement, and meaningful choices.
5. Use physical dice and normal voice/table interaction where those systems are part of the release target.
6. Save, exit, restart, and resume repeatedly without manual state repair.
7. Allow the party to diverge materially from initial hooks and ignore offered situations.
8. Advance enough in-world time for off-screen actors/processes to act repeatedly.
9. Return to previously visited places and observe coherent changes caused by elapsed time, actor goals, resources, player actions/inaction, and prior events.
10. Verify that new rumors, leads, opportunities, conflicts, requests, discoveries, or quest-like situations have emerged without being manually scripted for the test path.
11. Kill, retire, or otherwise remove a PC; introduce a replacement; continue without inheriting forbidden knowledge or corrupting campaign history.
12. Verify old opportunities can disappear, transform, succeed, or fail without the party, while new ones emerge.
13. Inspect representative world changes and verify their causes are explainable from persisted state/events rather than arbitrary AI invention.
14. Recover from representative user mistakes and application failures using supported recovery/admin mechanisms rather than developer database surgery.
15. Continue long enough that authored starting hooks are no longer sufficient to drive play; meaningful play must still emerge from the living world.
16. Create a second, unrelated campaign using the same executable and verify no state/content assumptions leak from the first.

The endurance test fails if continued meaningful play requires a developer to manually feed the campaign new quests, repair state, edit prompts/source files, or secretly script the party’s route.

## Long-horizon world-motion acceptance

A particularly important scenario is prolonged player absence from an area.

If the party spends substantial in-world time elsewhere and later returns, the earlier region should not be identical merely because it was off camera. Depending on established circumstances, actors/projects/conflicts may have advanced, failed, changed direction, resolved, worsened, or been replaced by new conditions.

The exact amount of change must depend on causality and simulation fidelity, not a requirement that “something dramatic must always happen.”

The returned-to region should be capable of containing:

- changed NPC circumstances and relationships;
- completed/failed projects;
- different prices, supplies, traffic, security, or services;
- changed faction power or priorities;
- consequences of unresolved threats;
- consequences of resolved threats;
- remembered effects of earlier PC actions;
- new rumors/leads/opportunities grounded in intervening events;
- old opportunities that no longer exist.

## Gate traceability

Every future gate/checkpoint should state which portions of this product definition it advances and which remain explicitly deferred.

Passing an intermediate gate means its scoped acceptance criteria are production-intended and verified. It does **not** mean the overall product is complete.

No later plan may silently weaken this product definition in order to make a checkpoint easier to pass. A deliberate product-scope change requires explicit human approval and an update to this document.
