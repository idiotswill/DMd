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
5. Use physical dice and natural voice/table interaction through the supported product flows.
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


## Product identity

DMd is a local-first autonomous tabletop Dungeon Master for sustained fifth-edition fantasy RPG campaigns.

The primary intended experience is a group of human players sitting together and playing a tabletop RPG naturally.

The computer replaces most of the work normally performed by the human DM. It does not replace tabletop play with a conventional computer RPG interface.

Players should normally interact through natural speech, physical dice, character sheets, occasional tactical/map displays, direct questions to the DM, and ordinary tabletop conversation.

The desktop DMd application is the central campaign runtime.

Companion player devices may later provide character sheets, maps, notes, private information and player-specific interfaces, but the initial production path must not depend on them.

A successful session should feel primarily like people playing D&D with a competent DM rather than people operating an AI application.

---

## First rules identity and commercial boundary

The first complete production game is fifth-edition D&D-style play based only on official material that is legally reusable in a commercially distributed product.

The engine may remain capable of supporting other rulesets later. That genericity must not make the first game vague or incomplete.

Where legally reusable official material exists, the implementation should follow it faithfully.

Where desired official D&D content is not legally distributable:

- do not copy or embed it;
- do not silently substitute proprietary material;
- either omit it or use clearly original compatible content that is legally distributable.

Every shipped rules/content asset must have traceable licensing/provenance.

Legal-to-sell is a product invariant, not a release cleanup task.

A complete Rules Coverage Ledger must pin the selected legal rules source/version and account for every gameplay/content family DMd claims to support.

---

## Character experience

Players should interact with recognizable tabletop character sheets.

A character is not merely an internal entity record.

The player-facing model must eventually expose all information required by the selected rules, including identity, descriptive information, species/ancestry where legal, class/level, abilities, proficiency, saves, skills, armor class, hit points, hit dice, speed, attacks, equipment, money, features, spellcasting where relevant, conditions, temporary effects, resources, and lifecycle state.

Character creation must happen through supported product flows. Players must not edit JSON, SQLite, source or internal state.

A character may survive, advance, change equipment, become injured, fall unconscious, die, retire or be replaced. Campaign history continues through replacement.

Character creation may also capture optional backstory, goals, beliefs, relationships, fears, obligations, organizations and meaningful possessions.

Player-provided backstory cannot unilaterally overwrite incompatible prebuilt-world facts. World-integrated backstory claims must be explicitly accepted before becoming world truth.

---

## Session Zero and table contract

Campaign creation must support a persistent table contract.

At minimum it should be able to establish:

- tone;
- seriousness/comedy balance;
- tactical-combat preference;
- exploration preference;
- social-roleplay preference;
- expected lethality;
- rules version;
- permitted character content;
- advancement method;
- optional rules;
- explicit house rules;
- PvP policy;
- PC theft/secret-action policy;
- retcon policy;
- content boundaries;
- preferred rules-explanation depth;
- beginner/experienced/mixed table;
- absent-player policy.

These settings persist with the campaign and must not silently change after software updates.

The default should remain as close as practical to the selected official rules rather than quietly adding DMd-specific house rules.

---

## Core table interaction philosophy

### Natural input

DMd must accept realistic player speech.

Players are not required to speak in formal command syntax.

Inputs may contain incomplete sentences, wrong names, speech-recognition errors, slang, profanity, table jokes, rules questions, dialogue, action declarations, several actions, statements from multiple characters, corrections, hypotheticals, false assumptions and changes of mind.

The conversation system should infer ordinary intent when doing so is safe.

The finished product must not require an artificial commitment keyword.

### Clarification policy

Ask for clarification when missing information would materially affect legality, target, player authority, movement, resource consumption, risk, tactical state, another PC's agency, irreversible consequences, or what is actually being attempted.

Do not ask merely because wording is imperfect.

### Player control

The DM never chooses a player's meaningful PC decision merely to keep play moving.

Another player cannot volunteer a PC for an important action without that controller's consent.

NPCs, world systems and the Director may act autonomously. PC agency remains with players.

### Rules questions are not actions

Players may ask whether an action is possible, what modifier applies, what a spell would affect, whether movement is legal, or what a rule means.

Answering a hypothetical/rules question does not commit the discussed action.

### Corrections

Before authoritative resolution, a player may correct mishearing, target, number, name or declared intent without creating a false historical event.

After authoritative state changes, correction uses an explicit provenance-preserving path rather than silent history rewrite.

---

## DM personality and adjudication standard

The target autonomous DM is grounded, reactive, rules-competent, evidence-conservative, dryly humorous when appropriate, permissive toward creative intent, unwilling to fabricate unsupported facts, willing to let players fail, and willing to let foolish decisions have consequences.

It runs the world rather than steering players through a predetermined plot.

Humor should usually follow player behavior, established NPC personality, consequences and recurring table jokes. It should not turn every scene into comedy.

### When to roll

Not everything requires a check.

A check normally exists because the outcome is meaningfully uncertain, success/failure are both plausible, and the distinction matters.

A roll cannot create facts the available evidence could not contain.

Natural 20 does not make an impossible action possible unless the selected rules specifically say otherwise.

### Evidence and repeated checks

A high roll improves what can be learned from available evidence; it does not conjure inaccessible truth.

Unchanged method + unchanged evidence + unchanged vantage should not permit unlimited rerolls.

A materially different tool, method, vantage, spell, helper, source or changed environment may justify another attempt.

### Unknown remains unknown

Uncertainty is legitimate campaign state.

Examples include unknown exact time, identity, provenance, motive, route, ownership, enemy capability and unverified NPC claims.

Narrative convenience is not sufficient reason to resolve an unknown.

### Rules hierarchy

The rules authority hierarchy is:

1. selected legal rules/content version;
2. explicit campaign house rules;
3. established campaign-specific precedent where applicable;
4. situation-specific adjudication where genuine discretion remains.

Language-model opinion is not an authoritative fifth source.

Substantially identical circumstances should receive consistent rulings unless a materially different fact changes the result.

---

## Passive, hidden and private information

Not every uncertainty should announce itself to players.

DMd must support passive and hidden resolution where the selected rules or good DM practice require it, including hidden Stealth opposition, NPC Insight, secret information checks and hidden enemy/world process outcomes.

The system must distinguish actual world truth, party-known information and individual character knowledge.

Player-facing surfaces must not leak hidden state.

A player may receive private discoveries, NPC messages, visions, knowledge, goals or handouts without automatically teaching the whole party.

---

## Table flow, listening and spotlight

Four humans do not politely issue one command at a time.

The system must handle interruptions, laughter, side conversations, rules questions, simultaneous declarations, PC-to-PC dialogue, out-of-character planning, changed minds, long quiet stretches, dominant speakers and split parties.

The DM must know when not to speak.

When players are actively planning or roleplaying with each other, DMd should usually listen rather than inserting itself after every sentence.

It should not repeatedly ask "What do you do?" when play is already occurring.

It should not offer action menus unless players ask for guidance or circumstances genuinely call for it.

Across a session, the Director should remain aware of spotlight distribution without enforcing artificial equal-time quotas.

---

## Speaker, player, character and authority identity

The runtime must distinguish human speaker, player identity, controlled PC/entity, in-character speech, out-of-character speech, DM-directed question and ordinary table chatter.

Misrecognized names must not transfer authority.

Session setup binds attending humans to player identities and controlled characters.

Support absent players, guest players, replacement characters, allowed controlled companions/summons and table-contract policies for absent PCs.

---

## PvP and inter-PC conflict

Session Zero defines whether and how PvP, PC theft, secret actions and inter-PC contests are allowed.

An attempted action against another PC does not silently determine that PC's voluntary response.

Where rules legitimately call for contested resolution, distinguish attempt, affected-player response, resolution and outcome.

---

## Player narrative ownership

Players may describe the cosmetic form of already-resolved outcomes.

Examples include a killing blow, spell appearance, celebration, mannerism, outfit, toast or inconsequential flourish.

Flavor may not silently grant extra mechanics or unsupported world facts.

PC thoughts, emotions and voluntary speech belong primarily to the player.

---

## Emotional register

The DM should recognize comedy, suspense, fear, grief, anger, triumph, uncertainty, intimacy, solemnity, relief and ordinary relaxed conversation.

Running jokes may persist, but must not be injected indiscriminately.

A serious moment should be allowed to remain serious.

---

## Difficulty and world fairness

DMd does not guarantee success.

PCs may die. Players may enter situations beyond their capability.

The world does not continuously rescale itself to make every encounter fair.

The world is also not an adversarial machine trying to defeat the players.

Consequences arise from causality, what actors know, evidence, relationships, resources, jurisdiction and willingness to act.

There must not be a hidden universal moral score that spawns level-appropriate punishment.

---

## Tactical philosophy

DMd must maintain enough spatial truth to adjudicate the selected fifth-edition tactical rules.

A grid/map should exist and may represent positions, distance, movement, reach, range, areas, obstacles, terrain, cover, elevation, line of sight and occupied spaces.

Players should not normally need to manually manipulate tokens for routine actions.

A player may say "I run up to him and stab him." DMd determines whether movement/range/path/rules permit it and either executes the legal action or explains the meaningful restriction.

The map supports rules; the map is not the primary command language.

### Complete tactical timing

The runtime must eventually handle applicable initiative, ties, surprise, turns, rounds, actions, bonus actions, reactions, reaction triggers, readying, movement interruption, opportunity attacks, concentration, durations, start/end-turn effects, ongoing saves, death saves, triggered features, controlled/summoned creatures, temporary effects and rule-required ordering.

### Enemy behavior

Combatants act according to perception, knowledge, intelligence, training, goals, morale, loyalty, self-preservation, relationships, orders and available capabilities.

Enemies may flee, surrender, negotiate, hide, raise alarms, protect allies, retreat, call reinforcements or refuse pursuit.

Combat AI must not be omniscient and not every opponent fights to the death.

### Tactical visibility

The runtime distinguishes actual battlefield truth from what each participant can currently see/know, including light, darkness, obscuration, hiding, unseen actors and fog of war.

---

## Environment and object interaction

Tabletop play allows attempts no UI designer predicted.

The runtime needs a general interaction/adjudication model for actions such as open, close, lock, unlock, break, push, pull, lift, drag, climb, jump, crawl, swim, dig, burn, extinguish, wedge, tie, cut, throw, hide, search, listen, smell, inspect, carry, drop, spill, barricade and improvise.

Relevant world objects need enough state/properties for rules and consequences.

DMd does not need molecular physics. It needs to avoid rejecting ordinary creativity because there is no dedicated button.

---

## World structure

Initial campaign creation uses prebuilt world templates.

The first production release does not need to invent an entire setting from nothing.

A world template may provide macro geography, settlements, cultures, factions, institutions, major NPCs, history, roads, political relationships, important locations, economic parameters, world rules and generation constraints.

Asterra is intended to become the first substantial real-world port after the generic world/content systems are mature.

Asterra remains content using the engine, never an engine assumption.

---

## Adventure structure and starter vertical slice

A world is not automatically an adventure.

The first production content must contain actual playable situations: motivated NPCs, locations, a settlement/base, travel/wilderness, a dangerous bounded location, investigation, social conflict, combat, treasure, secrets, discoverable information, interested factions and consequences.

Prepared material provides situations, not a mandatory script.

Players may cooperate, refuse, negotiate, flee, attack unexpected people, approach from another direction, overlook content, create their own objective or abandon the apparent adventure.

Before living-world/voice work is considered mature, DMd must have one small but complete legally distributable vertical-slice adventure that proves campaign creation, character creation, social play, exploration, investigation, tactical combat, inventory/loot, rest/resources, NPC consequences, save/restart and more than one plausible route.

At least one human playtest participant must attempt something the adventure author did not specifically anticipate.

---

## Procedural materialization

Prebuilt worlds cannot author every detail players may encounter.

DMd therefore needs procedural materialization for incidental farms, travelers, shopkeepers, interiors, alleys, employees, goods, wilderness detail and similar needs.

Procedural generation may initially create candidates.

Once players observe, interact with or otherwise canonize important detail, it becomes persistent campaign truth.

Returning later must not reroll contradictory reality.

Materialization must support mundane details, not only "interesting quest content."

---

## Living world

The world continues to exist when players are not observing it.

Relevant actors and processes may continue according to time, goals, knowledge, capabilities, relationships, geography, resources and previous events.

Not everything requires maximal simulation fidelity, but important developments need explainable causes.

World simulation should support meaningful schedules/travel where important, commitments, promises, debts, investigations, local law/enforcement, witness/evidence consequences, off-screen recovery and player-created projects where relevant.

---

## Opportunity and information model

A changing world produces information the players can perceive: rumors, gossip, letters, witnesses, prices, shortages, visible damage, construction, disappearances, guards, refugees, notices, requests and changed behavior.

Information may be true, false, incomplete, biased, outdated or manipulated.

Underlying world truth remains distinct from claims, beliefs and knowledge.

A quest is generally a player-facing framing of a situation rather than a hidden recurring `generate_quest()` result.

---

## Director

The Director controls attention, pacing and presentation, not reality.

Conceptually:

WORLD SIMULATOR
→ INFORMATION / OPPORTUNITY LAYER
→ DIRECTOR
→ NARRATION / TABLE

The Director may choose which supported developments deserve attention, delay presentation, combine compatible information, vary pacing, manage split-scene spotlight and allow quiet periods.

It may not invent convenient crimes/enemies, rewrite dice, change established facts, retroactively create causes, force outcomes or rescue players from their decisions.

### Campaign rhythm

The Director should avoid accidental repetition such as every town opening in a tavern, every road producing combat, every faction offering extermination work or every quiet period being interrupted by danger.

Variety must still be causally supported.

Quiet ordinary life is valid.

---

## Scene framing, zoom and time compression

Tabletop RPGs continuously change resolution.

DMd must support compression requests such as "we walk until something important happens", "we follow them to the inn", "we travel until nightfall" and "we spend the afternoon preparing."

It resolves ordinary intermediate activity and stops at the next meaningful decision/interruption.

Compression may not skip time, resource costs, world simulation, watches, triggered danger or consequences.

Free roleplay, exploration, tactical movement, combat, travel, downtime, rest, shopping and preparation are different resolutions of the same authoritative campaign, not disconnected toy modes.

---

## Preparation, rest, travel, downtime and economy

Preparation is gameplay.

Support choosing/preparing spells where applicable, acquiring supplies, replacing ammunition, buying equipment, identifying items, deciding what to carry, planning for weather/travel, lodging, information gathering, transport and loot division.

Tables may play these activities in detail or compress routine portions while retaining real costs and meaningful decisions.

Travel/rest must support applicable travel time, pace, route choice, navigation, food/water, light, exhaustion, weather, camp, watches, interrupted rest, short/long rest, resource recovery, spell preparation, hazards and encounters.

A long rest advances the world.

Economic play should support earning/spending money, barter, selling loot, unusual buyers, services, lodging, supplies, commissions, transport, negotiated prices, debts and receivables.

Availability derives from actual settlement/world state rather than a universal item catalog.

---

## Maps, clues, handouts and evidence

Important evidence may become a durable campaign object: maps, letters, notes, sketches, contracts, notices, diagrams, cargo records, inscriptions, clues and discovered documents.

Track actual contents, known provenance, carrier/custody, who has seen it, claims about it and unresolved questions.

Players may later inspect the same artifact again.

---

## Recaps and player-facing memory

Provide a durable player-safe memory layer derived from authoritative history.

Players should eventually be able to ask what happened last session, who an NPC was, what they know about a symbol, who owes money, what an inn was called, which situations remain unresolved, when an NPC was last seen and whether a route is known.

Answers are restricted to appropriate player/party knowledge.

Support session recap/transcript, NPC directory, known locations, clues, unresolved situations, commitments, important possessions and player-visible chronology.

This layer is derivative, never competing truth.

---

## Session start/end and exact suspension

Session start identifies attending players, binds characters, resolves content, restores exact state and offers a concise player-safe recap/immediate situation.

Session end does not invent a clean narrative endpoint.

Players may stop mid-dungeon, travel, town, pre-combat, mid-combat, negotiation or split scene.

Save/resume must preserve all authoritative transient state needed for exact continuation, including active initiative, current turn, remaining movement where applicable, reactions, concentration, durations, unresolved roll requests, pending material clarifications, split scenes, temporary effects, prepared resources, carried items, hidden actors, active negotiations and standing directives.

Already accepted events must not replay; unresolved actions must not disappear.

---

## Player-created plans may become the adventure

Players may decide to start a business, organize a performance, recruit informants, investigate a random shopkeeper, fortify a building, hunt an unexpected person, trade, join a faction, create a festival, start a feud, build a home or found an organization.

If possible within rules/world, the game should support consequences instead of responding "that isn't part of the quest."

Not every idea needs a bespoke subsystem immediately, but improvised goals must be able to become persistent situations.

---

## Failure without script rescue

Allow failed investigation, negotiation, lost targets, missed clues, lost fights, escaped enemies, wasted money, destroyed items, offended NPCs, failed missions and character death.

The Director must not secretly repair failure to preserve an expected plot.

Failure may create new circumstances, but not a hidden funnel back to predetermined success.

---

## NPC continuity and presentation identity

Important recurring NPCs should retain recognizable pronunciation, speaking style, vocabulary, temperament, relationships, attitudes, memories and relevant recurring mannerisms where established.

TTS may eventually support stable voice identity.

Presentation is not memory or authority; decisions still derive from authoritative state, beliefs, goals and rules.

---

## Local-first and reference hardware

The production gameplay loop works without cloud services.

Core play may not require GitHub, Drive, OpenAI, Anthropic, another external LLM or internet connectivity.

Optional cloud providers may exist.

The default supported local configuration should run comfortably on the reference machine:

- AMD Ryzen 5 8645HS;
- 16 GB RAM;
- NVIDIA RTX 4050 Laptop GPU;
- 6 GB VRAM;
- Windows x64.

Performance testing must include sustained multi-hour sessions and measure memory, VRAM, speech latency, DM response latency, TTS responsiveness and degradation/fallback behavior.

Model choice follows product experience + hardware constraints, not the reverse.

---

## Main desktop application

The desktop application is the central DM/session runtime.

Early production UI prioritizes usability over visual spectacle.

Core surfaces eventually include campaign selection/creation, session start/end, transcript, DM output, input state, roll requests/results, player decisions, character/session identity, tactical map when needed, campaign status and recovery/admin controls.

The first version does not need cinematic graphics.

---

## Companion application

Companion phone clients are later.

They may expose player identity, character sheet, HP/resources, inventory, abilities/spells, conditions, maps, logs/notes, private knowledge, advancement choices and player-specific rules reference.

Phones are clients of the authoritative central runtime, never independent sources of truth.

The main application remains playable when companions are unavailable.

---

## Rules explanation and learning support

The autonomous DM can answer rules questions quickly from the selected rules version.

It can explain what a player can do, why an action is invalid, concentration, advantage, movement and modifiers.

Experienced players should not receive constant unsolicited tutorials.

Explanation depth may be a player/table preference.

---

## Behavioral evaluation corpus

Maintain an autonomous-DM evaluation suite containing realistic messy inputs: transcription errors, missing punctuation, incorrect names, combined dialogue/actions, several PCs in one declaration, corrections, hypotheticals, jokes, profanity, repeated actions, impossible assumptions, unsupported information requests, split parties, PvP, chatter, vague pronouns, rules questions and interrupted speech.

The existing Asterra transcripts are valuable private/reference material.

Public actual-play may be studied for behavioral patterns, but copyrighted transcripts must not simply be copied into distributable content/training material without legal basis.

The goal is not to imitate a celebrity DM. It is to prove useful tabletop behavior.

---

## Finished-game acceptance principle

The final question is not only "Can DMd execute the rules?"

It is:

"Can people forget about the software for long stretches and simply play D&D?"

Memorable moments should routinely arise because players had strange ideas, rules gave them structure, dice introduced uncertainty, NPCs reacted according to circumstances, the world remembered, consequences accumulated, and the DM knew when to adjudicate, describe, ask, listen, or stay silent.
