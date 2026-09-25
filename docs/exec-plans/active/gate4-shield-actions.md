# Gate 4 paid shield actions and physical source controls

Writer: environment_audit. Branch `codex/gate4-shield-actions`, base `8fa4484`.
Root owns canonical integration/PRs; area owns shared resolution/work/continuations.
This branch owns the new shield action helper, narrow dispatch arms, live shield AC
interpretation, table equipment/source-weapon choices, desktop controls and tests.

## Objective and authority

Make the source-created Goblin's shield/weapon preparation executable through the
real table. A shield must not vanish for free to permit a two-handed Shortbow.
The same operation must support trained PCs and source NPCs, retain real ItemIds,
custody, original accepted command, current armor and the single action budget.

Source: pinned SRD5.2.1 p92 (Shield uses a Utilize Action to don/doff; only training
grants its AC benefit; one shield at a time) and p191 (Utilize). Creature training
comes from its canonical stat block; PC training comes from its validated creation
profile. Goblin Warrior weapon facts remain p290, including mandatory extra damage
when the actual attack has Advantage. No display-name dispatch or client bonuses.

Relevant contracts: product definition preparation/inventory as real gameplay,
authoritative tabletop state, player-safe information and exact suspension;
Gate4 checkpoint and active plan; ADRs002/009-012/025/026. Neither this slice nor a
green isolated test completes Gate4 or the native encounter acceptance.

## Source decisions and public path

- `DonShield { shield: ItemId, hand: Hand }` selects a real intact quantity-one
  canonical shield in actor custody and an available hand. A shield already held
  in that hand may be donned; the operation cannot silently move/drop another item.
- `DoffShield` derives the actual donned identity, removes its hand/attachment and
  retains it in carried custody. This operation stows the removed shield; it does
  not drop it into an invented map location, transfer ownership or change quantity.
- Both require the active authorized actor, an available Action and no pending
  continuation. Spend one Utilize Action through the existing timing authority;
  accepted preparation ends continuous run-up and interrupts a strenuous-action
  rest. Invalid/stale/foreign/occupied-hand input leaves every field unchanged.
- Anyone may don a shield; lack of source training withholds its +2 AC rather than
  forbidding the object. Preserve existing natural defense and armor, including
  the source NPC's fixed-defense adapter and PC Defense fighting style.
- Existing `ActorEquipmentLoadout.command` records the exact accepted change.
  Replay and restore authenticate it through the existing versioned tactical event;
  no new queue, clock, authority aggregate or random identity is introduced.
- Only the authorized active actor/host gets shield choices. The table also exposes
  source-backed physical feature choices so a Goblin's Shortbow uses its printed
  action rather than losing mandatory source damage in a generic weapon action.
  Ordinary legal looted-weapon options remain available as ordinary attacks.
- The frontend retains original typed request/head/identity before sending and
  uses existing retry recovery. It never changes equipment optimistically.

## Acceptance and slices

1. Pure paid shield helper + shared dispatch and training-correct live AC. Test
   custody versus ownership, hand conflicts, repeated costs, stale/foreign/pending
   actions, original receipt and serialization/replay, PC/source training behavior.
2. Filtered table DTOs and source-feature/physical ItemId choices; actual shield
   controls and source attack submission using the existing command outbox.
3. Real SQLite Goblin creation/materialization -> paid doff -> next turn physical
   bow/raw dice -> paid don, with source armor, hands, ammo and budgets verified.
   Close/reopen, exact retry, export/restore, changed-head rejection and forged
   loadout command/current AC/historical-anchor rejection must remain atomic.
4. Focused rules/app/frontend checks, strict Clippy, format/diff, independent full
   and exact-head review. Root owns final canonical verify/CI/desktop acceptance.

## Non-goals and active remaining work

No general timed armor dressing (minutes), object economy, invented skill checks,
free shield removal inside Attack, broad source catalog, Multiattack, new mastery,
or second tactical scheduler. Full environment/object actions and other combat
families remain active Gate4. Beyond-encounter travel/preparation scheduling remains
Gate5; this does not change the Gate4 shield timing obligation.

## Status and next action

Source and existing loadout/action/app boundaries audited; root approved the scoped
contract and area author approved additive dispatch ownership. No implementation
or tests yet on this branch. Next: implement the shield helper and AC training
bridge, then table/UI and real persistence regressions. Do not compile until the
globally serialized slot is explicitly released (area currently owns it).
