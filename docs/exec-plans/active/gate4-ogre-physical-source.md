# Gate 4 — Complete Ogre physical source actions

Status: checkpoint1 source/package/closed-policy controls authored, 2026-10-01;
UNCOMPILED / UNRUN. The assigned
source implementation agent is sole writer of
`codex/gate4-ogre-physical-source`, reusing clean checkout `gate4-equipment-table`.
The previous equipment branch and commit remain preserved. Ogre creation and
attack execution remain closed. No PR or runtime acceptance is claimed.

Root transferred this checkout from clean reviewed `df0e0e2` after independent
plan amendment clearance, SHA256
`b2960d12592a93380e0e3cd918ce6a4189b1de7d9ff305bd74b579309380d4f4`.
This writer/scope commit precedes source edits. Checkpoint1 is limited to the
immutable Ogre definition, full-pin loader, installed package checks and closed
physical source policy with authored controls. Ogre current admission remains
explicitly CLOSED; ID-only V1 lookup and historical admission must not gain it.
No attack adapter, OA, UI, pickup or Grapple activation belongs to this checkpoint.
All twelve old immutable revisions and allocation expectations remain exact;
the Ogre expectation is additive. No source-policy helper is claimed to execute
attacks before its later production consumer is integrated and reviewed.
Static formatting, JSON/manifest and byte audits only are allocated here; all
new compilation/tests and runtime evidence remain UNRUN. No push or PR edit.
Fresh fetch at writer takeover still resolves main to `d88a6923`.

## Objective, dependency and authority

Allow a normally created pinned Ogre to use both printed actions through actual
recoverable equipment, own-turn and opportunity attacks, current desktop controls
and durable recovery. Greatclub and both Javelin delivery forms must work before
Ogre becomes selectable. Three Javelins are three distinct finite physical items.
The resulting Large source also supplies a lawful later Grapple/LR producer;
this plan does not activate Grapple or satisfy that separate obligation.

Development base: source-count candidate
`f562f30a8e82931b9bd007f27ab67da2e5b0c6fc`, tree
`e5633723ca9cc038e0828d242eb71c64c2171ba7`. Its independent static review is clear;
focused verification completed normally:51 selected tests, compile/format and
strict Clippy pass. Full canonical/CI and application acceptance remain pending.
It inherits unaccepted foundationfa4,
Shovef9, Air3f and release8c. Main freshly fetched is
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`. These are development dependencies,
not accepted main. Reconcile all accepted prerequisites normally before final
acceptance; never merge a stacked PR into its development base.

Read root AGENTS, product definition, Gate4 checkpoint/protocol, ADR024/029,
source-weapon and gear-count plans. This advances faithful reusable-source
mechanics, real physical custody/action economy, normal creation and exact
save/restart. Catalog enumeration stays Gate6; Ogre does not close the Gate4
weapon, monster-running, Grapple or complete tactical families. No Gate5 work.

Design inputs, fully reviewed before source authoring:

- `tooling/ogre-source-preflight-2026-09-30.md`, SHA256
  `5019edbbbc997a8b44dac333fbe96a308493e84fe867654501e40710accf5b66`.
- `tooling/ogre-source-preflight-independent-review-2026-09-30.md`, SHA256
  `24fb0714abfce3cff1bf8dff509ed519fd4c65810df91b4b1f651be9fc10fcd5`.

## Source and bounded interpretation approved by root

Use only the pinned official SRD5.2.1 PDF, SHA256
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Ogre is printed page312; ordinary weapon properties are pp90–91 and monster
equipment guidance is p255. Preserve exact attribution and immutable old files.

The block is Large Giant, AC11, HP68 (8d10+24), speed40 feet, STR19/DEX8/CON16/
INT5/WIS7/CHA7, initiative−1, PB2, CR2/XP450, Darkvision60, passive Perception8,
Common/Giant. Gear is one Greatclub and three Javelins. Greatclub is +6, reach5,
2d8+4 Bludgeoning. Javelin is +6, melee reach5 or ranged30/120, 2d6+4 Piercing.
It has no printed Multiattack, mastery, immunity, resistance or extra trait.

Root approves this exact normalization under the owner's standing Gate4 authority:
the Ogre's printed formulas override ordinary weapon base dice only, while the
named weapons retain relevant physical Two-Handed/Thrown properties and legal
grips. `TwoHandsV1` describes this specific source's ordinary hand capability,
grounded in its Greatclub use. Neither the hand count nor preservation of every
unprinted weapon property is represented as a literal stat-block quotation or
a universal rule about Giants, Large creatures or two empty loadout slots.
The independent preflight finds this bounded interpretation coherent; review the
actual source/adapter against it before current admission.

Represent two printed actions by three closed execution forms: `greatclub`,
`javelin-melee`, `javelin-thrown`, with clear corresponding labels. The two Javelin
forms are alternatives consuming the same one available Action, not extra uses
or a Multiattack. This avoids adding mixed delivery to unrelated source programs.
Ordinary recovered equipment retains ordinary1d8/1d6 damage for another actor;
the Ogre formula belongs to its exact source, never the Item definition.

## Scope and implementation checkpoints

1. **Immutable source/package and physical program.** Add `ogre-v1.json`, its
   exact loader/pin and installed-file/manifest/NOTICE checks. Keep current
   admission closed until the coherent own-turn/OA/UI path below is present.
   Count3 uses the existing reviewed count planner; creation has four distinct
   quantity-one items, initially stowed, no armor/shield/ammunition. No caller can
   supply counts, formulas or hand anatomy. AC remains11.
2. **Own-turn source weapons.** Extend the closed feature-to-physical-weapon
   mapping only after full-pin lookup. Preserve ordinary planning: actual item
   custody/state, equipment transition/receipt, hands, ability, attack bonus with
   current modifiers, range/reach, cover, underwater, exhaustion, advantage and
   damage type/flat modifier. Existing Goblin/Skeleton matching stays strict.
   Derive only Ogre base damage from the immutable printed program, in both live
   planning and retained reconstruction. Thrown Javelin uses Strength and Thrown,
   never Shot/ammunition. Critical requests are4d8/4d6 with flat+4 once. Actual
   hit or miss relinquishes the exact thrown Item through existing completion.
3. **Physical source opportunity attacks.** Add a typed choice binding feature,
   item and legal grip. It may reuse a retained CreatureFeature option identity
   only with exact reconstruction. The old gripless CreatureFeature choice must
   refuse this new program; no intrinsic fallback may bypass physical planning.
   Eligibility uses current EffectiveHands/custody before offering an option.
   Greatclub requires both usable hands at attack time, while a held one-hand
   Greatclub with the other hand free may select a lawful two-hand attack grip.
   Held Javelin permits melee OA; stowed/thrown delivery does not. Commit exactly
   one Reaction, with no Action, BeginFeature, draw/equipment change or ammunition
   cost. Validate the same retained crossing, Reaction window, source pin,
   selected before-image and physical receipt after cold restore.
4. **Actual application and UI.** Expose the three source forms and a physical
   source OA option carrying only exact item/feature/legal grips. Omit the new
   DTO field when empty to preserve old serialized offers. Keep legacy/intrinsic
   buttons and histories intact. Distinguish all three Javelins using stable item
   choices; do not depend on list position after refresh. Normal creation derives
   item_count4 and no ammunition from the catalog. UI supplies no damage, actor
   authority, invented pin or hidden target information.
5. **Active-map ground weapon recovery.** Implement the missing production pickup
   prerequisite described below. The existing Equip operation accepts only already
   carried weapons; thrown completion leaves real Location custody and a ground
   placement. Changing test custody directly cannot satisfy ordinary-loot acceptance.
6. **Recovery, source review and admission.** Add current admission only when
   every printed form, OA and ground recovery path exists and the full coherent
   delta is reviewed.
   Run focused tests, canonical verification, both-platform exact-head CI and
   packaged native play. Preserve earlier failures and exact source-qualified
   evidence. Reconcile accepted prerequisites; expected-head merge and separate
   literal-main verification precede acceptance.

Relevant seams: `tactical_creatures`, `tactical_definitions`,
`tactical_creature_equipment`, `attacks/creature_weapon`, `attacks/planning`,
`attacks/opportunity`, attack validation/completion, `table_creatures`,
`table_attacks`, `table_movement`, table protocol, desktop tactical API,
AttackForm/OpportunityForm and installed content verification. Extend the existing
physical planner/resolver; do not add a parallel source-attack engine.

### Ground recovery prerequisite identified during independent plan review

The exact139f976 plan review identified a real missing command: neither table nor
tactical action exposes ground pickup; Equip/Unequip require Entity custody.
The original ordinary-loot acceptance is retained and this production prerequisite
is now explicit. It is not discharged by a synthetic Item or private state edit.

The pinned SRD's Attack action (printed p177) explicitly includes picking a weapon
up in its one equip-or-unequip allowance, before or after an attack. Extend that
existing transition with an explicit ground-pickup choice and actual custody
receipt. Do not silently reinterpret accepted legacy Equip payloads. The new
choice is currentflow5-only in both policies; the immutable original command path
and old receipt serialization remain exact. It spends the same single attack
equipment allowance, not another free transfer or an invented mandatory Action.
It creates no extra attack, Action, item grant, ammunition or ownership change.

Bind the actual active actor/controller, one intact quantity-one physical weapon,
its current-map Location custody and exact ground record. Require an available
physical hand through EffectiveHands, ordinary5-foot unobstructed interaction
reach from the actual footprint/height, and a player-visible/reachable item. Do
not use a monster's extended attack reach as automatic object-manipulation reach.
Wrong scene, inaccessible height/geometry, remote/foreign item, occupied/reserved
hand, consumed/destroyed item, duplicate pickup or stale ground proof refuses
before payment or mutation. The reach rule is this bounded physical interaction
interpretation, not a quotation that every creature has identical anatomy.

Atomically move that same Item from Location to the actor's custody, assign the
selected hand and remove only its active ground placement. Keep owner, quantity,
definition, original grant and prior throw/drop cause unchanged. The accepted
journal plus equipment receipt retain the exact prior ground location/position
and causal origin needed to authenticate a pending attack after pickup; current
post-pickup custody alone is insufficient historical proof. Update actual receipt,
source attack and after-equipment validators together. There is no append-only
duplicate live ground authority and no copying of source damage onto the Item.

Expose reachable ground weapons as stable Item choices in the actual Attack form
and any existing after-equipment choice; raw client IDs, position or visibility
claims cannot establish access. A genuine other actor picks up a previously thrown
Javelin before its attack and rolls ordinary1d6 base damage. Cover before and after
equipment allowance, no second change, same-command retry, changed-body no-write,
cold pending attack/damage and independent portable restore from accepted history.
Native play must include a real pickup through the new control. Generic nonweapon
interactions, off-turn looting and travel back to retired scenes remain separate
requirements, without claiming their completion here. Before authoring this
checkpoint, review the concrete receipt/authority changes against these rules.

## Execution and old-history boundary

Flow5 is the current released-candidate executor; Counterspell has not received
a later implemented allocation on this base. The new Ogre producers and physical
source OA choice are admitted only under exact flow5 in Live and Historical mode.
There is no old Ogre producer to admit through a legacy paid-continuation allowlist.
Recheck the allocated execution version before implementation/integration. Preserve
old source commands, source scheduler routes, numeric tags and original accepted
retry lookup before current admission. Keep V1 tactical/picker bytes, all old full
pins, all29 fixture blobs and five original replay suites unchanged. No historical
missing pin is filled from current admission. Public Grapple guards stay closed.

## Required acceptance

| Boundary | Positive and adversarial proof required |
| --- | --- |
| Source/package | Exact printed facts and reviewed interpretation; wrong/missing/tampered installed source refuses with no writes; old source pins remain exact. |
| Physical creation | Four distinct qty1 items through actual catalog/creation; wrong count, duplicate/nil/existing IDs and quantity forgery refuse atomically; retry/reload grants nothing twice. |
| Printed forms | Real own-turn Greatclub, Javelin melee and Thrown; physical hit/miss/critical; all consume one Action, actual hands/items and printed dice, preserving generic situational rules. |
| Finite custody | Throw each of the three real Javelins, covering hit and miss; no fourth unrecovered throw; exact source/loadout/ground positions survive cold/portable continuation without replenishment. |
| Ordinary loot | A real other actor uses the supported active-map Attack pickup to acquire the actual dropped weapon and rolls ordinary dice; no copied Ogre source damage. Exact same-item custody/ground transition, real hand/reach/visibility, before/after allowance, cold/retry and hostile requests are required. |
| Opportunity | Real movement crossing offers the exact held weapon and legal grip; one Reaction, printed damage, no Action/draw; unavailable supporting hand and old gripless choice refuse. |
| Cold/retry | Pending own-turn attack/damage and OA survive real file SQLite close/reopen, independent portable restore and exact original retries; changed bodies/foreign controls/stale handles preserve the complete store. |
| UI/native | Normally create the pinned Ogre; see four distinguishable items and all forms; play physical attacks/throw/OA, real other-actor pickup and pending cold resume in the verified package. No direct state repair or harness replaces native reachability. |
| Compatibility | Every old allocation/fingerprint, immutable content, original capture and accepted command remains unchanged; all original continuation suites execute on the receiving head. |

Current EffectiveHands unit fixtures may test the new OA planner's immediate hand
availability, but do not prove later accepted Grapple reservations or release
refresh. Full Grapple temporal/history/controller/session/Finish/native integration
remains a separate required slice. The actual Large Ogre/Huge Dragon failed-save
route (DC14 vs Dexterity+6/raw1) remains UNMET until those real commands run.
Automatic failure/LR additionally needs a genuine causal condition producer, such
as a lawful Knock Out; no HP, condition, size or LR injection is permitted.

## Validation, risks and exact next action

The original reviewed plan commits changed only this plan. Checkpoint1's authored
source and controls are recorded below; there is no runtime result for them yet.
Main risks are leaking printed dice onto ordinary loot, bypassing supporting-hand
checks through the old OA variant, double-paying source Action/Reaction costs,
replenishing thrown gear, changing legacy serialization and advertising partial
printed actions. The acceptance matrix addresses each explicitly.

One writer per branch. The local heavy slot belongs to the separate guarded-core verifier;
no parallel Cargo/npm/compiler/database/native operation is authorized here.
Static authoring/review may proceed once this plan is independently reviewed.
Next: independent complete checkpoint1 source review, then allocated focused and
receiving-head verification. Keep admission closed and review the concrete next
adapter scope before expanding. The separate ground-recovery plan must reconcile
actual after-equipment timing and custody receipts before that later producer.
Run appropriate focused checks and `./scripts/verify` when allocated the slot;
never promote static source or a test-only constructed definition to acceptance.

## Checkpoint1 author handback — 2026-10-01

Writer/scope authorization was committed first at `51dfc3d`, based on reviewed
`df0e0e2`. The pinned p312 Ogre is now an additive immutable `ogre-v1.json` with
exact statistics, three Action forms, one Greatclub/three Javelins and the reviewed
TwoHands annotation. The loader validates it against the existing V1 vocabulary;
the original tactical catalog and every earlier immutable source remain unchanged.
The source registry now holds thirteen revisions, while the current catalog still
contains eleven sources. Ogre is explicitly excluded and ID-only lookup remains
V1/unknown. No historical Ogre producer is invented.

Historical table creation bypasses current catalog admission, so a common profile
guard rejects this registry-only source after exact lookup. The guard is also used
by initial/retained profile validation, preventing a manually invented profile from
turning immutable lookup into gameplay authority. Existing source creation and
historical paths retain their prior semantics. No source-specific bypass creates
an Ogre for tests.

The sealed `OgreWeaponProgram` descriptor exposes exact-pin, three-form read-only
facts: actual ordinary weapon definition, source delivery, Strength, printed bonus
and printed damage. It checks retained Two-Handed/Thrown vocabulary and never
changes equipment definitions, creates an attack, pays a cost, or claims a current
attack consumer. Checkpoint2's ordinary physical source adapter is its intended
first production execution consumer; OA and pickup remain later reviewed work.
Current intrinsic/physical attack adapters, scheduling and UI are unchanged.

The package manifest adds the exact Ogre file and updates only NOTICE's existing
row for appended attribution/interpretation text. Installed-content validation
requires the declared file, correct length/checksum and exact compiled bytes even
after a source cache has initialized. Existing Tauri and portable packaging copy
the whole content directory; their configuration is unchanged. This is static
distribution-path evidence, not a built or launched package.

Eight new controls are authored: full printed facts; full-pin/ID-only/current
admission boundaries; invented retained-profile refusal; closed physical forms
and ordinary-versus-printed damage; exact four-position Individual allocation
without ammo; exact manifest bytes; actual pure table Live/Historical refusals
with/without a pin plus an old-source positive control; and installed package
missing/undeclared/changed/rehashed refusal, complete-store equality and repair
recovery. Allocation positions are not accepted ItemIds, materialized Ogre gear,
or count-bearing gameplay. The adversarial event/profile controls are explicitly
synthetic negatives. The package case will use the existing real SQLite runtime
when executed, but has not run here.

Both pre-existing count tests retain all twelve original vectors/omission and
fingerprint assertions; only Ogre is excluded from those old-source loops and
tested additively. No original capture, original replay suite, admission picker,
source-control, transport or combat execution field has changed.

Direct rustfmt and Git whitespace checks, strict JSON/manifest verification and
old-content/fixture byte inspection are static author evidence. An initial call
through the rustup proxy could not choose a default formatter; the existing
workspace toolchain's direct rustfmt was then used without changing configuration.
No Cargo/compiler/npm, unit/integration test, database, UI/native, branch push or
PR edit ran. All eight new controls remain UNCOMPILED / UNRUN. No current Ogre,
own-turn/OA/pickup/native positive or genuine Grapple/LR obligation is satisfied.
