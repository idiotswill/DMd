# Gate 7 — Persistent living-world simulation

Status: **Planned**

## Product requirements advanced

- off-screen autonomous world motion;
- causal actor/process evolution;
- explainability/provenance;
- law/witness/evidence consequences;
- projects/relationships/economy pressures;
- player-created persistent projects.

## Objective

Make relevant actors and processes advance when players are elsewhere, producing durable explainable consequences instead of freezing off-camera.

## Entry conditions

Gate 6 and mandatory human vertical-slice checkpoint accepted.

## Acceptance criteria

At minimum demonstrate useful production fidelity for:
- NPC goals/activity;
- faction activity;
- travel/movement/schedules where important;
- projects;
- relationships;
- resources;
- local threats/conflicts;
- economic pressure;
- commitments/promises/debts;
- investigations/law enforcement;
- witness/evidence consequences;
- recovery/injury processes;
- player-created projects/businesses.
Changes must be causal and provenance-inspectable.

## Architecture invariants

Actors act from available knowledge/beliefs/capability/resources/geography, not omniscient truth. Director is not allowed to manufacture world events. Simulation fidelity may vary, but authoritative causes cannot be narrative invention.

## Explicit non-goals

Long-tail procedural materialization, opportunity presentation/Director, voice.

## Required failure/recovery behavior

Simulation can resume deterministically/recoverably from authoritative state. Partial failures cannot leave time advanced without corresponding events/state. Expensive/background processing must fail safely.

## Merge/pause boundaries

Codex may merge in-scope Gate 7 simulation architecture/save changes autonomously after exact-head verification when they preserve causal/provenance invariants. Pause at the end of Gate 7 with representative causal-world evidence and Gate 8 handoff.
## Candidate workstreams

World clock/process scheduling, actor goal execution, faction/project systems, causal provenance, simulation recovery/performance.

## Deferred requirements

Procedural detail/opportunities Gate 8; presentation/Director Gate 9.

## Open questions

Choose selective simulation fidelity deliberately; avoid attempting to simulate every citizen at identical detail.

## Production integration acceptance

Advance substantial in-world time while party is elsewhere. Return to an earlier region and observe several coherent changes caused by persisted actors/processes. For representative changes, an admin can answer why they occurred from stored state/events. Also demonstrate a relevant process that did *not* change because prerequisites/resources were absent.
