# Rules Coverage Ledger

Status: **Source inventory established; mechanics implementation remains in progress in Gate 2**.

[The machine-readable ledger](rules-coverage-ledger.json) is the implementation/evidence record. [The source inventory](srd-5.2.1-inventory.json) records source chapter coverage, glossary membership and named catalogs. [Source provenance](srd-5.2.1-provenance.md) pins the official English SRD 5.2.1 and its commercial CC BY 4.0 terms; the [distribution notice](../../content/srd-5.2.1/NOTICE.md) must accompany source adaptations.

## What is accounted for

The ledger has 55 rules/content families. Every family has one numeric `primary_gate`, legal/source citations, explicit scope, status, mechanical test evidence, production integration evidence, player acceptance evidence and final Gate 14 ownership. Empty evidence arrays mean no such evidence is claimed. An inventory entry inherits its owning family's gate/status; listing a spell or creature does not implement it.

The source inventory covers all fourteen substantive chapter groups, with printed page citations. Pages 2–4 are contents/index navigation; their entries are represented by the chapters and catalogs rather than counted as mechanics. It additionally names:

| Source category | Entries | Meaning |
|---|---:|---|
| Rules glossary | 155 | Every named glossary entry; each mapped deliberately to its owning family |
| Spells | 338 | Every top-level spell-description heading, pages 107–175 |
| Magic items | 258 | Every top-level magic-item heading, pages 209–253 |
| Monster/NPC/animal stat blocks | 330 | Every named stat block, pages 258–364 |
| Classes / subclasses | 12 / 12 | All selected-source classes and subclasses |
| Backgrounds / species | 4 / 9 | All selected-source choices |
| Feats | 17 | All Origin, General, Fighting Style and Epic Boon entries |

These counts describe source entries, not distinct permutations: variants within a heading (such as item bonuses, lineages, spell choices, transformations, feature options or summoned stat blocks inside spells) remain obligations of that parent entry. Equipment tables, trinkets, class features, spell lists, Metamagic, Invocations, poisons, traps and toolbox rules are covered by explicit families/page ranges. Expand these into per-asset executable records and evidence as their owning implementation gate ingests them. No parent family may be declared complete while its source subentries are silently omitted.

The PDF's visible contents and page headings take precedence over bookmark destinations: several supplied bookmarks land on a preceding page. Named catalog headings were extracted from the pinned PDF and reviewed against chapter structure; they do not ingest copyrighted books or third-party transcriptions.

## Implementation ownership

- **Gate 2:** legal pin and completeness accounting; validated character modifiers; D20/proficiency/advantage; attack/HP/damage foundations; conditions/resources; timing/reaction, rest, spellcasting/concentration and passive/secret primitives.
- **Gate 3:** supported character creation/sheets and the first text-first table flow.
- **Gate 4:** full spatial combat, all timing/effect/condition interactions, target geometry, visibility, tactics, death saves, mounts, unarmed/grapple, weapon mastery and combat spell execution.
- **Gate 5:** full exploration/social/travel/rest/crafting/economy, advancement/multiclass, environmental/curse/poison/trap rules, noncombat magic and item use.
- **Gate 6:** complete legally reusable catalogs/options plus production content integration. Gates 4/5 own mechanical dependencies. Gate 6 must account for every catalog entry and its variants, not only content used by the starter adventure.
- **Gates 9/13/14:** autonomous interpretation/explanation, distribution notices/recovery, and human end-to-end acceptance remain cross-cutting. These do not absorb unfinished rules mechanics without an explicit roadmap change.

Primitives and complete families have separate rows where needed. For example, `condition-primitives` belongs to Gate 2 while `combat-conditions` owns complete tactical semantics in Gate 4. A typed marker alone is not full condition support. The same distinction applies to spellcasting vs complete spell effects/catalogs, timing vs complete encounters, and recovery primitives vs actual rest gameplay.

## Updating and validating

Use `planned`, `intentionally_deferred`, `implementing`, `implemented`, `mechanically_tested`, `production_integrated`, or `player_accepted`. All current rules rows are planned/deferred; this source slice does not claim gameplay tests or user acceptance. Every deferred row names its receiving gate through `primary_gate` and explains scope. Optional toolbox rules are inventoried even when disabled by default; opting in must be explicit campaign configuration. Non-SRD content is outside this selected-source inventory and cannot enter by familiarity or through a claim of generic compatibility.

When implementation advances, add exact test names/files, production scenarios/heads and player acceptance reports to the matching arrays. A feature may only advance to `mechanically_tested` with test evidence, to `production_integrated` with mechanical and real application evidence, and to `player_accepted` with all three. Gate acceptance remains governed by checkpoints; the ledger cannot waive it.

`cargo test --locked -p dmd-domain --test rules_coverage_ledger` runs offline in normal workspace CI. It checks the pin, complete chapter span, reviewed catalog counts, unique names/IDs, source-page bounds, every inventory-to-family relationship, gate assignment and evidence for advanced statuses. Negative cases demonstrate rejection of orphaned families/entries, duplicate ownership, removed catalog entries and unsupported completion claims. It checks consistency against the reviewed source inventory; it is not a claim that software can infer legal scope or prove faithful gameplay from JSON alone.
