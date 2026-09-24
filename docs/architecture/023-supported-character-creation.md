# ADR 023 — Source-derived supported character creation

Status: **Accepted — Gate 3; see the [integrated checkpoint](../checkpoints/gate-03-desktop-table-loop.md).**

## Decision

The first supported creation profile is SRD 5.2.1 Human Fighter 1 Soldier with Skilled.
These are honest fixed supported identities, not pretend full catalog selectors. Players
assign all six standard-array scores, choose the permitted Soldier boosts, two class
skills, one Human skill, three Skilled skills, size, two languages, Fighting Style,
Gaming Set proficiency and starting equipment. Mastery grants select all three currently
supported weapon definitions in any order. More choices and complete catalogs remain
owned by Gates 4/5/6; this is not the final character product.

The source is the existing official CC BY 4.0 pin. Character creation pp19–22,
Fighter47–48, Soldier83, Human86, feats87–88 and equipment89–96 ground every grant.
Tough is not a feat in this source. Soldier boosts apply only to Str/Dex/Con, with no
old-edition species ability bonuses. Initial HP is 10+Con, not a rolled d10. Fighter
equipment option C and Soldier option B supply 205GP; source-priced purchases debit
integer copper. Arrows are bought in multiples of twenty. Gaming Set proficiency is
granted independently of buying a physical set.

`build_character` consumes typed choices and an entity ID plus the exact supported
pack, deriving `BuiltCharacter { mechanics, profile, equipment }`. Callers do not supply
AC, HP, modifiers, bonuses or resource maxima. `CharacterProfile` retains immutable
creation choices, grants, source pages, initial loadout and remaining money; equipment
records are starting inventory data for the application's identity-owned items. Backstory
is unaccepted player narrative, never newly established world truth.

`validate_character_profile` reconstructs source derivation and rejects altered grants,
prices, counts or derived profile fields. `RulesAction::CreateCharacter` requires an
existing authorized active world/character identity, initializes rules if absent, or adds
one new mechanical entity without replacing previous entities, rolls, rulings or history.
Root table/application integration owns identity creation, controller binding, atomic
persistence, current inventory and the normal desktop workflow.

## Selected functional features

Defense derives +1 AC while the created character wears armor. Archery adds +2 only to
ranged weapon attacks and outstanding-request reconstruction. Second Wind creates a
saved raw d10 request with Fighter level as modifier, spends one of two uses, respects
the combat Bonus Action budget, and heals on accepted input. Short rests restore one
use; long rests restore all. There is no caller-supplied healing total.

Human Resourceful grants Inspiration after a completed long rest. If already inspired,
the new grant creates a persisted optional transfer decision. Mechanics wait for the
controller's `ResolveInspirationTransfer`, with an explicit eligible recipient or None
to decline. No PC decision or recipient is invented by the engine. Queries remain usable.

Savage Attacker accepts two complete raw weapon-damage sets and the player's choice of
either, including the worse set. It requires a recorded combat turn, rejects a repeat in
that turn and rejects spell damage. Critical weapon dice remain raw; ordinary modifiers
are not doubled. An optional Heroic Inspiration replacement records its original set,
die index and replacement, spends Inspiration and preserves both original inputs. Replay
re-resolves exactly these inputs rather than choosing a better roll or rolling again.

## Explicit remaining scope

All Fighter masteries are recorded and visible, with execution owned by Gate 4. Complete
weapon properties, wielding/equipment changes, ammunition expenditure and spatial
legality remain Gate 4; existing attacks continue their explicitly limited single-attack
primitive scope. A carried Shortbow is not a claim of full ammunition/hand automation.
Tool/item use and wider rest scheduling remain Gate 5. Skilled tool-choice variants,
other origins/classes/species, progression and complete catalogs retain Gates 5/6.
Character sheet feature text must reveal these limits; unsupported declarations must
remain unresolved or follow explicit trusted adjudication rather than silently claiming
the feature executed. Unrecorded turn timing cannot grant unlimited Savage Attacker.

## Compatibility and verification

New opt-in mechanical feature state and raw Savage Attacker records default absent and
omit absent fields on serialization, preserving existing Gate 2 meanings and replay inputs.
Schema 3 table integration is owned by the concurrent persistence slice. The source
catalog is the same tracked JSON embedded in the rules module and shipped with manifest
integrity and attribution. Source/negative tests validate choices and reconstructed
profiles; transition tests cover authority, dice, recovery, pending decisions, rejection,
serialization and complete event replay. These tests do not replace Gate 3 production
desktop acceptance.
