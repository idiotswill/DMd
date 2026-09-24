# Gate 4 tactical source definitions

Status: Implemented; focused verification passed, awaiting root integration and review.
Branch `codex/gate4-combat-definitions`, based on `14a94d6`.

## Objective and boundaries

Add a separately versioned, source-derived tactical definition catalog for the Gate 4
resolver. Preserve the existing kernel definitions and their replay meaning. This slice
does not implement execution or claim that a definition is a playable feature. The root
Gate 4 plan owns integration, authority, persistence, desktop acceptance and gate scope.

Read AGENTS, Gate 4 checkpoint, product tactical philosophy/timing/visibility, ADRs
018–020 and 024. The source is the pinned SRD 5.2.1 and existing CC BY attribution.
Weapon rules and the complete weapon table are on source pages 89–91. Selected spells
and creatures retain their exact source page and deliberately bounded catalog identity.
Full catalog population belongs to Gate 6; tactical mechanics remain Gate 4.

## Acceptance and planned slices

1. Model typed weapon properties/masteries and source-faithful attack ranges, damage,
   handedness, ammunition and exceptional requirements; include every source weapon.
2. Add reusable spell/effect and creature-feature descriptors with bounded source entries
   sufficient to exercise tactical integration. Validate identities, values, references,
   combinations and schema/source pins; reject unknown fields and contradictory data.
3. Register the new JSON in the existing manifest, retaining NOTICE/source provenance.
   Add meaningful source and negative validation tests. Coordinate focused checks with
   the root to avoid concurrent builds, then commit for independent review/integration.

## Decisions and risks

- New `tactical.json` and `tactical_definitions.rs`; no edits to kernel.json, existing
  RulesPack/AttackDefinition/SpellDefinition, domain/state/spatial contracts or UI.
- Root adds the public module export on integration. A standalone integration test can
  include this module directly until that shared-file change lands.
- No campaign names, fixture-only policies or inferred creature behavior enter definitions.
- Structural validity alone cannot authenticate arbitrary replacement source content;
  production integration must pin installed bytes to the reviewed source asset.
- Source property/mastery semantics are typed identifiers with source-clause Rustdoc;
  execution remains the resolver's responsibility. All 38 weapons (including firearms,
  fixed-damage Blowgun and mounted Lance) are represented. Prices/weight are not duplicated.
- The ten spells are Burning Hands (114), Fog Cloud (133), Hold Person (141), Shield
  (161–162), Shield of Faith (162), Thunderwave (169), Magic Missile (146), Cure Wounds
  (121), Fire Bolt (131), and Dancing Lights (121–122). Their descriptors preserve source
  target/area/visibility, timing, material, duration, ongoing saves and object clauses.
- Five represented stat blocks are Goblin Warrior (290), Skeleton (325–326), Wolf and
  Warhorse (364), and Young Red Dragon (318). Adult Red Dragon (318–319) and Cultist
  Fanatic (278) have **selected features only**, with machine-readable omissions. The
  latter retains its real Pact Blade and Hold Person 1/Long Rest; no invented spell grants.
- The schema covers these entries' effect families, not every spell/monster mechanic.
  Summoning, shape-changing and additional tactical feature families remain Gate 4 work;
  full source catalog population remains Gate 6. No row is promoted by this definition slice.
- Updated the manifest regeneration helper and distribution test to include tactical.json;
  regeneration must not silently remove the newly reviewed asset.

## Validation and next action

The final focused run passed all eight `dmd-rules --test tactical_definitions` tests and
the `dmd-domain --test distributed_rules_pack` test. Strict Clippy for the definition test,
rustfmt and `git diff --check` passed. The tests compile the real production module directly
while the shared lib.rs export is owned by root. Source exceptions, contradictory data,
unknown fields, references, partial coverage and exact manifest bytes are checked.
Regenerating the manifest with the updated helper retained all five declared assets.
The final tactical asset is 62,423 bytes, FNV-1a64 `7d9f1b02b523f400`.

No full build was run in this slice: the root coordinates serialized workspace verification.
No execution or Gate 4 acceptance claims follow from these definition tests.

Next: root reviews/cherry-picks this commit, exports `pub mod tactical_definitions;`, pins
installed tactical bytes to the embedded catalog, and wires supported effects through the
authoritative encounter resolver and table flow. Root performs full integrated verification.
