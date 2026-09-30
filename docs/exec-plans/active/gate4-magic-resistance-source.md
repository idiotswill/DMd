# Gate 4 corrected Night Hag source and Magic Resistance prerequisite

Status: plan-only checkpoint, 2026-09-30. No implementation or executable verification of this slice.
Branch: `codex/gate4-magic-resistance-source`; no PR yet.
Writer: `gate4_air_recovery`, sole writer assigned by the coordinating root.
Development parent: exact Air source checkpoint `1a0a5e75bfbde98c7621c421fbdc9d5b4750ab92`, inspected clean in the reused `gate4-shield-missile-runtime` checkout after root fetched it. That parent is an **unaccepted development dependency**, not verified main. Root owns its separate Air branch/PR and all integration, publication and heavy verification.

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
- The current admission helper still returns all immutable entries; failing to separate it before adding a second Hag revision would incorrectly admit both versions as current. This is a concrete required change.
- Source classification must be exhaustive over actual producers. Repeated effects can outlive their original casting resolution; classification cannot require that live cast or trust only the effect's source-name string. Complete canonical-clause/replay proof remains an implementation obligation to inspect before claiming this lane finished.
- Source glossary p185 resolves the magical/nonmagical criterion. No source ambiguity currently requires an owner ruling. If a newly discovered clause cannot be represented faithfully without expanding this bounded slice, record the exact source/proof issue and return it to root before coding that expansion.
- Local verification constraints and Air/release/Shove integration are scheduling/acceptance dependencies, not permission to weaken source or compatibility checks.

Next action: root reviews the exact plan-only commit and confirms implementation assignment. Only then implement the bounded source/admission/classification slices above on this branch. At this checkpoint the only changed file is this plan; no source payload, runtime, test, manifest, historical fixture or accepted evidence has changed.
