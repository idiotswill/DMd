# Gate 4 — Guarded attack equipment continuation

Status: PLAN ONLY, 2026-10-04. Root transferred sole plan writing to
`gate4_ci_oct4` in `gate4-ground-equipment-lifecycle`, branch
`codex/gate4-ground-equipment-lifecycle`, from clean
`8ec12c3b1b24fe0d1ab28abd17bad99c37be4492`, tree
`491df9770cd150bc198d6fcd3e1e401b1741d0cc`. No source changes, compiler/test
execution, database/native work, publication or CI operation are authorized by
this assignment. Root and an independent peer must review this concrete plan
before root explicitly transfers source writing. Root owns the heavy slot.

The parent [ground recovery plan](gate4-ground-weapon-recovery.md) remains the
full before/after acceptance contract. Its guarded foundation PR57 is frozen in
its separate checkout. This branch neither edits that checkout nor treats its
development parent as accepted main. Reconcile prerequisites and actual fetched
main normally before eventual acceptance; never merge into an unaccepted stacked
development parent as a substitute. There is no PR for this follow-on branch yet.

Root read and accepted the scope of the source-grounded proposal
`tooling/ground-8ec12c3-next-lifecycle-proposal-2026-10-04.md`, SHA256
`da2a7af9908108f09c154da57bd1eb480d927474a291ef8a04ca21c27f2ef5ea`.
That proposal and the fifteen foundation controls provide navigation and static
design evidence, not executed pickup, replay or acceptance evidence.

## Objective, authority and boundaries

Implement the actual private rules-owned after-equipment continuation: a real
physical attack completion produces bounded work; the shared pump selects it
after its consequences; its current controller Applies one equipment operation
or Declines; the decision retains its exact physical evidence and has a narrow
local inverse. This is one cohesive producer/consumer checkpoint, not a schema
checkpoint or a before-only pickup release.

This advances product-definition requirements for authoritative rules, complete
tactical timing, physical object identity, player control, hidden-information
boundaries and exact suspension. It follows ADR018 typed immutable-input rules
transitions, ADR020 original-history restore preflight and ADR024 trusted table
commands/idempotence. Read those ADRs, root AGENTS, product definition,
[Gate4](../../checkpoints/gate-04-tactical-encounters.md) and the
[gate protocol](../../checkpoints/gate-execution-protocol.md) before writing.

The pinned SRD5.2.1 contract and before/after allowance in the parent plan remain
unchanged. Only a normal physical attack on the actor's own AttackAction window
may privately opt in; no extra Action, attack, Item or inventory grant is created.
Existing Equip/Unequip semantics and Thrown intrinsic draw remain unchanged.
The new continuation does not expand Opportunity, Reaction, BonusAction, Nick,
nested Cleave, Ready, intrinsic, spell or Grapple authority.

Public action, retained-state and rules-enabled restore admission stay closed
unconditionally for every new ground/after-equipment choice or record under both
Live and Historical policies. Internal admission additionally requires exact
flow5; no executor allocation, implicit migration, source pin change or historic
raw-tag change. No runtime flag, privileged bypass or deserializable preparation
may turn the private checkpoint into a second execution API.

App/UI offers, real before-pickup commit, complete ordinary/source integration,
original accepted history, file SQLite/cold/portable recovery and packaged player
play remain required later in this same Gate4 objective. None is discharged by
private producer tests. General travel/economy remains Gate5; this plan moves no
existing Gate4 acceptance there.

## Concrete typed contract

Use additive default/omit-None fields, preserving old absent-field JSON and
fingerprints exactly. Types below fix authority and lifetime; mechanical naming
may follow adjacent domain conventions without changing this contract.

| Addition | Required content and owner |
| --- | --- |
| `AfterAttackEquipmentIntent::Choose` | `after_equipment: Option<_>` on both `WeaponUseChoice` and `CreatureWeaponUseChoice`. Mutually exclusive with every explicit `equipment_change`, before or old-after. No default pause. |
| `TacticalAction::ChooseAttackEquipment { work, choice }` | Exact `TacticalWorkKey`; choice is `Decline` or `Apply(AttackEquipmentOperation)`. No source/position/cost/before-image/allowance/post-state input. |
| `TacticalWorkKind::AttackAfterEquipment` | One non-roll occurrence registered through the existing bounded work allocator and trace. Allocate no raw-roll role, tag or request. |
| Optional `TacticalAttackAfterEquipment` on `TacticalResolution` | Its work item/key, original attack identity/actor/turn/window, actual submitted physical choice and typed physical source identity, actual completion command and completion-parent key, and queued/selected stage. Selected stage retains the exact selecting command. No copied damage program or replacement attack. |
| `after_equipment_parent: Option<AttackEquipmentCompletionParent>` on `TacticalWeaponAttack` | Default/omit-None. Contains `work: TacticalWorkKey`, `pause: Knockout | Graze`, `paused_by: CommandMeta`, `accepted_raw: Option<RollRequestId>` and `suspended_outcome: WeaponAttackOutcome`. Only an opted-in actual suspension writes it; no declaration-time or predicted parent. |
| Optional final after decision on `WeaponAttackReceipt` | The same cause/work/selection binding plus trusted decision command, `Declined` or `Applied { operation, equipment_before, ground_before? }`. Ground evidence is required exactly for Pickup and preserves the actual ground vector index. No unfinished final decision is fabricated at declaration. |

The after cause must retain enough to compare the original physical choice,
actor/window and source with the actual completed history entry, and to replay
the original declaration later. For ordinary sources record that source kind;
for a printed physical creature source retain its exact source pin and feature
identity. Do not infer unused allowance from today's budget or from a receipt
that currently omits the original equipment choice. Derive it from the real
declaration before the attack is cleared. The final receipt carries the required
cause even after its resolution and temporary selected record retire.

Exactly one matching completed physical history entry must exist. Bind its
origin, actor, turn, window, weapon, target, delivery, ability, grip, ammo, purpose
and final outcome. Reject duplicate/missing entries, a Pending outcome, different
source/choice, any spent explicit equipment allowance or duplicate after record.
Work resolution identity is the original resolution origin; occurrence is the
actual allocated one. Completion/selection/decision commands must have the right
campaign, actor/session authority and causal order; a completion-created ground
origin may follow declaration and still legitimately precede the later Pickup.

## Producer and actual completion ancestry

Extend `tactical/attacks::complete` at the existing physical completion cut.
Preserve old after-equipment, ammo, same-Item thrown custody and ground creation,
and final weapon-history outcome. Then derive/register one after occurrence for
the explicit legal unused allowance and clear `resolution.attack`/`hit_review`
as today. No live fake Finishing attack may survive to reconstruct a thrown Item
as carried, repeat damage or repay a cost. No opt-in means the old path is exact.

The after frame is pushed before `apply_damage`/Graze pushes its resulting
vitality, effect, concentration and fall children. It is a later singleton frame,
never a simultaneous sibling of those consequences. Existing pump flight-loss
derivation still runs before it selects later work. Queued presence is not a
waiting condition. Same-attack Pickup uses the ground record actually created
by completion, including its completing command, not a pre-attack receipt.

Ancestry must cover every real completion route:

1. Ordinary miss/automatic miss completes while the existing `FinishAttack`
   occurrence is entered by `continuations::start`.
2. Ordinary rolled or fixed damage completes inside the actual `AttackDamage`
   start/finish scope. Hit-review resume retains its genuine existing ancestry;
   do not replace that cause with the current command or invent an occurrence.
3. When actual damage pauses at KnockoutChoice, retain that exact entered
   AttackDamage key only for the opted-in attack. The direct
   `choose_knockout` continuation validates that retained key/node, attack/raw
   identity where applicable and stage, enters it before completion and vitality
   children, and restores the previous in-process marker on success or error.
   Keep the existing distinct vitality occurrence allocation and command cause.
4. When `FinishAttack` pauses at MasteryChoice, retain that exact entered key.
   Direct `choose_mastery` validates and re-enters it around completion and
   Graze consequences, including Decline, then resets the marker. Do not select
   a merely recent node of the same kind. Existing non-opted-in behavior stays
   unchanged.

Concretely, initialize `after_equipment_parent` to None at physical admission.
At `apply_damage`'s real `preview.awaiting_choice` branch, before returning the
KnockoutChoice stage, require the active trace node to equal the entered
AttackDamage occurrence and write the parent record with `pause = Knockout`,
the current trusted `meta`, current Hit/critical outcome and `attack.damage_roll`.
If a damage raw exists, its request must equal the attack's deterministic
AttackDamage key for this parent, and its accepted result must be the one used
by the actual damage preview. Fixed damage has no invented raw result. At
`finish`'s actual Graze MasteryChoice branch, capture the entered FinishAttack
key, `pause = Graze`, current `meta`, Miss outcome and `attack.attack_roll`;
an automatic miss retains None. A present attack raw must match the exact attack
request/origin and its miss result; the FinishAttack node must belong to that
attack's actual trace lineage. Neither branch allocates a completion/follow-up
work item at this pause: the attack is still incomplete.

On direct knockout or mastery choice, require the retained pause to match the
current stage, unchanged attack origin/source/intent, suspended outcome and
accepted raw identity; verify the full exact parent node and same resolution.
Retained `paused_by` must bind the command that actually established that pause,
not the later chooser. Enter this node around completion and its child production,
then restore the prior trace marker before calling pump. The actual chooser is
recorded as `completed_by` in the new after cause; copy the validated parent key
there before the completed attack (and temporary parent record) is cleared.
For non-pausing paths obtain the completion key directly from the current entered
node. No latest-kind lookup or synthetic completed node is permitted. Local raw,
stage and lineage checks do not replace eventual original accepted-command replay
of the pause and decision. With absent intent, never write the optional field or
wrap the direct continuation in new ancestry behavior; preserve its existing bytes
and execution path exactly.

The new work's parent is the verified completion node. Direct choice commands
remain their actual completion metadata; they do not impersonate earlier raw
submissions. Restore/current validation recognizes these new optional parent
records, but the checkpoint's unconditional public guard still rejects them.
The transient trace `active` field must be reset at transition boundaries and
cannot be used as a serialized proof. Test wrong/stale/sibling parent forgeries.

## Selection, ownership and retirement

Add a cohesive private `tactical/attack_equipment` module (or an equivalent
focused submodule) for the producer, selection, decision and validation. Use
the existing `turns::push_frame`/`work_trace::register`; do not introduce another
queue. `continuations::start` selects the exact work into its retained record,
removing it from the frame exactly once. Only Selected makes the pump wait.

Extend both pump waiting/empty-frame handling, `turns::choose`, turn validation's
selected count/work inventory/non-roll suspension, and work-trace live-occurrence
inventory. Validate exactly one queued occurrence before selection and exactly
one selected retained owner afterward, with no duplicate frame/pending/raw work.
The selected choice is not `ChooseTurnWork` and has no raw roll. Treat it as
independent actor response authority in trace ownership; no inherited area-host
ordering consent can authorize the decision. The resolution must not retire,
accept another attack, EndTurn, Conclude or Finish while this work is unanswered.

The private decision boundary validates exact flow5, campaign/head, selected key,
cause/history identity and current trusted controller/session authority before
applying a choice. Decline performs those checks without `can_act`, a living
actor, free hand or available Item requirement. The retained actor's controller
must be able to decline after incapacity/death; former item ownership and the
current turn actor alone are not continuation authority. Eventual app attendance
and opaque session binding remain separate mandatory integration checks.

Apply derives current legality at the decision command's cut. On success, first
record the final decision on the exact history entry, retire the selected record
and resume the same pump. Rejected Apply has no state mutation: the same choice,
prior damage, ammo, attack budget, ground history and all consequences remain
unchanged. Duplicate/stale decision commands cannot apply the operation twice.
Actual public retry lookup remains ahead of future admission, as ADR024 requires.

## Sealed physical preparation and decision inverse

The existing `PreparedPickup::new` is a before-attack constructor: it requires an
available attack and `plan()` applies BeforeAttack/Normal. Do not relax it with a
caller allowance flag or run it against a fabricated unpaid state. Extract only
the shared physical derivation under a private owning module. Add a separate
after constructor that can be reached only from the validated selected record
and actual trusted decision command; it rederives the already-earned allowance
from that record without requiring attacks remaining or an unspent Action.

The preparation holds the original state borrow and complete physical before-image
for one call. It cannot be deserialized, created with a supplied candidate, or
consumed against a different image. It derives active encounter/scene, actual
current capability, source-proven anatomy and EffectiveHands, exact Item identity,
intact quantity-one definition, custody/unique ground record, current object
knowledge, ordinary five-foot point reach, unobstructed access and hand capacity.
Reuse the foundation's uniform non-disclosing refusal and spatial object query.
Equip/Unequip rederive current carried custody/occupancy through their real
equipment logic, without constructing an attack grip or repeating old after work.

Consume only the derived same-Item custody/ground/loadout delta. Preserve owner,
quantity, definition, original grant and earlier drop/throw cause. Do not alter
HP, ammunition, attack cost, motion or any unrelated inventory. An unavailable
originally intended item is not auto-replaced; the controller can select a legal
alternative or Decline. Pickup of the same just-thrown weapon is allowed only by
this later actual ground route; old Equip's just-thrown prohibition remains.

Separate the existing guarded public physical planner from the crate-private
production preparation used by tactical internals. Both use the same physical
calculation; the public wrapper retains its unconditional new-authority refusal.
Private producer tests must reach that actual preparation and completion path,
not a test-only alternative implementation. Do not expose an accepted-state
setter or a test/runtime bypass flag. Shared intent validation preserves the
normal-purpose own-turn AttackAction and equipment-change mutual exclusion.

Add a distinct after-operation inverse, not an extension that rewinds the attack.
At the immediate decision cut it validates the recorded cause/selection and exact
expected current equipment and Item/ground post-image, then locally restores only
this operation's before-image and original ground-vector position. Equip/Unequip
restore only their actual equipment delta; Pickup additionally restores its one
physical Item and ground entry. Reprepare from that local decision-cut image and
compare the derived operation/evidence. Reject changed custody, wrong ground cause
or index, forged/current hand images and unrelated edits to the claimed delta.

Never reset attack sequence, HP, spent ammunition, conditions, position, fallen
state or completed receipts to declaration time. Do not demand today's equipment
equal a much older final receipt after later legitimate play. This inverse is
local consistency at the decision cut; exact accepted command replay must later
authenticate original admission and every intervening cut. Matching forged
request/raw/receipt copies cannot become positive history proof (ADR020).

## Fail-closed consumer integration in this checkpoint

Extend `has_unimplemented_ground_records` (or a clearly named shared superset)
to detect new declaration intent, temporary completion-parent evidence, queued or
selected after work even if its record is missing, retained after records and
final decisions in every history entry. Keep kernel/tactical validation and
rules-enabled restore guards effective at earliest anchor, later snapshots and
current state, including missing encounter/flow or malformed ownership shapes.

Extend the common action guard outside Live-only admission to include the new
choice command itself, including no-effect Decline, ordinary/source attack intent,
OA intent and nested Cleave intent. Retain before Pickup's existing guard and
reject declaration-time AfterAttack/Pickup. Both policies reject all new public
authority at flows1–5 and unsupported versions. Direct public weapon preparation
rejects new choices and retained evidence too. No old fixture or retry is rewritten.

Propagate the optional intent through `creature_weapon::begin_creature_weapon`'s
conversion into `WeaponUseChoice`; preserve source eligibility, exact printed
pin, activation, range and damage. Every old constructor explicitly supplies None,
including ordinary/source OA and test helpers. Explicitly refuse new intent in
OA and non-normal/nested Cleave consumers even behind the public guard, so later
guard removal cannot accidentally create allowance. Separately added Ogre/Grapple
consumers require deliberate integration with their writers; do not edit their
checkouts or lift their existing admission. New enum compilation grants nothing.

No app option or UI is enabled. Update only additive type consumers needed to
compile and unconditional negative guards/controls. `rules_restore::command_origins`
must eventually enumerate removed-ground/equipment origins, completion/selection
and decision metadata, bound to exact audit/action/event chronology; full accepted
restore integration remains blocked here. Do not present a record walker as replay.

## Meaningful controls and execution sequence

After exact plan review and explicit source-writer transfer, first record that
review/status in this plan, then implement the above as one coherent guarded
checkpoint. Freeze a clean commit for independent complete-diff review before
root allocates any compiler/test slot. Controls are authored UNRUN until their
actual commands finish and logs are read. Do not weaken existing assertions.

1. Old absent fields serialize identically; old Equip/Unequip and Thrown timing
   stay exact. New intent/explicit operation conflict and wrong purpose/window
   refuse before payment. Positive constructed inputs are labeled private.
2. Drive actual internal attack begin, raw resolution and completion for miss,
   hit, fixed damage, knockout Apply and Graze/Decline. Assert one work/receipt,
   real completion parent and metadata, cleared attack, no second damage/ammo/cost.
   Do not insert a selected work item or completed receipt as positive setup.
3. Drive actual vitality, concentration, fall and unconscious-drop children before
   selection; assert bounded ancestry/frame ordering and no early queued wait,
   simultaneous ordering escape, re-equip after drop or inherited area authority.
4. Actually throw through completion, then recover that same Item from its actual
   newly created ground origin. Preserve owner/grant/definition/quantity/vector
   order/history. Cover a legal different ground item and carried Equip/Unequip.
5. Prove successful Apply with the last attack already spent. Occupied/reserved
   hand, changed custody, invisible/blocked/distant ground point or lost capability
   refuse with complete input equality and selected work retained. Decline still
   succeeds with zero legal items or a dead/unconscious controlled actor.
6. Wrong actor/session/campaign/head, resolution/occurrence, completion parent,
   source/history binding, duplicate work/decision and stale command reject.
   Full no-write assertions include earlier payment and all actual consequences.
7. Round-trip the after inverse at its real constructed decision cut; preserve
   unrelated HP/ammo/movement/conditions/ground entries. Reject forged before or
   current images and declaration-time geometry substituted for decision truth.
8. Exercise actual public dispatcher/planner and restore-preflight refusals for
   both policies, all relevant flow versions, nested OA/Cleave choices, pending
   parent/queued/selected/final/anchor records and Decline that would erase itself.
   Retain all fifteen foundation controls and explicit shared consumer refusals.

Private tests may construct the original baseline character/map image, clearly
labeled; subsequent work/selection/throw/drop/decision evidence must come from
the real private producers. Such fixtures establish internal behavior, not an
accepted command journal, real source admission, durable store atomicity or UI.

Root later allocates `cargo fmt --all -- --check`, strict changed-crate all-target
Clippy and focused domain/rules/app tests. Record exact test names/counts and
reject zero-match results. Preserve all29 frozen fixtures and execute all five
original receiving suites unchanged. Use `./scripts/verify-fast` during allocated
iteration and `./scripts/verify` before implementation-complete claims. All
canonical checks, exact-head Linux/Windows CI and receiving-head verification
remain required; no command listed here is permission to run beside root's slot.

## Remaining production obligations and handoff

Before public activation, consume the actual sealed before-pickup transition in
`attacks::begin_with_source`: today it commits planned loadout/ammo but initializes
the ground before-image to None and never transfers ground custody. Removing its
guard alone is incorrect. Commit the same Item/ground change atomically only after
all source/target/hand/ammo/budget checks, retaining authentic images in pending
and history receipts. Cover selected and different picked-up weapons.

Integrate ordinary and separately reviewed printed-source attacks through that
one physical boundary. Another actor's recovered Javelin/Greatclub keeps ordinary
damage; the source actor's exact printed dice do not become Item properties.
Reconcile all source/OA/Cleave/Grapple fields and refusals deliberately. Full Ogre
admission remains gated on all of its original printed forms, finite gear and OA.

Add player-safe ground options and an independent item/hand selector, explicit
later-choice opt-in, and a separate owned selected-after view because the completed
attack is absent. Route session/attendance/source control through the retained
actor and exact opaque work handle. Decline is available with no legal Apply
options. Preserve exact payload/command identity across uncertain acknowledgment,
close/reopen and focus changes; no client-supplied mechanics or hidden-ID leaks.

Then prove genuine admitted original history with real file SQLite, exact accepted
retries, complete-store no-write negatives, cold reopen and independent portable
restore from the earliest actual anchor at pending attack/damage, selected after
work and final decision. Include same-attack actual throw/recovery, another actor
recovering the original source Item, intervening damage/fall/hands/custody changes
and finite-gear reuse without extra attacks. Preserve all old capture bytes and
five original replay suites; synthetic positive state/receipt edits cannot replace
them. Packaged native player/source pickup and cold continuation, independent
full-diff review, canonical checks, accepted-main integration, exact receiving-head
CI and protected merge remain mandatory. Gate4 stays active until its full gate
review; no next gate begins on this checkpoint.

Current validation: plan/source inspection only; implementation absent and every
new control UNWRITTEN/UNRUN. Risks requiring review are causal parent retention,
queued-versus-selected waiting, spent-budget admission, old serialization,
decision-cut inverse scope and cross-branch source constructors. No acceptance
is reduced. Exact next action: root and peer review this clean plan commit/tree;
resolve findings in the plan; only then transfer source writing explicitly.
