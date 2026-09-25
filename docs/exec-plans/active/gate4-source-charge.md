# Gate 4 source melee action and Charge continuation

Writer: environment_audit. Branch: `codex/gate4-primary-attacks`.
Starting integrated checkpoint: `935a207` (reviewed casting `81c1a4e` plus the
separately reviewed departure helper `5435743`). The helper's execution evidence
belongs to the movement author's pending combined batch.

## Objective and governing scope

Complete the source Warhorse Hooves/Charge action through the real tactical reducer:
source admission, one action cost, raw attack and damage, conditional Prone, nested
vitality/concentration and deterministic replay of every pending stage. Root AGENTS,
Gate4, the product's source fidelity, authoritative movement, provenance and exact
combat resume requirements govern this slice. Existing source catalog bytes remain
unchanged.

SRD5.2.1 page364 gives Hooves +6, melee reach5ft and 2d4+4 Bludgeoning. If a Large or
smaller creature was approached20+ feet straight immediately before the hit, add2d4
Bludgeoning and Prone. There is no saving throw. The implementation must not import
an older-edition Strength save or a bonus attack. The reusable admission may accept
other source Action melee attacks only when every applicable effect already has a
complete supported path; unsupported feature/effect choices reject before cost.

## Ownership and design

- Own `tactical_attacks.rs`, `tactical/attacks/**`, focused attack tests and this plan.
- Movement author approved only the `CreatureAttack` action/dispatch addition in
  `tactical.rs` and removal of the unused annotation from `straight_approach` when
  this source becomes its real consumer. No other shared turn/work/pump changes.
- The public choice names target, canonical feature ID and optional actual weapon
  ItemId. Actor, printed bonuses, dice, reach and conditional effects derive from
  the active actor's immutable creature profile. No client DC/modifier/proof fields.
- Invoke the existing creature feature scheduler for source timing and use limits.
  Spend the central Attack action exactly once. Keep the ordinary source attack in
  the existing TacticalResolution attack slot, raw work and vitality children.
- Snapshot the validated straight-movement origin and endpoints before admission
  clears continuous movement. Retained proof validates actor/campaign/head, endpoint,
  traveled distance and actual approach geometry. The app must also authenticate
  that original movement command through accepted-event replay.
- No placeholder equipment for intrinsic attacks. Required physical implementations
  must be held, intact and in actual actor custody; reject unsupported source use.

## Acceptance and verification

Add public reducer tests with real source construction and actual movement: Charge
after20ft; below20ft; turn/direction/action interruption; too-large target; normal miss;
critical doubles base and extra dice; Prone immunity; source/foreign/stale/unavailable
input unchanged; retained proof/raw request tampering; serialized replay before raw
attack, damage, knockout choice and target concentration as applicable. Verify that
ordinary opportunity Hooves does not acquire own-turn Charge from old movement.

Run focused attack/movement/turn tests and strict domain/rules Clippy only after the
shared compiler handoff. Root owns full canonical checks, application origin audit,
desktop control/projection and production acceptance. New source code is not yet
implemented or tested at this planning checkpoint.

## Remaining Gate4 work

Source mixed Multiattack and legendary actions, ranged source attacks/physical weapon
extras, remaining mastery saves/movement/ongoing modifiers, grapple/shove and broader
spell/zone/summon/transformation paths remain explicit active obligations. This slice
does not accept Gate4 or redefine those obligations. Next action: implement retained
own-turn source admission and Charge proof in the owned attack modules, then obtain
fresh movement/source review before the serialized test batch.
