# Gate 4 — Recover physical ground weapons during an attack

Status: bounded foundation plan amendment only, 2026-10-04; no implementation or
runtime acceptance. Root assigned `gate4_ground_oct4` as sole writer of
`codex/gate4-ground-weapon-recovery`, reusing `gate4-source-creature-control`.
Fetch and inspection found clean `28defe5a64348b819be028af13d22d3175dfc771`,
tree `fe1c1c27834746741452947cc6b82c045452b9e0`, with no interrupted edits.
The old catalog-cache branch remains preserved at619ada0. No command, schema,
source, test or admission change exists. The full before/after contract below
remains mandatory; the next checkpoint deliberately keeps all new public
ground-pickup authority closed while its shared physical foundation is developed.

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
2026-10-04 is `dbf1d633460473183324b4ec519e8d1980884b5c`; this checkout has not
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
Root read and agreed its concrete proposal. Independent review of checked plan
`28defe5` found no actionable design defect: memo
`tooling/ground-28defe5-plan-independent-review-2026-10-01.md`, SHA256
`927907546373beb0b0b59c7d2c77b7d2ec0fdf70cbd1d90aeb6f4aeb466ba9a8`.
The decisions below preserve that reviewed complete contract. The bounded
foundation checkpoint added on 2026-10-04 still requires review before source
authoring. Neither external note is implemented behavior or runtime proof.

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

1. The complete before/after plan at `28defe5` is independently reviewed. Review
   this narrower foundation amendment before source changes; it changes the
   development order, not acceptance or the required final contract.
2. Implement and review only the guarded foundation described below. All new
   producer and retained-state authority remains denied in both execution
   policies at every flow version. Pure helper controls are not accepted play.
3. Separately review and implement the owned after-equipment lifecycle, then
   connect the real application/projection/UI and ordinary physical attack path;
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

## Next bounded checkpoint: guarded physical foundation

This is a development dependency, not a pickup release. Source authoring starts
only after root reviews this exact plan commit and explicitly returns the writer
assignment. The intended checkpoint contains the following cohesive changes:

1. Add `AttackEquipmentOperation::Pickup { item, hand }` and a typed
   `AttackGroundPickupBefore` record in `dmd-domain`. The record contains the
   original complete `ItemInstance`, exact `TacticalGroundItem`, encounter,
   scene and location identities, and actual `ActorEquipmentLoadout`. Add
   `ground_pickup_before: Option<AttackGroundPickupBefore>` to
   `TacticalWeaponAttack` and `WeaponAttackReceipt`, with serde default and
   omission when None. Field names must agree across their constructors and
   reconstruction. Old choices, receipts, fixtures and fingerprints remain
   byte-for-byte unchanged. The record is evidence to check, never permission.
2. Add one crate-internal object-point admission query in the spatial module.
   Derive the observer from the actual campaign/encounter participant. Use current
   awareness, real senses, light, obscuration and exact stored object point; do
   not call creature perception with a fake participant. Sight checks begin at
   the real observer center, as existing perception does. Blindsight may establish
   object perception through clear effect; blindness excludes ordinary sight,
   and creature-only Tremorsense/contact does not establish it. Bound query work
   using the existing spatial capacity discipline. Independently require physical
   reach with a clear-effect ray from an actually occupied actor cell to the point.
   Compute ordinary five-foot reach in half-foot units from occupied cell centers,
   including vertical cells and the real Large/Tiny footprint. Test the endpoint
   convention explicitly: the Item is its stored point, not a fabricated cell or
   target volume. No extended weapon reach or host knowledge enters the query.
3. Add a rules-owned, non-deserializable preparation object whose fields and
   constructor are private to the physical-equipment implementation. Construct it
   only from the actual immutable input state, trusted origin, actor, derived
   AttackAction window and typed choice. It retains its original state borrow and
   internally derived candidate for the duration of one planning call. No external
   caller can supply a candidate, receipt, validation flag or consumed-allowance
   flag. Revalidate the complete physical before-image before any later consuming
   operation; a preparation cannot authorize a changed state. The preparation
   performs no authoritative mutation and publishes no generic candidate setter.
4. Derive a coherent candidate only after the existing admission checks: current
   active map and controller, visible/reachable intact quantity-one known weapon,
   exact Location custody and unique matching ground record, usable receiving
   hand, no conflicting holder, and no prior operation on this attack. Use one
   non-disclosing refusal class for hidden/foreign/unknown Item probes. Change
   only custody, that ground record and the chosen hand in the candidate; preserve
   Item identity, owner, quantity, definition, grant and prior drop origin. The
   sealed object derives and retains the original receipt before-image.
5. Route ordinary physical planning through this shared preparation before
   `carried_item` resolves the selected weapon. Its internal planner consumes the
   prepared equipment operation once, then applies existing grip, source, ammo,
   mastery, target and budget checks. Keep the original accepted choice in every
   receipt; no accepted command is rewritten to disguise Pickup. Old Equip and
   Unequip use their existing behavior, including the just-thrown prohibition.
   Existing source attacks reach the same physical planner; Ogre's exact-pin
   printed adapter remains separately owned and closed. A recovered weapon's
   ordinary damage definition never inherits source-creature printed dice.
6. Add a private bounded inverse shared by physical reconstruction. Validate the
   expected current Item/equipment/ground image before restoring only the recorded
   pickup transition in a local reconstruction. Reject mismatched identity, cause,
   location, custody, duplicate ground record, current hand or intervening mutation;
   never rewind unrelated changes. The inverse consumes the receipt as data and
   rederives the same sealed preparation; it does not authenticate the original
   controller/geometry cut. Only later real accepted command replay can do that.

The existing public `prepare_weapon_attack` entry and every tactical producer
continue to refuse Pickup. The lower private implementation may be tested directly
with explicitly labeled constructed inputs; that is no public execution authority.
After assembly, ordinary/source callers share the same internal physical code,
but none can reach its new branch through an admitted command in this checkpoint.
Do not add `after_equipment`, `ChooseAttackEquipment`, retained after work, a new
pump/frame branch, after decisions or after receipt fields in this checkpoint.
Their reviewed contract above remains required before eventual public admission.
Do not create a partially owned choice merely to make before Pickup runnable.

### Explicit admission and compatibility guard

Add one shared predicate for new ground records, including live physical attack
receipts (ordinary and creature sources) and all retained weapon-history entries.
Rules tactical-state validation rejects it unconditionally before execution,
including states with no encounter/flow. Domain deserialization may understand
the additive record but must not make it runnable. Rules-enabled restore also
rejects new records at the earliest anchor and through every later snapshot;
semantic replay cannot grant authority to their encoded Pickup actions. Any new
physical consumer discovered during authoring must be added to this boundary.

Guard command choices outside the Live-only execution check: ordinary Attack,
CreatureWeaponAttack, OpportunityAttack and nested Cleave weapon selections all
refuse Pickup under Live and Historical policies at flows1-5 and any unsupported
version. Cover direct public physical planning as well. A mutation which later
retires its new state cannot bypass this action check. Existing accepted retry
lookup stays first, and no historical event fixture is regenerated. Table offers
remain carried-only and app/source-controller authorization cannot enable the
new operation. Shared Grapple consumers must explicitly refuse it if present
after deliberate integration; their separately guarded authority is unchanged.

This temporary checkpoint guard is stricter than the final exact-flow5 contract.
Removing it requires a separately reviewed complete before/after producer,
original-cut replay, app/UI ownership and recovery implementation. It must not
be removed merely because pure helper tests pass. No source admission changes,
executor allocation, implicit upgrades, new public inventory mutation or direct
state edit are part of the checkpoint.

### Checkpoint controls and handoff

Author controls for new/old serialization; private preparation with selected and
different weapons; exact identity/owner/grant retention; supporting/reserved hands;
duplicate ground records and stale custody; ordinary damage; geometric boundary,
height, occlusion, darkness and special-sense cases; same refusal for hidden and
unknown probes; private inverse round-trip and rejected forged before/current
images. Label constructed helper images as such. Assert complete input equality
on every failed operation. Do not present direct helper output as admitted play.

Add unconditional public refusal controls for both policies, every relevant
producer (including nested choices), new records in current/history/anchor states,
and new raw data with superficially matching requests. Preserve old absent-field
round-trips and original history bytes. Check all exhaustive enum consumers rather
than granting new behavior to silence a match error. This checkout predates the
private Grapple resolver branch; reconcile that consumer with its writer explicitly.

Source handoff must identify the exact clean commit/tree, full diff, authored
controls, unchanged corpus hashes and remaining acceptance. Root owns the heavy
slot; no Cargo/npm/rustc/build/test, database, native run, CI mutation or push is
authorized for this assignment. Root schedules focused domain/rules/app checks
after static review, followed by the complete slice's canonical verification.
Until actually run, all authored controls remain UNRUN. Public positive pickup,
after-work ordering/Decline, real SQLite/cold/portable replay, UI and packaged play
are explicitly UNMET and remain later checkpoints in this same Gate4 objective.

## Acceptance and current limits

| Boundary | Required evidence |
| --- | --- |
| Real custody | Same original Item changes Location to Entity custody once; actual ground record removed, grant/owner/prior cause retained; native control reaches this route. |
| Source separation | Another actor uses ordinary1d6 Javelin or1d8 Greatclub base damage; exact Ogre source still uses its printed dice through its own adapter. |
| Economy and timing | One before-or-after allowance, no extra attack/Action; real after-interruption unavailable-choice recovery; no OA pickup; before pickup of selected or different weapon. |
| Geometry/privacy | Real map/height/hand/perception input; wrong/hidden/remote/foreign/destroyed/stale choices reject without revealing hidden facts or changing state. |
| Persistence | Real file SQLite and independent portable restore at pickup-owned choice and pending attack/damage; exact original retry, changed body/foreign actor/stale handle leave complete store unchanged. |
| Compatibility | All29 frozen fixtures and five original receiving suites retain exact bytes and execute; old Equip/Unequip and source pins/fingerprints remain exact. |

Status remains PLAN ONLY. The complete `28defe5` contract is independently reviewed;
this 2026-10-04 bounded foundation amendment still needs root review. No schema,
source, test, UI or source admission change and no runtime evidence exists. Git
whitespace/static diff and non-document identity checks are the only verification
appropriate to this amendment. Root owns the heavy slot. Exact next action: return
this clean plan commit and writer ownership to root for review before any bounded
source-authoring assignment. Keep new ground authority and Ogre creation closed.
