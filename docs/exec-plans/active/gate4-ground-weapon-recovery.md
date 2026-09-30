# Gate 4 — Recover physical ground weapons during an attack

Status: plan/preflight only, 2026-09-30. Root is sole writer of
`codex/gate4-ground-weapon-recovery`, reusing the clean historical
`gate4-source-creature-control` checkout. Its old catalog-cache branch remains
preserved at619ada0. No command, schema, source, test or admission change exists.

## Objective and dependency

A combatant can pick up an actual reachable ground weapon through the ordinary
Attack action equipment allowance, preserving its identity, custody and prior
history. Another actor must be able to recover a thrown source weapon and use
ordinary weapon damage. This is the mandatory production prerequisite identified
by the independently reviewed Ogre plan, not general noncombat inventory work.

Development base is source-count candidate
`2559bfe7404e642b11ea36a1323de7a35e0cb56c`, tree
`1832e561008d621099e6e8afaa77cad6104e7708`, draftPR55. It inherits unaccepted
foundationfa4, Shovef9, Air3f and release8c. Fresh fetched main remains
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`. These are reviewed development
dependencies, not accepted main. Reconcile accepted prerequisites normally and
never merge a stacked PR into its development base.

The separate Ogre writer owns immutable source, printed attack adapters and
physical source opportunity attacks. This branch owns the general active-map
ground transition and its app/UI/recovery integration. Agree shared enum and
receipt interfaces before source authoring; do not concurrently edit either
checkout. Normal integration must retain both independent objectives and tests.

Read root AGENTS, product definition, Gate4 checkpoint/protocol, ADR018/020/024,
the physical-weapon and encounter-release plans. This advances authoritative
physical custody/action economy, source fidelity, real player controls and exact
session suspension. It does not close the inventory, combat-actions or weapon
families, full Ogre admission, Grapple, old-scene revisiting or Gate4 as a whole.
General object interaction/travel/economy retains its existing receiving gates.

## Rules and concrete existing gap

The pinned SRD5.2.1 PDF SHA256 is
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Its printed p177 Attack action includes weapon pickup within one equip-or-unequip
before or after an attack. It grants no extra attack or Action. Monster equipment
guidance p255 preserves ordinary recovered equipment use: the Ogre's printed
damage belongs to its source, not to the Javelin Item. Neither source is changed.

Current `AttackEquipmentOperation` offers only Equip/Unequip, both requiring
Entity custody. Generic weapon preparation checks custody before the equipment
transition; source attack facts have another earlier check. Actual thrown and
unconscious-drop producers leave the same Item in Location custody with a
`TacticalGroundItem` position and causal command. There is no pickup producer.
The existing `TacticalWeaponAttack` before-image contains loadout and ammunition,
so reconstruction cannot yet authenticate an original ground-custody transfer.

External source inspection `tooling/ogre-ground-pickup-consumer-notes-2026-09-30.md`
records these actual seams and unresolved after-attack timing. It is not an
implemented design or runtime result.

## Required design boundaries

1. Add an explicit ground-pickup operation; preserve the old accepted Equip and
   Unequip payloads and semantics. Admit new producers only under exact current
   flow5 in both Live and Historical policies. Original accepted retries precede
   current admission, with all old raw tags/pins/fixtures unchanged.
2. Use the existing single Attack-action equipment allowance. No separate free
   transfer, hidden Action tax, extra draw, bonus attack, item grant or ownership
   reassignment. Opportunity attacks cannot pick up equipment. Shared consumers,
   including the separately guarded Grapple core, must not inherit new authority
   merely because an enum match compiles; each needs its own reviewed admission.
3. Bind the actual acting controller, current active battlefield and one intact
   quantity-one weapon with exact Location custody and a matching current ground
   placement. Require a usable physical hand via EffectiveHands, visible/reachable
   item knowledge and unobstructed ordinary5-foot interaction reach using actual
   footprint and height. This bounded manipulation interpretation is explicit;
   extended attack reach and a Large/Giant tag do not automatically extend it.
4. Atomically change only the same Item's custody, selected hand and current ground
   placement. Keep owner, quantity, definition, original grant and prior journal
   cause. A previous different owner is not a reason to reject dropped loot.
   Foreign campaign, wrong scene/custody, destroyed/consumed item, duplicate ground
   record, inaccessible position/height, stale input and occupied/reserved hand
   refuse before any mutation or new payment.
5. Preserve an authenticated before-image containing original custody, map/ground
   position and causal drop identity, in addition to the actual equipment image.
   Reconstruct the same admission for pending attacks/damage after a cold/portable
   restore. A caller-supplied post-pickup state, matching forged request and raw
   roll, or current carried Item alone cannot supply original proof. Omit any new
   optional receipt field when absent so old serialization/fingerprints stay exact.
6. Support both before and after equipment allowance. Before pickup may acquire
   the very weapon used for the attack or a different weapon. After pickup must
   be validated at its real causal execution point, including an intervening
   movement/fall, loss of usable hands or changed custody. Do not obtain a remote
   Item or strand an already paid/resolved attack because its planned after choice
   became impossible. A real retained owned choice/decline may be needed; inspect
   the common work queue and original after-equipment semantics before choosing
   the precise domain representation.
7. Distinguish an existing ground Item from the weapon newly thrown by this same
   attack. The latter's actual ground cause does not yet exist at declaration.
   Resolve that concrete legal case in the design instead of silently treating a
   pre-attack ground receipt as its proof or inheriting old Equip's prohibition.
8. App offers and UI use stable real Item identities, legal hands and exact
   before/after choices. The server derives access, cost, position, source and
   damage. Hidden/unreachable ground items must not be disclosed through option
   lists or identifier probes. A failed/uncertain acknowledgment retains the exact
   submitted command; it must not manufacture a replacement pickup.

## Plan-first checkpoints

1. Read the actual before/after physical planner, source adapter, completion,
   visibility/geometry and historical authentication consumers. Resolve the
   receipt and after-choice lifecycle in this plan, including recovery when the
   proposed after operation becomes unavailable. Obtain independent review of
   the concrete design before source changes. No public bypass or new schema is
   preapproved merely by this initial plan.
2. Implement the bounded pure admission and current versioned producer/receipt,
   preserving all legacy paths. Meaningful controls cover physical identity,
   hands/reach, exact allowance and historical reconstruction. Do not mutate
   existing fixture states to claim a genuine pickup history.
3. Connect the real application/projection/UI and ordinary physical attack path;
   integrate the Ogre source adapter deliberately after its separate review.
   Full source admission stays closed until all required printed forms/OA/pickup
   and controls exist.
4. Prove genuine accepted play: normally create/equip actors, actually throw/drop
   the weapon, have another actor pick it up and attack with ordinary dice. Cover
   before/after, repeat pickup, all finite Javelins and same-turn throw/recovery
   as resolved by the reviewed rules design. No replacement Item or direct state
   edit may discharge this evidence.
5. Run focused checks and canonical verification, exact-head Linux/Windows CI and
   packaged native pickup/cold continuation. Review the full diff, reconcile
   accepted main, merge with expected-head protection and separately verify main.

## Acceptance and current limits

| Boundary | Required evidence |
| --- | --- |
| Real custody | Same original Item changes Location to Entity custody once; actual ground record removed, grant/owner/prior cause retained; native control reaches this route. |
| Source separation | Another actor uses ordinary1d6 Javelin or1d8 Greatclub base damage; exact Ogre source still uses its printed dice through its own adapter. |
| Economy and timing | One before-or-after allowance, no extra attack/Action; real after-interruption unavailable-choice recovery; no OA pickup; before pickup of selected or different weapon. |
| Geometry/privacy | Real map/height/hand/perception input; wrong/hidden/remote/foreign/destroyed/stale choices reject without revealing hidden facts or changing state. |
| Persistence | Real file SQLite and independent portable restore at pickup-owned choice and pending attack/damage; exact original retry, changed body/foreign actor/stale handle leave complete store unchanged. |
| Compatibility | All29 frozen fixtures and five original receiving suites retain exact bytes and execute; old Equip/Unequip and source pins/fingerprints remain exact. |

Status is plan/preflight only. The existing Ogre review establishes why pickup is
required; it does not review this as-yet unresolved after-choice representation.
Root owns this branch. The local heavy slot remains reserved by the guarded-core
verification lane; no Cargo/npm/database/native operation is authorized here yet.
Exact next action: complete the consumer preflight and concrete receipt/timing
decision, then independent plan review before any implementation.
