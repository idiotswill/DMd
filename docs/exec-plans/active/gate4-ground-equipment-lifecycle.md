# Gate 4 — Guarded attack equipment continuation

Current status: draft PR60 at aff47ba failed its first actual CI compile. The
private validator findings are resolved by static review; a moved receipt and
incorrect test session type need the narrow correction recorded below. Root is sole
writer, 2026-10-04. All23 rules controls and the app preflight remain UNRUN;
runtime and production acceptance are not established.

Historical source transfer: root explicitly
transferred sole SOURCE-WRITER ownership to `gate4_ci_oct4` at clean amended plan
`caded2228bb841bec6bf056f0d0e7a7202b95b49`, tree
`e32451a35397054ecf95d3620a7b77796e8d5190`. Root's complete plan review is CLEAR:
`tooling/ground-caded22-root-plan-review-2026-10-04.md`, SHA256
`368a28f1017c5db42bcdfdf5cff4b783dca7e9a11c986fbff281137750357dec`.
Independent amendment rereview is CLEAR:
`tooling/ground-caded22-plan-amendment-independent-review-2026-10-04.md`, SHA256
`a75778e24764e750960d422515961b7a1be93d6cb5a10e065700637ecb02bb33`.
Both complete reviews were read. This status is committed before source edits.
Direct rustfmt and Git/static inspection are authorized; Cargo/compiler/tests,
npm/build, database/native, publication and CI operations are not. Root Air87840
retains the sole heavy slot. New controls are now AUTHORED/UNRUN; see the source checkpoint below. No other
checkout is writable, and every new public/restore authority stays closed.

Original plan-only status, retained as history (superseded by source transfer):
2026-10-04. Root transferred sole plan writing to
`gate4_ci_oct4` in `gate4-ground-equipment-lifecycle`, branch
`codex/gate4-ground-equipment-lifecycle`, from clean
`8ec12c3b1b24fe0d1ab28abd17bad99c37be4492`, tree
`491df9770cd150bc198d6fcd3e1e401b1741d0cc`. No source changes, compiler/test
execution, database/native work, publication or CI operation are authorized by
this assignment. Root and an independent peer must review this concrete plan
before root explicitly transfers source writing. Root owns the heavy slot.

Review amendment, 2026-10-04: root transferred sole PLAN-ONLY writing from
`gate4_ci_oct4` to `gate4_ground_oct4` at clean
`0c3ac0028d916aec5c5b14dded2af2c1baee6bd7` to resolve the two findings in
`tooling/ground-0c3ac00-lifecycle-plan-independent-review-2026-10-04.md`, SHA256
`ec6fd606e068b56601e3f811cc81b730894f9782b45f2bbba57ecf1ee47e16d6`.
This amendment requires root and a different peer to rereview the exact clean
commit before any source transfer. The amendment author does not self-certify
independent review. All other branches remain frozen.

The parent [ground recovery plan](gate4-ground-weapon-recovery.md) remains the
full before/after acceptance contract. Its guarded foundation PR57 is frozen in
its separate checkout. This branch neither edits that checkout nor treats its
development parent as accepted main. Reconcile prerequisites and actual fetched
main normally before eventual acceptance; never merge into an unaccepted stacked
development parent as a substitute. Root will publish a draft against this exact
development parent for first CI, without merging into that unaccepted parent.

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
vitality, effect and concentration children, including immediate unconscious
drop/end-concentration changes. After those successful changes, explicitly call
`falling::queue_losses` while the actual completion parent is still entered and
before leaving that scope. Its later-pushed fall frame therefore precedes the
after frame without making the two simultaneous siblings. The pump fallback
must not be the first producer of a flight loss caused by this completion:
correct frame order alone does not prove the BeginFall node's parent. Queued
presence is not a waiting condition. Same-attack Pickup uses the ground record
actually created by completion, including its completing command, not a
pre-attack receipt.

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
   children, queues resulting flight losses before leaving, and restores the
   previous in-process marker on success or error.
   Keep the existing distinct vitality occurrence allocation and command cause.
4. When `FinishAttack` pauses at MasteryChoice, retain that exact entered key.
   Direct `choose_mastery` validates and re-enters it around completion and
   Graze consequences, including Decline, queues resulting flight losses before
   leaving, then resets the marker. Do not select a merely recent node of the
   same kind. Existing non-opted-in behavior stays unchanged.

Concretely, initialize `after_equipment_parent` to None at physical admission.
At `apply_damage`'s real `preview.awaiting_choice` branch, before returning the
KnockoutChoice stage, require the active trace node to equal the entered
AttackDamage occurrence and write the parent record with `pause = Knockout`,
the current trusted `meta`, current Hit/critical outcome and `attack.damage_roll`.
If a damage raw exists, its request must equal the attack's deterministic
AttackDamage key for this parent, and its accepted result must be the one used
by the actual damage preview. Fixed damage has no invented raw result. At
`finish`'s actual Graze MasteryChoice branch, capture the entered FinishAttack
key, `pause = Graze`, current `meta`, Miss outcome and the actual accepted
`attack.attack_roll`. The Graze pause requires that real raw to match the exact
attack request/origin and its miss result; the FinishAttack node must belong to
that attack's actual trace lineage. Neither branch allocates a completion/follow-up
work item at this pause: the attack is still incomplete.

Preserve `attacks/validation::validate`'s blanket rejection of retained
`automatic_miss` attacks. Current `tactical_weapons` derivation produces automatic
misses only for underwater ranged/thrown delivery beyond normal range; the only
bundled Graze weapons, Glaive and Greatsword, are melee with no range, and ordinary
creature-weapon planning grants no mastery. Thus automatic-miss Graze suspension
is not a reachable positive source path and stays closed. A real non-Graze
automatic miss completes immediately through its entered FinishAttack node,
without an attack raw or a pause-parent record, and may produce the opted-in
after work there. Test this separately from real rolled-miss Graze Apply/Decline.
Do not change a source profile, inject this combination into positive setup or
add an unreachable validator exception; a future genuine source combination
requires a separate reviewed contract.

On direct knockout or mastery choice, require the retained pause to match the
current stage, unchanged attack origin/source/intent, suspended outcome and
accepted raw identity; verify the full exact parent node and same resolution.
Retained `paused_by` must bind the command that actually established that pause,
not the later chooser. Enter this node around completion and its child production.
On success call `falling::queue_losses` before leaving the entered node, then
restore the prior trace marker on both success and error before calling pump.
This follows the existing `continuations::{start,finish}` wrappers: their loss
queueing occurs inside the scope, whereas `apply_vitality_from_cause` does not
queue losses and the later `turns::pump` fallback cannot recover a departed
parent. Keep the existing distinct vitality occurrence; no synthetic completion
or fall-parent occurrence is added. The actual chooser is
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
2. Drive actual internal attack begin, raw resolution and completion for rolled
   miss, hit, fixed damage, knockout Apply and rolled-miss Graze Apply/Decline.
   Separately drive a real underwater non-Graze automatic miss, with no fabricated
   attack raw or pause-parent record. Assert one work/receipt, real completion
   parent and metadata, cleared attack, no second damage/ammo/cost. Retain the
   existing restored automatic-miss/natural-twenty refusal. Do not alter source
   profiles or insert a selected work item/completed receipt as positive setup.
3. Drive actual vitality, concentration, fall and unconscious-drop children before
   selection; assert bounded ancestry/frame ordering and no early queued wait,
   simultaneous ordering escape, re-equip after drop or inherited area authority.
   Include real direct-choice damage/incapacity or concentration ending that
   causes a non-Hover flight loss. Assert the BeginFall node's exact retained
   AttackDamage/FinishAttack parent and its frame above after work, not only final
   execution order/counts; check transient-marker reset and outer atomicity on
   errors. Absent intent keeps its original path and bytes.
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

## Source checkpoint authored, 2026-10-04

The bounded private lifecycle is authored. The original source-writing status
commit is f7510731038ae85f58030566bb7fc8141ba60f77. No new command or state is
publicly admitted. There has been no Cargo/compiler/test/npm/native/database/CI
execution in this assignment; all new controls are AUTHORED/UNRUN. Direct rustfmt
and Git whitespace/constructor/diff inspection are static evidence only.

Implementation joins the real attack completion and shared continuation stack.
Normal ordinary and exact printed physical CreatureAction declarations propagate
the omitted intent. Actual pauses retain entered AttackDamage/FinishAttack keys;
direct choices re-enter those parents and queue flight losses before leaving.
After work is queued before vitality children, selected only after they finish,
and is not a raw roll or simultaneous ordering option. Selection retains its
own actor response authority. A player-controlled actor's current controller
must make the decision; host identity is not a substitute. Decline does not
require capability or any legal equipment operation.

The decision prepares and consumes a borrowed physical candidate atomically.
The inverse is restricted to its still-selected immediate decision cut and
compares the complete restored input, preserving all unrelated attack results.
Same-attack thrown-item evidence binds its actual completion ground origin.
Equip/Unequip use the existing carried-item operation body. No fresh attack budget,
new damage, source profile, before-pickup custody commit or public option is added.
Final receipt consistency is not authenticated historical replay, and its inverse
must never be applied to equipment after later play.

Twenty-one rules controls are authored in tactical/attack_equipment_tests.rs.
They drive actual private begin/raw/completion/decision producers for misses,
rolled and fixed hits, real underwater automatic miss, knockout, Graze, actual
throw/recovery, source-built Goblin physical intent, source-built Chimera flight
loss, concentration and unconscious drops. They also cover selected ownership,
no raw/simultaneous/duplicate selection, stale/foreign/controller/session/cause
refusals, inverse forgeries, omitted fields and public guards. The initial images
are explicitly constructed private fixtures. Canonical build_character and
build_creature produce Fighter/Goblin/Chimera source facts; no source definition
is edited to manufacture an automatic-miss Graze combination. Their subsequent
continuation records are produced by actual implementation, never inserted as
positive setup. These fixtures do not authenticate accepted journals.

One additional app unit control calls the real validate_rules_export preflight
on forged intent, temporary parent, orphan queued work, selected record and
retired final receipt in current, later-snapshot and earliest-anchor positions.
This is a negative pure preflight control, not a database or portable-play proof.
The existing foundation preflight test remains intact. The common guard also
covers orphan trace nodes, and app work projection/source control expose no new
command or card. Both policies and flows1–5/unsupported versions stay closed.

Mechanical None initializers were required in ordinary source-compiled rules,
domain and app fixture constructors. They preserve existing assertions and absent
JSON; the five protected legacy receiving suite files, frozen captures/raw
artifacts and content files were not edited. The original fifteen foundation
controls retain their bodies/assertions apart from the necessary None initializer
in the existing action-guard constructor. Record exact blob-parity evidence in
the clean handoff; no replay pass is claimed.

Static risks for independent complete review: actual direct-parent/raw lineage,
selected versus queued ownership, physical inverse boundary, source feature pin
binding, strict current-controller authority, additive constructor/exhaustive-match
coverage and old absent serialization. All source/new tests still require actual
compiler/Clippy/execution results. Tests may reveal fixture or implementation
errors; no passing result is inferred from authored assertions or rustfmt.

Exact next action: freeze and hand the clean source commit/tree and full diff to
root for independent complete review. Root owns any later execution allocation
and publication. Do not run beside its heavy slot. Before-pickup source custody
integration, application offers/session admission, all original replay/cold/store
and portable recovery, native acceptance, canonical checks, receiving-head CI and
accepted-main integration remain mandatory. Gate4 is not complete.

## Review correction plan — 2026-10-04

Root read the full independent source review
`tooling/ground-58077e7-complete-independent-review-2026-10-04.md`, SHA256
`36268f2c29250953fc0d4874f20a7c8acfa48677d9f2e49df09d36247510f84e`,
and independently completed the full source/control review. No public bypass was
found: the existing public and restore guards reject both shapes below. Correct
the private consistency contract before publication, preserving all old controls.

1. Require a unique relevant completion-kind node for the admitted own-turn single
   physical attack. A forged second FinishAttack under the genuine AttackRoll can
   otherwise replace a real Graze pause's parent while preserving kind and ancestry.
   Keep exact accepted-raw identity and causal path checks. Apply uniqueness at
   pause validation, actual completion and retained after-record validation; do not
   choose the newest node or add a serialized receipt. Shield's real SpellProgram /
   ResumeHit work remains distinct. The raw=None branch is checked without inventing
   a positive fixed-damage melee Knockout source.
2. Inventory all AttackAfterEquipment trace nodes at stable private boundaries.
   The queued/selected record must own exactly one matching trace occurrence; no
   record means no orphan trace occurrence. Preserve immediate selected-cut inverse
   and actual decision/pump retirement. No alternative queue or schema is needed.
3. Add real-producer Graze same-kind sibling rejection and selected extra-retired /
   trace-only orphan negatives. Prove the counterfeit is a well-formed generic DAG,
   then assert actual private continuation rejection and complete unchanged state,
   including transient marker, raw and earlier cost. Keep the old wrong-kind and
   duplicate-frame negatives and all existing positive consequence/choice controls.

This plan precedes source changes. Only the private runtime, its new controls and
this plan may change. No source/content/fixture/public-admission/profile/workflow
change, compiler/test/native/DB execution or publication is part of this correction.
Root Air87840 retains the heavy slot. Freeze the fix for independent delta review;
all previously recorded production integration and Gate4 obligations remain open.

Correction authored after plan5b0393f: one shared private completion-node check
requires a unique relevant kind at pause, resume validation, completion and retained
after validation. The after validator now inventories every trace node of its kind
as well as live work. Two new controls use actual producer pauses/selections and
generic-valid DAG counterfeits; old21 controls remain unchanged. All23 rules controls
and the app preflight control remain UNRUN. Direct rustfmt/check and Git whitespace
checks pass; these are static checks only. Exact fix awaits independent delta review
before publication or actual compiler/test allocation.

## Reviewed correction and first-CI checkpoint — 2026-10-04

The preceding authored checkpoints are historical. Corrected source is
`6adbd3c2e831b8cd158f44c9a332f019f990f90f`, tree
`85c69146e4fd2c4ad2884a8263a3a598292b8e96`, after plan-first5b0393f.
Root completed the full original source review and read the full independent
review that identified the two findings, then authored the bounded fix.
The independent correction review is CLEAR:
`tooling/ground-6adbd3c-correction-independent-review-2026-10-04.md`, SHA256
`a5b908992f073004e3fb6268d089a454ae8c54610921c455db0fae6758e2d72c`.
Its audit-v2 SHA256 is
`84fc899c25d6a1cb952a08ca08852903b6711b13baee5f004885c8660afb721e`;
the complete correction patch SHA256 is
`c61ea0db16dedcf12c1881eb66049a0337cb7798823b44e210a25da653d4d411`.
Root read the whole correction review. No additional static blocker remains.

The review checks the unique real completion parent and the complete after-work
trace inventory, including selection/inverse/pump retirement. Both controls use
actual private producers and prove generic DAG validity before private rejection.
All21 earlier controls remain unchanged, now23 plus the unchanged app preflight.
All42 protected blobs and21 raw captures retain their exact prior identities.
No public/restore guard, content input or accepted fixture changed.

This documentation checkpoint changes no source. First Linux/Windows CI will
provide compiler and runtime evidence on its own exact head; all new controls
are still UNRUN locally and no passing result is inferred from review. Root's
Air87840 remains the sole local heavy/native slot. The complete parent pickup
objective, app/UI integration, accepted original replay, SQLite/cold/portable and
native verification, canonical checks, prerequisite/main reconciliation and
protected merge remain required. A green private checkpoint alone will not
close ground recovery or Gate4.

Exact next action: independently review this docs-only delta, publish the draft
against codex/gate4-ground-weapon-recovery at8ec12c3, inspect actual exact-head CI
output and fix concrete failures. Do not merge into the development parent.


## First CI compiler correction plan — 2026-10-04

Draft PR60 published aff47bae20679192463d341caa629d6353500353 against8ec12c3;
Linux run37213005279 / Windows37213005216 are the first actual compiler runs.
Root read actual Linux job111467794097 output, SHA256
bb0d93f31b862001d285633b343144ccd743beb8cec4b7a7ebc622ab57a028ac.
It reports E0382: assignment evaluates its receipt-moving RHS before the LHS
closure reads receipt.cause.origin.id. Copy that CommandId into a local before
assignment and use it for lookup; do not clone the entire receipt or alter choice
semantics. E0425/E0433 identify SessionId as unknown in the new test module.
The initial interpretation was a missing import. Independent source tracing then
established the actual type is PlaySessionId (domain command.rs and ids.rs); use
that existing type at both sites, without adding a new type or alias. MSRV
jobs111467794204 and111467793908 agree on the original compiler errors.
No Rust test harness ran in these failed compile jobs; Windows frontend evidence
is separate. Preserve every failed log and all assertions/fixtures.

This plan precedes the two source edits. Root will make only those compile fixes,
run direct rustfmt/Git checks, obtain independent delta review and publish a new
exact head for full CI. Air87840 keeps the local heavy slot; no local Rust/compiler
or native/DB execution is authorized alongside it. All runtime/production and
accepted-main integration obligations above remain required.

Review refinement before publication: source checkpoint78e9c2 copied the receipt
origin correctly but added an invalid SessionId import; it was not executed or
published. Independent review caught the real domain name. Root inspected both
domain declarations and amends the test references to PlaySessionId, already
available through the existing domain glob import. Preserve the intermediate
checkpoint as unverified history; only the corrected final head receives CI.
