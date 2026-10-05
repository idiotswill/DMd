# Rules Coverage Ledger

Status: **The 13 scoped Gate 2 families and two scoped Gate 3 families have mechanical and production integration evidence. The twelve Gate 4 families are actively implementing; 28 later-gate families remain intentionally deferred. No player acceptance is claimed.**

[The machine-readable ledger](rules-coverage-ledger.json) is the implementation/evidence record. [The source inventory](srd-5.2.1-inventory.json) records source chapter coverage, glossary membership and named catalogs. [Source provenance](srd-5.2.1-provenance.md) pins the official English SRD 5.2.1 and its commercial CC BY 4.0 terms; the [distribution notice](../../content/srd-5.2.1/NOTICE.md) must accompany source adaptations.

Current integration note, 2026-10-04: PR48 is accepted at main
`dbf1d633460473183324b4ec519e8d1980884b5c`, with separate successful literal-main
checks recorded in the [release plan](../exec-plans/active/gate4-encounter-release.md).
Older release-candidate statements below remain historical evidence. No Gate4
family status or player-acceptance claim changes.

## What is accounted for

The ledger has 55 rules/content families. Every family has one numeric `primary_gate`, legal/source citations, explicit scope, status, mechanical test evidence, production integration evidence, player acceptance evidence and final Gate 14 ownership. Empty evidence arrays mean no such evidence is claimed. An inventory entry inherits its owning family's scope and gate accounting; a primitive family's integration status does not declare every source subrule implemented. Listing a spell or creature does not implement it.

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
- **Gate 5:** full exploration/social/travel/rest/crafting/economy, advancement/multiclass, environmental/curse/poison/trap rules, noncombat magic and item use, including ritual execution and longer casting-time orchestration.
- **Gate 6:** complete legally reusable catalogs/options plus production content integration. Gates 4/5 own mechanical dependencies. Gate 6 must account for every catalog entry and its variants, not only content used by the starter adventure.
- **Gates 9/13/14:** autonomous interpretation/explanation, distribution notices/recovery, and human end-to-end acceptance remain cross-cutting. These do not absorb unfinished rules mechanics without an explicit roadmap change.

Primitives and complete families have separate rows where needed. For example, `condition-primitives` belongs to Gate 2 while `combat-conditions` owns complete tactical semantics in Gate 4. A typed marker alone is not full condition support. The same distinction applies to spellcasting vs complete spell effects/catalogs, timing vs complete encounters, and recovery primitives vs actual rest gameplay.

## Gate 2 evidence and limits

The 13 `production_integrated` rows cite exact test files and names at application head `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c` in [PR #18](https://github.com/idiotswill/DMd/pull/18). Mechanical tests check deterministic rule behavior; application scenarios use `CampaignRuntime`, actual SQLite state/audit/event commits, local content, restart, queries, and export/restore/replay. The checkpoint and execution plan retain the final exact-head verification and CI evidence; this ledger does not independently declare a gate accepted.

The evidence applies to each row's stated Gate 2 scope. Shared production paths are exercised alongside kernel-specific boundary tests; it does not claim that every rule variant has a separate application test. Character option grants, complete tactical condition consequences and simultaneous trigger ordering, full preparation/spell effects/geometry, complete catalogs and full rest/camp play retain their later-gate owners. The `Ritual` glossary member remains inventoried under `spell-primitives`, whose scope explicitly assigns ritual execution and longer casting-time orchestration to Gate 5. No ritual implementation is claimed. The shipped `dancing-lights` adaptation implements concentration/duration bookkeeping only, as reported by the supported-content query.

The source-license row proves source pinning, attribution retention and loading of the declared local pack. The release packaging/distribution audit remains Gate 13. All `player_acceptance` arrays remain empty: automated tests and intermediate gate integration do not replace the full-game and human acceptance required by Gate 14.

## Gate 3 evidence and limits

Only `play-rhythm` and `character-creation` advance, using exact application/rules tests
at foundation head `52ffab23c5e67db7d6ee40a622533b08d44f10fd` and the packaged scenario
in the [Gate 3 checkpoint](../checkpoints/gate-03-desktop-table-loop.md). Initial creation,
correction, dice, ambiguity/crash recovery and campaign isolation used `f4f1ad2`; the final
`05ff272d16ab7d893e0c39950f07bd5176c82e3e` package verified pending-roll completion,
readable labels, saved resources and session end/restart/continuation. All final artifact
checksums and corrected license notices were inspected.

Creation is the supported Human Fighter 1 Soldier/Skilled profile, not the complete catalog.
The text path uses trusted host-established check context and conservatively leaves unknown
intent unresolved. Full catalogs remain Gate 6, tactical execution Gate 4, noncombat and
progression Gate 5, and autonomous interpretation/DM behavior Gate 9. Kernel rest/Inspiration/
Savage Attacker primitives do not claim desktop controls or broader feature execution.
All `player_acceptance` arrays remain empty; this technical scenario is not a human playtest.

## Updating and validating

Use `planned`, `intentionally_deferred`, `implementing`, `implemented`, `mechanically_tested`, `production_integrated`, or `player_accepted`. The 13 Gate 2 and two Gate 3 rows record scoped production integration. The twelve Gate 4 rows use `implementing` to record the active gate, without claiming complete family coverage. The other 28 retain `intentionally_deferred`, their receiving gate and their existing obligations. Every deferred row names its receiving gate through `primary_gate` and explains scope. Optional toolbox rules are inventoried even when disabled by default; opting in must be explicit campaign configuration. Non-SRD content is outside this selected-source inventory and cannot enter by familiarity or through a claim of generic compatibility.

Gate 4's [active plan](../exec-plans/active/gate-4-tactical-encounters.md) records exact
source and bounded production evidence, including the verified initiative/turn path
merged in PR32 at 12ed29a. Current physical-encounter, reaction, area and privacy work
does not close any complete family. Full actions/masteries, grapple/mount/underwater
consequences, spell mechanisms, enemy behavior, improvisation, encounter finish and
packaged integrated acceptance remain required. Existing family scopes, ownership
are unchanged; no unfinished mechanic moves to a later gate. Bounded evidence in
the combat-actions and spell-effects rows now records the actual Shield-hit source
cases at74c2cabf8fd8af44cdc2a58fc4bb7a5c6e9acfcb: canonical723 GNU Rust tests,
including the owned Mage's file-SQLite/cold-retry scenario. Both families remain
`implementing`. PR45 is merged as a0b12d2 with all six source and separate main
checks passing. The same two rows now include source-qualified ffc3 evidence for
Magic Missile: real owned responses, uniform private acknowledgments, concentration
children, first aid/death, exact receipts and independent cold recovery pass on
Linux and native Windows. The ffc3 packaged desktop also survives full restart at
the selected owned Shield decision and completes all six physical faces and ordered
impacts. Its bounded plan records the artifact and read-only outcome audit. Final
integrated canonical/CI/review and missile merge/main proof remain pending;
Counterspell, Ready release and the other required mechanisms remain open.
This is technical evidence, not human acceptance.

The bounded [Air Elemental source admission](../exec-plans/active/gate4-air-elemental-source-admission.md)
adds a separate complete immutable SRD pages258–259 source and exact current
creation pins while retaining V1 definitions and historical picker bytes. Integrated
3f3e359 passes all six CI jobs (783Linux/785Windows,60table,all8originalFlow4).
Its verified Windows package completed actual source creation, atomic overlap
refusal, corrected placement, owned initiative with pending-request cold restart,
and five-foot flight with airborne cold restart in the existing campaign. Prior
resources, identities and histories survive. The plan qualifies labeled QA dice
inputs and the read-only capture scope. Accepted main `dbf1d63` is reconciled with
all 411 non-document files unchanged from `3f3e359`. Integrated `f932c73` now passes
canonical verification (782 GNU Rust tests, zero failures) and all six CI jobs
(783 Linux /785 MSVC); the plan records exact logs and independent review.
Final documentation-head checks/review, protected merge and literal main proof
remain outstanding. No evidence array or family status
advances here. Air Form special geometry, Multiattack and Whirlwind execution remain
Gate4 obligations; source data and explicit unavailable boundaries do not close
`monster-running`, `combat-actions` or Gate6 `monster-content`. The later Shove
slice still owes its genuine paid Prone-immunity continuation and recovery.

The bounded [Magic Resistance source prerequisite](../exec-plans/active/gate4-magic-resistance-source.md)
now records Linux source verification and the remaining Windows coverage repair.
The reviewed helper correction passes32 synthetic controls at1cea684; this is not
actual corrected Cargo coverage or positive MR gameplay. The historical authored
checkpoints below retain their original status; current evidence and exact failed
Windows attempts are qualified in the MR plan. No family or evidence array advances.

Historical MR source checkpoint: the bounded prerequisite
adds a separate selected Night Hag revision with its printed trait, exact same-ID
revision lookup and current-admission separation. Private save categories derive
from canonical spell work, retained source clauses or the audited nonmagical breath
and Concentration causes. Tests for source/admission/content integrity, synthetic
category composition and real Hold Person-Fiend/breath controls are authored only;
no compile, test, native or production evidence is claimed and no evidence array
advances. Real positive MR via Counterspell and a genuine repeated-save mechanism
remain Gate 4 requirements, as does old/new same-ID application coexistence after
the separately accepted encounter-release integration or an original pre-encounter
capture. Historical source payloads, picker bytes and accepted fixtures stay frozen.

Historical coexistence checkpoint: the locally integrated release/source candidate includes an independently
reviewed genuine old/new Hag coexistence child of the frozen Shield-Hit replay
suite. It completes original pending work, explicitly upgrades/releases, creates
the current revision and authors both revisions' later real spell execution and
cold persistence controls. The imported test is **UNCOMPILED/UNRUN**; its presence
does not close coexistence or provide positive Magic Resistance gameplay evidence.
Both development dependencies remain subject to separate acceptance.

The [encounter release plan](../exec-plans/active/gate4-encounter-release.md) now
owns the approved flow 5 completion/highwater and retained-scene design. Integrated
8c03f9f connects authenticated release/replacement/session handling, recovery and
desktop controls and passes canonical775 GNU Rust tests plus all six CI jobs
(776 Linux/778 native Windows). All59 table cases and eight unchanged original
flow4 continuations pass after genuine original-source capture/baseline completion.
The verified8c Windows package exercises two actual encounters, closed/active-session
Finish and cold Finished/pending-attack recovery with the same actors, paid resources,
items and absolute Mage Armor deadline. The plan distinguishes native evidence from
the separate real file-SQLite ammunition/drop-custody, replay and refusal families.
Final evidence head d4 passed independent review and all six checks, then PR48
merged with expected-head protection as dbf1d63, whose full tree equals d4.
Separate literal dbf main runtime checks remain pending; complete Gate4 families
remain implementing. These checkpoints add
no passing evidence to the machine-readable ledger and advances no family status.

When implementation advances, add exact test names/files, production scenarios/heads and player acceptance reports to the matching arrays. A feature may only advance to `mechanically_tested` with test evidence, to `production_integrated` with mechanical and real application evidence, and to `player_accepted` with all three. Gate acceptance remains governed by checkpoints; the ledger cannot waive it.

`cargo test --locked -p dmd-domain --test rules_coverage_ledger` runs offline in normal workspace CI. It checks the pin, complete chapter span, reviewed catalog counts, unique names/IDs, source-page bounds, every inventory-to-family relationship, gate assignment and evidence for advanced statuses. Negative cases demonstrate rejection of orphaned families/entries, duplicate ownership, removed catalog entries and unsupported completion claims. It checks consistency against the reviewed source inventory; it is not a claim that software can infer legal scope or prove faithful gameplay from JSON alone.
