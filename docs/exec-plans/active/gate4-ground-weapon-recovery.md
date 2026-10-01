# Gate 4 — Recover physical ground weapons during an attack

Status: concrete plan amendment only, 2026-10-01; no implementation or runtime
acceptance. Root temporarily transferred sole writer ownership to the assigned
plan author from clean `053f736e27d5897d6cf197c5a7b0bf8f20113d7d` on
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
foundationfa4, Shovef9, Air3f and release8c. At the original plan checkpoint, fetched main was
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`. These are reviewed development
dependencies, not accepted main. Reconcile accepted prerequisites normally and
never merge a stacked PR into its development base. The current main observed on
2026-10-01 is `dbf1d633460473183324b4ec519e8d1980884b5c`; this checkout has not
merged it. Earlier d88 observations remain historical; root coordinates normal
prerequisite integration and final exact-head verification.

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
transition. Source `creature_weapon::facts` separately checks the actual Item's
required definition, not custody; preserve both checks in their proper consumers.
Actual thrown and
unconscious-drop producers leave the same Item in Location custody with a
`TacticalGroundItem` position and causal command. There is no pickup producer.
The existing `TacticalWeaponAttack` before-image contains loadout and ammunition,
so reconstruction cannot yet authenticate an original ground-custody transfer.

External source inspection `tooling/ogre-ground-pickup-consumer-notes-2026-09-30.md`
records these initial seams and unresolved after-attack timing; its early source
custody wording is corrected above. The complete source-grounded design memo is
`tooling/ground-053f736-equipment-lifecycle-independent-design-2026-10-01.md`, SHA256
`f86ca70db8c3d2f00b89ed84b3c2b67471e0fff328b0b1df2ccbd25a837648cc`.
Root read and agreed its concrete proposal. The decisions below reconcile it into
repository memory, pending independent review of this exact checked plan before
source authoring. Neither external note is implemented behavior or runtime proof.

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
   became impossible. Use the explicit opt-in, separate retained work and owned
   Apply/Decline lifecycle decided below; preserve original after-equipment semantics.
7. Distinguish an existing ground Item from the weapon newly thrown by this same
   attack. The latter's actual ground cause does not yet exist at declaration.
   The later Pickup consumes that actual new ground record if currently legal;
   never use a pre-attack ground image as its proof or change old Equip's prohibition.
8. App offers and UI use stable real Item identities, legal hands and exact
   before/after choices. The server derives access, cost, position, source and
   damage. Hidden/unreachable ground items must not be disclosed through option
   lists or identifier probes. A failed/uncertain acknowledgment retains the exact
   submitted command; it must not manufacture a replacement pickup.

## Concrete wire and receipt decision

Add only typed choices and executor-derived evidence; names below describe the
intended domain contract, not already implemented APIs. New optional fields use
default/omit-when-absent serialization, preserving old bytes and fingerprints.

| Addition | Authority and shape |
| --- | --- |
| `AttackEquipmentOperation::Pickup { item, hand }` | Selects an actual Item and receiving hand only. Supported in BeforeAttack and in the new selected after-work Apply. Declaration-time `AfterAttack/Pickup` refuses; it cannot prove a future ground state. |
| `after_equipment: Option<AfterAttackEquipmentIntent>` on both `WeaponUseChoice` and `CreatureWeaponUseChoice`, with sole intent `Choose` | Explicit opt-in to a later decision. Requires `equipment_change == None`. Absent retains old behavior and creates no new pause. An old after operation or any before operation, including Pickup, excludes this intent. |
| `ChooseAttackEquipment { work: TacticalWorkKey, choice: Decline | Apply(AttackEquipmentOperation) }` | Resolves exactly the selected after occurrence; no client-supplied position, source, formula, cost, before-image, allowance boolean or post-state. |
| Optional before-pickup receipt on `TacticalWeaponAttack` and its `WeaponAttackReceipt` | Exact executor-derived Item before-image, `TacticalGroundItem`, encounter/scene/location identity, plus actual actor equipment before-image. Required exactly for before Pickup, absent for other operations. The original command binds the pre-action spatial/authority cut through replay. |
| `TacticalWorkKind::AttackAfterEquipment` and an optional retained `TacticalAttackAfterEquipment` on the resolution | Binds exact work key, originating attack, actor, action window, completed weapon-receipt identity, completion command and requested intent. Exists independently of `resolution.attack`; no RollRequest/raw role. Only one eligible own-turn physical attack starts this resolution, and no OA/nested source grants another follow-up. Reject duplicate retained authority. |
| Optional final after decision on `WeaponAttackReceipt` | `Declined { work, chosen_by }` or `Applied { work, chosen_by, operation, equipment_before, ground_before? }`. Ground image is required exactly for Pickup. Selection, decision and completion identities must agree; chronological proof comes from exact accepted commands. |

The allowance is derived from the actual declaration and AttackAction window.
No source or caller provides a remaining-allowance flag. Before Pickup followed by
throwing that weapon has spent the allowance; it cannot request recovery after
the same attack. Thrown's existing intrinsic draw permission remains distinct.
No Opportunity/Reaction, BonusAction or unrelated command family admits Pickup or
the new follow-up. The shared guarded Grapple consumer explicitly refuses the new
operation until its separate reviewed scope supports it. Preserve all Grapple guards.

Require exact flow5 under both Live and Historical at every new producer/choice
boundary, including Decline: a command that retires all its new state must not
evade a state-only version guard. Retained new receipts/work require that same
executor. Original retry lookup remains first; no old paid continuation is changed.
No source pin migration, current source admission or implicit execution upgrade.

## Shared preparation and causal execution

Introduce one rules-owned `prepare_attack_equipment` boundary for ordinary and
future source attacks. It accepts actual state, trusted origin, actor, derived
window and typed operation, and derives a sealed non-deserializable preparation
plus before-image. A preparation is bound to the exact input state/call lifetime,
or its complete before-image is revalidated at consumption. Never expose a generic
accepted candidate state or "pickup already checked" flag. The candidate is local
rules computation, not another public mutation path or a source admission bypass.

For before Pickup, check actor knowledge before disclosing identity/geometry errors;
then validate current map, real custody/ground record, intact quantity-one known
weapon, usable hand and manipulation reach. Derive the same Item's Location-to-
Entity custody, selected hand and removal of one ground record in a local candidate.
Pass the coherent candidate to ordinary/source physical planning before its
selected-weapon carried-item check. Preserve the actual original choice/receipt
and consume that operation internally once; do not apply it again in the old
before planner or hide it by rewriting the accepted command. Only after target,
source, mastery, Action, ammo and supporting-hand checks all succeed commit pickup,
costs, source activation and pending dice in the existing atomic transition.
Picking up a different weapon must leave a legal attack grip and supporting hand.

The future Ogre adapter supplies only its separately validated exact-pin printed
program/damage; this boundary owns physical preparation and custody proof. Its
checkpoint2 plan at `c205759` is separately under review. Do not implement its source
adapter here or lift its current/profile/Historical creation guard. Another actor
using the recovered Item keeps ordinary weapon dice. Integration must use this same
preparation boundary and reconstruct the original before Pickup exactly once.

For an opted-in after choice, preserve the existing completion cut: attack outcome,
ammunition, same-Item thrown custody/ground cause and any original old after operation
complete before downstream damage consequences. The old Equip/Unequip behavior,
including refusal for the just-thrown weapon, is unchanged. Create the new follow-up
only for its explicit unused allowance, **below** the resulting damage/Graze effect,
concentration and fall frames. Retain it when `resolution.attack` is cleared; never
leave a fake Finishing attack that can re-run damage or reconstruct the thrown Item
as still carried. The completed physical receipt remains available for the follow-up.

`turns::pump` drains child frames and queues flight losses before selecting later
work. Register the new occurrence in the existing bounded work trace, not a parallel
queue. It cannot share a simultaneous ordering frame with its prior consequences
or set a global wait before those children drain. Once selected, retain it as an
owned non-roll choice. Extend pump, ordering, work-trace and turn validation so the
resolution cannot retire, accept another attack, EndTurn, Conclude or Finish around
unanswered work. Do not expose it through raw-dice submission or ChooseTurnWork.

Apply rederives actual capability, EffectiveHands, current custody, ground record,
visibility and reach at that command's real causal cut. Decline checks the exact
selected work and current controller/session authority but does not require a free
hand, an available weapon or mechanical ability to act. A controller can decline
after its actor becomes unconscious/dead; the pending optional operation must not
strand the paid attack. Invalid/stale Apply is a full no-write rejection and leaves
the same decision available for a valid Apply or Decline, without undoing/repeating
the earlier attack, damage or costs. No automatically selected alternative. Record
the decision before retiring the follow-up and resuming the shared pump.

Same-attack thrown recovery uses the actual ground record produced by completion,
including its real completing-command origin. It needs no invented pre-attack
ground receipt. Current adjacent reach can allow it; a distant target, forced
movement, fall, blindness, changed custody or lost hand can make it unavailable.
The actor may then choose another legal operation or Decline. A later genuinely
available attack can provide its own allowance; do not invent an extra attack or
resource grant to prove finite weapon recycling.

## Ground admission, original-cut proof and later cuts

Add a bounded ground-point query in the spatial module, shared by offers and
acceptance. Reuse actual awareness/senses, illumination/obscuration, clear sight
and clear physical effect/reach primitives. The exact stored Item point/height
and actual occupied actor footprint/vertical cells define the ordinary5-foot
manipulation check. Document/test the object-point metric against the existing
participant-distance convention; do not fabricate a creature/volume to call
entity perception. A visible point through an obstruction is not necessarily
physically reachable. Host knowledge, a remembered floor cell, creature contact,
Tremorsense of a creature or a supplied ItemId is not current object knowledge.
Unknown, hidden, remote or foreign ID probes must not disclose hidden item facts.

Check exact Item/campaign identity, Intact state, quantity1, known weapon definition,
actual scene/location and exactly one matching active ground record, with no holder
or conflicting hand assignment. Require actual hand anatomy/reservations. Different
prior owner is allowed; keep owner, definition, quantity, original grant and journal
cause unchanged. Location custody alone is insufficient without that live record;
Finished-scene receipts do not grant current-map access. A repeated drop of the same
Item under a later cause is a distinct cut, not reusable earlier pickup permission.

The before-image supports reconstruction but is not self-authenticating authority.
Extend `attacks/planning::reconstruct`'s current equipment/ammo inverse with only the
recorded pickup transition. Verify the expected current physical image before
restoring original Item custody/ground entry/equipment in a local reconstruction;
do not rewind unrelated later mutations. The genuine original command replay proves
its then-current geometry, perception, source and controller. After choices are
replayed at their own accepted command cut, not original attack geometry or today's
post-fall state. Completion may create the same-attack ground origin; require exact
causal ordering without falsely requiring that drop to precede attack declaration.

Extend `rules_restore::command_origins` for nested removed-ground origins, equipment
images, completion and decision metadata. Bind exact audit/action, original attack,
window, source, selected trace key, chronology and event outcome. Earliest pre-tactical
anchor replay must reproduce every snapshot and final current image; matching a
forged request and raw faces is insufficient. Preserve ADR020's integrity trust
limit rather than claiming cryptographic authenticity of a rewritten entire history.
Completed turn receipts can retire under existing turn-history rules; their full
older proof remains in accepted journal/snapshots. Do not add an unbounded duplicate
inventory ledger or remove earlier throw/drop history.

## Application/UI and concrete verification targets

Extend carried-only `table_attacks::options` with safely admitted ground choices and
an independent equipment item/hand selector so before pickup of a different weapon
is expressible. The server determines legal choices; UI emits stable actual IDs.
Expose a separate owned after-work view because today's `attack_decision` requires
a live `resolution.attack`. Route authority from the retained attack actor through
normal table session/attendance/source control, not the current turn actor or former
item owner. Current controller authorizes the continuation; the original command
retains the original admission proof. Preserve exact opaque binding, presentation
history and uncertain-acknowledgment retry behavior across close/reopen/restore.

AttackForm gains explicit later-choice opt-in and the selected after view gains
Apply/Decline with a work-key-based focus identity. No legal options must still
show Decline. Keep old draw/stow choices and accepted absent-field views exact.
The UI cannot create a replacement request after an uncertain acknowledgment or
supply ground positions, cost, source pin, damage or accepted after-images.

Author controls for selected-versus-different-weapon before Pickup; one allowance;
same-attack throw and actual later Pickup; remote/changed circumstances followed by
Decline; damage/concentration/fall/unconscious-drop ordering without re-equip;
unchanged prior payment on rejected Apply; exact finite Item/owner/grant history;
supporting/reserved hand, duplicate/missing ground, wrong scene/height/obstacle;
blindness/darkness/hidden-ID non-disclosure; player/source control and session
binding; wrong work/occurrence/actor/stale handle/changed payload full no-write;
flow1–4 Live/Historical rejection including no-effect Decline; and genuine original
retry plus real file SQLite/cold/independent portable restore at pending attack,
pending damage, selected after choice and final decision. Forge ground cause,
position/custody and matching request/raw evidence only as labeled negative restore
controls. Real admitted play, original history execution and native UI proof remain
mandatory; a direct state edit or synthetic candidate is never a positive substitute.

## Plan-first checkpoints

1. Consumer preflight is complete and the concrete decisions above are checked
   in as a plan-only amendment. Obtain independent review of this exact plan
   before source changes; resolve any record/lifecycle findings there first.
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

Status remains PLAN ONLY. Root agreed the external design; this concrete checked
amendment still needs independent review. No schema, source, test, UI or source
admission change and no runtime evidence exists. Git whitespace/static diff checks
are the only verification appropriate to this amendment. Root owns the native Air
heavy slot; no Cargo/npm/build/test/database/native operation or push is authorized.
Exact next action: return this clean plan commit and writer ownership to root for
independent review before confirming a bounded source-authoring assignment. Keep
Ogre creation closed until its complete forms/OA/pickup/UI path is reviewed.
