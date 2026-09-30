# Gate 4 — Grapple and Escape lifecycle

Writer: root, following the source/domain and derived-hand author's handback.
Branch: `codex/gate4-grapple-lifecycle`. Status: **guarded foundation independently
reviewed; draft publication for first executable checks, no runnable Grapple
gameplay or executable verification yet**.
Date: 2026-09-30.

Development base: `387b74241d1870964be0e88cb0f3216e151c9b55`, tree
`ffd06e4d73cca2188014c0120b6e702df89935bb`, containing the Shove development
parent and normal integration of corrected Air source `021421c4`. These are
unaccepted development dependencies, not evidence of a merged feature. At initial
plan creation, a fresh `git fetch origin main` resolved main to
`c4d8c34c19b5c92eca789f292f99632a0107d861`. The supplied development checkout is
was clean and stays on its explicitly assigned base. PR51 and other branches belong
to root; this plan does not authorize editing them.

## Objective and authority

Implement an ordinary own-turn Unarmed Strike Grapple, its durable physical grip,
the target's Action to Escape, and source-faithful ending/release through the
existing rules, table, persistence and desktop path. The result must survive
actual suspension, restart, accepted retry and original-history restoration.
It advances Gate04 actions, conditions, movement interactions, source fidelity,
physical dice, private actor control and exact recovery. It is not completion
of Grappling, Unarmed Strike, tactical encounters or the product.

Read: root `AGENTS.md`, `docs/product-definition.md`, roadmap and gate-execution
protocol, Gate04, ADR025/026/028, and active unarmed/Shove plans. Their remaining
Gate4 requirements remain in force. In particular, dragging/carrying grappled
creatures, special grappling attacks/body parts, class/feat exceptions, broader
equipment allowances, PvP consent and the remaining reaction/Ready families
remain Gate4 work. None moves to Gate5. Offstage/no-turn tactical deadlines and
aftermath timing also remain Gate4 obligations.

Pinned English SRD5.2.1 PDF, independently rehashed:
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
The complete relevant source clauses were read in the local verified extraction:
pp7–8, 14–15, 86, 89–92, 177, 180, 182–184, 187, 190–191 and 290/305.

| Source | Required behavior |
| --- | --- |
| p190, Unarmed Strike | Target within 5 feet; target chooses STR or DEX save. DC and escape DC are 8 + grappler STR modifier + PB. Target at most one size larger; an actual free hand is required. This is not an opposed Athletics check or an attack against AC. |
| p182, Grappling | One creature per used hand/body part; that part cannot target another creature until the grapple ends. Target spends its Action for STR(Athletics) or DEX(Acrobatics) against the grapple's escape DC. Grappler Incapacitated or distance beyond grapple range ends it. Grappler may release at any time, without an Action. |
| p182, Grappled | All Speed is zero and cannot increase. Attack rolls against anyone other than the grappler have Disadvantage. Drag/carry is an optional movement capability with the stated extra-foot cost and size exceptions. |
| pp7/187, pp8/182 | Voluntary failure belongs to saving throws; Escape is a skill ability check. Inspiration can reroll a die in either D20 test; Legendary Resistance cannot turn an Escape check into a success. |
| pp182/191 | Losing Fly Speed can cause a real fall; Hover is the stated exception. Unconscious implies Incapacitated and drops held things. A grapple does not itself grant flight, structural support or a carry path. |
| pp177/89–92 | Attack-action weapon equip/unequip permission, real held shield/weapon identities, ammunition loading and untrained armor remain relevant. Empty equipment slots alone do not establish anatomy. |

## Findings which constrain the design

1. `tactical/shove.rs` already provides the relevant owned save, source DC,
   paid ordinary Attack opportunity and common failed-save machinery. Its source
   pins, private control and exact pending-work proof are precedents to reuse,
   not permission to create another resolver. Light/Nick attacks and source
   Multiattack must not become generic Grapple replacements.
2. `WeaponLoadout` has two `Free`/`Item` slots. Creature equipment materialization
   defaults every source to those slots, including Air. There is no authenticated
   anatomy or grapple occupancy. A wolf, elemental, arbitrary mechanical sheet,
   creature type or human-readable name must not acquire hands from that default.
3. Conditions already retain their source through
   `tactical_effect_adapter::condition_effects`. Existing speed and attack
   condition readers can consume a projected Grappled condition. A separate
   stored legacy `ActiveEffect` would create competing authority.
4. `tactical/shove/geometry.rs` currently refuses Push involving any Grappled
   actor or grappler as unsupported coupled movement. The new, fully proved
   relation must allow ordinary forced displacement of one body and then apply
   the range rule; unknown old effect records must not be guessed into grips.
5. Movement commits each segment in `tactical/movement.rs`; fall landing changes
   position before vitality in `tactical/falling.rs`. Vitality and effect
   installation can cause Incapacitated/concentration/drop consequences. Break
   evaluation belongs at those real changes, not only at the next turn or end of
   a submitted route. A route that leaves and later re-enters range has already
   broken the grapple.
6. Free release during an existing wait needs additional proof. The global raw
   pending guard currently excludes it. More importantly, attack validation
   rederives mode from live conditions, including Grappled Disadvantage
   (`tactical/attacks/validation.rs`, `validate_source`). Merely letting Release
   through the guard would strand a pending attack against a third creature.
   This is a required design seam, not grounds for an idle-only release rule.
7. `movement::validate_opportunity` compares an active unanswered window's
   options with the current resolver's exact vector. Effective hand reservations
   make those options sensitive to release. This is a different temporal case
   from an issued roll: refresh the unanswered menu causally, rather than freeze
   it or reinterpret an already selected attack. `advance_segment` already
   rescans every participant before departure and allows a prior automatic
   Unavailable actor to become eligible; preserve that behavior.
8. Source damage can depend on the attack's mode: the actual Goblin Scimitar
   adds damage only for Advantage (`attacks/creature_weapon::source_damage`,
   also the intrinsic attack adapter). A sealed context must govern derived
   source damage as well as dice count. Existing unfinished `FlightLost` falls
   also rederive live flight loss; release needs separate new-grip causal proof
   without weakening that old validation branch.

## Bounded admission and source anatomy

The first actor authority is a successfully reconstructed, actually created
Human character profile, with a typed ordinary two-hand capability returned by
the source creation adapter. Do not authorize by a loose `species_id == "human"`
check, display name, description, `Humanoid` tag, bare sheet or two empty slots.
Represent the anatomical capability separately from what is currently held.
Document the ordinary Human body interpretation explicitly; p86 identifies the
species but does not enumerate a hand count. Source identity plus the admitted
creation model, rather than client anatomy input, owns that capability.

For a genuine Host-controlled source grappler, propose one narrowly audited
immutable revision of the existing Goblin Warrior definition carrying a typed
ordinary-hand capability. Its actual p290 Shortbow attack uses a physical weapon
which requires two hands (pp90–91), and its Scimitar/Shield Gear uses the existing
physical equipment path. This is the proposed source evidence for admission,
not a rule that every creature with Gear or a Fey/Humanoid tag has two hands.
Root and independent source review accept this bounded ordinary two-hand
normalization, and the reconstructed Human normalization above, as engineering
interpretations rather than quoted hand counts. They do not assert a universal
maximum number of anatomical appendages. No inference that another source has a
maw/tentacle grant is allowed. Actual immutable payload/admission implementation
still requires its own review; this decision is not executable source evidence
or permission to claim the source-controller matrix complete.

Use an optional typed source anatomy field omitted from old definition JSON,
and a separate new immutable revision/file/current-admission entry, following
the full-pin registry. Preserve all old definition fingerprints and the old
ID-only V1 API. Old unannotated source profiles remain valid, usable targets,
and able to Escape; they do not receive new grappler anatomy by default.
Installed content, the actual manifest generator, exact entry-set assertions,
current creation pin, and coexistence tests must all reflect the added revision.
Reconcile the independent Hag revision by exact source identity during root's
later integration; never replace Air or an old revision by an ID-only match.

Root's source investigation identifies an existing genuine coexistence starting
point: `crates/dmd-app/tests/fixtures/reactions-v1-upgrade-100c7da.json` retains
the old Goblin alive at HP4 in settled flow2. The planned production proof, after
release5 integration, is explicit `UpgradeExecutionTo(EncounterReleaseV1)`
**first**, then the actual owner's EndTurn to clear its empty retained
Attack-action window, `ConcludeHostilities`, `FinishEncounter`, and normal
current-catalog creation of the exact new Goblin
pin in that same campaign. Preserve the original Goblin/source throughout, then
verify cold accepted retries and original export/restore. This is a source-read
candidate path identified by root, not an executed test or an already accepted
release outcome. The corrected order matters: release5's live-execution guard
forbids a fresh legacy-flow2 EndTurn, while its explicit upgrade guard permits
this settled empty Attack-action window. It avoids assuming a new historical capture is necessary and
does not authorize creating an old pin through the new live picker.

New Grapple admission checks, before payment: actual active-turn controller;
living actor able to act and harm this target; authenticated anatomy and chosen
free hand; legitimate ordinary Attack-action opportunity; current located target;
size, five-foot body distance and physical contact geometry; exact source pins.
No creature's longer weapon/feature reach extends ordinary Unarmed Grapple.
Unsupported Air Form/contact geometry remains an honest private refusal.
Known immunity does not silently refund or select another action: an admitted
attempt resolves its save and pays its attack; immunity yields a recorded no-effect
result with no retained grip. Do not leak a secret immunity through target lists.

Use the Shove controller boundary for this slice: reject every pair where both
actors are player-controlled, including same-owner PC/creature combinations,
before cost. Permit actual owned actor versus actual Host-controlled opponent,
the inverse, and Host/Host opponents when both sources have required capabilities.
Require mutual enemy relations and no ally relation for a new harmful attempt.
Autonomous actors are not silently represented by Host. An opaque target handle,
Host issuer or PvP prose does not establish consent. Full consent remains Gate4.
Existing relation release/Escape remains available to its actual controller even
if dispositions later change; do not rerun harmful-attempt consent at cleanup.

## One authoritative grip and one shared continuation

Propose an optional `RulesState` grapple attachment with stable deterministic grip
identity, original paid command/window, grappler and target, chosen typed hand,
authenticated source/anatomy pins, established range/DC, final save proof and
establishment cause. It survives ordinary turn resets. The shared resolution
contains only current attempt/Escape/ending work and its causal receipts; it is
not the lifetime authority. Empty/absent serialization must preserve old bytes.

Derive one Grappled view per live relation through the existing condition reader.
Do not store another condition in `rules.effects` or a duplicate mutable effect
group. Multiple incoming grapples retain their distinct sources and DCs; successful
Escape selects one relation and ends that source, leaving other grapples intact.
One anatomical hand cannot be reserved by two attempts/grips. Multiple outgoing
grips are bounded by the proved actual hands, not an arbitrary one-grapple limit.

Do not invent an `ItemId` for a creature. Keep physical `Free`/`Item` facts and
grapple occupancy distinct, with a single shared effective-hand query combining
them. Every relevant live caller must use it: equipment changes, weapon grips,
two-handed attacks, one-handed ammunition loading, shield don/doff, physical
spell components and subsequent grapple admission. Pure planners must receive
the authenticated reservation view from the rules adapter, never from a client.
No assignment overwrites a grip and no cleanup deletes a real held item.
Unarmed damage can still use a kick/headbutt where lawful; a reserved hand cannot
be silently reused for a distinct creature. Special body-part attacks require
their own typed source mapping before admission.

The effective-hand audit must cover the real reader/writer chain, not only the
new Grapple button:

| Production seam | Required use of grip authority |
| --- | --- |
| `tactical_weapons/equipment.rs`, shared weapon planners and creature-weapon adapter | Before/after equipment allowance, two-handed use, ammunition loading, throw and physical assignment; keep real Item custody. |
| `attacks/intrinsic.rs`, `attacks/opportunity.rs`, `attacks/planning.rs`, `attacks/validation.rs` | Actual held implements, current unchosen options, selected attack admission and historical physical equipment reconstruction use the appropriate current or sealed reservation view. |
| `tactical/shields.rs`, `tactical_spells.rs`, `tactical_spells/binding.rs` | Shield don/doff and real S/M/free-hand exceptions; releasing a grip does not alter a paid spell or fabricate a held focus. |
| `tactical_inventory.rs`, `tactical_creature_equipment.rs`, legacy kernel guard | Two physical slots never prove anatomy; no Item/grip collision or bypass through the legacy free-hand flag. No-authority old actions keep their old path. |
| `tactical_vitality_adapter.rs`, vitality and condition continuations | Grip reconciliation is independent of Item dropping and cannot be skipped by an empty-loadout return. |
| App `table_attacks`, `table_movement`, `table_shields`, `table_equipment`; desktop `AttackForm`, `ShieldForm` | Use the same authoritative availability. A physically Free but reserved hand must not be labeled usable; owned occupancy disclosure must not reveal a hidden target's identity. |

Grapple starts or spends one existing ordinary Attack attack, with payment once
before the target's save choice. Reuse source modifiers, body armor training,
conditions, cover/Dodge and physical dice semantics from the inspected save path;
do not apply Magic Resistance to this nonmagical save. Target chooses STR/DEX,
including source-backed automatic failure, explicit voluntary save failure,
Inspiration and an actual source Legendary Resistance choice after failure.
Successful resistance ends the paid attempt; final failure installs the grip
unless immunity or an authenticated intervening cause has invalidated it. Preserve
the original attempt and final no-effect reason; no refund or automatic rechoice.

The selected hand is reserved during the attempt. An explicit grappler withdrawal
before establishment ends that paid attempt without inventing target dice. It is
not a way to recover the attack or free-hand allowance. Its canceled request, if
one was issued, retains its identity and cause; retries reproduce that result.

Include the one source Attack-action weapon equip/unequip allowance for this new
Grapple path, using the existing typed operation and physical identity rules. An
explicit before change can free the chosen hand; otherwise after-result change
is an owned continuation with an explicit decline. Record one allowance for this
attack, not a fake weapon attack receipt. Recheck after changes against any grip
actually established; never unwind the accepted save to force an impossible equip.
Shield don/doff remains its source Utilize Action. Applying this allowance to old
unarmed/Shove paths remains separate recorded Gate4 work unless root explicitly
extends this slice; do not change their historical request shapes or behavior.

Escape is the target's own-turn Action, independent of Attack attacks, selected
against one current incoming relation. The target explicitly selects Athletics
or Acrobatics. Use the real PC skill proficiency or exact source skill modifier,
untrained armor, Poisoned/Frightened and other applicable check conditions. Keep
the established source escape DC; do not substitute the source stat block's spell
DC or recompute it from a later altered grappler. No STR/DEX save proficiency,
opposed check, automatic saving-throw failure, voluntary fail-save command or
Legendary Resistance applies to Escape. Choosing not to attempt Escape is free;
after its Action is paid a failed check leaves the grip and gives no refund.

## Release, breaks and authenticated temporal context

Release names a proved live relation and its actual grappler/controller. It is
available off turn and during raw-roll and non-roll suspension, with no Action,
Bonus Action or Reaction cost. It removes only that relation/hand reservation.
No visibility test can strand the controller's own grip. Public presentation of
the release still follows knowledge rules; the command grants no extra sight.

The temporal rule has three distinct cases. An already issued physical request
retains its authenticated issued grapple context; an unanswered action menu uses
current legal options; an already initiated physical consequence retains its
authenticated initiating cause. Later newly admitted work uses current truth.
None of these cases grants an old saved `RollMode`, copied option vector or
whole-state snapshot independent authority.

For issued work, retain the exact new relations at the relevant source admission
and request issue, with immutable establishment proofs and subsequent actual
end causes. Reconstruct only their condition/hand contribution when validating
that work. Continue the existing source, actor, equipment, position, unrelated
condition and request checks, including the existing physical-item/ammunition
undo used to validate admitted attacks. The same derived hit circumstances must
govern request mode, accepted kept faces, hit outcome, critical handling and the
source damage program. In particular, release cannot add/remove the Goblin
Scimitar's Advantage-derived extra damage on an already issued attack. This is
not permission to restore unrelated old conditions or equipment.

Original accepted-command replay proves the relations existed at the cut, the
cut is complete for its reader, and the exact later controller/cause ended them.
A forged or incomplete cut, substituted source/work parent, or fabricated old
grip/release must fail retained structural checks and original semantic replay.
Live admission must never use a historical cut as permission. The proposed
concrete records and their retirement rules are specified below.

For an active unanswered OA window, a lawful release refreshes its affected
options with the ordinary current `movement::options` resolver and effective
hands, recording the original window identity and the exact release cause.
Preserve original window origin, reactor/mover, from/to crossing, step, frame,
work ancestry, offered order, reaction budget and already answered decisions.
The updated vector must still equal the current resolver's vector; do not simply
remove the equality guard. Do not reissue rolls, auto-decline, auto-select a newly
legal weapon, replace an owner or eagerly pump beyond an unanswered child.
An already selected OA belongs to the sealed-work rule, not menu refresh.

After the existing children finish, `advance_segment` must keep scanning every
current participant in its existing deterministic order. Attack/Declined stays
answered; prior automatic Unavailable may be reoffered when currently eligible.
Do not freeze a reactor roster or rewrite completed choices. A genuine automatic
unavailability still uses the existing causal path, never a fabricated decline.
The motivating two-handed melee/reach weapon plus occupied other hand is a
supported pure planner invariant. Current Human starters and Goblin source gear
do not establish its genuine table acquisition, so label that control synthetic.
Unarmed already works without a free hand for these first admissions; freeing
a hand alone does not create their first five-foot opportunity reach.

Apply this audit to weapon, unarmed, creature and spell attacks; saves/checks;
accepted-hit/Shield and Magic Missile collection/ordering; liquid landing;
knockout/mastery/after-equipment and other owned choices. An already admitted
attack is not canceled/re-rolled because Disadvantage later ended. A later
attack uses the new condition image. An Escape request whose selected grip is
explicitly released becomes obsolete: cancel only that unfinished request with
a typed cause, retain any already accepted faces, retain paid Action and let the
common pump finish its remaining consequences. Do not cancel unrelated rolls,
replace a turn controller or auto-answer an owned material choice.

Automatic source breaks are causal consequences, not owner decisions: remove
outgoing relations when the grappler becomes Incapacitated, including derived
Unconscious/Paralyzed/Stunned causes, and when actual source displacement exceeds
the established range. Integrate after actual condition/vitality/position changes
before subsequent mechanics and before new fall/movement admission. Ending the
condition does not automatically restore a Dodge already ended by Speed zero.
Newly unsupported/impossible source anatomy or immunity must not leave a corrupt
grip; audit only actual admitted producers and require explicit semantics before
admitting new transformation producers.

Reconcile outgoing grips after the real vitality/effect transition independently
of `tactical_vitality_adapter::drop_held`. That helper returns early when no Item
is held; a hand occupied only by a grip must still release on the grappler's
Incapacitated/death transition. Include all actual derived Incapacitated causes,
not only the existing newly-Unconscious item-drop comparison. Object custody,
shield assignment and relation authority stay distinct.

Death needs an explicit distinction. A dead grappler cannot sustain an active
physical grip; record this as a dead/inert-grappler consequence alongside existing
vitality/drop processing, including direct death without an intermediate saved
Unconscious image. Target death alone is not the p182 release rule. Page180 says
conditions/effects return upon revival if their durations remain; it does not
literally specify every intermediate condition on a corpse. Root accepts keeping
the living holder's occupied grip until release/range/another actual ending cause
as the lifecycle interpretation, consistent with that revival rule and p182's
endings. Root also accepts the dead/inert-grappler ending as a physical engineering
interpretation. Neither is presented as a quoted death clause. Forbid dead-target
Escape and do not claim corpse carrying is implemented. Incapacitating the target
alone likewise does not release its living holder.

Installing Grappled can cause non-Hover flight loss. Use actual fall/liquid/damage/
concentration children in the existing frame stack and correct causal ancestry.
The falling body may move beyond grip range and end it; neither body is silently
carried, suspended by the other's hand, or moved through an unsupported solid.
Retain an already triggered fall's authenticated cause across later release and
test that restoration does not invent a second landing or erase accepted dice.

Specifically, a legal ordinary Human grip can reduce an actual Large Chimera's
non-Hover Fly Speed to zero. Current `falling/validation.rs` rederives unfinished
`FlightLost` from live flight support; release before landing would restore
healthy flight and fail that check. Use the new explicit grapple-flight-loss
cause below, with the exact establishing work and historical grip cut, for this
new producer only. Keep old `FlightLost` validation unchanged. Still validate
the retained actor's current pre-landing position and exact destination/surface
against actual geometry, the fall's work parent and all landing/damage ancestry.
Release cannot cancel a triggered fall or restart a child. The genuine fixture
must place the Human on actual support and the adjacent Chimera legally airborne;
the real Air target supplies immune no-effect, not a fabricated nonimmune Hover
positive.

## Movement until the dragging slice

Expose an explicit new self-only movement intent to an actor maintaining a grip:
the target stays where it is. Show that exceeding range ends the grip. Existing
ordinary Move remains unchanged for actors without new grip authority. Require
the explicit choice when outgoing grips are present, even when the proposed
route currently stays in range; do not infer whether the owner intended dragging.
Movement inside range retains grips, at normal self-only movement cost. At the
first committed crossing beyond range, end the relevant grip and resume only
the remaining legal path. An OA still resolves before departure and can prevent
the crossing or cause an earlier break. Target Speed zero prevents its own
ordinary movement; it does not prohibit source forced motion.

Retain the self-only admission and its original outgoing grip IDs even if the
last grip is released while an OA suspends this movement. Original movement
origin/path, accepted prefix, cursor, spent budget, crossing and target position
remain unchanged. The explicit intent does not become invalid merely because
it is no longer required for a newly admitted move. Each later uncommitted segment
still uses current capability/allowance; do not recost the committed prefix,
refund movement, move the former target or resurrect a completed/interrupted
route. This is a genuine new-slice scenario to test. An incoming grip acquired
mid-movement is not yet genuinely admitted by this own-turn-only Grapple slice;
future reaction/Ready/special producers must join the same prefix proof explicitly.

Shove Push moves its selected body only. For new proved grips, apply actual
geometry, retain grips still in range, and end those beyond range without dragging
the other body. Replace the blanket new-grip refusal only for relations the
lifecycle understands; unsupported legacy/special coupling remains explicit.
Blocked Push leaves geometry/grips unchanged; accepted no-effect never refunds
the paid attempt. Preserve every private Host geometry review boundary.

No Drag/Carry action is accepted in this slice, and no shortened path, extra cost,
automatic release, silent decline or substitute mode pretends to implement it.
The UI explains the self-only choice and the pending dragging capability. Actual
dragging remains a named Gate4 follow-up: coupled swept volumes/support, source
size exceptions and cost, multiple grips/cycles, before-crossing opportunities,
terrain/jump/fly/liquid/fall interactions, interruption/recovery and private
geometry. This temporary admission limit is not permission to close Gate4.

## Proposed optional records and cut validation

The following is the reviewed schema contract. Its source/domain types and local
shape controls are now authored; they grant no runnable authority at this
checkpoint. Use existing `CommandMeta`,
`TacticalWorkKey`, `TacticalRollKey`, `Hand`, source-pin and spatial types. New IDs
have separate deterministic domains; no raw-role occurrence is consumed for a
free release. All optional fields default to None and omit None; producers write
None for empty attachments and retained validation rejects noncanonical empty
attachments. All new records deny unknown fields. Their allocation is
proportional to real admitted grips/work, with checked
counts; do not invent a scene, command, grip or history cap.

| Attachment/type | Proposed exact contents and role |
| --- | --- |
| `RulesState.tactical_grapples: Option<TacticalGrapples>` | Version1 plus deterministic ordered `active: Vec<TacticalGrip>`. This is the only live relation authority and is absent when active is empty. Pending attempts live in the resolution, not as provisional live conditions. |
| `TacticalGrip` | `declaration: TacticalGrappleDeclaration` owns `id`, original paid `CommandMeta` and attack window, `grappler`, `target`, selected `Hand`, typed anatomy/source proof, original body positions, range and escape DC. `save: TacticalGrappleSave` owns the chosen save/request/final proof; `established_by` and `work` retain the actual establishing command and work. ID derives from the actual paid command, actors and selected hand. Use full creature pins; Human proof requires the reconstructed admitted character profile, not a string or a new client hash. The source proof records the approved ordinary-hand interpretation. |
| `TacticalResolution.grapple: Option<Box<TacticalGrappleResolution>>` | `activity: Option<GrappleActivity>` with boxed Attempt/Escape stage variants, immutable `proofs: Vec<TacticalGrip>`, `cuts: Vec<GrappleReadCut>`, `ends: Vec<GrappleEndReceipt>`, and `opportunity_refreshes: Vec<GrappleOpportunityRefresh>`. Activity data reuses paid save/check and existing request/work identities; an attempt retains its selected-hand reservation before a live grip exists. Activity may be absent while other work consumes proofs. This does not execute a second queue. Proofs may retain an ended grip only while suspended work references it; they never project a live condition or reserve a hand. |
| `GrappleCutKey` / `GrappleReadCut` | Key is existing work key plus closed reader discriminator: `AttackAdmission { attack: CommandId }`, `RequestIssue { roll: TacticalRollKey }`, or `FlightLoss { actor: EntityId }`. Record exact issue command, sorted unique relevant grip IDs referencing `proofs`, and `source_attack: Option<GrappleCutKey>` for an inherited attack descendant only. The reader determines the complete relevant actor/relation set; the client cannot select a convenient subset. |
| `GrappleEndReceipt` | Grip ID, actual causing `CommandMeta`, and a closed cause: owner `Released`; `Escaped { roll, work }`; `Incapacitated { work }`; `Dead { work }`; or `OutOfRange { work, moved_actor }`. Work-bearing causes refer to the exact existing effect/vitality/position producer and its retained ancestry/receipts. Owner release has no invented executing work node; any affected wait is identified by its existing cut/window. Original replay proves the prior grip and actual transition. |
| `GrappleOpportunityRefresh` | Original window work key, movement/window origin, reactor and step; causing end-receipt reference; exact previous and resulting ordered option vectors. The chain explains an update to the existing window, without replacing its origin, crossing, frame or offered/answered history. Unchanged vectors do not need a refresh record. |
| `TacticalMovement.grapple_self_only: Option<GrappleSelfOnlyAdmission>` | Original explicit movement command plus sorted original outgoing grip IDs. Their immutable proofs are retained in the resolution. This remains valid after release; it records why the admitted choice was required, not a demand that those grips stay active. |
| `TacticalFallCause::GrappleFlightLost` | Establishing grip ID, actual consequence command, establishing work key and `FlightLoss` cut key. The existing `TacticalFall.path`, actor, origin, stage and work trace retain all geometry and child progress. Do not duplicate a mutable fall path or change old `FlightLost`. |

The closed reader keys deliberately distinguish an attack's admitted source
program from a later request. For example, damage and Inspiration descendants
which belong to that admitted attack inherit its proven mode-dependent program;
they do not become a new attack when release occurs between attack and damage
dice. A genuinely new attack has a new admission/cut and uses current conditions.
Use a cut only for a reader which actually consumes new grip authority. Existing
no-authority histories never gain an empty cut as a generic opt-in switch.

The live grip and a retained proof must be byte-equal while both exist. A cut's
IDs must be unique, known, source-valid and compatible with its actors, hand
ownership, source reader and exact work node. A retained proof which is no longer
live needs its unique chronological ending receipt. Rederive the request and
all source-derived damage using this narrow historical relation view; reject
missing/extra cuts, mode-only evidence, changed save/DC, mixed source revisions,
wrong work kind/parent or an end receipt preceding establishment or a cut at
which that grip is claimed live. A descendant request may legitimately issue
after release while referencing its earlier source attack cut. Original
replay proves the complete as-of relation set and each real paid/physical result;
local consistency alone does not prove a genuine earlier history.

Refresh validation checks the unchanged original window/crossing/work, the real
owner-bound release or automatic end, each previous-to-next vector link, and
latest vector equality with ordinary current options. Replay checks the prior
vector against its actual pre-release state. No cut replaces that live equality.
The free-release command is applied atomically to relation authority, receipts
and affected owned projections, with existing accepted-retry-first handling. It
must not alter an unrelated pending request or advance its cursor simply to make
the new image validate. Escape obsolescence cancels only its own pending ID and
retains payment, accepted faces and cause. Attempt withdrawal uses the analogous
owned cancellation proof; it is distinct from ending an established grip.

For a new causal fall, replay must show the actual grip establishment reduced
the actor's sustaining non-Hover flight to zero at its initiating cut. Local
validation checks its exact source/grip/work link and existing geometry/child
proof. Later release explains why live flight may differ; it cannot substitute
for that initial proof. Old `FlightLost` continues to require its original live
loss query, and old unsupported/legacy Grappled effects cannot select this case.

Retire resolution-only proofs, cuts and receipts only with their last retained
consumer; completing a source attack is insufficient while its child still
depends on it. After the resolution retires, the original journal and existing
raw/result history authenticate past work. A release with no suspended consumer
needs its normal accepted command/result, not an unbounded second lifetime ledger
in `RulesState`. Recovery anchors must reject injected active grips **or** retained
proof/cut/refresh/self-only authority; collect every new origin for the same
original-anchor replay used by other tactical provenance. A nearest snapshot
containing a convenient old grip is not a substitute for its original commands.

Record-shape constraints from independent review `318649ed` are part of the
implementation contract:

- A work key identifies its enclosing resolution and occurrence; a paid roll key
  identifies its actual action origin, role, subject and occurrence. Nested work
  may have different resolution and paid origins. Do not equate them. The local
  shape controls keep both identities; later rules must match actual node kinds,
  parent records and source programs, and original replay must prove producers.
- Establishment, flight loss and range break can share one accepted command.
  Their ordering requires the actual later child ancestry, rather than a strict
  event-sequence increment or an occurrence-number comparison. Free owner
  release is a separate accepted command. `source_attack` is a direct,
  same-attack AttackAdmission reference with identical relation IDs; its source
  cannot itself inherit, and another request cannot stand in for that admission.
- A pending Attempt reserves its exact `(grip ID, actor, hand)` even when the
  live attachment is None. Only reconstruction of that same paid admission may
  recognize its own reservation. A resisted/withdrawn attempt or an ended proof
  does not reserve a hand. The later effective-hand implementation must consume
  the provisional reservation and install live authority atomically.
- Final saves retain either the canonical physical roll reference or the real
  `TacticalSaveDecision` for automatic/voluntary failure. Automatic failure has
  no physical request; voluntary failure retains its issued request. The chosen
  ability, issue/finalizer, actual LR decision and final success remain distinct.
  Later rules/replay must authenticate final arithmetic, ownership, relevant
  source conditions and the existing LR receipt; these shape records cannot
  authorize a forged success/failure or fabricate dice.
- A live grip can outlast its establishing resolution. Its historical work
  reference does not demand that the retired trace still exist as current work.
  Consumer resolutions copy its immutable proof and require equality while it
  remains live. Preserve proofs/ends/cuts until the actual last consumer retires,
  including completed falls with concentration children, refreshed OA chains and
  the original self-only route. Do not retire them merely when an attack ends.
- The no-authority path and recovery guards inspect provisional and retained
  fields, including orphan self-only movement and causal fall fields, rather
  than only live grips. Old schemas/executors cannot acquire authority by
  deserialization. Full origin collection and original-anchor replay validation
  are prerequisites to removing the temporary checkpoint rejection below.

## Version, wire, ownership and recovery proof

Observed role tags are 1–14, 16, 17 and ShoveSave 18. Propose append-only
`GrappleSave = 19` and `GrappleEscape = 20`, leaving tag15 and all old UUID domains,
roles and occurrence identities unchanged. Owner release and relation identity
use their own deterministic source IDs, not forged raw dice occurrences.

Propose additive new commands/work kinds and optional state/cut fields on the
existing flow4 spine, admitted only with authenticated new Grapple authority.
There is no reason yet to reinterpret any old command on a state lacking that
authority. Prove this with original-history parity, including pending attacks
and old generic Grappled effects. Schema4 absence and old flow1–4 behavior must
remain byte/operation-identical. Do not allocate flow6: it is reserved for the
future Counterspell executor after root's release5 work. Root will integrate
this feature with release5 explicitly, including retained-grip preflight and
release/aftermath obligations; this branch does not implement release5 itself.

The required compatibility proof is path-specific. New attempt/Escape/release/
self-only commands enter only the current flow4 executor; earlier flow versions
reject them. Live ordinary readers take the existing path when no new grip
authority or authenticated retained consumer exists. New sources have immutable
new pins, not modified old definitions. New cuts are produced only by real new
grip reads; old raw roles, paid receipts, work stamps, accepted pending vectors,
fall causes and generic Grappled effects retain their original interpretation
and serialized bytes. Do not add a global historical-roll trust flag or relax a
raw-pending guard for arbitrary commands: the exception authenticates this
specific owner's live grip/attempt before any mutation.

The admission constraint supporting the temporal proof must be checked in code:
ordinary Grapple begins on its actor's turn without an existing resolution, and
is not an OA/Ready replacement in this slice. It therefore cannot introduce a
new grip into previously issued unrelated old work. Release/automatic ending
can only affect work exposed to an actual new grip; proof records preserve that
fact even after the last live relation ends. Audit every actual producer before
relying on this claim. Old generic effect records alone do not create such an
exception. Integration order is source/anatomy and optional domain, strict rules
and old-history controls, app/restore/UI, then root's explicit release5
integration and combined original-history verification. No flow6 is allocated.

If the temporal/hand audit finds that old pending work must change even without
new grips/cuts, stop and report the counterexample. Do not hide a semantic upgrade
under an optional field. Propose an explicit later execution boundary/order to
root before changing old interpretation; no speculative flow number allocation.

Use normal canonical table command envelopes, stable opaque actor/relation/work
handles, actual controller checks and original accepted-retry-first ordering.
Target save/check and grappler release/equipment decisions belong to their own
controllers; simultaneous work ordering belongs to the current-turn controller.
Area delegation does not authorize independent release or another actor's roll.
Keep cause actor, rolling actor, current-turn actor and issuing player distinct.

Retained validation checks deterministic IDs, no duplicate hand reservation or
legacy condition identity, authentic source pins, exact paid opportunity/check,
final raw result plus any LR receipt, relevant chronological work ancestry, and
every end cause. Original replay authenticates provenance which a local snapshot
cannot establish alone. Extend recovery-anchor detection for new optional
authority, old-schema rejection/preflight, export/restore and active-grip release
preconditions. Never discard grips merely to finish/replace an encounter.

## Planned implementation and acceptance

1. **Review this amended plan first.** Anatomy/death interpretations are accepted
   as recorded below. Root must review the proposed exact optional records,
   temporal branches and compatibility proof before gameplay edits. No
   implementation is authorized by this plan's existence.
2. **Source and domain.** Authenticated PC capability and audited immutable
   creature revision, optional grip/cut authority, pure condition/hand projection
   and strict structural validation. Add meaningful legacy byte/pin controls.
3. **Rules and lifecycle.** Paid ordinary attempt, real save, skill Escape,
   free release/withdrawal, actual source breaks, genuine flight/fall children,
   explicit self-only movement and one-body forced movement. Use existing work
   frames, failed saves, physical dice and atomic transition paths throughout.
4. **Application and desktop.** New owned commands/capabilities, private-safe
   projections, cancellation/receipt history, durable original-anchor recovery,
   actual controls for hand, target save, Escape skill, release and movement.
   Reuse the integrated UI focus/remount framework; each new stage/key/actor
   resets draft rolls and reachable focus. Do not duplicate it or expose raw
   internal coordinates/IDs. Do not optimistically spend a hand, Action or die.
5. **Verification when root frees the heavy slot.** Focused rules and real
   default-stack file-SQLite tests, desktop checks, canonical scripts, fresh
   independent exact-head review and actual packaged native acceptance. Then
   remote exact-head checks, protected merge and literal-main proof as coordinated
   by root. Development parent success is not evidence for this candidate.

Required meaningful controls and production evidence:

- Actual STR versus DEX source modifiers and choice, PC/source proficiency,
  automatic versus voluntary save failure, armor/cover/conditions, Inspiration,
  source LR, current size/reach/physical contact and before-cost refusal. Confirm
  Escape skill modifier is different from a saving throw and LR/fail-save cannot
  answer it. Test both failure and success against the established DC. The
  current genuine LR source is the Huge Adult Red Dragon, outside ordinary
  Human/Goblin size admission. Test its correct size refusal; label isolated LR
  integration tests as synthetic until a real size-legal source grappler is
  admitted. Do not enlarge a PC or shrink that dragon to claim production LR
  acceptance. The eventual genuine positive remains a recorded Gate4 obligation.
- One paid ordinary attack/Action, no double cost on retry, no Light/Nick or
  Multiattack substitution, no hand invented for Air/wolf/generic sheets, exact
  source revision coexistence and actual finite Goblin ammunition creation.
  Test two real hands/two targets and distinct incoming sources without arbitrary
  scene or grip caps; remaining source body parts stay unsupported honestly.
- A real installed Air target, created with its current catalog pin and actual
  accepted Host assignment, physically fails the save: paid immune no-effect,
  no grip, no forced Prone, no invented fall. No mutated immunity sheet can stand
  in for this source test. Additional pure synthetic invariants are labeled.
- Real hand conflicts with shield, weapon equip, two-handed attack, ammunition,
  and a second grapple; explicit lawful release restores only that reservation.
  Before/after weapon allowance preserves real items. Spell S/M and two-handed
  melee/reach OA hand invariants require focused pure controls; the first
  admitted Human Fighter/Goblin grapplers do not yet establish a genuine
  spellcaster or two-handed melee-weapon acquisition path. Do not fabricate such
  table evidence; source admission of those positive cases remains Gate4 work.
- Release on another actor's turn at raw attack against a third creature, after
  accepted attack before damage/Shield completion, pending Escape and non-roll
  choices. Original faces and request IDs persist where relevant; later attacks
  use current conditions. A rejected foreign/stale release changes nothing.
- Real Goblin Scimitar source-damage control: use accepted creation/equipment,
  a Human's maintained grip and a separate actual Shove to make a reachable
  opposing source target Prone. On the Goblin's real turn, its attack at that
  target has Prone Advantage canceled by Grappled Disadvantage. Release during
  its issued Normal attack: the same faces, Normal request and base source damage
  remain, with no newly acquired Advantage extra die. A later new legal attack
  against the still-Prone target must use current Advantage and the real source
  extra damage. Prove actual positions, dispositions, turn sequence and source
  program; this is a proposed genuine fixture, not an executed result.
- Unanswered OA refresh: pure controls cover added two-handed options and a
  newly legal longer-reach actor, exact refresh cause/vector chain, no change to
  selected attacks/answered decisions, and ordinary rescan of prior Unavailable.
  Real Human/Goblin movement covers owner release while the outgoing grappler's
  explicit self-only route waits at an actual OA: preserve original crossing,
  prefix, expenditure and stationary target, then continue current legal steps.
  The first source scenario must not be mislabeled as a real two-handed menu
  change. Rejected refresh/cut/controller mutations leave the full state intact.
- Selected Escape canceled by actual holder release retains its paid Action and
  accepted faces, cancels only its pending request and never becomes a voluntary
  fail-save or free retry. Non-roll save/after-equipment/knockout/landing and
  simultaneous-work choices remain owned, reachable and unanswered. New
  after-equipment choices use live effective occupancy without rewriting the
  admitted source attack; other source-derived selected consequences retain
  their original proof. Audit Shield/Missile collection against its current
  source/response contract without inventing new respondents or reopening paid
  responses solely to demonstrate release.
- Multiple incoming grapples: attacking either holder still has Disadvantage if
  another grip supplies it; one Escape/release removes only the selected source.
  Target-only Incapacitated/death, grappler Incapacitated/direct death, exact
  range threshold, leaving/re-entering route, blocked/clear Push and OA-before-
  departure all produce source-specific results and preserved ancestry.
- Itemless outgoing grappler: real paralysis/Incapacitated and direct-death
  producers end the grip even with no Item in either physical slot. Target-only
  Incapacitated/death preserves the living holder's relation/reservation. Use
  actual admitted source casts/damage and original replay; keep any unavailable
  positive explicitly unverified rather than injecting condition/HP state.
- Genuine airborne non-Hover source target against a legal supported grappler
  from a real platform: Grappled causes the real fall/landing/damage chain and
  range break. The installed Large Chimera is a candidate for a Medium Human
  grappler; verify its exact source and live creation/geometry before using it.
  Do not enlarge a PC or modify a creature. Reopen at landing choice/dice and any
  genuinely reached concentration child. Include Grip to Chimera flight loss
  to free release **before landing**: authenticated initiating cause, exact
  retained path and original child identities survive despite restored live
  flight. No second fall or canceled accepted dice. Old no-grip FlightLost
  controls retain their old validation behavior.
  Source immunity and Hover controls do not claim a nonimmune flying case by
  manufacturing Air's condition list. If an admitted source/setup cannot supply
  a particular positive, record the gap and add the real source prerequisite.
- Real PC creation and exact source creation/control commands in file SQLite,
  no SQL/entity/clock/condition injection. Cold-reopen and identical accepted
  retry at every genuinely reachable material cut: paid save choice/raw,
  installed grip, turn
  crossing, Escape raw/result, release during interrupted work, after-equipment,
  movement/OA and fall children. Preserve full original journal and prove
  independent semantic export/restore continuation from its true anchor.
- Host, owner and unrelated viewer DTO/history checks: private source stats,
  hands, hidden relations and release causes cannot leak through controls, errors,
  revisions or cancellation summaries. Foreign/same-owner PvP and autonomous
  Host substitution reject before cost. No source/body label is an authorization.
- Hostile retained/restore images: coherent fake hand capability/pin, forged grip
  or DC/save result, missing/duplicate source node, forged old condition cut,
  rewritten release controller/cause, canceled-ID substitution, impossible
  distance/end chronology and altered paid budget all reject atomically. Include
  coherent forged OA-refresh chains, omitted relevant grip cuts, changed Goblin
  damage program, false flight-loss cut and injected recovery-anchor authority.
- Original no-new-grip histories, including old generic Grappled effects,
  pending attacks/OA windows, source damage, cancellation, physical equipment and
  FlightLost children, retain exact canonical state/export bytes, raw requests,
  operation stamps, receipts and continuation. New optional authority must be
  rejected by historical schema/executor routes instead of being ignored.
- Desktop production flow: actual PC and Host-source directions, physical saves
  and Escape checks, off-turn release available during waits, blank new dice,
  correct focus/scroll and actor handoffs, clear self-only movement choice,
  restart and equipment disclosure. Packaged source/artifact hashes and state
  audit must identify the actual played executable. Component tests alone do
  not establish native acceptance.

Update the witnessed-memory hook/geometry/work-reservation design for every new
material root, grip condition change, release and forced fall/break boundary.
The existing 1674 candidate estimate is qualified to its audited source families;
it is not a universal limit or an admission cap. Re-derive checked input-dependent
counts and temporary/history memory for the new family, preserving all-observer
witnesses at each real material step. Do not claim static counts are timings or
silently drop samples to fit them.

## Current status, risks and exact next action

Root approved bounded source/domain implementation from exact amended plan
`3338fbf51a2cba8b7b3938c2b292e76e617323dd`, tree
`af3e4fd38f61afff4b8e3a3baafeff1a3af8713c`, after the independent record-shape
review in `tooling/gate4-grapple-amended-record-shape-review-2026-09-30.md`
(external SHA256
`318649ed1b20c6163698ff7d1b3afb55b0ada68fe9ea20bcd881ae39cfe51c6d`).
Root separately approved a narrow fail-closed guard for every new live,
provisional, retained or raw-only Grapple record until the full resolver exists.

Authored checkpoint scope:

- New immutable `goblin-warrior-v2.json` differs from the frozen Goblin only by
  typed `TwoHandsV1` anatomy. A source loader enforces that equality. The current
  registry selects its exact new pin; immutable lookup and ID-only helpers retain
  V1. The actual manifest generator and distributed exact-file assertion include
  the seventh payload. Other sources receive no inferred hands.
- The pure anatomy query reconstructs the actual supported Human creation
  profile/mechanics, or validates the actual creature profile and full source
  pin. No human-readable name/type or default two-slot inventory grants anatomy.
- Domain records implement the attachments/table above, physical versus no-die
  save evidence, provisional reservation identity and append-only raw tags19/20.
  Local shape validation checks identities, nonempty ordered live hands, direct
  inheritance, chronological/causal endings and retained consumer references.
  Existing constructors write None, and absent fields retain old JSON omission.
- Kernel and tactical validation reject all new Grapple records, including new
  raw roles without an attachment. Restore-anchor and old-schema preflight
  exclusions are authored. This is an intentionally unaccepted checkpoint:
  structural validity never makes these states executable. No command, work-kind
  producer, condition projection, resolver, table action,
  desktop control or functional Grapple/Escape/release is exposed.
- Authored synthetic domain controls cover save evidence, one-hand occupancy,
  provisional/retired distinctions, nested origins/direct inheritance, same-
  command ancestry, retained movement/OA/fall consumers, optional absence,
  unknown fields and stable old/new roll tags. Source controls read the genuine
  old Goblin export without rewriting it, preserve its fingerprint and JSON
  value, reconstruct its Human, and label pure new-source construction separately
  from production coexistence. Hostile runtime and old-schema/duplicate-field
  controls assert rejection. Fresh current-Goblin table fixtures select the new
  current pin; historical captures and journals remain unchanged.

Verification status: direct toolchain `rustfmt` parsing/formatting and Git diff
checks only; the actual manifest generator was run as source metadata generation.
All authored Rust tests are **unrun**. No Cargo build/check/test/clippy, npm,
database, native UI or GitHub publication was performed. Root holds the heavy
slot for independent original-history work. Compilation, behavioral source
admission, byte-exact replay, cold continuation and native acceptance are not
established by this checkpoint.

Source-binding correction, 2026-09-30: independent review of source/domain head
`660631a4ca5f7547b4cf45d3517ee2d8dc41402d` (tree
`f138ad514b790e40a15991141eb9db28e539bdb3`) found that current compiled Goblin V2
admission lacked an exact installed-byte check in `load_rules_pack`. General
manifest integrity alone permitted an omitted entry or a self-consistently
rehashed changed payload. The external review is
`tooling/gate4-grapple-660631a-independent-review-2026-09-30.md`, SHA256
`774c84fe5336a2e52509223637f89a9147fe8eda10280c7977d9cf36ce8443e8`.
Root confirmed the finding and authorized only this narrow correction before
further source/gameplay work.

The corrected loader requires the explicit Goblin V2 declaration, re-reads its
file, and checks declared byte length, FNV checksum and exact compiled bytes.
Recoverable content errors request an installed-package update without changing
campaign data. One authored runtime test covers missing, undeclared and rehashed
packages after compiled-cache warmup and real campaign initialization. Each case
rejects a query and an otherwise valid command, compares the whole export after
normalizing only its export timestamp, then restores the genuine package and
accepts the same action/context through the same runtime at the unchanged journal
head. Each `execute_rules` invocation creates a new internal CommandId, so this
is a positive repaired-package control, not an accepted-command retry identity test.
These controls remain **unrun**; the correction has only direct rustfmt and Git
diff checks. No payload, source fixture, gameplay guard or Grapple scope changes.

Review amendment, 2026-09-30: independent review of plan commit
`2213790271b6553d389450f22f819f003672b4d1` (tree
`9fbcbc46a98284e61f275962c636baf86ee3fdb8`) identified the live OA equality,
source-damage, suspended movement, causal flight-loss and itemless cleanup seams.
The full corrected external review is
`tooling/gate4-grapple-plan-independent-review-2026-09-30.md` outside this repo;
its corrected SHA256 is
`fb3e420ce25488669d505aae54efa6d506c37d7b5f426a37182c37e069a86c71`.
Its initial base-tree transcription was explicitly corrected by the reviewer.
Root accepted its substantive findings and the documented Human/Goblin ordinary
two-hand and death/living-holder interpretations. Root authorized this plan-only
amendment, not gameplay implementation. The later source/domain authorization
and authored-only status are recorded above; the gameplay acceptance controls
remain future work.

Principal remaining review risks are the exact temporal record/reader coverage,
all effective-hand callers, strict original-replay/retirement proof and
flight/forced-displacement ordering. The immutable source revision needs
independent review and executable content/source verification. The full resolver
must replace the checkpoint guard with authenticated source/admission/result
validation, exact consumer retention/retirement, origin collection and original
replay before any gameplay acceptance or release publication. A guarded draft for
CI is separately authorized below. These are engineering/source proof
obligations within Gate4, not requests to waive product acceptance.

The narrow installed-source correction was committed as
`522d427b0387a5a2f415e0973888a1a2b46e1638`, tree
`3fe2fbe443268d8a7767819beb4b03428eb0869a`. Independent corrective review is
clear by static inspection only: external
`tooling/gate4-grapple-522d427-corrective-independent-review-2026-09-30.md`,
SHA256 `cdd25e4d86d5d774756e38af9023cb4e399c5c63a4ae3e65e2212389a673dc6b`.
All authored tests remain unrun. The original 660631a finding remains valid for
that original head; it was not erased by the correction.

Root next authorized normal dependency integration and a bounded plan only.
Merge `c3f3e73488840c5f27aa3f099ec702d893b203b1`, tree
`b5f6893df46eebe86b6d187281105b2f2f85b865`, preserves corrected source/domain
522d427 and published Shove `c1124004dfb3fcc84d105b512a5fe4f9502a7727`.
It merged cleanly and changes only the reviewed Air setup test and Air/Shove
plans. No Grapple source/domain/guard change or executable acceptance follows.

The bounded derived-hand checkpoint is recorded in the
[derived effective-hand plan](gate4-grapple-effective-hands.md). It preserves
physical Item slots, unknown-anatomy compatibility, early execution rejection,
source-validation ordering and historical/current reader separation. Its caller
matrix names the preparatory composition and the full-lifecycle/UI work excluded
from that checkpoint. Root reviewed and approved plan commit
`e3e22e75ef7e2a2be4867b2052424957097d3707` (tree
`b11b30b85700d06322d7d9881c87bfbeaac6a24f`) and then authorized its implementation.
The source-derived current-hand view and included existing planner/app-option
composition are now authored, with focused pure controls; all remain unrun.
Physical Item loadouts, current source validation, execution guards, source
damage programs and old captures stay unchanged. Selected attack reconstruction
explicitly refuses new authority until original admission can be authenticated.

Root approved deferring the own-Attempt exclusion API and hostile identity
controls until the real authenticated admission caller exists. Current derivation
always reserves a pending Attempt. No public mask, unused bypass flag or synthetic
command acceptance is introduced. This sequences the checkpoint; the complete
resolver obligation remains in this plan. The detailed plan records exact caller
coverage and qualifies synthetic fixture setup, source-only controls and deferred
full-lifecycle evidence.

The complete bounded hand delta at `ec886ac147e91bb670692f69c46cf8f36fe83000`
has now passed independent static review; its exact memo/hash and qualifications
are recorded in the derived-hand plan. Root resumes sole branch ownership and
authorizes a guarded draft PR for first remote compilation and regression
evidence. This supersedes the prior author's publication hold only, not the
full resolver, acceptance or guarded-authority contract. The draft initially
targets published Shove c1124004 for a bounded review and must never merge into
that development branch. Final accepted dependency integration, main retargeting,
exact-head review/checks and relevant production evidence remain mandatory.

Exact next action: publish the reviewed foundation plus this documentation,
inspect actual CI results and fix justified findings. Original fixtures and
early guards remain intact. Release owns the sole local heavy verification slot;
no simultaneous Cargo/npm/database/native operation is authorized. Genuine
old/new Goblin coexistence, release integration and the complete resolver,
transport, UI, privacy and native lifecycle remain unaccepted Gate4 work.

Root subsequently transferred sole branch writing to flow4_import_review for the
bounded plan-first [dependency/coexistence integration](gate4-grapple-development-integration.md).
That plan pins reviewed Shove f9 with Air/release as unaccepted dependencies,
preserves the 801 lint correction and every Grapple execution guard, and imports
only the independently reviewed genuine Goblin child plus exact parent append.
No resolver authoring, heavy verification or publication is authorized there.
