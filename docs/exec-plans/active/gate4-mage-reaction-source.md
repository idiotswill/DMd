# Gate 4 Mage reaction source capabilities

Writer: bootstrap_audit. Branch `codex/gate4-mage-source`, based on PR33 integration
`9910ce7`. The Ready writer owns domain casting/reactions, spell program compilation,
binding/retained programs and the shared tactical scheduler. This source slice owns
tactical definitions/catalog/manifest, needed creature profile/equipment source
adapters, source feature scheduling and their focused tests. One writer per file.

## Objective and source boundary

Provide a genuine source NPC and source capabilities for Gate4's Shield/Counterspell
reaction pipeline. Preserve root AGENTS, Gate04, ADR026 and the product's source,
player-control, bounded content and exact replay requirements. This is source/data
and authorization plumbing, not completed reaction execution, native acceptance or
full catalog completion. Full remaining Gate4 mechanics stay in Gate4.

Pinned English SRD5.2.1 source anchors:

- p305 Mage: Medium or Small Humanoid (Wizard), Neutral; AC15 includes Mage Armor;
  HP81(18d8), Speed30ft, printed abilities/saves/skills; Wand; Common plus three
  languages. Intelligence spell save DC14. Protective Magic is one shared3/day
  feature choosing Counterspell or Shield in response to that spell's trigger.
- p120 Counterspell: level3, Reaction to seeing a creature within60ft cast with
  V/S/M components, Somatic component only, instantaneous. Actual Constitution
  save; failure dissipates the interrupted spell, wastes its activation but does
  not expend its spell slot. No automatic level comparison or invented check.
- pp161–162 Shield: existing canonical descriptor, Self, V/S, +5AC including the
  triggering attack until next own start, prevents Magic Missile damage.
- p145 Mage Armor: base13+Dexterity, willing unarmored creature,8hours, ends when
  wearing armor. Its included source AC must be represented honestly, never called
  natural armor or fabricated permanent innate protection.
- p96 Arcane Focus Wand physical gear; SRD257 spellcasting source components and
  p186 per-day feature recovery on Long Rest remain applicable.

The source text is pinned outside the repository in
`../research/srd-5.2.1/SRD_CC_v5.2.1.txt`; legal attribution remains the distributed
CC BY4.0 NOTICE/source provenance. No non-SRD source grants enter by familiarity.

## Typed contract and ownership

Add `ReactionTrigger::SeenCreatureCastingWithComponents` and closed
`EffectDescriptor::InterruptSpellCasting { ability: Constitution,
preserve_spell_slot: true }`. Range60, one visible creature, no upcast effect.
The Ready writer maps this descriptor to its owned program/interrupt continuation.

Add `FeatureActivation::Reaction` and `CreatureActionCost::Reaction`. The ordinary
serialized `BeginFeature` route must reject unproven reactions. Only a crate-private
`begin_creature_reaction_feature(state,current,meta,actor,selection)` source hook is
available to the authenticated live-window adapter. It derives source/issuer/counters
and invocation metadata; the shared scheduler pays the actual Reaction exactly once.
Protective Magic uses feature-level `PerLongRest { uses:3 }`; neither spell receives
an independent three-use pool. No caller boolean or arbitrary prepayment bypass.

Mage representation is explicitly `SelectedFeatures`: include the faithful printed
statistics and Protective Magic source, real Wand custody, and only source clauses
actually modeled. Omitted actions/spells remain named. Mage Armor preparation needs
a coordinated honest source-baseline/lifecycle decision before its code is written;
the current generic no-gear armor adapter otherwise treats printed AC as perpetual.
The root and Ready writer have been notified of this specific decision dependency.

## Verification and next action

Plan written before implementation. Read current source/descriptor/scheduler files,
settle included Mage Armor representation, add bounded source descriptors/adapters
and strict malformed-source/counter/authority tests. Preserve old canonical definition
fingerprints by omitting newly optional fields on existing entries. Update actual
manifest bytes after source changes; test full distributed pack integrity. Obtain
independent source review, then focused tests and strict lint in the shared compiler
slot. Root canonical/UI precedes the protocol writer; no compiler is running here.
The reaction driver remains a separate verified integration dependency. No source
or production acceptance has been claimed for this new slice.
