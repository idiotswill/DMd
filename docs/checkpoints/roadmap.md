# DMd future-gate roadmap

Status: **Owner-approved roadmap — reconciled against main on 2026-09-24**

## Purpose

This roadmap turns the finished-product contract into bounded implementation gates.

It does not replace `docs/product-definition.md`.

Every gate must advance the production path, state which product requirements it advances, identify what remains deferred, and include production integration acceptance.

Future gates are behaviorally specific but should avoid premature implementation design.

Only the active gate receives a detailed execution plan.

## Preconditions before Gate 2

1. Gate 1 remains accepted on current `main`; re-verify rather than re-opening completed work by assumption.
2. The roadmap/bootstrap PR has integrated the approved product expansion, Gate 2–14 checkpoints, traceability, Rules Coverage Ledger requirement, vertical-slice checkpoint, and gate execution protocol.
3. Root `AGENTS.md` has been reconciled with the owner's autonomous-within-gate merge / end-of-gate pause authorization.
4. The bootstrap PR has passed exact-head repository verification and has been merged.
5. `main` has been refreshed after that merge.

The bootstrap merge is not an end-of-gate pause. Codex should then start Gate 2 on a fresh branch.

## Gate sequence

| Gate | Name | Player/product capability |
|---|---|---|
| 1 | Campaign persistence foundation | Trustworthy durable campaign/runtime foundation. |
| 2 | Commercial fifth-edition rules kernel | Legally distributable, auditable rules foundation and rules completeness ledger. |
| 3 | First playable desktop table loop | Human can create/open a campaign, create/select PCs, play text-first, roll physical dice, save/restart. |
| 4 | Tactical encounters | Complete authoritative tactical/spatial combat path with natural declarations. |
| 5 | Noncombat adventuring | Exploration, social play, travel, inventory, rests, downtime, economy, split scenes and open-ended interaction. |
| 6 | World templates and starter adventure | Reusable worlds/content plus a complete legally distributable vertical-slice adventure. |
| — | Mandatory vertical-slice human playtest | Four humans prove the production app is actually playable before autonomy/world-simulation complexity accelerates. |
| 7 | Persistent living-world simulation | Off-screen actors/processes advance causally and explainably. |
| 8 | Procedural materialization and opportunity layer | Unauthored detail becomes coherent persistent reality; new situations emerge from world state. |
| 9 | Autonomous DM and Director | Messy natural text/table interaction is interpreted, adjudicated and narrated like a competent DM. |
| 10 | Local speech and table runtime | Natural voice becomes the normal local-first multi-hour interface. |
| 11 | Asterra production world port | A substantial real world stresses the generic engine without engine-specific coupling. |
| 12 | Companion player experience | Phone clients provide sheets/maps/logs/private info while authority stays central. |
| 13 | Recovery, administration, performance and release | Installable, recoverable, configurable, performant product path. |
| 14 | Endurance acceptance | Same production build proves sustained, enjoyable, unscripted campaign play. |

## Milestones

### Foundation complete
Gate 1.

### First mechanically playable DMd
Gates 2–5.

### First complete tabletop vertical slice
Gate 6 + mandatory human playtest.

### First autonomous campaign-capable DMd
Gates 7–10.

### First substantial real-world content build
Gate 11.

### Full player-device experience
Gate 12.

### Release candidate
Gate 13.

### Finished DMd 1.0
Gate 14.

## Gate completion rule

A gate is not complete merely because isolated subsystems work.

Every gate must answer:

- Can a normal user reach the capability appropriate to this gate?
- Does it operate on authoritative campaign state?
- Does persistence/restart preserve it where applicable?
- Do failure paths preserve integrity?
- Is the capability connected to the production application path?
- Are later requirements still explicitly deferred rather than accidentally omitted?
- Did work introduce campaign-specific assumptions?
- Did provider output become authority?
- Did the implementation silently weaken an earlier invariant?
- Is exact verification evidence available on the reviewed head?

## Implementation discipline

For each gate:

1. establish/verify entry conditions;
2. create an active execution plan for the bounded task;
3. split work into coherent PR-sized slices/branches where useful, preserving one writer per branch;
4. open PRs early enough for CI;
5. implement through production-intended boundaries;
6. test mechanics and integration;
7. test failure/recovery;
8. inspect complete diffs;
9. run required exact-head verification;
10. update checkpoint/plan/PR;
11. merge in-scope PRs autonomously only after their exact heads satisfy the required checks and acceptance;
12. refresh `main` after every merge;
13. perform an integrated final-gate review on the merged head;
14. pause at the end of the gate with an evidence-based summary and wait for the owner before beginning the next gate.

Codex must not silently weaken `docs/product-definition.md`, waive failed acceptance criteria, or hide debt to make a gate pass.

Do not pull attractive future-gate features forward unless a minimal prerequisite is genuinely required.
