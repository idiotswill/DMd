# Gate 4 source physical weapon actions

Writer: environment_audit. Branch: `codex/gate4-primary-attacks`, starting at the
reviewed Charge checkpoint `bf2e38d`. Root owns the app/UI and source creation;
movement owns shared falling/turn work. This slice owns attack modules, attack
source/choice domain types, one approved public action/dispatch and focused tests.

## Objective and source boundary

Connect Goblin Warrior Scimitar and Shortbow (SRD5.2.1 page290) to actual physical
items through the public reducer. Both have +4 and 1d6+2 damage, with an extra 1d4
of the same type when the attack has Advantage. Scimitar is melee reach5ft and
Slashing; Shortbow is ranged80/320ft and Piercing. Their mandatory advantage clause
must survive cancellation, raw damage, critical doubling and same-type defenses.

New source creatures have real stowed weapons, leather armor, a donned shield and
finite ammunition. Source attacks must use the ordinary Attack action's one explicit
weapon equip/unequip allowance. It cannot doff the shield for free or waive the
shortbow's two-handed grip. Item custody, not ownership, authorizes borrowing.

Intrinsic Charge behavior remains unchanged. No Multiattack, legendary activation,
source conditional rider or unrelated mastery expansion belongs to this slice.
Source attacks whose printed facts cannot yet be represented faithfully reject
before spending any action, source use or ammunition; the remaining Gate4 work
stays active, never silently reclassified as catalog work.

## Contract and implementation

- `CreatureWeaponUseChoice` supplies target, real weapon ItemId, grip, ammunition
  stack and explicit equip change. The source feature determines delivery and
  attack/damage; the client cannot supply modifiers or a contradictory ability.
- `TacticalAction::CreatureWeaponAttack { feature_id, choice }` selects the active
  source actor. The existing source scheduler verifies its Action activation.
- The existing physical planner must represent the source base attack exactly:
  legal ability/grip/property facts, printed base numbers and canonical range/reach.
  A deterministic matching normal weapon plan is retained for equipment/history;
  source-defined advantage damage remains part of the same retained attack.
- `TacticalAttackSource::CreatureWeapon` retains the source pin/feature plus the
  existing `TacticalWeaponAttack` record. Its `weapon()` adapter keeps the same
  reservation, physical completion, raw dice and vitality/concentration queue.
- Before admission all source/equipment/geometry checks pass. Accepted admission
  spends one action, reserves one real arrow, records exact original loadout, and
  binds source invocation. Reconstruction checks those facts again before raw input.
- Existing `attack.origin` and `weapon().equipment_before.command` remain audit
  origins; no synthetic authority or additional event stream is introduced.

## Verification

Build only after bootstrap releases the serialized slot and movement's falling batch
if ready. Add public source-created/materialized Goblin scenarios: stowed scimitar
equips into the free hand beside the shield; stowed shortbow fails while shield is
donned, and succeeds only with actual free hands; real ammo spends once and leaves
a Spent tombstone; borrowed versus foreign/missing/wrong-kind items; short/long/out
of range and hidden-target privacy; Advantage and cancellation, critical extra dice;
before/after equip, paused attack/damage/knockout/concentration replay; forged source,
receipt, range, raw request, ammo count and grip rejection with unchanged inputs.

Run focused attacks, movement and turn tests, then strict domain/rules all-target
Clippy, format/diff checks and independent exact source review. Root owns canonical
workspace/application/desktop verification and the eventual Gate4 summary. This is
the planning checkpoint; no new source code or successful test claim yet.
