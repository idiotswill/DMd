# Gate 4 — Tactical encounters

Status: **Planned**

## Product requirements advanced

- authoritative spatial combat;
- fifth-edition timing/action economy;
- natural-language-to-tactical commands via typed command path;
- visibility/stealth;
- enemy behavior/morale;
- environment interaction;
- exact combat save/resume.

## Objective

Support complete production-intended tactical encounters while allowing players to speak ordinary intent instead of manually operating the grid for routine actions.

## Entry conditions

Gates 2–3 accepted; Rules Coverage Ledger identifies tactical rows owned here.

## Acceptance criteria

Implement applicable:
- grid/spatial geometry and positions;
- movement/path legality, terrain, reach/range/areas;
- line of sight, cover, lighting/obscuration;
- initiative/rounds/turns/ties;
- actions/bonus actions/reactions/readying/opportunity attacks;
- attacks/damage/healing/conditions/death;
- concentration/durations/start-end-turn triggers/ongoing saves;
- hidden/unseen participants and player-safe fog of war;
- summons/controlled entities as required;
- environmental improvised actions;
- enemy perception/knowledge/goal/morale-constrained behavior;
- fleeing, surrender and negotiation;
- exact mid-combat save/resume.

## Architecture invariants

Spatial/tactical state is authoritative structured state, not narrative text. Combat planners cannot use hidden knowledge unavailable to the actor. AI can propose behavior but rules/core validates and resolves.

## Explicit non-goals

Living-world simulation outside encounter scope, autonomous table DM, voice, cinematic graphics.

## Required failure/recovery behavior

Invalid movement/actions do not partially mutate. Mid-encounter restart preserves exact initiative/effects/resources/visibility. Hidden information does not leak through player UI/logs.

## Merge/pause boundaries

Codex may merge in-scope Gate 4 architecture/save/content changes autonomously when they are required by the accepted checkpoint and verified on the exact head. Do not weaken rules fidelity, persistence, player agency, or information visibility. Pause at the end of Gate 4 with integrated combat evidence and the Gate 5 handoff.
## Candidate workstreams

Spatial model, encounter/timing engine, visibility, combat commands, enemy policy, map UI, effect persistence, integration tests.

## Deferred requirements

Broad travel/economy/downtime Gate 5; sophisticated autonomous language Gate 9; speech Gate 10.

## Open questions

Exact map rendering technology and tactical UX can be chosen during execution planning. Avoid requiring drag-token interaction as the command language.

## Production integration acceptance

Run a complete multi-round encounter through the real desktop application with physical player dice. Include a reaction, concentration/ongoing effect, limited visibility, blocked/difficult movement, improvised environmental action and an opponent who flees/surrenders/negotiates. Save/exit/resume while initiative is active and finish with correct durable consequences.
