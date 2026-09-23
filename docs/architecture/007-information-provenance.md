# ADR 007 — Information provenance

Status: **Accepted for Gate 0 foundation**

## Problem

Mystery, politics, deception, rumors, documents, and faction intelligence fail if the engine collapses all information into a single truth store. The source of an assertion may also be something other than a speaking NPC: a ledger, proclamation, inscription, anonymous rumor, or institution can introduce information into play.

## Decision

DMd distinguishes four concepts:

- **Fact** — accepted world truth for a validity interval.
- **Claim** — an assertion that exists regardless of whether it is true.
- **Belief** — an individual or faction's internal proposition with confidence and basis.
- **KnowledgeRecord** — durable access to an established fact or claim, identified by a stable `KnowledgeId`.

A claim is never promoted into a fact merely because it exists or because a social check succeeded.

## Information agents

`AgentRef` identifies an information-holding actor as either:

- an `EntityId`, for an individual/creature/character;
- a `FactionId`, for institutional knowledge and collective operational beliefs.

This allows a faction to know a route, believe an enemy is weak, or act on false intelligence without requiring that knowledge to be duplicated onto every member NPC.

## Claim sources

`ClaimSource` may be:

- an agent (entity or faction);
- an item, such as a letter, ledger, map annotation, recording, or document;
- a location, for inscriptions/notices/environmental messages;
- unknown, for genuinely unresolved hearsay or anonymous information.

The source is provenance, not truth status.

A document whose author is unknown can therefore assert something without DMd inventing an author. If authorship is later established, that is separate world information rather than a retroactive prerequisite for the claim to exist.

## Belief basis

Beliefs may be based on:

- a materialized fact;
- a claim;
- direct observation of an event;
- an inference composed from other bases.

Snapshot validation checks fact/claim basis references. Direct observation event IDs are checked against the append-only journal by persistence/replay because historical events are not all embedded in `CampaignState`.

## Knowledge boundaries

`KnowledgeHolder` may be:

- an entity agent;
- a faction agent;
- explicit shared-table knowledge.

Each materialized `KnowledgeRecord` has its own stable identity so persistence, corrections, migrations, and provenance do not have to address records by display text or vector position. Repeated observations or reminders may create journal events without requiring duplicate current-state knowledge rows.

Out-of-character table discussion does not create character or faction knowledge. Narrative and NPC-dialogue renderers must receive only the information visible to the relevant holder/context.

## Consequences

This model supports, without inventing special cases:

- unreliable witnesses;
- forged or authentic documents;
- propaganda;
- conflicting testimony;
- anonymous rumors;
- NPC lies;
- faction intelligence;
- partial investigations;
- later correction of false beliefs;
- documents whose provenance is still unresolved.

## Explicitly deferred

This ADR does not yet define:

- evidence strength/scoring;
- automatic Bayesian belief updates;
- trust/reputation dimensions;
- social-check mechanics;
- document authenticity mechanics;
- rumor propagation simulation.

Those systems may consume these records but must preserve the distinction between truth, assertion, belief, and access.
