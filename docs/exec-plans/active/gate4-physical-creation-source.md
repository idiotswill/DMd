# Gate 4 — Pinned physical character creation

## Current evidence — controlled native QA completed, 2026-10-07

PR #62's source-created weapon route was exercised in the packaged Windows
application on October 6–7. Root operated normal desktop controls; a separate
auditor read only copied JSON and recorded UI observations. The entered dice faces
were controlled automated QA values. They exercise the physical-roll input path,
but are not evidence of human physical throws or a human table playtest. Gate 4
acceptance remains pending. The historical dated reports below retain their
original scope; their earlier UNRUN/current-next-action statements are not the
current status of this native route.

### Source, package and retained evidence

The executed source was `48f7c92b63bd2f9c586d2622d1875342f004b093`, tree
`eee68a87b8fec25f86d78cf032ab48509b0172cf`. CI run `37423150396`, Windows job
`112136794242`, produced artifact `11406378413`. Its archive SHA256 is
`a550ef65de502f9cae77ff717e2b40c15962e98d04054e7ff095b0fa94a64530`.
The portable `DMd.exe` SHA256 is
`f20f13bc2beb0bdb95a5b534f7887f401dd5b1609ee8caf334f322dfb6ed68b9`.
The verified package provenance is
`tooling/physical48f-package-completed-provenance-2026-10-06.json`, SHA256
`9952cfe7f06f6fd757e2e9e65481800962db20f07cf0a9ad3cb65cd1770692e7`.

External paths here are relative to the retained workspace
`C:/Users/jadra/Documents/ChatGPT/DMD`, outside this Git checkout. The package is
under `artifacts/physical48f/package/portable/DMd.exe`; copied native evidence is
under `tooling/native-physical48f-oct06/`. These artifacts are not checked into Git.
The following names are relative to that evidence directory:

| Evidence artifact | SHA256 |
|---|---|
| `copied-capture-audit-final-through-seq39-v3-2026-10-07.json` | `c50cae365523e1f89ec46d8f4a6bd7bab7ff1d855856fc1a5b0453f2b0ee9080` |
| `independent-physical48f-final-seq39-receipt-2026-10-07.json` | `d7f66ce0feb2a6d9770765d2e7e2e1c5d9f97f4bf7b40f12ec9ab4ea782f1a81` |
| `independent-physical48f-through-seq32-receipt-2026-10-07.json` | `d76646e889eb9b66e4a1cdfa1ff2443c1f6d5850395705752e29ee3c91f27995` |
| `graze-seq13-15-copied-mechanical-audit-2026-10-07.json` | `64bbe3c9b08a6ae2d727772b19ce8f9089f6e3e71d04c58f5fdc3f99f4b9cf3e` |
| `glaive-seq24-27-28-copied-mechanical-audit-2026-10-07.json` | `17662b5155e092e908ec1be991fbc95ecbab9817dbd064f96c50cbe9586af2ea` |
| `equipment-resume-copied-audit-receipt-2026-10-07.json` | `d15ea8f3734212c6a0227dee5d3d920b2f138139dccfb6a488003a18af046c67` |
| `ui-observations-oct07.jsonl` (563 rows, 10,890,032 bytes) | `cf46043b6bce0c7af6ec547532e24709e28584e36d7bb2a155716f8c630031af` |
| `operator-log.md` (completed October 7 append) | `75a7010bd62f78b1f67d1de59cd9fca830ed209e37c39b9602744a1433391b64` |

The operator log was finalized after the copied audits; their earlier operator-log
snapshot hashes remain historical and are not substituted with the final hash.
The frozen JSONL ends at `2026-10-07T07:29:54.211Z`. The final audit's first external
mechanical script stopped before output on a `name` lookup; the saved campaign
field is `display_name`. The original was preserved and a separately saved v2
changed only that lookup. Its successful receipt identifies its script SHA256;
`tooling/independent-physical48f-final-audit-correction-2026-10-07.json` records the
exact inverse-byte comparison. No capture or earlier report was replaced.

The separately pinned Windows source-job receipt
`tooling/physical48f-package-ci-outcomes-2026-10-06.json`, SHA256
`a675124baa6a30ea53839e6ba524a6db10b2b5bb293f5ebbfd4490d5316ccef0`, records 812 Rust
passes in 50 nonempty harnesses and desktop 130/130 tests in 20 files, with
svelte-check reporting zero errors/warnings. These are results for source `48f`,
not fresh checks on this documentation commit. No compiler, Cargo, npm or tests
were run by the evidence writer.

### Acquired source and actual item identities

QA campaign `0cc2a691-7836-4d5e-99b2-b19fec25053e` was created through the desktop
with an explicit practice-combat table contract. Both source-created characters
retain ruleset `srd-5.2`, version `5.2.1`, catalog schema `1`, profile
`human-fighter-soldier-level-1-physical-v1`, fingerprint `e2d57011783b3fae`.
Both have Strength 17 and maximum HP 11. Their immutable creation events and
single materialization receipts bind these actual purchases and allocations:

| Character / actor | Purchases and remaining money | Actual weapon / leather armor ItemIds |
|---|---|---|
| Glaive: character `4910a586-68f0-4187-80a2-61d6086d32b4`, actor `d95538b9-8fe7-46d5-a181-c2c8e513c810` | Glaive 2000 cp + leather armor 1000 cp; 17500 cp remains | `6f88dff3-a173-447f-b53e-ce6c3a279b10` / `25c8e0a4-532e-4bd3-8c58-3acb67bcfad8` |
| Greatsword: character `7155ca12-2af2-442f-9c56-a72cb3eb776c`, actor `bf2deaa7-a4d4-4fa6-82e5-0d439a724db8` | Greatsword 5000 cp + leather armor 1000 cp; 14500 cp remains | `9c11209c-4c98-4cac-99b7-02d1e249c1fb` / `64411b07-2b41-4bb4-8421-19a75788b06e` |

Each quantity is one, with owner and custody equal to that character's actor.
Both retained the selected weapon mastery and worn purchased leather armor.
The receipts report exact creation commands/events, player controllers, source
profiles, purchases and allocations; display names alone are not source evidence.
The route did not mutate Items, source profiles, HP, payment or SQL to manufacture
eligibility. The original four Items and two materialization receipts remain at
sequence 39; actual attacks ready/retain the same weapons with TwoHands.

### Observed mechanics and persistence

- Greatsword's first Attack Action used BeforeAttack Equip. Controlled d20 face 1
  left a real Graze choice at sequence 14; accept at 15 applied only Strength +3,
  taking Glaive HP 11 to 8 without a damage roll. The other player's inspected
  view had no Graze controls and its copied state remained unchanged.
- Glaive equipped its purchased weapon on its own turn, entered d20 face 1 and
  declined Graze at sequence 21. Greatsword remained at 11 HP. Glaive retained
  the weapon after EndTurn. An earlier unarmed opportunity was explicitly declined;
  it is not counted as the later held-Glaive reaction.
- At sequence 23, Greatsword's voluntary move from x15 to x20 feet paused at x15
  before leaving the held Glaive's ten-foot reach; Glaive stayed at x5. The owner's
  accepted reaction at 24 had no equipment operation. D20 face 12 +5 hit AC14;
  genuine response ordering/refusal led to pending 1d10 at 27. Face 4 + Strength 3
  dealt 7 at 28: Greatsword HP 11 to 4, movement completed to x20, five feet spent,
  one Reaction and no additional Action/equipment allowance. Raw coordinates use
  half-feet: x30 to x40 and movement cost 10, not a ten-foot move.
- Greatsword returned ten feet west in the same turn and entered ordinary-miss
  face 2 +5 =7 against AC14. Graze choice at 31 was declined at 32: Glaive stayed
  at 8 HP, all seven accepted rolls were unchanged and no damage roll was issued.
- Both actors ended their turns. Round 3 Greatsword hit with face 12, then ordered
  responses and Glaive continued without Shield. Pending damage at 38 was the
  printed 2d6; faces [1,1] + Strength 3 dealt 5 at 39, taking Glaive HP 8 to 3.
  Greatsword stayed at 4 HP. Its one Action remained spent; no Savage Attacker
  use, extra equipment or duplicated raw result occurred. No raw request or
  attack resolution remained, while the encounter itself remained active.

The full audit compares 40 captures and 10 complete live/closed/reopened triples:
creation (sequence 0), characters-created (4), pending Greatsword attack (13),
Graze choice (14), Glaive opportunity (23), paid attack (24), pending damage (27),
movement completion (28), pending Greatsword damage (38) and completion (39).
Each triple compares complete typed campaign arrays, schema objects/descriptions
and all eleven migration rows. All 39 transitions between captured cuts retain
exact accepted-raw prefixes and prior historical typed rows. The final journal
spans sequences 1–39 and contains nine unique accepted raw-request IDs, without
replacement results or cancellations. This is copied-record validation, not a
fresh Rust semantic replay of these captures.

All three protected campaigns remain completely equal to their before-launch
logical rows: bridge `f5da4b10-50f8-47ef-9ad9-e0a921ad0bdc` at sequence 0,
Shove `a2858187-20b5-45da-9062-2a9c1d2d7b46` at 91 and Expiry
`e48574bc-4d06-4d5e-987e-66e7d67b2590` at 75. Root observed normal closes and
reopens using the same verified EXE. The final process chain after the Glaive
attack was PID2044 → 18752 → 7372 → 8812 → 5108; the completed operator log
records exact starts/windows and earlier processes. Root observed Glaive 3/11
after the final reopen, returned through the ordinary dropdown to Native bridge
Oct06 A (the protected bridge UUID above), observed Host setup, closed normally
and confirmed no DMd process/window remained. These process/UI observations are
separate from copied-data equality. The final bridge-closed capture equals all
four campaigns in the sequence-39 reopened cut; it is one closed cut, not an
eleventh triple.

### Limits and exact next action

The October 6 equipment-prepared reopened cut is missing. Its live/closed cuts
and the separate October 7 before/after-resume cuts agree at sequence 6; they
do not retroactively prove the missing triple or a forced crash. The native run
does not claim a critical, Savage Attacker use, every Graze variation, arbitrary
accepted retry, portable export/restore or relocation, complete privacy coverage,
reference-hardware performance, human physical dice or a human table playtest.
The broader automated cases retain their own source/check identities; this native
receipt does not replace their original-history/recovery/hostile-state acceptance.

This evidence advances normal source acquisition, owned physical weapon play and
exact combat suspension/resume. It does not complete the integrated Gate 4
encounter with physical player dice, concentration/ongoing effect, limited
visibility, blocked/difficult movement, environmental improvisation and an
opponent who flees/surrenders/negotiates. The product's human-play and endurance
requirements remain unchanged, as does the original acceptance matrix below.

Next: independently review this complete documentation delta and the cited frozen
evidence, then root reconciles all remaining PR #62 acceptance and obtains required
checks on the final evidence head before any protected merge. Verify literal merged
`main` afterwards. The evidence writer has not pushed, merged or run fresh canonical
checks. Gate 4 stays active; Gate 5 must not begin from this receipt.

## Native evidence documentation plan — 2026-10-07

Root allocated sole documentation writing on PR #62 to the copied-capture auditor.
A fresh fetch confirms local and remote `codex/gate4-physical-creation-source` at
`48f7c92b63bd2f9c586d2622d1875342f004b093`, tree
`eee68a87b8fec25f86d78cf032ab48509b0172cf`, with a clean working tree.

Record the completed controlled native QA route in this plan and the Gate 4
checkpoint, with a roadmap pointer. Bind the source/package, saved-capture and
operator evidence separately. Preserve every existing acceptance requirement and
historical result with its original attribution. The source-plan, Gate 4 production
integration and product-definition physical-player requirements remain unchanged;
entered QA faces do not establish human physical throws or a human playtest.

Scope is these three Markdown documents only. No production, test, content,
database or runtime changes are allocated. Verify exact artifact hashes, factual
claims, links and the complete diff; audit all other Git blobs unchanged. Freeze
the coherent documentation result for independent review. Root retains publication,
required checks on the final evidence head, protected merge and merged-main
verification. This work does not close Gate 4 or begin Gate 5.

## Receive accepted Shove main before renewed CI — 2026-10-06

Root fetched authoritative main5afc992e62bb967aceec69db53f19b6347b70855,
treefcac4fae77d31a83efc0667180c2ed40c8731b10. The published Physical correction
5d9f7eb961078e05d74cfe7eba4ebceb0a7164a9 has no fresh CI because its PR cannot
merge cleanly with accepted main. A merge-tree preview finds one textual
conflict: Physical's original/current character retry tests and main's Shove
retry test were inserted at the same describe-block opening.

Commit this receiving plan first, then normally merge the complete accepted
main. Preserve both complete test bodies as separate sibling tests, including
their original inputs, exact retained requests, invocation comparisons and
assertions. Keep all other source from the automatic three-way union. Review
the six automatically combined paths (table API/transport tests, restore,
protocol, runtime and table harness registration), and audit every unchanged
parent blob and both additions. Do not import any unaccepted development branch.

This integrates two already reviewed Gate4 paths without changing their product
scope, source pins, controller authority or historical acceptance. Root remains
sole writer. No local compiler/test/native work is allocated while Offstage
owns the heavy slot. Publish the reviewed receiving head for fresh CI; all prior
709/5d9 evidence retains its literal attribution, and required native/canonical
acceptance remains open.

## October 6 exact-head CI correction

Both canonical platforms on `709ec08f9f847fa7ae958d99c9591b142be69bec`
failed the unchanged `distributed_srd_manifest_verifies_kernel_source_and_license_bytes`
control. Linux job111766060948 and Windows job111767169942 report that the
actual manifest contains the approved `character-creation-physical-v1.json`,
while the expected exact file set still lists only the six older entries.
Root read the full test, manifest and regeneration script. The new asset is
already required by the approved design and installed-content validation.

Commit this correction plan first, then add only that literal path to the
test's expected BTreeSet. Keep every prior filename and the exact-set equality,
catalog integrity load and exact campaign ruleset resolution. No production,
content byte, checksum, old creation input or replay fixture changes are needed.
This updates the distribution contract for the approved additional asset;
it does not relax integrity checks or alter any old source definition.

Preserve the actual prior local result: formatting and strict affected-package
Clippy passed, and 133 tests in eleven complete harnesses passed, including all
three new table application cases. The following historical missile harness
was interrupted after seven named successes without a final eight-test summary;
five later harnesses were not reached. The cause is unknown and no process is
running on recovery. These results are not a completed focused186 or canonical
pass. Full original evidence remains outside Git. After correction, review the
complete exact delta, verify the distribution harness and remaining controls,
rerun required exact-head CI and retain the outstanding native acceptance.

## Authored coherent source handback — 2026-10-05

Implementation now follows the approved design below on this independent branch;
no Ground, Ogre, Grapple or Offstage changes were imported. Production changes
cover the immutable physical creation catalog and installed-byte validation,
strict full-pin domain/builder path, explicit outer/nested creation actions,
profile-derived equipment materialization, pre-creation anchor and exact nested
history checks, schema1–3 raw compatibility preflight, and current desktop creation.
The typed catalog fingerprint is `e2d57011783b3fae`; the original sixteen entries,
kernel/source/tactical/creature payloads and old input/actions remain unchanged.

Authored acceptance tests use actual table creation and purchases, real ItemIds,
controller-owned opaque transport, per-command file reopen and independent portable
restore. They cover both weapons' natural1 Graze, ordinary-miss Graze, decline, hit and
critical dice, held Glaive10ft opportunity with one Reaction/no equipment change,
pending movement continuation, exact accepted retries including the original
pre-session creator, original outer/nested/audit correspondence, rejected hostile
anchors/profiles/imports, schema1–3 future authority/null shadows, legacy schema3
absent/null controls, and full database row equality after rejected import/migration.
Desktop tests cover actual current catalog prices/purchases/masteries/full source
submission and uncertain exact retry, while retaining original creation retries.
Installed missing/undeclared/rehashed current source refusal is also authored.
No positive test edits a purchased Item, mastery, character profile, paid state,
HP, or SQL projection to manufacture acquisition or attack eligibility. Negative
import/migration inputs are explicitly cloned/poisoned only to prove refusal.

Validation so far: direct rustfmt parsed/formatted changed Rust files successfully
and `git diff --check` passed. Static audit
`tooling/physical-creation-authored-static-audit-2026-10-05.md` records the five
original installed payloads plus their manifest entries, all36 raw/test/capture
fixture paths and all42 pre-existing Rust test/helper blocks in modified test
roots preserved verbatim. Original CharacterCreationInput and both original
CreateCharacter variant bodies also remain verbatim. Its SHA256 is
`0bfe29a1c8913733d30d86cdc476fd001cc9e339a9296df6b773e2609e12db9c`.
All production acceptance tests remain UNCOMPILED/UNRUN.
No Cargo, compiler, npm, database, native application, CI, publication or merge was
executed by this source owner. Root retains the sole serial verification slot and
must independently review the complete source/test bodies before allocating runs.

Next action: root/peer full exact-head review, then root-owned focused Rust and
desktop checks, correction review and full verify-fast/verify/exact-head CI. Native
current creation→sheet purchases→prepare→Graze→Glaive opportunity and restart
evidence remains required; it is not waived by these authored tests. Later receiving
into Ground and other Gate4 work requires an explicit plan-first normal merge and
union review. Keep Gate4 open until its integrated acceptance is demonstrated.

## Approved implementation assignment — 2026-10-05

Root independently read the complete334-line plan and actual creation, inventory,
legacy registration and restore seams, rehashed the pinned PDF and visually checked
page91. Review `tooling/physical-creation-c05c18c-root-design-review-2026-10-05.md`
approves this complete design. Root explicitly transfers sole SOURCE WRITER to
`ground_next_oct5` on this branch from clean `c05c18c18d7c181eac88ad393b602c6e42073c7b`.
This status commit precedes implementation. Complete the coherent normal acquisition,
materialization, current UI, Graze/Glaive OA and file-SQLite/portable route below,
without routine checkpoint permissions. All old source/input/action/history
boundaries remain required. Return the entire source and new tests for independent
review. No Cargo/compiler/npm/database/native/publication is allocated here; direct
rustfmt is coordinated with root. All authored acceptance remains UNCOMPILED/UNRUN.
Ground remains separately frozen and must not be imported or edited.

Historical plan-only status and reviewed contract follow.
Status: PLAN ONLY, 2026-10-05. Root assigned `ground_next_oct5` sole plan writer
on `codex/gate4-physical-creation-source`, checkout `gate4-physical-creation-source`,
from authoritative main `32c0c682c4dbb235e1f9a119643c5d8626d5cb71`, tree
`c189a2fc618569b270fa8f87f7cb0577ff1fc32e`. A fresh fetch of origin/main still
resolves to that exact head. No PR exists. No implementation or runtime execution
is authorized until root independently reviews this complete design and assigns
source writing. Root owns every compiler, Cargo, npm, database, native and CI slot.
Ground `d43e079` remains a separate frozen root-owned candidate. Do not import it,
Ogre, Grapple, Offstage or other unaccepted branches implicitly.

## Objective and gate traceability

A normal user can choose genuine source-priced Greatsword and Glaive starting
purchases, select their Fighter masteries, create the character through the desktop,
materialize the actual purchased ItemIds once, and use real Graze and a held
longer-reach Glaive opportunity attack. Creation, equipment, physical rolls and
pending continuation must survive actual SQLite restart and independent portable
restore with original accepted-command retry. The completion boundary is this
entire production route; a catalog/registry-only checkpoint cannot complete it.

This advances product-definition sections What finished game means, Feature-
completion rule, First rules identity and commercial boundary, Character experience,
Complete tactical timing, Tactical visibility and exact suspension. Gate4 owns
source-faithful physical attacks, mastery, hand/reach rules and recovery. ADR023
owns original creation, ADR020/018 semantic original-history replay, ADR025 strict
state compatibility, and ADR026 interruptible tactical work. Add an ADR023 current-
source addendum during implementation without rewriting its original decisions.

Non-goals: broader classes/species/levels, ongoing shops/trade/loot/economy/rests,
new attack or Graze mechanics, NPC source revisions, Ground/Grapple activation,
voice/autonomy and any Gate5 work. These remain required in their existing gates.
This limited current catalog does not redefine final character-product coverage.

## Source evidence and immutable inputs

The existing official English PDF outside the repository was rehashed:
`research/srd-5.2.1/SRD_CC_v5.2.1.pdf`, 6,031,375 bytes, SHA256
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
This equals `content/srd-5.2.1/source.json`. The exact source is
<https://media.dndbeyond.com/compendium-images/srd/5.2/SRD_CC_v5.2.1.pdf>.
Printed/PDF page91 was rendered and visually inspected on2026-10-05, including
column headings and adjacent rows. Glaive is20GP =2000cp, quantity multiple1;
Greatsword is50GP =5000cp, multiple1. Both have Graze; Glaive is1d10 Slashing,
Heavy/Reach/Two-Handed, and Greatsword is2d6 Slashing, Heavy/Two-Handed. Both
weigh6lb. Existing tactical.json already retains their combat definitions.
Render evidence outside Git: `tooling/physical-creation-srd-91-2026-10-05.png`,
SHA256 `bad4d77dc8646c0d68bccbfeb4ecfbe41a932be6e64810e320963417b8f8a02a`.

Keep original `character-creation.json` (16 entries), `kernel.json`, `tactical.json`,
`source.json`, every earlier immutable creature asset, and all historical raw
exports/captures byte-exact. Keep `starter_catalog()`, original creation input,
original outer/nested action variants and their emitted absent-field bytes and
meanings. Do not weaken the existing16-entry registry assertion or any old history
control to make new acquisition pass. Source data remains CC BY4.0 with the original
attribution/license and recorded adaptation. Add the new asset to the existing
manifest regeneration script and installed-content validation; do not change old
payload manifest entries. NOTICE may add an accurate adaptation paragraph and
therefore gets its own new checksum.

## Concrete source and retained data model

Add immutable `content/srd-5.2.1/character-creation-physical-v1.json`, parsed as the
existing strict `StarterCatalog` shape. Its `schema_version` is1, `ruleset_id`
`srd-5.2`, `version` `5.2.1`, and `profile_id`
`human-fighter-soldier-level-1-physical-v1`. Retain starting_money_cp20500, fixed
Human/Fighter1/Soldier/Skilled choices, source-page list and all16 old rows unchanged
in original order; append Glaive then Greatsword with the prices/page91 above.
The scope identifies the two additive physical purchases and continuing catalog
limits. No consumer infers a source from display names or possession.

Add strict domain `CharacterCreationSourcePin` with required fields:
- `ruleset_id: String` = `srd-5.2`;
- `ruleset_version: String` = `5.2.1`;
- `catalog_schema_version: u32` =1;
- `profile_id: String` = `human-fighter-soldier-level-1-physical-v1`;
- `definition_fingerprint: String`, exactly16 lowercase hex characters.

Fingerprint the complete typed catalog using compact `serde_json::to_vec` and
FNV1a64, following the existing creature-definition fingerprint convention. The
registry checks every field against the one immutable current definition; format
validation alone is insufficient. Future changes require another immutable profile
identity/revision; neither ID-only lookup nor a matching price admits an unknown pin.
This fingerprint identifies local semantic content, not cryptographic publisher or
save authenticity. Distribution bytes are independently bound by the manifest.

Append `creation_source: Option<CharacterCreationSourcePin>` to CharacterProfile,
with default None and omission on serialization. None (including explicit null in
a legacy profile) means only the frozen original derivation; a V2 purchase cannot
be justified with None. Some requires the exact registry pin and current derivation.
Do not put it on `CharacterCreationInput`: every existing input and old control
body remains unchanged. The new profile carries immutable source selection but
not standalone accepted-creation authority; original-history validation supplies
that authority. No mutable trusted flag or caller-provided derived sheet exists.

Public pure queries/builders: keep `starter_catalog()` and `build_character()`
unchanged in meaning; add `physical_starter_catalog()`,
`current_character_creation_source()`, and `build_character_from_source(input,
entity_id, source, pack)`. The latter requires a nonoptional exact pin. Reconstruct
retained profiles by explicit absence/current-pin dispatch; validate the entire
resulting profile and intrinsic mechanical sheet as today. An unknown pin cannot
fall back to legacy. Factor shared derivation only if complete old result equality
is retained and independently tested.

## Legacy kernel registration boundary

The observed `build_character` registers every purchased old catalog weapon in
`MechanicalEntity.attacks/attack_proficiencies` and validates it through
`pack.attack`. Immutable kernel.json contains only Club, Dagger and Shortbow.
The tactical physical planner instead derives actual Item custody, weapon source,
proficiency and mastery from current inventory and CharacterProfile. These are
separate existing responsibilities (`validate_character_intrinsics` documents it).

For the explicit current creation revision, keep legacy kernel registrations equal
to the purchased intersection with the frozen three original weapon IDs. Validate
those registered definitions normally through the exact loaded pack. Greatsword
and Glaive remain actual purchased/profile equipment and use tactical physical
planning; they are not registered as incomplete generic kernel attacks. Use the
fixed original catalog weapon set, not a permissive `pack.attack(id).is_ok()`
filter whose result could vary with arbitrary pack contents. Old creation output
stays identical. A current character buying only Greatsword may have no legacy
kernel attack registrations; this is valid while its physical Item works through
the actual tactical route. No extra damage/ability/proficiency data is supplied
by input and no alternate kernel pack is constructed.

Retain inherited refusal of generic kernel attacks when tactical inventory exists,
and the application refusal of execute_rules for table campaigns. Add controls
showing physical Greatsword/Glaive options work after materialization while generic
kernel permission/attack requests for those IDs remain unsupported without writes.
Do not claim generic kernel feature expansion or silently register simplified
versions that omit physical/mastery rules. This is an explicit bounded producer
for the already complete tactical weapon authority, not a separate combat engine.

## Commands, validation and original creation provenance

Add `TableAction::CreateCharacterFromSource { character_id, entity_id, player_id,
source: CharacterCreationSourcePin, input: CharacterCreationInput }` and exact
nested `RulesAction::CreateCharacterFromSource { entity_id, source, input }`.
Both source fields are mandatory nonoptional typed structs. Missing, null,
duplicate, unknown member, unknown revision and altered fingerprint fail closed;
no current pin is filled in on behalf of the caller. Original CreateCharacter
continues to reject the new purchases under both live and historical replay.

The outer producer uses the same host/setup-only boundary and current campaign,
player, unique non-nil character/entity identity constraints; derive the whole
built profile before modifying the cloned candidate. Apply exactly the matching
nested action at the same original CommandMeta, then retain the derived profile.
No existing mechanical entity/profile may be replaced, and other characters,
rolls, resources, events and currency stay unchanged. The nested pure resolver
validates source, entity and original issuer/sequence as the old creator does.
Application production persists this new nested action only inside its matching
outer table event; ordinary execute_rules remains refused for a table campaign.
Pure builder/resolver tests are mechanism evidence, not accepted table history.

Keep existing atomic BEGIN IMMEDIATE state/event/audit/presentation commit and
saved accepted-response lookup before current session/revision/ownership checks.
A retry uses the exact original outer action, pin, generated IDs and envelope.
A different source/body under an accepted CommandId refuses without replacement.
No second source ledger or SQL schema is needed.

`rules_restore` must recognize the new outer action as requiring a nested rules
event; match entity, input and complete source exactly, with the same envelope,
issuer/actor/session/campaign/sequence and accepted audit correspondence as old
creation. Old/new outer and nested variants cannot substitute for each other.
Reject standalone new rules-creation journal events claiming table creation and
relabelled opaque events. Replay the original accepted command through the exact
builder, comparing every original event/outcome, later snapshot and current image.

New authority requires its original pre-creation anchor: extend existing anchor
checks to reject any earliest snapshot already containing a non-null
creation_source in a table profile (including current-source profiles nested in
any receipt, which tactical-anchor rules already prohibit). Do not grandfather
current creation under the legacy pre-journal anchor exception. Validate every
current profile was introduced by its original accepted new table action through
replay, even when no inventory has been prepared yet. A valid source pin or a
matching forged current/latest-snapshot pair cannot replace missing creation
history. Later original events cannot change immutable source/purchases/masteries.

## State/schema compatibility and physical materialization

Remain within current campaign schema4 and existing inventory schema1, with the
new explicitly omitted profile extension and explicitly named action semantics.
Portable envelope and old event version constants remain unchanged; older builds
already reject unknown new action/profile fields, while this build preserves old
variants exactly. No migration inserts a source pin or changes old snapshots.

Schemas1,2,3 must reject any non-null future creation_source, including nested
profile maps, before a Value transform could discard/overwrite it. Extend
persistence's typed legacy preflight and `reject_legacy_encounter`, plus domain
shape checks for non-null current creation at a pre4 state. Probe nested map values
without allowing a duplicate character key or duplicate field with a null shadow
to conceal source authority. Preserve the prior absent/null legacy behavior:
missing/null creation_source is not authority; duplicate source fields, malformed
non-null types, null/absent required pin members, unknown pin members and duplicate
pin members are invalid. Do not serialize away a non-null future source during
upgrade. Tests cover both key orders and every codec/portable/open migration path,
with complete transaction rollback and all original old-schema tests unchanged.
Current schema4 accepts only known source semantics after rules validation.

`starting_equipment_plan` derives its source.profile_id from the actual validated
profile revision, not always starter_catalog(). Its existing full retained
creation_profile includes the complete new pin; no weaker ID-only receipt check
is added. Materialization uses the existing allocations/receipts, one ItemId per
weapon, real quantity1/custody/ownership/loadout, supplied unique bounded IDs and
one grant only. Grant provenance remains the original PrepareEquipment command,
separate from the earlier CreateCharacterFromSource command authenticated by replay.
No new arbitrary inventory acquisition/grant action is necessary. Repeated, spent,
dropped, transferred, missing or destroyed gear must never reopen provisioning.
Old source pins, receipts and materialization bytes remain identical.

## Current desktop and privacy

`character_creation_options` returns the current immutable catalog plus a required
`source: CharacterCreationSourcePin` field in its options DTO. This is a current
query, not retroactive presentation history. Preserve the old pure catalog API.
CharacterForm keeps familiar prices/remaining gold/mastery selectors; TableApp's
submit callback uses CreateCharacterFromSource and that exact received pin. Keep
the old action in typed transport validation/labels/retry decoding for saved
requests; add the new variant deliberately. Technical fingerprints are not normal
form controls. Pending/uncertain submission keeps the exact source/IDs/request and
locks resubmission as today. Profile TS adds the optional omitted creation_source;
old sheet views and projection bytes stay absent and unchanged.

Use current public equipment preparation, AttackForm and OpportunityForm; they
already support generic physical weapon definitions and explicit grip. Verify
both new weapons appear only after actual materialization and Glaive TwoHands is
required. Money and private character/source information stay in the same owned
sheet boundaries. No new catalog capability means possession or grants. All old
history bindings and player/Host projections must continue exact replay.

## Coherent implementation and acceptance slices

1. After independent plan review and explicit source allocation, add the immutable
   current catalog/pin, strict profile derivation, legacy registration split,
   current outer/nested command and schema/provenance boundaries together. Keep
   old APIs/actions/capture bodies. Add meaningful negative/old-output controls.
2. Complete actual application materialization, current desktop submission,
   transport retry and original-history restore. Do not leave a public catalog
   pointing to unimplemented creation or a new pin admitted without replay.
3. Prove actual file-SQLite play below, review the entire exact source, then root
   allocates focused/frontend/canonical/CI/native execution. Intermediate commits
   support review but do not establish a separate catalog-only finish line.
4. Deliberate normal receiving into Ground/Grapple later, after exact dependency
   review/verification and a plan on that branch. This branch's cases use main's
   existing physical route and do not need Ground/Ogre or their optional fields.

Required real scenarios use normal Host table creation with the new exact pin,
current mastery selection, accepted PrepareEquipment, actual map/initiative and
physical faces. They must never insert an Item/profile/mastery/source/HP/payment
or SQL row directly. Two ordinary created PCs suffice, with explicit table consent
for practice combat; no incoming NPC source dependency is needed.

- New creation buys Greatsword and Glaive at5000/2000cp, preserving all16 old prices
  and20500cp initial budget; actual money is input-derived. Reject over-budget,
  wrong multiples/duplicate entries, incorrect/unknown pins and reused/nil IDs.
  Wrong price/quantity/mastery in retained profiles or receipts is a negative.
- Individually materialize both Items; demonstrate exact identities, custody and
  source receipts and refusal of a second grant. New-only-weapon creation works
  with empty legacy attack registrations and a real physical tactical option.
- Actual Greatsword and Glaive Graze on rolled misses (including natural1), both
  accept and Decline. Preserve the physical attack raw, source mastery and one
  Attack Action; accepting applies only the ability modifier, emits no damage
  dice request and cannot become a hit/critical or accept caller damage totals.
- Actual hit/critical weapon damage uses printed2d6 Greatsword or1d10 Glaive and
  exactly doubled critical dice, through ordinary physical requests/choices.
- A held Glaive OA triggers on genuine voluntary exit from10-foot reach, starting
  outside an ordinary5-foot weapon's reach. Own-turn accepted BeforeAttack Equip
  readies it; a later actual crossing presents the owned OA. Require TwoHands,
  one Reaction, no additional Action or equipment/pickup allowance. Finish physical
  dice and any real mastery consequence, then resume the saved movement. Reject
  wrong Item/one-hand/foreign controller/stale request with complete store equality.
- Cold-close/reopen the actual file and independently portable-restore before new
  creation/materialization retry, pending AttackRoll, Graze decision, DamageRoll,
  OA offer/paid roll and resumed movement. Execute the same original request on
  both routes; compare outcomes and whole authoritative state, then exact saved
  accepted response on retries and unchanged persistent export on changed body.
- Hostile source/current+matching-latest snapshot/profile/receipt/audit/outer versus
  nested event/envelope mutations and missing pre-creation anchors fail before
  writes. Snapshot every destination table/row, including an existing independent
  genuine campaign, and compare after refusal; count-only assertions are insufficient.
- Compare old creation results/serialized profile omission, all16 catalog entries,
  original action and nested-event JSON, and immutable received raw/capture hashes.
  Append new suites rather than rewrite old fixtures or weakening old assertions.
- Widget/API tests cover both current purchases, exact pin in submitted action,
  validation, uncertain exact retry and unchanged legacy retry decoding. Native
  packaged play uses ordinary creation/preparation, Graze and Glaive OA, physical
  dice and genuine save/exit/resume with source package identity recorded.

## Planned paths and verification

Domain: `character_creation.rs`, source shape/schema validation.
Rules: `character_creation.rs` plus a focused current-source module, `kernel.rs`,
`kernel/engine.rs`, retained profile validation and `tactical_inventory.rs`.
Persistence: `snapshot_replay.rs` typed future-authority probes and every existing
open/upgrade/portable caller (inspect for complete coverage before source).
App: `table_protocol.rs`, `table_engine.rs`, `rules_restore.rs`, `table_runtime.rs`,
source equipment projection only if required; preserve transport commit order.
Desktop: `table-api.ts`, `TableApp.svelte`, `CharacterForm.svelte` and relevant
creation/transport/widget tests. Content/package: new source, NOTICE, manifest,
`scripts/update-rules-manifest.py`, installed distribution checks. Docs: ADR023
addendum, this plan and existing scope/coverage references where genuinely advanced.

Root schedules `cargo fmt --check`, strict affected domain/rules/persistence/app
all-target Clippy, original character_creation/tactical_inventory/tactical attack
and opportunity controls, new source/schema/real table-loop cases, original five
legacy replay harnesses and protected content/raw audits. Exact counts/names must
be captured from actual execution, not inferred here. Then `scripts/verify-fast`,
canonical `scripts/verify`, desktop npm check/test/build and all required exact-head
Linux/Windows/MSRV/guard CI and native package acceptance. Fresh-head results must
follow every correction; earlier source-only or other-branch results do not transfer.
No relaxed stacks/profiles/test threads, disabled guards or reduced assertions.

## Risks, current evidence and exact next action

Risks: accidentally widening old creation/kernel behavior, treating a pin as
history authority, source smuggling through schema3/duplicate JSON, current-only
catalog lookup during old receipt validation, old presentation byte drift, and
UI retries that regenerate a current source or Item ID. The explicit boundaries
above address each; independent design/source review and runtime evidence must
confirm them. No legal/source-price uncertainty remains for these two rows; the
new asset's actual fingerprint will be computed only when it exists and reviewed,
never invented in this plan.

Completed here: root/authority documents and relevant current consumers read;
clean authoritative main freshly fetched; original PDF pin and complete page91
visually verified; concrete proposed data/actions/schema/replay/legacy boundaries
recorded. Only this plan is authored. No source asset, code, test, package manifest,
Cargo/npm/compiler, database or native application changed or ran.

Exact next action: commit this plan and return its clean SHA/tree for root's
independent full design review. Remain plan-only until root assigns source work.
Then complete the whole normal acquisition-to-play route and fix concrete findings
without changing acceptance. Gate4 stays open; no gate completion or Gate5 start.
