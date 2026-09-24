# ADR 025 — Tactical spatial truth and compatible campaign state

Status: **Accepted for the Gate 4 foundation in PR #23; complete encounter acceptance remains pending.**

## Product requirements and source

Gate 4 advances authoritative spatial combat, source-faithful targeting/movement, actor
perception, environmental interaction and exact suspension. The selected mechanical source
is the pinned English SRD 5.2.1, especially pages 13–18 and glossary pages 176–191. Source
adaptations retain the shipped notice. This decision does not claim a complete tactical
application or move any of the twelve Gate 4 ledger families to a later gate.

## One spatial authority

Introduce an optional typed tactical aggregate in campaign schema 4. Coarse world locations
and scenes retain their existing meaning; encounter coordinates do not overload a location
identifier. Participants reference real campaign entities. Existing rules state remains
the authority for HP, rolls, conditions, resources and initiative; no independent tactical
copy of those values may drift from it.

The spatial model separates physical obstruction, sight obstruction, cover, illumination,
obscuration and support/elevation. Creature footprint, movement modes and senses are durable
source-derived capabilities. Position arithmetic and all collections have explicit bounds.
The five-foot square grid follows the selected source's diagonal movement/range convention;
finer fixed units represent Tiny spaces and geometry without floating-point replay drift.
Source rules do not prescribe every raster/cover algorithm: implementation conventions and
genuine GM geometry adjudications must be identified rather than attributed to an invented
source clause. Invalid or ambiguous geometry fails before committing any movement.

Pure geometry functions derive paths, costs, cover, visibility and area membership. They do
not mutate authoritative state. The tactical resolver commits accepted consequences and
records provenance through the existing atomic application boundary. The renderer may
preview/display geometry; it cannot authorize it or provide final modifiers/damage.

Dash retains the selected ordinary/special speed (SRD p.180). The default interpretation
increases only that speed's allowance, while every mode subtracts shared movement spent
(p.188). Speed 30/Fly 60 with Dash using Speed therefore has allowances 60/60; Dash using
Fly has allowances 30/120. The source requires a choice but does not explicitly settle
transferring that extra movement to another mode; this boundary convention preserves the
choice without treating extra movement as a change to Speed. Alternative treatment requires
an explicit persisted campaign adjudication. The query receives derived selected-speed
counts; it never stores or grants a second movement budget.

Arbitrary bounded integer direction vectors support aiming between grid axes. Sampled
occlusion cannot prove that several solids collectively grant Total Cover: uncertainty
requires an authored ruling. Sight and Blindsight still check actual candidate rays.
Magical darkness and independent heavy fog are distinct causes; Truesight bypasses only
darkness within its range. Dim-only light has no Bright radius. Unaware creatures acquire
no new sensory contacts or current self-position, while their remembered information stays
available. Projection also has a deterministic shared work limit: dense pathological
queries return Capacity instead of partial or unfiltered information.

## Perception and knowledge

Actual position, present sensory contact and remembered position are different values.
Current contact is derived from current geometry/senses/conditions; stored memory records
only what an actor acquired, with observation provenance. A last-known marker must never
track an unseen actor's current position. Tremorsense locates but is not sight. Blindness,
blindsight, invisibility, darkness and special senses require pairwise evaluation.

Player and enemy views are projections, not raw encounter state. Enemy policy receives only
its actor view, established capabilities and durable goals/morale; it returns a proposal
which rules validate. Guessed/unseen attacks address a declared location rather than quietly
following an entity's hidden position. Secret actions must not leak through token IDs,
transcript, recap, roll reasons, pending choices or error messages.

## Durable continuation and replay

Movement interruption, reactions, Ready triggers, multi-target effects and follow-up dice
must be explicit typed saved continuations before desktop integration. Action resolution
may pause for another controller; the UI cannot stand in for the scheduler. Accepted raw
faces and chosen proposals are replay inputs. Replay must not rerun an ambient planner or
silently reinterpret historical text with a new grammar. Legacy table/rules event semantics
remain supported; new tactical semantics get explicit versioned dispatch.

The following encounter slice owns detailed timing/continuation semantics. The foundation
is incomplete until linked to that production path; standalone geometry tests do not accept
Gate 4.

## Compatibility

Schema 3 to 4 adds an absent encounter without inventing combat. Upgrade current images
atomically after strict metadata/JSON preflight. Preserve immutable snapshots, journal,
audit and observation bytes; decode historical snapshots through the registered migration
chain. Reject non-null future encounter data under historical schemas rather than silently
preserving it. No new aggregate storage tables are required solely for encounter state;
portable format 2 can retain its existing envelope while explicit state versions evolve.

All pending SQLite migrations run inside one outer transaction. SQLx receives the already
acquired connection through `run_direct`; this preserves nested migration savepoints and
keeps the opening future Send for the native desktop command boundary. The public generic
Acquire wrapper produced a Windows compilation failure and is not used here.

Existing rules definitions and their historical outcomes remain stable. New tactical source
definitions are separately versioned and validated. Loading new metadata does not imply
that its effect execution or full source catalog is already complete.

## Required evidence

Geometry tests must cover boundaries, sizes, corners, elevation, movement modes, cover,
all area shapes and sensory distinctions. Projection tests must inspect serialized output
for hidden data. Migration tests must compare exact old snapshot/history bytes, reject
corrupt/future-state smuggling and verify atomic rollback. Integrated rules/application
tests and the packaged multi-round scene remain prerequisites for final Gate 4 acceptance.
