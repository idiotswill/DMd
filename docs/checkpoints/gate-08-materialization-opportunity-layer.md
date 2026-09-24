# Gate 8 — Procedural materialization and opportunity layer

Status: **Planned**

## Product requirements advanced

- unauthored but coherent detail;
- persistent materialization;
- rumors/leads/requests/conflicts/discoveries;
- information propagation;
- opportunity lifecycle;
- repetition control;
- mundane procedural reality.

## Objective

Allow play to continue outside authored detail and convert living-world motion into perceivable, persistent situations without a random quest vending machine.

## Entry conditions

Gate 7 accepted.

## Acceptance criteria

Support:
- deterministic/contextual candidate generation;
- incidental NPCs/locations/interiors/goods/environment detail;
- persistent canonization after observation/interaction;
- provenance/knowledge for generated detail;
- rumors/claims/leads/requests/offers/conflicts/discoveries/opportunities;
- information propagation with bias/staleness/falsehood;
- opportunity expiration/transformation/success/failure without player participation;
- mundane materialization, not only dramatic content;
- repetition controls grounded in available developments.

## Architecture invariants

Procedural model/provider output is proposal only. Materialized facts require validation and authoritative commit. Generation cannot overwrite established facts or campaign isolation.

## Explicit non-goals

Full conversational autonomous DM/Director, voice, Asterra production port.

## Required failure/recovery behavior

Generation/provider failure leaves current state intact and supports retry/fallback. Revisit loads persisted material rather than rerolling contradictory facts.

## Merge/pause boundaries

Codex may merge in-scope Gate 8 content/materialization architecture autonomously after exact-head verification and legal provenance checks. Do not replace causal world development with unconstrained narrative generation. Pause at the end of Gate 8 with persistence/revisit/opportunity evidence and Gate 9 handoff.
## Candidate workstreams

Materialization pipeline, procedural content constraints, opportunity/information layer, knowledge propagation, persistence/revisit tests.

## Deferred requirements

Director/table presentation Gate 9; speech Gate 10; Asterra stress test Gate 11.

## Open questions

Model/algorithm choices remain open. Acceptance is behavioral/causal, not tied to one generator.

## Production integration acceptance

Players deliberately travel somewhere the template did not fully author. DMd materializes enough coherent detail to continue. They interact, leave, world time advances, and on return the same persisted reality has changed only through causal events. At least one new player-facing situation must emerge from world state rather than authored starter hooks.
