# Gate 6 — World templates and starter adventure

Status: **Planned**

## Product requirements advanced

- reusable prebuilt worlds;
- production content packs;
- adventure situations;
- maps/clues/handouts/evidence;
- legal content provenance;
- first complete tabletop vertical slice.

## Objective

Make DMd host real reusable settings and a legally distributable starter adventure through production content architecture.

## Entry conditions

Gates 2–5 accepted. Content manifest/catalog foundation from Gate 1 remains enforced.

## Acceptance criteria

Support versioned:
- rules/content/world packs;
- macro locations, settlements, factions, institutions;
- important NPC templates;
- roads/routes, cultures, calendars/settings;
- creatures/items/hazards needed by starter content;
- adventure situations, secrets, clues, treasure, handouts;
- deterministic campaign creation;
- durable evidence objects and knowledge visibility;
- a small but complete starter adventure with social, exploration, investigation, tactical combat, resources/rest and multiple plausible approaches.
Create two independent campaigns from the same template and prove isolation/divergence.

## Architecture invariants

World template/content is immutable/shared; campaign materialization is mutable/campaign-scoped. Engine remains setting/name independent. All distributable content has provenance/license status.

## Explicit non-goals

Asterra production port, living world autonomy, procedural long-tail materialization, voice.

## Required failure/recovery behavior

Missing/incompatible/corrupt packs fail closed before runnable play. Content update cannot silently replace the campaign's pinned version. Campaigns remain recoverable through raw diagnostics.

## Merge/pause boundaries

Codex may merge in-scope Gate 6 world/content/architecture work autonomously after legal provenance and exact-head verification. The mandatory human vertical-slice playtest is an external acceptance requirement: engineering work may be merged, but Gate 6 must not be treated as fully accepted for progression to Gate 7 until that playtest passes or the owner explicitly directs otherwise. Pause at the Gate 6 boundary with the playable build and playtest instructions.
## Candidate workstreams

World/content schema, authoring validation, starter legal content, adventure/evidence objects, world creation UX, integration fixtures.

## Deferred requirements

Off-screen simulation Gate 7; procedural materialization Gate 8; autonomous DM Gate 9; Asterra Gate 11.

## Open questions

Exact first starter setting/adventure creative content can be chosen here, but must be legal to distribute and not Asterra-dependent.

## Production integration acceptance

Create two campaigns from the same production world template. Play the complete starter vertical slice through the desktop runtime, proving social, exploration, investigation, combat, evidence, inventory, rest and restart. Then run the mandatory human vertical-slice checkpoint before Gate 7.
