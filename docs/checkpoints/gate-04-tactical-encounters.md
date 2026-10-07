# Gate 4 — Tactical encounters

Status: **Active — owner authorized after Gate 3 on 2026-09-24; acceptance pending.**

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

## Bounded native evidence — Physical creation, 2026-10-07

PR #62 source `48f7c92b63bd2f9c586d2622d1875342f004b093` was exercised through
the packaged Windows desktop: normal source-created Greatsword/Glaive purchases
and materialization, Greatsword Graze accept and ordinary-miss decline, Glaive
miss/decline and held ten-foot opportunity, printed weapon damage, and normal
save/close/reopen. The [Physical creation plan](../exec-plans/active/gate4-physical-creation-source.md#current-evidence--controlled-native-qa-completed-2026-10-07)
records the exact source tree, package/executable identity, campaign/ItemIds,
operator observations, artifact locations and hashes.

Independent copied-data checks cover 40 captures, ten complete logical triples,
exact raw/history prefixes and three unchanged protected campaigns. Glaive's
reaction dealt 7 and resumed five feet of movement with one Reaction; the final
Greatsword hit dealt 5 from 2d6 [1,1] +3, leaving Glaive at 3 HP and Greatsword at
4. No pending attack/raw request remained. The final bridge-closed cut matched the
completed campaign data; actual process exits/reopens are separately documented
operator evidence. The missing October 6 equipment reopened cut remains missing.

All entered faces were controlled automated QA inputs through physical-roll
prompts, not evidence of human physical throws. This advances source acquisition,
owned combat and exact suspension/resume but does not satisfy the complete
production integration acceptance above. Native critical/Savage variations,
portable recovery, comprehensive privacy/performance and human-play evidence are
not inferred from this route. PR #62 final evidence-head review/checks and any
merge verification remain separate work. Gate 4 remains active and unaccepted;
the product contract and every acceptance criterion above remain unchanged.
