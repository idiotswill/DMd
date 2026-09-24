# Gate 2 — Commercial fifth-edition rules kernel

Status: **Planned**

## Product requirements advanced

- commercially distributable first rules pack;
- Rules Coverage Ledger and licensing/provenance;
- deterministic application-owned rules resolution;
- character mechanics and derived values;
- physical/digital dice integration;
- rules questions/adjudication hierarchy;
- primitives needed by tactical and noncombat gates.

## Objective

Establish a legally distributable, auditable and production-intended fifth-edition mechanics foundation. The gate must prove completeness accounting, not merely representative mechanics.

## Entry conditions

- Gate 1 accepted.
- roadmap/product expansion bootstrap merged.
- exact legal rules source/version verified from current official sources and recorded before content ingestion/implementation that depends on it.

## Acceptance criteria

At minimum:
- create the Rules Coverage Ledger from the exact selected legal source;
- account for every rules/content family with gate ownership;
- implement the core resolution model required by later gates;
- ability checks, saves, proficiency, advantage/disadvantage;
- attack/AC/damage/healing foundations;
- conditions/effects/resource expenditure/recovery primitives;
- initiative/timing/reaction/concentration primitives needed later;
- rest/resource primitives;
- spellcasting primitives;
- character-derived modifiers and validation;
- rule-driven RollRequest/Result/ResolvedRoll through existing dice boundary;
- explicit rules-query path that does not commit an action;
- campaign house-rule representation and ruling provenance;
- passive/secret resolution primitives;
- deterministic tests and invalid-action tests;
- no provider/LLM decides authoritative rules outcomes.

## Architecture invariants

- AI/provider output never authoritative.
- trusted issuer/actor separation remains.
- rules/content remains versioned and content-isolated.
- raw persistence remains content agnostic.
- rules behavior must be reproducible from authoritative inputs.
- legally non-reusable D&D material must not enter shipped content by convenience.

## Explicit non-goals

- complete tactical map/battlefield;
- autonomous DM;
- voice;
- living-world simulation;
- Asterra port;
- polished desktop UX;
- implementing every ledger row that properly belongs to Gates 4/5/6, but those rows must already be assigned.

## Required failure/recovery behavior

Invalid, impossible, stale, unauthorized or incompatible rules actions fail without authoritative mutation. Rules/content version mismatch fails closed through the runnable-campaign boundary. A failed provider/helper explanation cannot change state.

## Merge/pause boundaries

Codex may merge Gate 2 PRs, architecture records, and checkpoint updates autonomously after exact-head verification when they satisfy the approved Gate 2/product contract.

Do not silently narrow the product or claim proprietary/unlicensed material is distributable. If official licensing evidence is genuinely ambiguous enough to threaten commercial distribution and no clearly safe implementation exists, surface that as a real blocker.

Pause for the owner at the end of Gate 2 with the complete rules-source/provenance decision, merged PRs, verification evidence, ledger state, remaining debt, and Gate 3 handoff.

## Candidate workstreams

- legal rules-source/provenance inventory;
- Rules Coverage Ledger;
- character/rules kernel;
- effects/timing/resource model;
- rules query/adjudication model;
- integration with existing dice/app/persistence boundaries;
- mechanical + property/regression tests.

## Deferred requirements

Tactical geometry/combat completion (Gate 4), full noncombat systems (Gate 5), world/adventure content (Gate 6), autonomous DM (Gate 9), voice (Gate 10).

## Open questions

- Exact licensed source/version and attribution text must be verified from current official sources.
- Which optional rules ship enabled/disabled by default?
- Which legally unavailable iconic D&D content needs original compatible substitutes later?

## Production integration acceptance

Through the real `dmd-app` runnable campaign path, create/load a representative legal campaign/character, execute core rules actions using the real dice boundary, durably commit results, restart/reopen and prove identical authoritative mechanics. Demonstrate invalid actions and content/version failures do not mutate state. The Rules Coverage Ledger must show no unassigned family.
