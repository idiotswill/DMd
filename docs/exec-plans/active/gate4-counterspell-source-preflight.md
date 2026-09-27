# Gate 4 Counterspell source preflight

Status: read-only source/code audit recorded on 2026-09-27. This is a proposal for
the next bounded prerequisite, not an accepted architecture or runtime change.
The active implementation remains
[Magic Missile target Shield](gate4-shield-missile-runtime.md). No Counterspell
executor, new execution version, content mutation or Gate5 work is authorized by
this document. Branch: `codex/gate4-shield-missile-runtime`.

## Objective and boundaries

Identify the source and compatibility work required before adding Counterspell to
the existing tactical stack. This advances the product-definition requirements for
source-faithful rules, controlled entities, complete tactical timing, exact
suspension and the real production path. ADR026/ADR028 and Gate04 continue to govern
authority, independent response ordering, immutable historical interpretation and
gate acceptance. A pure spell helper or new schema alone is not feature completion.

The immediate proposal is a separately reviewed immutable source-resolution and
explicit profile-admission prerequisite, including source-derived Magic Resistance
and authenticated magical-save classification. Do not silently change existing
profiles, waive the trait, match human-readable creature names, or treat the new
trait as unconditional advantage on every saving throw. The final architecture
decision remains open for independent review.

## Pinned source findings

The local `research/srd-5.2.1/SRD_CC_v5.2.1.pdf` outside this checkout was rehashed
for this audit. Its SHA256 remains
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`, matching
`content/srd-5.2.1/source.json`. The same-directory text extraction was inspected.
Use this pinned SRD5.2.1 source, not rules recollected from another edition.

- SRD p120: Counterspell is a Reaction to seeing a creature within 60 feet casting
  a spell with Verbal, Somatic or Material components. Counterspell itself has
  Somatic components and requires a Constitution save. Failure makes the spell
  have no effect; its Action, Bonus Action or Reaction is wasted. If cast with a
  spell slot, that slot is not expended. The interruption is not an attack roll or
  an ability check against the spell's level.
- SRD p305: Mage Protective Magic uses Intelligence, save DC14, and a shared
  three-per-day pool for Counterspell or Shield. The existing typed definition
  represents that pool as the feature's `PerLongRest(3)` usage, with no separate
  per-spell uses. Counterspell is level3 and Shield level1. This provides a genuine
  source creature route; do not invent a currently unavailable PC creation grant.
- SRD p311: Night Hag has Constitution save +3 and Magic Resistance, which gives
  advantage on saves against spells and other magical effects. Its at-will Magic
  Missile is level4; only Material is waived, so its V/S casting can satisfy the
  Counterspell component trigger when actually perceived and in range.

`content/srd-5.2.1/tactical.json` already has typed Counterspell
`InterruptSpellCasting { ability: Constitution, preserve_spell_slot: true }`, the
Mage grant, and the Hag missile grant. However, the Hag's `traits` array is empty
and its adaptation explicitly lists `magic-resistance` as omitted.
`content/srd-5.2.1/NOTICE.md` records that omission.
`tactical_definitions.rs::MonsterTrait` currently supports Pack Tactics and
Legendary Resistance, not Magic Resistance. Correct Counterspell against a new
source-faithful Hag therefore requires more than exposing the existing interrupt
descriptor.

## Compatibility finding: definitions are immutable source identities

`tactical_creatures.rs::creature_definition_fingerprint` hashes the serialized
whole `CreatureDefinition`. `source_for_profile` resolves only the current
definition ID and then demands the exact pinned ruleset/version/fingerprint.
`tactical_creatures/profile.rs` builds and validates the profile against that
definition. Editing the existing Hag trait array in place would invalidate existing
profiles and genuine exports, even where their earlier play never used the trait.

Spell source identities are also affected. `tactical_spells.rs::source_pin` hashes
the tuple of spell, optional whole creature definition and optional feature. A new
creature trait consequently changes its spell program source fingerprint too.
Retaining an old creature ID while silently rebuilding a program from its new
definition does not preserve historical interpretation.

Evaluate a versioned immutable source registry addressed by the full pinned source
identity, including ruleset ID/version, definition ID and definition fingerprint.
Keep each historical definition available with its original canonical identity;
let the fresh catalog select the corrected definition explicitly. Spell program
resolution must likewise reconstruct the exact historical spell/creature/feature
tuple and verify its existing fingerprint. An ID-only fallback to the newest
definition, accepting an unknown fingerprint, or disabling pin validation is not a
compatibility solution.

The minimum lookup audit includes these current-ID paths, not only profile lookup:

- `tactical_creatures.rs::source_for_profile` and profile build/validation;
- `tactical_spells.rs::plan_spell_cast_authorized`, `validate_spell_plan`,
  `source_pin` and `source_components`;
- `tactical_spells/creature.rs::plan_spell_from_feature` and
  `tactical/casting.rs::validate_actor_source`;
- `tactical/continuations.rs::save_modifier` and
  `tactical/initiative.rs::preview_initiative_circumstances`;
- every other creature-definition lookup used by derived statistics, traits,
  source feature usage, spatial admission, replay and source DTOs.

This is an audit starting set, not a claim that all lookup paths were enumerated.

Explicit admission of the corrected source for new profiles is required. Decide
separately whether an existing profile can be upgraded by a journaled operation or
must use an explicit replacement/admission path. Any proposed upgrade must preserve
old creation/cast evidence, retain current HP, expended resources, control and other
live state according to reviewed rules, and reject unsafe in-flight changes. It
must never rewrite an old profile pin or earlier event as if the new trait had
always existed. The representation, migration policy and safe admission boundary
are open architecture questions; this audit does not select one.

## Magic Resistance and saving-throw provenance

`tactical/continuations.rs::save_request` currently combines condition-derived
circumstances with default external circumstances. It has no authenticated magical
effect category. The future Counterspell save must derive Constitution +3 and
Magic Resistance advantage from the admitted Hag source, with ordinary advantage/
disadvantage cancellation, physical faces, Inspiration and supported failed-save
choices retained by the existing machinery.

Add a typed source-backed Magic Resistance trait, and derive the save category from
the authoritative typed producer and retained cause. The client must not supply a
trusted `magical` flag; neither the creature name, raw-roll reason text nor any
earlier spell damage is proof. Specify classification for direct spell saves,
magical creature effects, nonmagical effects and concentration saves separately.
In particular, a concentration save after magical damage is not automatically a
save against that spell. Preserve each older execution's accepted request/mode and
source interpretation; do not inject advantage into historical pending rolls.

## Existing casting seams and required future contracts

The pure program compiler already emits `SpellProgramNode::InterruptSpellCasting`.
The real encounter binding's `ExecutableSpellKind` has no interrupt variant, so
that descriptor is not a production Counterspell path. `tactical/casting.rs::begin`
currently admits and commits the spell within the same command; flow4 has no
durable pre-commit Counterspell window.

The existing pure lifecycle has useful semantics: casting spends the action type;
`SpellCastAdvance::Countered` marks a still-casting spell countered and ends only its
matching new concentration without the commit-time slot expenditure. Ordinary
interruption is a separate path and cannot be substituted for the Counterspell
slot exception. Source feature use already paid at admission and an expended
Reaction are not a spell slot and must not be refunded by that exception. Existing
slot reservations still constrain nested casting. A held Ready spell's release is
not another casting trigger.

Future runtime work needs an authenticated pre-commit parent cast and an independent
response window within the existing resolution/frame stack. Reuse source admission,
component/range/perception checks, current-turn ordering and respondent ownership,
but make each trigger's delegation independent. Selected Counterspell becomes a
real child cast that may itself be countered; only its successful effect produces
its parent's Constitution save. Keep the original parent uncommitted until the
response work settles. Do not accept a client-provided countered/succeeded boolean.

Current hit and missile Shield completions prove an installed Defense effect.
Countered Shield needs an explicit, versioned child outcome and retired casting
proof; do not forge `completed_shield` or discard the paid child. A countered hit
Shield must resume the same accepted hit, including natural20/cause behavior. A
countered missile Shield must settle its selected response, continue the independent
respondents/amount barrier, and confer no missile prevention. Include all retired
countered casts in the shared cast/work occurrence namespace.

## Genuine capture and acceptance prerequisites

Before changing flow4 casting interpretation, finish and verify the current missile
production path. Capture real flow4 pauses through actual table requests/SQLite:
selected hit Shield, selected missile Shield, partial amount collection, all amounts
before impacts, partial impacts, and the concentration-child pause immediately
before the automatically resumed final singleton. Also capture an ordinary current
spell at its first genuine raw pause: its cast is already committed and must not
gain a new response window when resumed under future code. There is no genuine
current pre-commit casting pause to capture; editing a saved phase/version would not
provide historical evidence.

Preserve the existing PR42 and four flow3 captures byte-for-byte, including source
profile pins, program fingerprints, accepted envelopes, presentation history and
pre-tactical anchors. The source registry must restore and continue their original
creation/casting journals without rewrites before acceptance of any new source.

The later Counterspell runtime acceptance needs real source Mage/Hag commands,
component/sight/range refusals before cost, both response arrival/order choices,
shared Mage pool accounting, nested countering, Constitution/Magic Resistance
physical rolls, successful/failed saves and matching concentration cleanup. Cover
both interrupted Shield parent types, old/new retry identity, private projections,
file close/reopen at every persisted stage, independent restored continuation and
hostile earlier-image refusal with normalized full no-write comparisons. These are
required future evidence, not authored tests or passing results in this audit.

## Exact next action and verification status

1. Finish the current Magic Missile slice, integrate verified PR45/main normally,
   and retain exact-head focused/canonical/CI evidence. This audit does not release
   the shared heavy build slot or permit another runtime slice to bypass that work.
2. Obtain an independent design review of the immutable registry, spell-source
   reconstruction, explicit corrected-profile admission and magical-save category.
   Resolve the open migration/version boundary in an ADR and a bounded execution
   plan before implementing the prerequisite.
3. Implement and verify that prerequisite with genuine historical exports unchanged
   and explicit new source admission. Then capture the verified flow4 production
   pauses and review the bounded Counterspell stack design before changing runtime
   execution semantics. Stay within Gate4; no acceptance is waived or moved to Gate5.

Evidence for this document is source/code inspection and the rechecked PDF hash.
No new source registry, trait, magical-save category or Counterspell runtime was
implemented. No compilation, tests or production acceptance ran for this preflight.
