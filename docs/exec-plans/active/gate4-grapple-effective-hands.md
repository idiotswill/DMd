# Gate 4 — Derived effective-hand reservations

Status: **plan only, awaiting root review before implementation**.
Writer: `shove_publish_correction`, coordinated by root; sole writable checkout
`gate4-attacks`, branch `codex/gate4-grapple-lifecycle`. No PR for this checkpoint.
Date: 2026-09-30.

## Objective, authority and exact starting point

Prepare the existing physical action planners to account for a hand reserved by
an ordinary Grapple. Use one derived reservation view with the real physical
loadout, keeping Item custody and relation authority separate. This checkpoint
does not expose a playable Grapple, Escape, release or dragging action.

The parent [Grapple lifecycle plan](gate4-grapple-lifecycle.md) remains the full
feature contract. This bounded checkpoint advances the product definition's
authoritative tactical timing, environmental interaction, private information,
exact suspension and recovery requirements. It cannot satisfy their production
acceptance by itself. Gate 4 and all its remaining families stay active; no
obligation moves to Gate 5. Root AGENTS, the gate-execution protocol, Gate04 and
the parent plan's ADR025/026/028 and pinned SRD authority remain binding.

Received clean source/domain head was
`522d427b0387a5a2f415e0973888a1a2b46e1638`, tree
`3fe2fbe443268d8a7767819beb4b03428eb0869a`. Its installed Goblin V2 binding
correction passed independent static review, not executable tests. Review:
external `tooling/gate4-grapple-522d427-corrective-independent-review-2026-09-30.md`,
SHA256 `cdd25e4d86d5d774756e38af9023cb4e399c5c63a4ae3e65e2212389a673dc6b`.
The original missing-binding finding on `660631a` remains recorded in the parent
plan; it is not retroactively relabeled a passing head.

Root authorized normal integration of published Shove
`c1124004dfb3fcc84d105b512a5fe4f9502a7727`. Merge
`c3f3e73488840c5f27aa3f099ec702d893b203b1`, tree
`b5f6893df46eebe86b6d187281105b2f2f85b865`, completed without conflict. Relative
to `522d427`, it changes only the reviewed Air Shove setup test and the Air/Shove
plans (three files, 174 additions / 20 removals). It preserves both parents and
adds no Grapple gameplay. Shove's corrected execution remains unverified here.

The external next-slice map was read in full and checked against the current
planner/component/shield/app callers:
`tooling/gate4-grapple-effective-hands-next-slice-map-2026-09-30.md`, SHA256
`d8ddf7780880d31ef944a8aacec8ac3b5115b0ac0061c373816d833e000c6c73`.
That map supplies design input; this checked-in plan defines the proposed scope.

## Representation and authorization boundaries

Keep `HandAssignment::{Free, Item}` and all existing physical loadout wire bytes
unchanged. A creature is not an Item. No fabricated ItemId, third physical slot,
second stored occupancy attachment, serialized mask or client permission flag
is introduced. Derive a nonserialized view with private/internal construction
and read-only hand predicates from actual RulesState plus relevant source proof.

Physical Free means no Item occupies the slot. Effective availability additionally
requires that no live or provisional relation reserves it. Unknown anatomy is
not zero hands: it refuses only new ordinary Grapple admission and does not
disable existing equipment, source spells or intrinsic attacks for Mage, Hag,
Air or another unannotated source on the no-new-authority path. Full Human
creation reconstruction and exact annotated Goblin source pins remain the only
initial ordinary Grapple anatomy admissions.

| Record/context | Required interpretation in this checkpoint |
| --- | --- |
| Live outgoing grip | Reserve its proved grappler's exact hand. An incoming Grappled relation alone reserves none of the target's hands. |
| Pending Attempt before final outcome | Reserve the selected hand even when there is no live attachment. A completed, resisted or withdrawn attempt is not provisional occupancy. |
| Attempt's own admission reconstruction | Exclude only that exact authenticated declaration identity at its original cut: grip ID, paid origin, actor and hand must all match. No public `ignore_pending`, actor-only match or arbitrary mask. This internal context is unavailable as live command authority. |
| Ended retained proof/cut/receipt | Reserve no current hand. Never treat a retained proof vector as live occupancy. Its possible historical use is a separate later authenticated consumer. |
| No new authority | Preserve the previous physical-only path, error/source-query ordering and omitted JSON fields. Do not invoke new anatomy admission for every old action. |
| Missing/temporarily taken RulesState | Missing authority is not proof of an empty reservation set. Pass the actual separately held RulesState where required, or fail the relevant internal context explicitly; never use `unwrap_or_default` to erase occupancy. |

The preparatory derived view can check exact source identity, record shape,
actor/hand uniqueness and physical collision. Those checks do not prove that
synthetic records came from accepted commands. Keep the early kernel and tactical
`has_unimplemented_grapple_records` rejection, old-schema rejection and recovery
anchor guards intact, including provisional, retained, raw-only, self-only and
causal-fall fields. Every public command/query acceptance path must still refuse
this unimplemented authority. Pure controls exercise the internal planner seam
and must be labeled accordingly.

Do not add a trusted public constructor for reservations. The existing public
weapon/component entry points must preserve no-authority behavior; any entry
point lacking the exact new context must reject new authority instead of silently
passing an empty view. Reuse one underlying planner algorithm. Source-bound
read-only availability shared with the application may expose predicates, never
an API accepting caller-invented reservations or hidden relation identities.

## Physical validation ordering and recursion avoidance

Keep `tactical_inventory::validate_loadout` a physical Item/custody primitive.
The actual call chain is:

`ordinary_grapple_anatomy` → `validate_creature_profile` →
`creature_current_armor` → `tactical_inventory::validate_loadout`.

Adding the full anatomy/reservation factory inside that primitive would recurse.
Validate physical source equipment first, establish anatomy/source authority,
derive reservations, then check Item/reservation collisions as a separate layer.
Pass the resulting context through slot predicates instead of repeatedly
reconstructing the whole creature from each predicate. Existing source and
starting-PC equipment materializers continue creating real Items only.

An explicit planner `WeaponAttackInput.loadout` can be a before-image rather
than the state's present physical equipment. Compose reservations with that
explicit image; do not silently replace it with current inventory. Keep current
action context distinct from own-Attempt admission and sealed historical attack
context. The latter remains unavailable for new Grapple authority in this
checkpoint: no saved cut alone can authenticate an earlier attack.

## Caller scope for the proposed implementation

Short rules paths below are beneath `crates/dmd-rules/src/`; app paths are beneath
`crates/dmd-app/src/`. Desktop components are beneath
`apps/desktop/src/components/`.

| Caller | Included work | Excluded or preserved boundary |
| --- | --- | --- |
| `dmd-rules/src/tactical_weapons/equipment.rs` and `tactical_weapons.rs` | Use the shared derived predicates for draw/equip, OneHand, TwoHands, one-handed ammunition loading and before/after equipment changes. Check the chosen physical before-image. | Unequip/throw clears matching Item slots only; never remove a reservation. Preserve independent Thrown draw, ammunition identity/count, the existing single Attack weapon-change allowance and old plans. No fake WeaponUseChoice or weapon receipt for Grapple. |
| `tactical/attacks/planning.rs`, `creature_weapon.rs`, `intrinsic.rs`, `validation.rs` | Thread current authenticated planner context through the shared source weapon path; identify and protect reconstruction boundaries. | Keep existing item/ammunition undo and source pins. Do not implement new historical cuts, trust current reservations for a selected historical attack, or replace Goblin Scimitar's original Advantage-dependent damage program. Old no-authority reconstruction remains exact. |
| Ordinary unarmed, source body attacks and Shove | Add compatibility controls proving that the hand helper is not a generic free-hand requirement. | Lawful kicks/headbutts, intrinsic body attacks and Shove stay available with both hands occupied. No inferred body-part grant, new unarmed equipment allowance or changed Shove request shape. |
| `tactical/shields.rs::change` and app `table_shields.rs::options` | Use the same derived selected-hand availability; validate conflict before Action spending and expose the same legal donning hands. | Doff only the actual shield Item and preserve other reservations. No free shield allowance, reprice or new armor source semantics. |
| `tactical_spells.rs::validate_spell_components`, `tactical_spells/binding.rs`, `tactical/casting.rs` | Compose S-only free-hand and M-access checks with reservations; preserve actual held-material sharing and source component waivers. | Legacy `spellcasting.free_hand` cannot override new authority. No generic Focus/ComponentPouch grant: production `material_fact` still refuses unsupported grants. No refund, rebind or repricing of an already paid spell after release. |
| App `table_casting.rs` | Continue using the shared `bind_spell` result for offered legal variants; propagate source-bound availability rather than duplicate policy. | No UI-side component rules or removal of source waivers. No synthetic spellcasting grappler presented as genuine admission. |
| `tactical/attacks/opportunity.rs`, `tactical/movement.rs`, app `table_movement.rs::opportunity` | Use the shared current-hand predicates for unselected weapon options, including the app's separate two-hand check. Keep the full current option equality and participant rescan. | Do not implement release/menu refresh or weaken `validate_opportunity`. An already selected OA is historical admission, not a new live menu. Keep Answered/Declined distinct from prior automatic Unavailable. |
| App `table_attacks.rs::options` | Audit candidate enumeration and use shared availability only where the actual chosen grip/equipment change is known; keep candidate versus executable-plan distinction explicit. | Do not blanket-filter a carried weapon merely because a current Item occupies the hand: an allowed explicit before-attack unequip/draw may make it legal. No duplicate planner or invented optimistic equipment mutation. |
| App `table_equipment.rs`, `table_projection.rs`, desktop `AttackForm.svelte` / `ShieldForm.svelte` | Audit current physical-hand consumers and record the eventual owned disclosure seam; preserve all current DTO bytes and labels in this guarded checkpoint. | New occupancy DTOs, relation handles, reserved/free UI labels, draw-choice UI changes and privacy/native acceptance belong to the full resolver/app checkpoint before guard removal. ShieldForm continues consuming backend legal hands. |

Current options may use current derived occupancy. A future lawful release must
refresh an unanswered OA menu causally, preserving crossing, work, owner and
answered chronology; `advance_segment` must still rescan prior Unavailable
participants. That implementation and its retained refresh proof are excluded
here. Likewise, sealed attacks and damage descendants require source/replay-
authenticated original cuts, not a fallback from historical to current context.
If the preparatory composition cannot preserve the old path without solving
those temporal seams, stop that change and bring the concrete counterexample to
root before expanding this checkpoint.

A selected attack's sealed equipment admission and its later owned
after-equipment choice are distinct cuts. The latter must use current occupancy
when that new choice is accepted; it cannot reuse a historical reservation view
to overwrite a currently occupied hand. This checkpoint may prepare the shared
physical operation with explicit context, but must not fabricate sealed history
or enable that new lifecycle to demonstrate it.

## Proposed authoring sequence and meaningful verification

After root reviews this plan, author one coherent checkpoint with the derived
view, collision/availability predicates and the included caller composition.
Keep the physical validator, old wire representations and execution guards
unchanged. Review all call sites for missing RulesState and historical context
before connecting them. Do not add dead-code bypass flags solely to make a
synthetic fixture execute through production command admission.

Author focused internal/pure controls that establish behavior across the shared
planners, not merely mirror the helper implementation:

1. Live outgoing and pending Attempt each reserve the exact selected hand;
   provisional occupancy works without a live attachment; incoming and ended
   proofs reserve none. Duplicate/colliding reservations and an Item/reservation
   overlap reject. Own-admission exclusions reject wrong ID, origin, actor or hand.
   A missing separately held rules context cannot erase a reservation.
2. Use actual weapon definitions: one usable hand plus a reservation permits a
   lawful one-hand attack/throw, rejects TwoHands and one-handed ammunition
   loading, and preserves custody/quantity. Explicit unequip frees an Item only;
   thrown aftermath retains the other reserved hand. Include the actual Goblin
   weapon planner path, with source damage checks left intact.
3. Shield don collision rejects before costs; doff removes only the shield Item.
   Source/body attacks, kicks and existing Shove retain their old hand rules.
   Verify the public guarded path still refuses synthetic Grapple authority;
   these controls are not real accepted Grapple histories.
4. S-only versus held M/S sharing and source waivers retain their actual
   distinctions. A legacy free-hand bool cannot override a reservation. Pure
   focus/component-pouch and spellcaster-plus-grip controls must say they do not
   establish a production grant or admitted spellcasting grappler.
5. No-authority equality controls compare existing plans, request tuples, errors,
   source-query ordering and absent optional fields. Exercise old Mage/Hag/Air
   equipment and spells without ordinary-hand annotations, plus original Goblin
   V1 identity. Preserve the existing old-history fixtures byte for byte.
6. Current unanswered OA option controls cover both-hand conflicts and matching
   app/rules predicates. Keep selected attacks on the unchanged old path and
   reject unauthenticated new historical contexts. Two-handed melee/reach
   examples are pure planner controls until genuine source acquisition exists.
7. Retain hostile kernel/schema/restore-anchor controls for every new authority
   form. Prove the preparatory helper does not remove the early rejection. Full
   accepted release, refresh chains, Goblin damage across release, SQLite cold
   retries and native ownership/privacy evidence remain in the parent plan.

No test in this plan has been authored or executed by this checkpoint. Once
root allocates the heavy slot, run appropriate focused tests, `./scripts/verify-fast`
and canonical `./scripts/verify` on the actual candidate, followed by independent
full-diff review and all required exact-head CI. Failures require actual output
inspection and narrow fixes; older development success cannot validate this head.
Pure fixture success cannot be labeled gameplay, cold recovery or native proof.

## Non-goals, remaining risks and exact next action

No new Grapple command, work producer, paid window, save/check, live condition
projection, Escape, withdrawal, release, dragging/carrying, self-only movement,
grip death/Incapacitated cleanup, fall cause, OA refresh, historical cut producer,
transport capability, source revision, protocol tag or execution version is
implemented here. Itemless-grappler cleanup and living-holder preservation after
target death remain explicit full-lifecycle obligations. Root separately owns
release5/main integration, source coexistence, original-history receiving proof
and the witnessed-memory design updates.

The main risks are source-validation recursion, treating physically Free as
anatomy, losing separately held rules authority, trusting synthetic/retained
records, changing before-image equipment reconstruction, over-filtering legal
equipment choices, and recasting selected historical attacks as current options.
Keep each distinction explicit in code review and tests.

Current verification is limited to source/document reads, exact merge/parent
inspection and Git diff checks. No helper/gameplay code, Rust test, Cargo/npm,
database/native operation or remote publication was performed for this plan.

Exact next action: return this plan-only commit and its complete diff to root for
review, then freeze the branch. Do not start the helper or caller edits until root
assigns that bounded authoring checkpoint. Root retains the sole heavy slot.
