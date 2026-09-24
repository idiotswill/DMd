# Rules Coverage Ledger template

Status: **Gate 2 required artifact**

The ledger is the mechanical completeness/provenance inventory for the first production rules pack.

It prevents "generic engine support" or a handful of representative mechanics from being mistaken for complete fifth-edition support.

## Rules-source identity

- Ruleset/content license:
- Exact legal source/version:
- Source publication/revision date:
- Required attribution:
- Repository provenance record:
- Commercial-use review:
- Verification/decision record:
- Compatibility label/branding policy:

## Required columns

| ID | Rules/content family | Source citation/provenance | Legal-to-ship | Gate owner | Implementation status | Mechanical tests | Production integration | Player-facing acceptance | Notes/deferred reason |
|---|---|---|---|---|---|---|---|---|---|

Recommended statuses:

- unsupported;
- intentionally deferred;
- implementing;
- implemented;
- mechanically tested;
- production-integrated;
- player-accepted.

## Minimum inventory families

The actual ledger must be derived from the selected legal source, not only this list.

Inventory at least:

- core resolution;
- ability checks;
- saving throws;
- contests where applicable;
- advantage/disadvantage;
- proficiency;
- inspiration/heroic inspiration where applicable;
- movement;
- jumping/climbing/swimming;
- travel;
- vision/light/obscuration;
- hiding/stealth;
- initiative/surprise;
- action economy;
- reactions;
- readying;
- opportunity attacks;
- attacks;
- armor class;
- damage/healing;
- critical hits;
- dying/death saves;
- rests;
- exhaustion;
- conditions;
- concentration;
- spellcasting;
- spell targeting/range/areas/durations;
- equipment;
- weapons;
- armor;
- adventuring gear;
- tools;
- money/services;
- classes;
- subclasses;
- species/ancestries;
- backgrounds;
- feats;
- class features;
- species traits;
- spells;
- monsters/NPC stat blocks;
- summons/companions;
- mounts/vehicles where supported;
- environmental hazards;
- magic items where legally available;
- advancement/leveling;
- character creation;
- downtime/crafting where present;
- any optional rules intentionally supported.

## Gate allocation rule

Every row must have exactly one primary implementation gate or an explicit `not supported in DMd 1.0` decision that is compatible with the accepted product definition. Codex may not use this status to silently narrow a required 1.0 capability; a genuine scope conflict must be surfaced.

No row may remain silently unassigned.

## Completion rule

DMd may not claim "complete first rules pack" while rows needed by the target experience remain merely "generic primitive exists".

Gate 14 must validate representative interactions across the ledger through the production application, not only unit tests.
