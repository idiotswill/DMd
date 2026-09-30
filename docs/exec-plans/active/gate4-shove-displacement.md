# Gate 4 — Source-bound Shove and displacement

Status: implementation and tests authored; no executable verification or feature
acceptance. Writer: `gate4_shove_recovery`, exclusively. Root owns
integration, the heavy verification slot and publication. Branch:
`codex/gate4-shove-displacement`; root-created clean base and freshly fetched main
`c4d8c34c19b5c92eca789f292f99632a0107d861` (2026-09-28). No PR yet.

## Objective and authority

Deliver the ordinary own-turn Shove through actual domain/rules, table transport,
desktop forms and durable recovery: spend an Attack action attack, target chooses
Strength/Dexterity save, source-correct physical/automatic/voluntary save and any
Legendary Resistance resolve, then the shover chooses Prone or a real five-foot
push with support/flight/fall/liquid/concentration consequences.

Read root AGENTS, product-definition clauses on production completion, physical
dice, agency/PvP, information boundaries, environmental interaction and exact
suspension; Gate04, gate-execution protocol, active tactical/effect/closure/unarmed
plans and ADR025/026/028. The owner-approved external Shove design and reviewed
Air prerequisite govern this slice; their durable requirements are restated here.
Old plan statements are historical evidence, not a substitute for current code.

Pinned source: English SRD5.2.1, recorded by source.json and the repository
provenance document. External PDF SHA256
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Relevant pages: 14–16, 177–178, 180–191; Air Elemental 258–259 for the separately
owned source prerequisite. No 2014 opposed Athletics/Acrobatics interpretation.

## Scope and invariant decisions

- Five-foot body reach; target no more than one size larger; no free-hand rule,
  held Reach weapon extension, fabricated item, attack roll or critical branch.
  DC = 8 + authenticated shover Strength modifier + source proficiency. Exhaustion
  affects the target D20 test, not this DC. Targeting requires current location
  before hidden source checks; total cover, dead/self and may_harm apply.
- Spend exactly one existing Attack action attack or start the ordinary one-
  attack action. No new extra attack, bonus/Reaction/target movement cost. Retain
  the paid window, original issuer and distinct later selecting commands.
- Target alone chooses STR/DEX. Derive actual source save bonus, worn-armor
  training, applicable Dexterity cover, Dodge/Restrained and cancellation; use
  actual exhaustion, automatic failure, voluntary failure, Inspiration and
  enabled house rules. Do not globally change old save requests.
- Legendary Resistance precedes any outcome. Only the shover chooses Push/Prone
  after final failure. Success or resisted failure completes the paid attempt.
  No Shield-hit, weapon mastery or Savage Attacker work is fabricated.
- Prone immunity is accepted no-effect completion after the paid choice; preserve
  its source proof, do not attach an immune condition, refund, substitute Push,
  create a fall or reveal hidden immunity in option filtering/errors.
- All stages own selected work in the existing resolution/frame/trace. Child
  falls/concentration inherit actual cause/authority; no second queue and no
  synthetic Admin command. Add source-bound displacement/fall evidence where
  existing MovementEnd/FlightLost cannot prove the accepted Shove cause.
- A healthy actual flyer or Hover source is not made to fall by choosing a Walk
  primitive. Actual ledge/support loss and non-Hover Prone flight loss use the
  existing liquid/check/damage/vitality/concentration stack. No OA on push/fall.

## Controller and consent boundary

Resolve both controllers from actual PC ownership or CreatureController::Player;
source-NPC type and Host routing are not consent. Distinct-player actor pairs are
unsupported and must refuse before costs in all PC/source combinations. A
player-owned source cannot be driven by Host to bypass this rule.

Support an owned actor versus an actually Host-controlled opposing source
creature, the reverse, and genuine opposing Host/Host source creatures. Both
player-controlled actors are refused before costs even when they have the same
owner. Require mutual authored opposition and no ally relationship, alongside
actual controller/source authority and attendance. Do not claim that this enforces
unconstrained contract prose or infer consent from labels, saving-ability choice
or voluntary failure. Friendly/neutral and contract-specific cases without typed
evidence remain refused; typed policy/consent remains explicit Gate4 work.

## Reviewed geometry contract

Root reviewed the standard grid convention: a grid-direction displacement of
exactly ten half-foot units under existing max-axis grid distance, strictly
increasing existing occupied-space participant distance from the shover; retain
the true start/end/path and source cause. Client supplies direction/intent, not
a forced flag, mechanics, arbitrary shortened movement or final authoritative
position. Do not filter directional controls using hidden geometry.

Every Push intent enters the same retained Host geometry review, without exposing
hidden obstruction/unsupported distinctions to the player. Closed choices are
CommitExactPush (rederive and accept only exact legal full-body swept motion),
ConfirmBlockedNoMovement (only proven physical obstruction/bounds under the
recorded bounded convention), or ReturnToShover (retain the paid failed-save
opportunity for that actor's explicit new choice). No collision damage, refund,
automatic Prone substitution, arbitrary position/distance or invented support.
Unsupported Air Form/coupled movement cannot be confirmed as blocked or overridden;
only ReturnToShover is available. Mandatory Host review is the current bounded
adjudication scope and requires genuine native proof. Only the actual shover can
retry Push or change manually to Prone.

## Compatibility and shared dependencies

Root accepted additive dispatch: new Shove commands only on current ShieldMissileV1
(flow4), not old flows. No new flow number is reserved. New ShoveSave role tag18
is allocated after existing Medicine17; preserve the deliberate tag15 gap and all
old UUID tags. Confirm allocation again at later release5/dependency integration.
Old UnarmedStrike, source definitions, save modifiers, movement/fall producers,
UpgradeExecution 1→2, source fingerprints and historical serializers remain
unchanged. Optional new record fields skip serialization when absent.

- Air prerequisite: `gate4_air_recovery` exclusively owns
  `codex/gate4-air-source-admission` in gate4-content. Consume only root-integrated
  immutable source resolution and source-aware geometry boundaries. No invented
  Air profile or altered old catalog. Genuine Prone-immunity table/native
  evidence stays unresolved until that source dependency is verified.
  Its immutable local checkpoint is
  `4a27cb0776f1ba5efe0bc2c9fa4a3e782093039e`; this Shove branch remains on c4d8
  without that checkpoint. Root must reconcile the added creation source pin and
  source-aware save helper when integrating; source tests here use frozen V1.
- UI: root exclusively owns the reviewed focus/remount correction and local
  main reconciliation at `65b7606bd486120a1c650e44028e4195763a5b6c`. Integrate it
  precisely later; do not build a second focus/draft framework. Every fresh raw
  request is blank while its viewport remains reachable. Exact retries retain
  the original envelope rather than creating a fresh request.
- Release/history: root owns original flow4 producer captures/baseline and
  release5. Do not edit historical producers, fixture bytes or expected results.
  Archive/provenance/parent interpretation must be checked on the integrated head.

## Implementation sequence and acceptance

1. Closed Shove command/record/work/role, controller and paid-window admission;
   independently review concrete geometry and privacy boundaries early.
2. Source save derivation, target choice, physical/automatic/voluntary paths,
   resistance, shover outcome and immune accepted no-effect completion.
3. Authenticated actual push and Prone, support/flight/fall/liquid/concentration,
   exact selected work and causal validation, without changing old producers.
4. Table source/PC authority, stage-bound private capabilities and projection,
   ordinary desktop forms and proper fresh-request identity/remounts.
5. Genuine rules/source, actual file SQLite cold reopen/retry/export/hostile
   restore, privacy/controller matrices and meaningful UI regressions.
6. Root-coordinated source/UI/release integration, independent complete review,
   serial focused/fast/canonical checks, all six exact-head CI jobs, actual native
   play, immutable historical parity and protected merge/post-main verification.

Tests must cover legal full hands/size/range and pre-cost rejection, both actor
source kinds, actual save modifiers/cancellation and raw/automatic/voluntary
failure, genuine size-compatible Legendary Resistance and Air Prone immunity,
no-effect exact retry, lawful five-foot motion and physical consequences, all four
distinct-player refusal combinations, Host/source-owner spoofing, wrong-stage or
stale capabilities, no hidden DC/immunity/geometry oracle, and no OA on forced
motion. A source-size or unsupported setup failure is not evidence for a later
mechanic. Preserve original causes through nested child work and adversarial
state/trace/receipt mutations. Real desktop operation and pending-state restart
are required; tests alone do not complete the feature.

## Validation status and next action

2026-09-28: clean branch/base and current AGENTS/source/design/code reads confirmed.
This plan is the first repository write. No implementation or tests have run.
Root explicitly retains the sole heavy slot: **do not run cargo/npm/build/tests or
push** until coordinated. Static inspection, rustfmt, diff checks and coherent
local commits are allowed; label every authored-but-unrun test accordingly.

2026-09-28: root accepted the closed Host geometry choices, flow4/role18 direction
and narrowed controller boundary above. Same-owner PC/source rejection is required
alongside the distinct-player matrix; no custom policy interpretation is claimed.

2026-09-30: recovered the preserved draft without changing its base. Authored the
closed Shove record/role/work and source-bound reducer; source/controller refusal,
target save and Resistance, chosen consequence, exact Host geometry review,
actual flight/support/fall cause, and retained save/route/work/fall validation.
Added private stage/actor-bound transport capabilities and ordinary desktop
attempt/save/outcome/Host-review forms. No dependency code has been copied from
the Air or UI branches.

Authored, but not executed: reducer cases for paid full hands, source STR/DEX,
automatic/voluntary/physical/Inspiration results, actual size-compatible Dragon
Resistance, all same/different-owner PC/source refusal pairs, Host opponents,
clear/blocked/returned Push, healthy source flight, Prone flight loss, ledge
damage/concentration ancestry and hostile retained-state mutations. App cases
use normal creature creation and explicit Host assignment, actual file SQLite
close/reopen, mirror restore, exact and changed retries, wrong audience/stage and
stale capability rejection, private projection, forged export rejection, and a
real supported ledge over liquid with raw landing/damage dice. UI cases cover
explicit blank choices, opaque-stage remounts, all directions and closed Host
rulings, observer-scoped targets, budget disabling and uncertain-ack retries.
These test descriptions are intended coverage, not passing evidence.

The independent read-only reviewer identified fixture ammunition/control and
retained proof weaknesses before the checkpoint. Replaced the pure Goblin fixture
with a genuine Mage source, assigned the app source to Host through real commands,
and strengthened final-save authority/result, exact source-node ancestry and
accepted route/landing reconstruction. Independent exact-commit review remains
required. No native evidence, compiler result or gate acceptance is claimed.

Next: hand the coherent local checkpoint to the independent reviewer, fix any
findings, then let root integrate Air and the exact UI focus/remount correction.
Root owns serial focused tests, fast/canonical checks, historical flow4 producer
parity, release5 reconciliation, exact-head CI and genuine native desktop capture
(including fresh raw-face blanking/reachability and pending-state restart).
Actual Air Prone-immunity, Air Form refusal and healthy Hover proof must run only
against the integrated real source; do not replace them with invented mechanics.
The branch's behavior remains unverified until those checks succeed.

Remaining Gate4 work stays binding: Grapple/Escape/dragging, unarmed equipment
allowances, opportunity/Ready Shove, other mastery/spell displacement, teleport,
mounts, full zone contacts, improvised object/body/self actions, natural tactical
intent, typed consent, and all other twelve-family/eighteen-mechanism obligations.
No next gate begins and no scope reduction is implied by this bounded slice.
