# Gate 4 — Grapple and Escape lifecycle

Writer: `gate4_shove_recovery`, coordinated by root. Branch:
`codex/gate4-grapple-lifecycle`. Status: **plan only, awaiting root review; no
implementation or executable verification**. Date: 2026-09-30.

Development base: `387b74241d1870964be0e88cb0f3216e151c9b55`, tree
`ffd06e4d73cca2188014c0120b6e702df89935bb`, containing the Shove development
parent and normal integration of corrected Air source `021421c4`. These are
unaccepted development dependencies, not evidence of a merged feature. A fresh
`git fetch origin main` still resolves main to
`c4d8c34c19b5c92eca789f292f99632a0107d861`. The supplied development checkout is
clean and stays on its explicitly assigned base. PR51 and other branches belong
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
The two-slot normalization is an explicit anatomy interpretation to review
before authoring. No inference that another source has a maw/tentacle grant is
allowed. If that evidence is insufficient on review, select a source with an
affirmative audited capability; do not substitute a fabricated sheet or silently
claim the source-controller matrix is complete.

Use an optional typed source anatomy field omitted from old definition JSON,
and a separate new immutable revision/file/current-admission entry, following
the full-pin registry. Preserve all old definition fingerprints and the old
ID-only V1 API. Old unannotated source profiles remain valid, usable targets,
and able to Escape; they do not receive new grappler anatomy by default.
Installed content, the actual manifest generator, exact entry-set assertions,
current creation pin, and coexistence tests must all reflect the added revision.
Reconcile the independent Hag revision by exact source identity during root's
later integration; never replace Air or an old revision by an ID-only match.

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

The temporal rule is explicit: an already issued physical-roll request retains
its authenticated issued context across later free release. Later work uses
current conditions. Add the smallest optional grapple-condition cut to work
which can outlive a release: exact involved relations at the relevant source
admission/issue occurrence, with original grip proofs and later release/break
causes. It must not be a trusted cached `RollMode` or a saved whole-state override.
Reconstruct only that historical grapple contribution when validating the sealed
request; continue all other source, actor, equipment and request checks. Original
semantic replay must prove that those relations actually existed at that cut and
that the exact later command legitimately ended them. A forged cut, changed
origin, forged release or substituted work parent must fail retained validation
and full replay. Live action admission must never use an old cut as permission.

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

Death needs an explicit distinction. A dead grappler cannot sustain an active
physical grip; record this as a dead/inert-grappler consequence alongside existing
vitality/drop processing, including direct death without an intermediate saved
Unconscious image. Target death alone is not the p182 release rule: p180 preserves
ongoing conditions through death/revival when durations remain. Do not silently
free the living grappler's hand merely because the held target died. Preserve
the relation until release/range/other actual ending cause, forbid dead-target
Escape, and do not claim corpse carrying is implemented. The dead-grappler
physical interpretation must be reviewed explicitly, not mislabeled as a quoted
p182 clause. Incapacitating the target alone likewise does not release its holder.

Installing Grappled can cause non-Hover flight loss. Use actual fall/liquid/damage/
concentration children in the existing frame stack and correct causal ancestry.
The falling body may move beyond grip range and end it; neither body is silently
carried, suspended by the other's hand, or moved through an unsupported solid.
Retain an already triggered fall's authenticated cause across later release and
test that restoration does not invent a second landing or erase accepted dice.

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

1. **Review this plan first.** Resolve anatomy interpretation/admission and
   temporal-cut/death details with independent source review; record approval
   and chosen exact data shape before gameplay edits. No implementation is
   authorized by this plan's existence.
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
  spell S/M components and a second grapple; explicit lawful release restores
  only that reservation. Before/after weapon allowance preserves real items.
- Release on another actor's turn at raw attack against a third creature, after
  accepted attack before damage/Shield completion, pending Escape and non-roll
  choices. Original faces and request IDs persist where relevant; later attacks
  use current conditions. A rejected foreign/stale release changes nothing.
- Multiple incoming grapples: attacking either holder still has Disadvantage if
  another grip supplies it; one Escape/release removes only the selected source.
  Target-only Incapacitated/death, grappler Incapacitated/direct death, exact
  range threshold, leaving/re-entering route, blocked/clear Push and OA-before-
  departure all produce source-specific results and preserved ancestry.
- Genuine airborne non-Hover source target against a legal supported grappler
  from a real platform: Grappled causes the real fall/landing/damage chain and
  range break. The installed Large Chimera is a candidate for a Medium Human
  grappler; verify its exact source and live creation/geometry before using it.
  Do not enlarge a PC or modify a creature. Reopen at landing choice/dice and any
  genuinely reached concentration child.
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
  distance/end chronology and altered paid budget all reject atomically.
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

Only read-only source inspection and this plan have occurred on this branch.
No Cargo/npm/build/test, database, native UI or GitHub publication was performed.
Root holds the heavy slot for independent original-history work. No test result,
source admission, grip lifecycle or native behavior is claimed here.

Principal review risks are the anatomy interpretation/current-source revision,
the exact temporal cut needed for release during live-rederived work, all effective
hand readers, and flight/forced-displacement ordering. These are engineering/source
proof obligations within Gate4, not requests to waive product acceptance. The
dead-grappler interpretation and target-death distinction need explicit review.

Exact next action: commit this plan-only checkpoint, send root the full SHA and
the unresolved source/temporal decisions, and wait for root's review before any
implementation. Keep this checkout/branch the sole write target. Subsequent
approval must be recorded with authored and verified status kept separate.
