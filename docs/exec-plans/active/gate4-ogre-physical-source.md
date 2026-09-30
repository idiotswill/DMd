# Gate 4 — Complete Ogre physical source actions

Status: plan only, 2026-09-30. Root is sole writer of
`codex/gate4-ogre-physical-source`, reusing clean checkout `gate4-equipment-table`.
The previous equipment branch and commit remain preserved. No Ogre implementation,
source admission, tests, PR or runtime result is claimed by this checkpoint.

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
focused verification is running separately. It inherits unaccepted foundationfa4,
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
5. **Recovery, source review and admission.** Add current admission only when
   every printed form and OA path exists and the full coherent delta is reviewed.
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
| Ordinary loot | A real other actor acquires the actual dropped weapon by supported commands and uses ordinary dice; no copied Ogre source damage. |
| Opportunity | Real movement crossing offers the exact held weapon and legal grip; one Reaction, printed damage, no Action/draw; unavailable supporting hand and old gripless choice refuse. |
| Cold/retry | Pending own-turn attack/damage and OA survive real file SQLite close/reopen, independent portable restore and exact original retries; changed bodies/foreign controls/stale handles preserve the complete store. |
| UI/native | Normally create the pinned Ogre; see four distinguishable items and all forms; play physical attacks/throw/OA and pending cold resume in the verified package. No direct state repair or harness replaces native reachability. |
| Compatibility | Every old allocation/fingerprint, immutable content, original capture and accepted command remains unchanged; all original continuation suites execute on the receiving head. |

Current EffectiveHands unit fixtures may test the new OA planner's immediate hand
availability, but do not prove later accepted Grapple reservations or release
refresh. Full Grapple temporal/history/controller/session/Finish/native integration
remains a separate required slice. The actual Large Ogre/Huge Dragon failed-save
route (DC14 vs Dexterity+6/raw1) remains UNMET until those real commands run.
Automatic failure/LR additionally needs a genuine causal condition producer, such
as a lawful Knock Out; no HP, condition, size or LR injection is permitted.

## Validation, risks and exact next action

This commit changes only this plan. No source or runtime result exists yet.
Main risks are leaking printed dice onto ordinary loot, bypassing supporting-hand
checks through the old OA variant, double-paying source Action/Reaction costs,
replenishing thrown gear, changing legacy serialization and advertising partial
printed actions. The acceptance matrix addresses each explicitly.

One writer per branch. The local heavy slot belongs to the separate count verifier;
no parallel Cargo/npm/compiler/database/native operation is authorized here.
Static authoring/review may proceed once this plan is independently reviewed.
Next: independent exact-plan review, resolve findings, then implement checkpoint1
with no current admission and review source/physical policy before expanding.
Run appropriate focused checks and `./scripts/verify` when allocated the slot;
never promote static source or a test-only constructed definition to acceptance.
