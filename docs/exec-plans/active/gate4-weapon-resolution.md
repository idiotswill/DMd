# Gate 4 weapon resolution primitives

Status: kernel slice reviewed and locally verified; production integration pending.
Branch `codex/gate4-weapon-resolution`, base
`2d38f6160dcbe8704b4fc5581b96e95ba2ce6063`.

## Objective and boundary

Implement reusable source-backed weapon attack and consequence planning for all 38 SRD
weapons, ten properties and eight masteries. Advance Gate 4 weapon properties/masteries,
underwater combat, timing, targeting and player agency through primitives that the root's
authoritative encounter resolver consumes. This slice does not claim complete combat or
desktop acceptance. Source catalog expansion, inventory materialization, persistence,
continuation execution, geometry, UI and global gate closeout remain root-owned.

Relevant contracts: root AGENTS; Gate 4 checkpoint; product clauses on authoritative
spatial combat, complete timing, durable consequences and player choices; ADRs 009/010
(issuer/dice), 011/012 (atomic journal/replay), 018/020/024 (source authority/recovery),
and root's evolving ADR 026 encounter continuation contract.

## Source and state decisions

Only pinned SRD 5.2.1 supplies mechanics: attack/range/cover p.15; underwater, damage and
criticals p.16; Fighter mastery selection p.48; properties/proficiency pp.89–90;
all weapon rows p.91; ammunition p.96; Attack equipment permission p.177;
Exhaustion p.181. Source content is
the exact installed/embedded tactical catalog already reviewed in PR #23.

- Each physical weapon has a real ItemId and quantity one. Ammo remains stacked. Root
  materializes validated starting equipment and persists hand assignments; no synthetic
  copy identities and no profile equipment quantities used as mutable inventory.
- Existing mechanical ability scores and source PC profile/monster gear determine
  capability. An internal derived context supplies current spatial/timing facts; it is
  not deserializable player authorization. Root validates source identity and issuer.
- Loading belongs to the actual action/bonus-action/reaction opportunity. Light/Nick
  require different physical weapons; Cleave uses the same physical weapon and a prior
  melee hit. Typed receipts retain command/action/actor/turn and outcome provenance.
- Finesse ability, grip, optional mastery, secondary target and push distance remain
  explicit choices. No automatic selection on behalf of a PC.
- The result proposes typed resource/attack/mastery consequences. It does not spend
  inventory, mutate budgets, roll dice, write events, or bypass the encounter resolver.
- Cleave immediately continues its accepted triggering attack in the same action
  opportunity. Thus an Attack-action Cleave retains that action's per-attack equipment
  permission (pp.90/177), while Reaction Cleave does not. It creates no new paid action,
  resets no Loading limit, and keeps the exact parent window in its durable receipt.
  Root reviewed and accepted this interpretation for the continuation boundary.
- Attack modifier includes ability, trained proficiency, Archery and Exhaustion exactly
  once. Spatial/effect advantage and further active effects are composed by root.
- The planner validates immutable character grants separately from live armor. The
  inventory adapter must validate current armor and the Defense wearing-armor flag;
  strict legacy character validation remains unchanged. Legacy kernel attack-ID sets
  stay creation data; tactical availability comes from physical inventory identities.

## Scope and acceptance

Own new `crates/dmd-rules/src/tactical_weapons.rs` and submodules/tests, its lib export,
and this plan. Root additionally authorized a new domain `tactical_equipment.rs` plus
export for durable hand/receipt contracts. Root also authorized `character_creation.rs`
and its focused tests: any three distinct Simple/Martial source mastery choices are legal,
while historical defaults/profile replay stay valid. The starter shop and armor catalog
are unchanged. Root owns exposing these choices through the dynamic creation options/UI.

Acceptance includes custody/identity and malformed-input rejection; correct ability and
proficiency; handedness/free loading hand; reach/normal/long range; Heavy thresholds;
underwater Piercing exception and ranged automatic miss; Light/Nick/Cleave constraints;
ammo consumption and thrown-weapon release; versatile and critical damage including
Blowgun's fixed damage; all conditional mastery triggers, choices, expiry and turn limits.

Meaningful tests must cover boundaries and interactions, not merely mirror definitions:
same/different weapon identities, stacked ammunition and empty supplies, repeated Loading
across different opportunities, critical fixed/rolled damage, underwater Finesse/Heavy,
zero-damage mastery triggers, source-grant absence, secondary targets and replayable plans.

## Planned slices and verification

1. Agree the typed domain/API contract with root before substantive implementation.
2. Attack/equipment/history validation and deterministic profiles/resource plans.
3. Mastery offers and explicit validated choices/consequences with source-relative expiry.
4. Source interaction regressions, independent review, focused tests and strict Clippy.

Builds are serialized: root full verification, effects followup and runtime checks own
the initial slots. Do not start compilation until a slot is explicitly released. Use
focused `cargo test --locked -p dmd-rules tactical_weapons`, strict focused Clippy,
formatting and whitespace checks; root performs full integrated verification/CI.

## Risks and next action

Gate 3 creation retains starting equipment in the source profile but does not materialize
live ItemInstances. Root accepted ownership of that integration and of hand/receipt
persistence. Existing provisional Light tracking by definition string must be replaced
with physical ItemId receipts. The new kernel cannot be exposed directly as a player
permission surface, and arbitrary serialized contexts are not accepted commands.

Root independently reviewed all planner,
equipment, history and mastery files and confirmed fixed Blowgun damage against p.16.
Review found Exhaustion omitted from the attack modifier and immutable creation armor
incorrectly constraining live equipment. Both are corrected with regressions. Further
tests cover ordinary creature gear without importing stat-block feature riders, malformed
receipt identity, and inherited Nick/Cleave equipment timing.
The source reviewer independently read all four weapon modules against pp.16/89–90 and
found no further concrete blocker. Damaging-hit mastery outcomes count resolved damage
including temporary-HP absorption, rather than only HP loss.

Final native focused verification passed with one compiler job and no concurrent build:

- `cargo test --locked -p dmd-rules --lib tactical_weapons`: 17 tests passed.
- `cargo test --locked -p dmd-rules --test character_creation`: all 14 tests passed,
  including every source mastery kind, unchanged historical defaults and existing
  creation/feature/authority/replay regressions.
- `cargo clippy --locked -p dmd-rules --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.

The tests include all 38 source weapon definitions, normal/long/reach/Heavy boundaries,
underwater automatic miss with retained ammunition cost, Finesse and Archery classification,
versatile/critical/fixed damage, physical Light/Nick/Cleave identity and timing, Loading
windows, optional and mandatory mastery triggers, exhaustion, live-armor separation,
ordinary monster gear without stat-block riders, and malformed receipt rejection.

Production obligations remain explicit: root derives spatial facts, validates issuer and
catalog identity, reserves the paid action, spends ammo once even on an automatic miss,
persists pending choices/receipts and validates their canonical replay against journal
origins. Mastery consequences are not applied by this module. Controller-selected mastery
changes after a Long Rest, production equipment choices, durable runtime/UI integration
and end-to-end acceptance are not claimed by these pure-kernel tests.

Next: root integrates this coherent slice with inventory and encounter continuations,
executes realistic persistence/replay/UI paths and full verification/CI on the combined
exact head. This local kernel evidence does not close Gate 4.
