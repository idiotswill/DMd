# Gate 4 corrected Night Hag source and Magic Resistance prerequisite

Status: `486ce8b` passed Linux CI, including old/current Hag coexistence, but both
original native Windows attempts reached the six-hour limit. The reviewed CI
partition was published at `ad3b82b`. Its first actual isolated target reported
1 passed/61 filtered in 4116.01s, but the coverage guard correctly rejected six
missing doctest harnesses and produced no successful manifest. Root reviewed the
argument-placement correction plan; the bounded argv fix passed root and independent
review and all32 synthetic controls passed on1cea684, preserving the full default
workspace inventory and proof. No corrected Cargo command has run. Canonical/native evidence, accepted dependencies,
fresh complete partition/package verification and prerequisite acceptance remain
incomplete. The [Windows CI recovery plan](gate4-windows-ci-recovery.md) records
exact old/new results, failure evidence, proposed argv and unchanged obligations.
Branch: `codex/gate4-magic-resistance-source`; draft PR53 at
`ad3b82b7ba5741381ec6314b80c591f0d2f6251a`, freshly fetched on 2026-10-04;
tree `f6ed6762420e92332435eeb6843529a981dd0cd6`. The checkout is named
`gate4-shield-missile-runtime`.
Writer: root owns the reviewed source and verification/publication checkpoint
after gate4_ci_oct4 returned clean1cea684 and its handback. Root reviewed a1a9584 before
implementation; renewed writer/status commit 5b15083 precedes all source edits.
Production helper change is only the isolated argv placement and explanation.
All original 29 controls remain (28 unchanged, one strengthened), with three new
failure controls. Parser/schema/workflow and gameplay/source/test/fixture bytes
remain unchanged. Root read the complete correction and independent review, then
ran all32 pure controls successfully; the adjacent CI recovery plan records the
exact log and qualifications. Root owns execution, publication, dependency
reconciliation and acceptance. The prior29 passes did not verify this correction.
Original development parent: exact Air source checkpoint `1a0a5e75bfbde98c7621c421fbdc9d5b4750ab92`, inspected clean in the reused `gate4-shield-missile-runtime` checkout after root fetched it. Later reconciliation through3f is recorded below; the next accepted-main union is planned in the final section. Air remains an **unaccepted development dependency**. Root owns its separate Air branch/PR and all integration, publication and heavy verification.

## Objective and authority

Admit a new immutable Night Hag source revision that includes its actual Magic Resistance trait, and derive saving-throw circumstances from authenticated source causes. Preserve all old creature/spell identities, request modes, accepted envelopes, receipts and replay. This is the bounded source prerequisite already approved in [Counterspell source preflight](gate4-counterspell-source-preflight.md), using [ADR029](../../architecture/029-immutable-creature-source-admission.md)'s Air registry foundation. Magic Missile's merged prerequisite is satisfied; Air and all later integration remain separately subject to acceptance.

This advances [Gate 4](../../checkpoints/gate-04-tactical-encounters.md)'s source-faithful creature execution, saving throws, conditions, controlled-actor authority and exact suspension. The [product definition](../../product-definition.md)'s rules hierarchy, physical dice, player control, privacy, explainable provenance, local play and production-path completion remain binding. [ADR026](../../architecture/026-interruptible-encounter-resolution.md) and [ADR028](../../architecture/028-reaction-execution-and-work-ownership.md) retain their source, replay and owned-work boundaries. No requirement moves to Gate 5.

Counterspell's actual pre-commit response window, nested casts, interrupted Shield outcomes and direct Mage-to-Hag save remain a later Gate 4 runtime slice after the encounter-release integration. This prerequisite does not alter tactical execution versions or grant an unavailable player spell.

## Source evidence and immutable baseline

On 2026-09-30 the external `research/srd-5.2.1/SRD_CC_v5.2.1.pdf` was rehashed with SHA-256 `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`, matching installed `source.json`; the existing text extraction was read at the relevant clauses:

- SRD p311: Night Hag is a Medium Fiend, Constitution save +3. Magic Resistance grants advantage on saves against spells and other magical effects. Its at-will level-4 Magic Missile waives Material only, preserving Verbal/Somatic. The revision retains all printed statistics already represented and only removes `magic-resistance` from the selected adaptation's omitted-feature list. It is a complete stored **definition revision**, not a claim to implement the complete stat block; Coven Magic, Soul Bag, other spells/actions and other omitted features remain explicit.
- SRD p185, Magical Effect: created by a spell, magic item, or a phenomenon a rule labels magical. Elemental damage, a monster source, a reason string or a prior magical damage instance does not alone establish the category.
- SRD p120: Counterspell causes a Constitution save; failure wastes the casting action and prevents the spell's effect without spending its spell slot. These runtime semantics are recorded here for the later consumer, not implemented by this plan.
- SRD p305: Mage Protective Magic shares three daily uses across Shield and Counterspell, with Intelligence DC14. Do not modify that immutable grant or create separate per-spell pools.

Rechecked frozen files at the development parent:

| File | SHA-256 |
| --- | --- |
| `content/srd-5.2.1/tactical.json` | `5cb3e2e6965489c4c8e4f2f0ca643e0e3634ee84e2aed455260f1589fd330548` |
| `crates/dmd-app/src/table_creature_catalog_v1.json` | `adca5f6c4e7d5f316f9a8958c93e35a411493d201fac95f6938734e385e63dac` |
| `content/srd-5.2.1/air-elemental-v1.json` | `3693168e47498b03234e33b81c44bab2077cccd69f854852e12abf361f1fa772` |

None of these files is rewritten by this slice. Historical exports, old source producer checkouts and root's original-flow capture/baseline files remain untouched.

## Bounded implementation contract

### Immutable source and explicit admission

1. Add a separate complete immutable corrected Night Hag definition, provisionally `night-hag-v2.json`, with the same logical definition ID and a distinct full fingerprint. Never patch legacy `tactical.json`, derive an old revision by subtracting traits, or silently upgrade an existing profile.
2. Extend the small immutable registry to resolve both Hag revisions by full ruleset ID/version/definition ID/fingerprint. Keep `creature_definition(id)` and profileless/missing-pin historical fallback frozen to V1. Air's registry already resolves complete spell/creature/feature tuples and checks actor-to-program revision equality; audit and test those paths with **two revisions of the same ID**, not merely Air's distinct ID.
3. Separate current admission from all immutable revisions. `current_creature_sources()` currently returns every immutable source because Air was additive; change it to admit exactly the corrected Hag for new Hag creation while retaining unchanged current entries and Air. Old Hag remains resolvable and playable under its old pin, but is not a new live admission. Reject missing, legacy-Hag and forged source pins before creation mutation. An accepted old retry must return its saved response before current admission checks.
4. Preserve optional creation-pin serialization and normalization. A genuinely absent pin remains absent and means V1 permanently; `derive_intent` cannot insert the current revision. The frozen V1 picker/digests stay unchanged. Current source options carry only the admitted revision and accurate retained omissions.
5. Verify the separately declared installed revision's length/checksum and byte equality independently of compiled definitions. Missing, undeclared or modified/rehashed content fails with a recoverable content error and no campaign mutation. Follow ADR029's current-package requirement; this is not a DB/profile migration. Update the attribution notice and manifest without editing legacy source payloads.

### Magic Resistance and private source classification

Add typed `MonsterTrait::MagicResistance` and derive its presence only from the saving actor's exact admitted source revision. Do not copy the trait into a second mutable actor authority, infer it from actor/creature names, or add a client-controlled magical flag.

Use a private, non-Serde saving-cause classification built by authoritative producers and checked against retained source work. Enumerate all existing save-request routes before changing the helper. Classification and the trait feed the existing `save_conditions` circumstances, preserving automatic outcomes, advantage/disadvantage cancellation, ordinary modifiers, physical faces, Inspiration, voluntary failure and supported failed-save decisions. Reconstructed pending requests must use the same derivation as initial issue. Old source pins have no newly added trait and retain their modes, including later saves issued for unchanged old profiles.

| Producer/cause | Required classification/proof |
| --- | --- |
| Direct `SpellProgram` save | Spell, from the exact accepted canonical program and actor-bound spell tuple; no generic role/reason-string proof. |
| Repeated lifecycle save | Spell only after the retained effect/ticket and complete relevant trigger/condition clause agree with the canonical frozen source and its accepted source cause. Preserve the retained DC. Do not require a completed cast record to remain in the current resolution, or add durable category fields to the effect. Semantic history authenticates original installation. |
| Current creature breath-area save | Nonmagical for the currently admitted source breath clauses after source inspection; use the canonical private area program/full pin, not damage type or a blanket assumption that every future `SaveArea` is nonmagical. |
| Concentration after any damage | Concentration, not automatically a save against the spell that caused damage. No Magic Resistance from damage provenance alone. |
| Death/recovery rolls, landing checks and other non-save rolls | Keep their existing separate semantics; no Magic Resistance injection. |
| Physical mastery/unarmed saves, including Shove after integration | Nonmagical unless an actual source rule establishes otherwise. Reconcile independently developed Shove on root's integrated head; do not edit its branch. |
| Future magic-item or explicitly magical non-spell producer | Requires its own authenticated typed source classification and admission proof. An unknown/new source is not guessed from text or silently treated as an existing admitted category. No new such gameplay producer is added here. |

The repeated-save proof is a required implementation audit, not permission to accept a retained `definition_id` alone. Current `EffectSource` retains the source definition, actor, real command and ordinal; canonical effect shape and original full semantic replay must agree. A copied known source name or accepted unrelated command cannot manufacture a spell save. Preserve the pure-mechanism versus authenticated-application boundary explicitly.

### Exclusions and disclosure

No Counterspell executor/window, Ready release, new flow version, corrected-profile upgrade API, new damage/range/slot semantics, general content framework, expanded Hag spell list, magical-item executor, Air changes, Shove implementation, release implementation or witnessed-memory implementation is included. Do not use source completeness to imply executor completeness. Host catalog copy may explain supported/omitted capability; opponent-facing errors/DTOs must not reveal hidden traits or source identity. The private classification does not appear in transport input, public roll reasons, receipts or player-visible global counters.

## Planned implementation slices

1. After root reviews this plan-only commit, extend the checked-in architecture decision for the bounded same-ID revision/Magic Resistance contract. Add and inspect corrected source, registry admission split and installed-content validation; preserve old bytes.
2. Audit source resolution at profiles, equipment/weapons, policy, save/initiative modifiers, opportunities, component waivers, source grants/usage, canonical spell compilation, retained casting and actor/spell equality. Add meaningful same-ID old/new and cross-revision rejection controls.
3. Implement the private save-category derivation at all genuine producers and pending/replay validators. Audit repeated-effect authentication and concentration exclusions; add narrowly qualified mechanism tests.
4. Add real application admission/coexistence, negative gameplay, persistence/reopen/retry/restore and hostile-history controls. Update current source disclosure, distribution evidence and Rules Coverage Ledger while preserving the explicit positive-evidence limits below.
5. Independent full-diff review; root-owned focused/canonical verification and integration with accepted dependencies; exact-head source/persistence/native evidence before protected merge. Any actual failure is recorded and repaired without weakening a fixture or proof obligation.

## Acceptance and evidence limits

Required for this prerequisite:

- Frozen legacy source/picker hashes and every old creature/spell fingerprint retain their meaning. Both Hag profiles coexist; same-ID wrong-revision creature/spell tuples reject. The new source has Constitution +3, the correct typed Magic Resistance, original missile level/components/statistics and accurate remaining omissions.
- Genuine current Host source selection creates the new exact pin through the real table command. Missing/old/forged live pins fail atomically. Old accepted missing-pin and explicit-old-pin requests replay/retry unchanged without default insertion; existing profile history is never rewritten.
- New and old actors survive file-backed close/reopen, export/restore and independent semantic continuation. Verify actual accepted metadata, journal/audit/projection/binding consistency and normalized full no-write comparisons for rejected current/historical edits.
- Mechanism tests distinguish direct spell, source-matched repeated spell, nonmagical source effect and concentration; combine advantage with disadvantage and automatic-save conditions; reject forged source clauses/causes. Synthetic mechanisms are labeled as such.
- Actual admitted nonmagical breath against corrected Hag does not acquire Magic Resistance advantage. Actual Cultist Hold Person against the Fiend is the existing paid no-effect/type-mismatch control and creates no save, condition or fake advantage proof. A requested shape that the real source cannot target is never altered to manufacture a positive test.
- Installed source absence, omission from manifest and altered/rehashed bytes reject before writes even when compiled registry data is already cached. Distribution and source notice remain correct on Windows and Linux.
- Existing private source-control ownership, raw dice, exact accepted retry and historical presentation bytes remain unchanged. No private category or other observer's source information enters player DTOs/errors.

**Not accepted by this prerequisite:** a genuine positive Magic Resistance save through current gameplay. The actual supported Hold Person source targets Humanoids and cannot produce that positive against the Fiend Hag. The later Counterspell slice must prove a real Mage-to-new-Hag Constitution save with Magic Resistance; a genuine supported repeated-save Magic Resistance mechanism remains required later in Gate 4. Neither is waived, claimed from pure tests, or moved to Gate 5. Counterspell runtime still requires the original-flow capture proof and verified encounter-release integration.

## Verification plan and work restrictions

Root's original-source capture/baseline owns the exclusive local heavy slot. Until root reallocates it, this writer runs **no Cargo/npm/node/build/test, DB or UI operation**, and performs no push or merge. Static inspection, direct rustfmt where appropriate and `git diff --check` are permitted. No executable verification is implied by a parse/format check or authored test.

When root releases verification, the expected focused set is source definitions/profiles/scheduling, spell source reconstruction, tactical conditions/continuations/areas/concentration, application installed-content integrity and file-backed table-loop admission/control tests. Retain existing `legacy_tactical_replay`, `legacy_reactions_v1_replay`, `legacy_shield_hit_v1_replay`, original missile/ordinary-spell captures and accepted-envelope retry controls. Reconcile exact discovered test names on the implemented head; do not invent a passing command from this plan.

Run repository `./scripts/verify-fast` during authorized iteration and `./scripts/verify` before completion, including formatting, locked workspace check/clippy/tests and genericity/boundary guards. Run relevant frontend checks if DTO/UI code changes and installed-content packaging/native checks for the final integrated artifact. Required CI, exact-head independent review, old-history continuation and real application evidence remain root-owned acceptance gates. Air's parent evidence cannot substitute for this final head, and a source-only CI pass cannot complete Gate 4.

## Current findings, unresolved work and next action

- No new owner/product decision is required for the bounded prerequisite: the preflight already approved immutable correction/admission and private magical-save derivation. This plan chooses a separate full corrected Hag revision while preserving all other omitted features.
- Actual parent code inspection confirms full-pin creature lookup, same-tuple spell resolution, frozen V1 fallback, optional missing-pin replay and current installed Air integrity. These are inherited implementation observations, not acceptance of Air PR50.
- At plan review the admission helper returned all immutable entries; the authored implementation below now separates current admission before adding the second Hag revision. Verification must still prove only the corrected revision can be newly admitted.
- Source classification must be exhaustive over actual producers. Repeated effects can outlive their original casting resolution; classification cannot require that live cast or trust only the effect's source-name string. Complete canonical-clause/replay proof remains an implementation obligation to inspect before claiming this lane finished.
- Source glossary p185 resolves the magical/nonmagical criterion. No source ambiguity currently requires an owner ruling. If a newly discovered clause cannot be represented faithfully without expanding this bounded slice, record the exact source/proof issue and return it to root before coding that expansion.
- Local verification constraints and Air/release/Shove integration are scheduling/acceptance dependencies, not permission to weaken source or compatibility checks.

The coordinating root reviewed and approved plan `bfb9e9b516fcb7d064ff1e22bb51a2160d2064e0` and authorized the bounded implementation. The original plan-only status remains recorded by that commit; the following checkpoint supersedes its next action without claiming executable verification.

## Authored source checkpoint, 2026-09-30

- Added a complete separate `night-hag-v2.json`, typed `MagicResistance`, seven-file installed manifest and attribution. Canonical source comparison is explicitly a test: only the trait and corresponding omission change. Production does not reconstruct one revision from the other. Current admission selects one corrected Hag; full-pin and full spell-tuple lookup retain both. Current Host capability copy lists Magic Resistance, while the historical picker stays byte-identical.
- Installed Hag bytes are checked independently of compiled registry caching. The existing missing/undeclared/rehashed-source no-write helper now exercises both Air and Hag. The source generator's inherited five-file list and the distribution test's five-file expected set were two concrete Air maintenance omissions: preserved separately in `29e9a3414df6921abcea53e5ae0a40b8f1f0f2b8` and `6e8c9400cbe794ef9098915f1644b1dedaa972fb` before this Hag commit. Root owns the independently verified equivalent Air fixes; this branch's list/expectation extend to seven.
- Audited all four callers of `continuations::save_request`: direct spell SaveCondition, retained effect save, area save and Concentration. Each supplies a private non-Serde cause. The trait comes from the saving actor's exact source. Trait-absent actors return the old circumstances before this new category recognizer, preserving legacy pure lifecycle behavior; normal existing source/state validators remain authoritative.
- Direct Spell classification requires the validated canonical single SaveCondition program and matching actor/source tuple. Repeated classification requires the selected ticket's presence plus exact canonical retained condition, sole target-End/EveryOccurrence trigger, ability and success/failure behavior, overlap key/DC, supported expiry/group, source binding and deterministic effect/view/group IDs. The bounded 0..128 target-ordinal search mirrors retained casting's existing 128-occurrence limit. Stored DC/deadline survive changed caster mechanics or clock. This does not authenticate journal history or a Fiend target by itself: original semantic replay remains required. The retired-cast reviewer independently found no concrete source-equality/Rust issue in this dirty classifier, but did not compile or execute it; final whole-diff review remains open.
- Breath classification admits only exact frozen Chimera/young-red/adult-red source pins and their Fire Breath feature (SRD273,318-319). Concentration is nonmagical for this trait. Other causes needing classification fail explicitly. Direct Counterspell's InterruptSpellCasting node remains unsupported here; its later runtime must add and verify its exact private consumer. No generic magical bit, wire field, public reason, flow version or profile upgrade was added.
- Updated new live missile/Hag test constructors to use the actual current catalog pin. Left old rules V1 constructors, genuine historical fixture bytes and expected receipts unchanged.

Authored tests, all **unexecuted**:

| Location / test | Obligation |
| --- | --- |
| `tactical_creature_profiles::corrected_hag_is_a_distinct_complete_revision_without_upgrading_old_profiles` | Full source correction shape; both old/new full pins, unchanged mechanics, single current admission. |
| `tactical_spells::tests::same_id_hag_spell_tuples_resolve_full_revisions_and_reject_cross_revision_borrowing` | Two same-ID whole spell/creature/feature tuples, preserved components and cross-revision rejection. |
| `tactical::save_cause::tests::exact_trait_changes_only_authenticated_spell_category_and_uses_normal_composition` | Pure source category, old actor unchanged, Concentration/breath exclusions, unknown source rejection, cancellation/automatic precedence. |
| `tactical::save_cause::tests::synthetic_retained_clause_needs_no_live_cast_but_rejects_altered_source_shape` | Explicitly synthetic Fiend-target mechanism; no live cast, retained historical DC/deadline, local source/identity/trigger/condition/group/ticket adversaries. **Not accepted positive MR gameplay.** |
| `table_creature_cases::current_catalog_creates_corrected_hag_through_owned_transport_and_cold_restore` | Real current catalog; missing/old/forged live pin refusals with normalized full no-write export; file-backed admission, cold retry and independent restore. |
| `table_magic_resistance_cases::corrected_hag_has_real_fiend_no_effect_and_nonmagical_breath_controls_after_cold_restore` | Actual Cultist/Hag/Chimera source commands, paid Hold Person-Fiend no-effect, normal physical breath save, source Fire resistance, private projection, snapshot revision-swap rejection, cold reopen/restore/exact retry at real boundaries. |
| `rules_runtime::missing_undeclared_or_rehashed_corrected_hag_cannot_mutate_a_campaign` | Actual installed-content boundary after warming compiled source; unchanged full export after refusal. |

Validation performed: direct `rustfmt` parsing/formatting and `git diff --check`; Python manifest regeneration only as static content tooling. No Cargo/npm/node/build/test, database, UI, CI, push or merge was performed by this writer. These operations prove neither type correctness nor application behavior. Source hash rechecks and exact checkpoint SHA accompany the handback.

Final pre-commit static recheck retained all three frozen hashes in the baseline table above. New `night-hag-v2.json` is 2,731 bytes, SHA-256 `416d75d002f7dbb8500a4b8fef5a6d941eec14f694cf01d70349736ae89d05df`; its manifest FNV1a64 is `bc0d25b2db9c04ad`. This identifies the authored payload, not a passing runtime test.

Remaining acceptance gaps are explicit: parent Air is unaccepted; Shove/release integration, final full-diff review, focused/canonical CI/native checks and all genuine old-history suites remain root-owned. The current table parent forbids CreateCreature once an encounter exists; available genuine old-Hag captures are already in an encounter. Consequently real **same-campaign old/new Hag coexistence** still needs accepted release integration or a genuine original pre-encounter capture. No fabricated historical creation or rewritten fixture is substituted. The source/mechanism tests contain both pins but cannot close that application obligation. The direct and repeated positive MR gameplay requirements from the preflight likewise remain unwaived later Gate 4 work.

Next action: root independently reviews the exact committed checkpoint, schedules executable verification without competing with the exclusive baseline slot, resolves actual failures and performs acceptance-qualified integration. Do not publish/merge from this writer or start Counterspell runtime on this checkpoint alone.

## Independent review corrections, 2026-09-30

Root and a reviewer who did not author this slice inspected the complete `e349876254c95dcf92a52ecc765252d296fcdb2b` diff. The independent review found two concrete test defects before execution. Root now owns this branch and corrected both:

- The real Medium Hag placement `(75,45,0)` violated the existing ten-unit grid alignment. `(80,40,0)` is aligned; both occupied centers `(85,45,5)` and `(85,45,11)` remain inside the original Chimera cone from `(60,50,6)` toward `(90,50,6)`. All other actors remain behind the origin, preserving the intended single-target save. No geometry rule, source or gameplay assertion was weakened.
- Restoring the forged revision into the already-existing source campaign could fail merely because that campaign exists. The adversary now uses a fresh independent destination, compares all table row counts with its initial baseline after rejection, and restores the original genuine export successfully into that same destination. This checks actual rejection and atomicity without that masking failure path; the original source export also remains unchanged.

The reviewed source, frozen content and runtime implementation are unchanged by these test corrections. Full workspace format and diff checks passed on the authored checkpoint; final formatting and independent correction review are still required for the corrected head. No Rust compilation or test execution is claimed. Original baseline work retains the local heavy slot. Draft publication may start exact-head CI after correction review; source admission/coexistence, final integration, canonical/native evidence and genuine positive Magic Resistance remain acceptance obligations as stated above.

## Reviewed development integration plan, 2026-09-30

Root authorized this bounded local integration from the clean corrected source checkpoint `acf61abf188eda1f91a8007e799c43c410453cd8`. The fetched main reference is `d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`. This plan is committed before integration code. Normal local merges are now explicitly authorized; the earlier no-merge scheduling restriction is superseded only for the two development merges below. No push, remote-job cancellation, Cargo/npm/build, database, native UI or original-source capture is authorized. Root owns the exclusive fresh-target release verification slot.

1. Normally merge reviewed Air source `e915bbb9568f14a21c253bebf75d05ad2e91476e`, preserving both exact source revisions, seven installed source files and current admission. Air remains an unaccepted development dependency until its own acceptance.
2. Normally merge encounter release `8c03f9fb0058610fd37c0cfe7762e8b96d658f38`, which already includes the UI65 and fetched-main ancestry. This is an **UNACCEPTED release development dependency**. Its first canonical attempt failed at compilation; independent static review and a fresh-target retry do not constitute accepted executable or native evidence.
3. Reconcile the complete semantic union: authenticated Finished creation permission plus live full-pin admission; unchanged absent historical pins and accepted retry precedence; coherent replacement scene/participant locations before Air placement validation; release flow5 and completion/high-water history plus exact source-revision lookup. Preserve all frozen histories, retained legacy continuations and source payloads. Update only newly authored current test Begin/DTO fixtures that require flow5.
4. Import the independently reviewed child `support/source_revision_coexistence.rs` from the external draft with SHA-256 `d792e417b7435fe77cb8a69bba7ddfecd87906bc4680196bb40faad2ff94cb8e`, appending only its two-line module declaration to `legacy_shield_hit_v1_replay.rs`. Preserve the original parent's entire byte prefix, SHA-256 `0e722d6d46cb934c39eb2a74f778633612acd9b9be4f771886352b6d443d6564`, and the genuine partial-missile fixture, SHA-256 `fd410b848973510230ef5d1a4321636ef5df421ffc0067c19f7c5cfb1fb29cfc`. The draft tests actual old/new Hag coexistence, original pending work, authenticated release, current admission, full-pin spell execution, cold persistence, retry and hostile revision replacement. It is **UNCOMPILED/UNRUN**, and provides no positive Magic Resistance gameplay claim.
5. Inspect every conflict and the complete integrated delta, record exact checkpoint/tree and static checks, then return a clean local head for root's independent review. Report any broader unexpected semantic seam before expanding scope.

Reviewed external inputs are the release/Magic Resistance integration checklist (SHA-256 `5abaf0ee6b4713ec2e532a015a66c47740ac84141477f95c2dfda3f6f4d85077`), draft README (`0bac71d985f12e46dcd38d872973045b6a18f192124f7fd60b46f44982cf8dcc`) and independent draft review (`94848177d6ee02a5ad270da9d8fa7152280043b2c47d152fc795510bec13d98d`). Checklist corrigendum: its step7 phrase "corrected Constitution" is stale. Both original and corrected Hag sources have Constitution save +3; only the Magic Resistance trait and corresponding omission change.

Main acceptance remains blocked on accepted prerequisites, independent review of the final integration, exact-head executable/CI/native evidence and all remaining Gate 4 obligations. PR53 must not merge into Air; a later retarget to main is root-owned after prerequisite acceptance. Published `acf61ab` evidence cannot verify the forthcoming integrated tree.

### Local integration checkpoint

Plan commit `daa6492` preceded both normal merges. Air merge `4ffc14712328ac87363a63aad997f52652d32a39` retained the strict seven-file manifest generator/distribution expectations at its two conflicts. Release merge `0b15f0ba52cb81d191fba0ea6132c33df8757db2` has parents `4ffc147` and exact `8c03f9f`, with tree `7ec8dbb05a11ba9cce2c3c18b7bfe2f7bb461e6d`. These record development ancestry only; neither dependency is thereby accepted.

The release merge had three conflicted files. `Creature.test.ts` retains both source-pin/limits cases and the lasting-actor replacement case. `tactical.rs` includes both release and private save-cause modules, stages replacement actor locations before source placement validation against `next`, and then validates/activates the authored scene. The coverage ledger retains both source prerequisite disclosures and release disclosures. Auto-merged code retains current full-pin admission before the authenticated Finished exception, optional historical wire fields, full-revision initiative/profile lookup plus release high-water/history validation, and the release UI route alongside current source form pins. Only two new current test setups needed additional changes: the MR table case begins flow5, and the release recharge case supplies the exact Mage/adult-dragon source pins; no assertion or historical input was weakened.

The reviewed coexistence child is now wired into the genuine Shield-Hit replay suite. Its 1,265-line source remains byte-identical to the external draft hash above. The original 26,958-byte parent is retained as an exact prefix; only a blank line and the two-line child declaration are appended. The original partial-missile export and all imported release/flow4 fixture bytes remain unchanged. The child preserves original flow3 pending completion and old accepted creation before explicit upgrade/release, then tests new admission, same-campaign old/new profiles and full spell tuples through actual commands and independent cold recovery. This is an authored obligation, **not a passing test or positive MR save**.

Direct `rustfmt --check` on reconciled Rust and the imported child/parent, plus Git diff whitespace checks, provide static parse/format evidence only. Root still owns full independent integrated review, compilation, executable tests, exact-head CI, native evidence and acceptance. The branch remains local beyond published `acf61ab`; no publication or heavy operation occurred during this integration. The earlier same-campaign coexistence gap is now represented by an imported genuine-history test, but remains unclosed until execution succeeds on the final integrated head.

### Reviewed Air ancestry reconciliation

Root independently reviewed integrated `0b709239f62322fe3812b6a6714964e3aa148a86`,
tree `27a539ec4ca168ed09e16aa81ca4bd7945064935`, including the full genuine
coexistence child and all production seams. The static review found no actionable
defect; external review SHA256 is
`e513ac10c9004440747f10f8c1349014ccb89e0f73bdf632ed5a417632f924ad`.
The independently rerun byte audit is
`365dc47baacbca27d0ad867c9eafff045d57780b71320ec4c15953bb477ccaf7`.
Neither is executable or native acceptance.

Freshly fetched Air is now `3f3e3590e7977eca529168fa9069e8e1bca72434`, its
separately reviewed normal release8c integration. This MR branch already combines
the same Air e915/release8c production seams, plus MR and the genuine child.
Root will normally merge exact Air3f after committing this plan to align the
development ancestry. Expect documentation reconciliation only; inspect every
conflict and require production, test, content and historical bytes to remain
exact0b70923. Preserve both source/MR and Air/release disclosures. Any unexpected
semantic delta requires explicit review before publication.

Published acf remains untouched while its original runtime jobs finish. No local
heavy or native operation is part of this reconciliation. New exact-head CI,
canonical/receiving coexistence, accepted dependencies and final main/native
acceptance remain outstanding; PR53 must never merge into Air.


Plan checkpoint e1d5f4c preceded normal Air reconciliation merge
`fe862652151edee224969afb790c5e0b0839984e`, tree
`beb0f01fb48cd9f6c843ab01f9fd9688464d1630`. Only the coverage-ledger text
conflicted: keep the MR/coexistence paragraphs and the incoming accurate Air
parent-CI/integrated-pending disclosure. The complete delta from reviewed0b70923
contains exactly the Air plan, this plan and that ledger paragraph. All production,
tests, immutable content, genuine captures and the complete coexistence child are
byte-identical to reviewed0b70923. Root inspected the conflict and final diff;
whitespace checks pass. This is ancestry/documentation reconciliation only,
not new runtime verification. Published acf remains frozen for its existing jobs.

### Integrated-head failure and bounded correction plan — 2026-09-30

Root published reviewed `d77691ed52075ada15e74283d31d8efd287f9475` after acf
completed all six checks normally and its full evidence was preserved. The new
head passes four quick checks, including both all-target MSRV checks. Both runtime
jobs fail the newly added coexistence case at
`support/source_revision_coexistence.rs:183`: `TableRejected("This request
identity already belongs to different input.")`. The five original flow3 cases
pass on each platform. Linux job110057721270 completes the affected group5/1 in
135.83s; Windows job110057721765 completes5/1 in155.12s. Later suites never run.
The complete failed-head bundle SHA256 is
`9afc3e237e7ce7ae23e075e081bd1104322ab34907fa79997571ca1a25f3395b`;
Linux log `5eae46c51bd5ceb4d11d6296b1f7d805144232ae6aee9c099906c93867c41798`,
Windows log `005bb3bea9c46930dd8b428f99639b651a6b58de03d39998250753889b0d0090`.
Both jobs completed normally; neither produced a package.

Root inspected the actual immutable fixture and retry implementation. The old
Hag creation is command audit schema2, with its original version1 presented
transport request and response already retained in `table_transport_bindings`.
The new helper incorrectly called the legacy typed API, whose deliberate schema1
check rejects that cross-protocol reuse. Its comment claiming creation preceded
opaque bindings was wrong. The production refusal is correct; no source pin,
request identity or compatibility policy should change.

Before modifying the helper, root records this bounded plan: locate that exact
original binding by the saved creation command identity, authenticate its meta,
action, absent source pin and resulting event against the unchanged journal;
round-trip and retry the original full request through `submit_presented_table`;
require its exact stored response and whole-export equality. Retain the mistaken
cross-protocol submission as an explicit no-write rejection control. Keep all
three existing chronology cuts and every other coexistence assertion. Preserve
the parent suite and all original fixture bytes. Update the helper's provenance
comment, independently review the full correction, and publish a fresh verified
development head for CI. No local heavy run is authorized while release8c owns
the slot. This correction plan is not a passing coexistence result or acceptance.

Plan commit `7b37ae9efbed9c8c5161f81b89727cfdc2236380` precedes the test-only
correction. The helper now reuses the fixture's exact original presented request,
checks its saved meta/action/absent pin, binding event sequence and complete stored
response, and verifies the whole export after both the valid retry and deliberate
legacy-API rejection. Its original three call sites are unchanged. Direct Rustfmt
and Git whitespace checks pass. This is not a runtime pass: independent correction
review, new-head CI and local canonical verification remain required. The original
parent suite, captures, production code and source content are unchanged.

Independent correction review found the same mistaken typed retry separately in
the final independent cold-restore helper. It must be corrected before publication;
the first local correction alone is insufficient. Share the original-envelope
retry between the three chronology cuts and final independent restored runtime,
retaining per-operation whole-export checks and the existing all-bindings cold
retry loop. This is the same bounded test-only protocol correction.


## Accepted-main reconciliation before corrected CI — 2026-10-04 plan

Root is sole writer. Fresh remote main is dbf1d633460473183324b4ec519e8d1980884b5c;
Air is f932c73bf1f79cd0c5600431839a6ecece7877f7, tree
b654f37272f42b5115ee2c0ebdcc550f191f091e. MR aa8c36b is clean, fully reviewed
through the helper correction and documentation checkpoint. Its merge base with
Air is3f3e359; the incoming Air side changes exactly six documentation files,
370 additions/57 deletions. No production, test, content or canonical script
change is expected from that incoming history.

Before the next corrected-head CI, normally merge exact f932 into this branch.
Preserve both the accepted release/dbf evidence and MR-specific ledger disclosures.
Inspect any conflict; require every non-doc blob, all protected fixtures and raw
captures to remain exactly aa8. Unexpected source movement requires reconciliation
before proceeding. Record full tree/head and actual source parity, independently
review the complete receiving diff, then publish one corrected/reconciled head.
This brings accepted main into the ancestry; it does not accept Air or permit a
merge into its development branch. Later final-main verification remains required.

At15:29UTC old ad3 Linux and Windows remainder were still executing. Snapshot
8b5f72baec07ff645ed1271c27a08d34e13763b5044de0564904d0b85bf642da records their
state and the two unavailable partial-log responses. The isolated failure and
all six actual receipts/logs remain preserved. A new push may supersede unfinished
old jobs under existing workflow concurrency; such cancellation is not success.
All required fresh corrected-head partitions/union/package checks remain mandatory.
No local Cargo/native/DB work is part of this docs-only ancestry reconciliation.


## Accepted-main union checkpoint — 2026-10-04

Plan f3ed93f preceded the normal conflict-free merge
`d51886e8a6fed70a769100cc87707c0a202593c9`, tree
`b867eb9d5693b3c390b5bc9c19d7f522b91e49e8`, with exact second parent Airf932.
Accepted dbf is now an ancestor. The six incoming files are documentation only;
the ledger retains both MR paragraphs alongside the incoming Air/release evidence.
Root inspected the incoming ledger and retained MR delta. All other incoming docs
match the already reviewed Air head before the status clarification below.

All418 files outside docs/ remain byte-identical by Git blobs to aa8c36b and
therefore the reviewed correction1cea684; the pure suite result stays attributed
to that source head. This checkpoint adds no production behavior or passing
runtime result. The accepted release statuses are clarified using the same
reviewed evidence as expiry c9bbcb5, adapted to Air's later five-document context:
literal dbf's six checks
passed, while source8c retains its own canonical/native attribution. Historical
MR authored-only ledger paragraphs are labeled as historical; the MR plans retain
the actual Linux evidence, failed Windows attempts and new coverage repair.

Exact next action: independent complete receiving-head/documentation review,
then normal publication to PR53 with a fresh exact-head Linux/native partition
and package proof. Air remains unaccepted; no merge into its branch. Original
canonical/native/source acceptance and every remaining Gate4 family stay open.
