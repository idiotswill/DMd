# Gate 4 - Primary attack execution

Status: Active; implementation and production integration pending.

## Objective and ownership

Branch `codex/gate4-primary-attacks`, worktree `gate4-attacks`, based on root
`5e410c6` plus reviewed turn bridge `7f6db28` (local cherry-pick `0fa40d4`).
Connect canonical physical-weapon and source-creature attacks to the one authoritative
TacticalResolution, raw attack/damage dice, vitality, and concentration continuations.
This slice owns new domain/rules attack modules, tactical dispatch/work integration and
focused tests. Root owns application/table/UI, NPC equipment construction and live AC.
No source NPC equipment WIP from the root checkout is included in this branch.

## Contracts and scope

Product clauses: source-faithful rules authority, physical player dice, player/actor
identity separation, durable invalid-input atomicity, and complete production encounters.
Gate 4 tactical encounters and ADR025/026 govern spatial/state and continuation boundaries.
Pinned SRD5.2.1 pp15-18,89-90,176-191,255-257 and represented stat-block source pages.

- Accept only typed actor choices, physical ItemIds, source feature IDs and raw faces.
  Source definitions, geometry, effects, equipment and grants derive all numeric rules.
- Reuse existing weapon preparation and accepted weapon history for chosen ability/grip,
  Loading, Light/Nick/Cleave, ammunition and equip-before/after behavior.
- Retain one typed attack intent/resource receipt with exact original command provenance;
  restored pending requests must be reconstructed from canonical source and retained
  inputs, never arbitrary after-state images or player-authored modifiers.
- All attack, damage, knockout/mastery decisions and concentration follow-ups use the
  existing TacticalResolution. Creating action work must not re-observe a turn boundary.
  After-End source actions nest in the existing continuation and preserve its remaining
  source windows; they must not advance the turn before the attack finishes.
- Source feature activation pays once. Ordinary loot weapons use ordinary equipment
  proficiency; stat-block attacks use their explicit bonuses/formulas and source riders.
- No network, random generation, direct persistence, or player-control automation here.

## Acceptance and verification

1. Complete accepted ordinary primary attacks through raw hit/miss/critical and damage,
   fixed damage, temporary HP/death, controller-owned knockout and concentration work.
2. Preserve physical identities, quantity/custody, equip timing and accepted history;
   no duplicate ammo spending or action/refund on continuation, replay or restart.
3. Derived range/cover/perception/conditions, natural1/20, underwater automatic misses,
   and explicit source attack bonuses remain authoritative.
4. Unsupported source riders or mastery consequences reject before action/attack/ammo
   costs. An accepted attack never silently discards a mandatory or chosen consequence.
5. Focused tests serialize/replay each accepted boundary, corrupt retained requests,
   change issuer/actor/head, and verify unchanged originals after invalid operations.
6. Run focused Rust tests and strict Clippy in the globally serialized build slot.
   Root owns canonical full verification and real application/native acceptance.

## Planned slices and remaining Gate 4 work

- Define retained attack work and request reconstruction; connect ordinary weapon path.
- Connect source feature receipt path with canonical damage and supported hit clauses.
- Add explicit post-hit decisions and reuse existing damage/concentration source work.
- Validate authority, source/resource provenance and each restored stage; regression tests.
- Independent review, focused verification and coherent checkpoint for root integration.

Sap/Vex/Slow require typed non-condition effect modifiers, not magic labels or a second
TacticalFlow effect authority. Unsupported modifiers, Charge movement predicates,
reaction triggers/interrupts, improvised/environmental attacks and broader source feature
programs remain active Gate4 obligations until connected. They are not moved to Gate6.
The first closed pipeline will explicitly reject cases it cannot finish before spending
resources. This source slice alone does not claim an end-user feature or completed gate.

## Evidence and next action

Existing weapon/equipment/history/mastery, spatial cover/perception, tactical conditions,
turn bridge, vitality and grouped concentration APIs inspected. Turn bridge exact-head
read-only signoff complete; author reports20turn+14creature+31damage tests and strictClippy.
No new build has run. Next: implement the domain work contract and ordinary attack reducer,
then source hooks after root supplies verified NPC equipment/live-AC integration.
