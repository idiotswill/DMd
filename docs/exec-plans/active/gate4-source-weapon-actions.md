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

Public source-created/materialized Goblin scenarios cover: stowed scimitar
equips into the free hand beside the shield; stowed shortbow fails while shield is
donned, and succeeds only with actual free hands; real ammo spends once and leaves
a Spent tombstone; borrowed versus foreign/missing/wrong-kind items; short/long/out
of range and hidden-target privacy; Advantage and cancellation, critical extra dice;
before/after equip, paused attack/damage/knockout/concentration replay; forged source,
receipt, range, raw request, ammo count and grip rejection with unchanged inputs.

The focused batch passes all 104 tests: 62 attacks (including ten new source weapon
cases), 18 movement and 24 turn cases, with no failures or ignored tests. Log:
sibling `research/gate4-source-weapons/focused-second.txt`. The initial compilation
found a test-only enum typo (`Missing` belongs to `Custody`); the correction preserves
the intended missing-item assertion. Strict domain/rules all-target Clippy also
passes (`research/gate4-source-weapons/clippy.txt`); format/diff checks pass.
Independent exact committed-head review follows this tested source checkpoint.
The complete reducer and ten scenarios have already received independent read-only
source review with no blocker. Root owns canonical workspace/application/desktop
verification and the eventual Gate4 summary.

## Remaining production boundary

The real source Goblin starts with its shield donned and its weapons stowed. Its
Scimitar can use the normal attack equip allowance in the free hand. Switching to
the Shortbow needs an actual paid Doff Shield action, which is still active Gate4
work for the equipment controls. The bow tests use an explicitly prepared legal
shield-free starting state; they do not prove that the tabletop can yet perform
that transition. No source exception, free shield removal, or duplicate equip
allowance is granted here.

A subsequent static pass confirmed that the shared vitality reducer cannot resolve
damage against an already-dead body. Source weapon admission now rejects that target
before spending the Action or ammunition, after the generic location check. It does
not reject a living target at zero HP, nor does retained validation retroactively
reject a target killed by the committed attack. Two added regressions cover these
boundaries and lethal completion with serialized replay. Independent delta review
found no blocker, and both tests pass in the focused batch. Body/object durability
remains separate active Gate4 work.
