# Gate 4 — Source-bound Shove and displacement

Status: implementation authored; the first complete remote runtime runs failed
the Air setup. The published correction is frozen for its existing remote run.
The separate local Air/release integration checkpoint below is planned and
unverified; feature acceptance remains pending. Root owns the heavy verification
slot and delegated this 2026-09-30 integration to one branch writer. Branch:
`codex/gate4-shove-displacement`; root-created clean base and freshly fetched main
`c4d8c34c19b5c92eca789f292f99632a0107d861` (2026-09-28).

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

- Air prerequisite: root now exclusively owns
  `codex/gate4-air-source-admission` in gate4-content. Consume only root-integrated
  immutable source resolution and source-aware geometry boundaries. No invented
  Air profile or altered old catalog. Genuine Prone-immunity table/native
  evidence stays unresolved until that source dependency is verified.
  Its immutable local checkpoint is
  `4a27cb0776f1ba5efe0bc2c9fa4a3e782093039e`, imported by local merge
  `4567aaf` under root's explicit integration authorization. The new ledge
  creation supplies the exact Mage source pin, and the shared save helper keeps
  Air's immutable source-aware lookup. Original ID-only helpers remain frozen V1.
- UI: root exclusively owns the reviewed focus/remount correction and local
  main reconciliation at `65b7606bd486120a1c650e44028e4195763a5b6c`. Integrate it
  precisely; do not build a second focus/draft framework. Every fresh raw
  request is blank while its viewport remains reachable. Exact retries retain
  the original envelope rather than creating a fresh request.
  Local merge `4be7b3e` now preserves that exact commit in ancestry; new Shove
  prompts extend its existing kind/actor/key/stage identity and focus metadata.
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
accepted route/landing reconstruction. Independent review of checkpoint
`b7e4e71aa446e1537d16f1d02d2237e3d61258fb` (tree
`dbea4f13b62b88b3e6b731820e1eda39c14e05d6`) reported no remaining concrete
findings; that review was static only. No native evidence, compiler result or gate
acceptance is claimed.

2026-09-30 integration: root authorized the sole writer to merge the exact Air
and UI checkpoints locally, preserving both original commits. Spatial conflict
resolution retained Air's source-special geometry checks and Shove's separate
positive obstruction proof; UI conflict resolution retained the selected-actor
map and both Shove forms. No flow/release5 change was introduced.

Additional authored, unexecuted app cases select Air's exact pin from the real
installed Host catalog, create it normally, assign Host through accepted source
control and establish a supported PC beside actual Hover flight. Real file SQLite
reopen/independent restore/exact retry covers paid immune Prone with no effect or
fall, a healthy airborne five-foot Push, and an initially clear Air body pushed
toward a private partial solid: both Commit and Blocked must reject unchanged,
then Return preserves the paid save for an explicit immune Prone choice. The
source's +2 STR/+5 DEX and immunity/Hover facts are asserted from the created actor;
no custom immunity or flight fixture substitutes for that source.

The existing UI focus path is unchanged. Shove supplies its prompt identity and
focusable fieldset. Added TableApp interaction cases cover target save choice,
fresh physical save, shover outcome, Host review, returned blank direction/elevation,
fresh child damage and concentration dice, and absence of an unavailable Host
choice. The form checks and TypeScript syntax parse do not establish native focus
or gameplay acceptance. Rustfmt and diff checks passed; no cargo/npm/build/test
command ran in this integration task.

Independent static review of integrated checkpoint
`a09fee2c968b24bd27c245e8c24c60c82afba06e` reported no concrete findings. Root's
subsequent presentation review identified internal half-foot coordinates in the
Host push review. Those start/end locations now display feet east, feet south
and elevation in feet, consistent with the tactical map's units and orientation.
Only presentation is converted; transport and geometry values are unchanged.
This narrow correction passed a Svelte syntax parse and diff check; no build
or extra copy-only tests ran.

Next: root reviews the presentation correction and transfers the stable checkpoint
to the serial executable verification slot.
Root owns serial focused tests, fast/canonical checks, historical flow4 producer
parity, release5 reconciliation, exact-head CI and genuine native desktop capture
(including fresh raw-face blanking/reachability and pending-state restart).
Actual Air Prone-immunity, Air Form refusal and healthy Hover proof must now be run
against the integrated real source before those acceptance items can close.
The branch's behavior remains unverified until those checks succeed.

### Draft verification checkpoint — 2026-09-30

Root reviewed the complete presentation-only correction at
`3f8024bde89f8878bcc42197e42d60169182cceb` after independent full static review of
integrated a09fee2. Root then normally merged Air's documentation-only checkpoint
`1a0a5e75bfbde98c7621c421fbdc9d5b4750ab92`, preserving both dependency histories.
That reconciliation changes no production or test source.

Root authorizes a dependent draft PR targeting the Air source branch so remote CI
can find compiler, test and integration failures while the original-source flow4
baseline owns the local heavy slot. The draft also inherits UI65 through the
already-recorded development merge; it does not independently accept or duplicate
that UI objective. Before final acceptance, merge/reconcile the verified Air/UI
prerequisites and retarget main so the remaining review covers the Shove slice.
No prerequisite, test or native acceptance is waived by draft publication.

This supersedes the earlier publication hold only. No local Cargo/npm/build/test
run has been performed for this draft. Root must inspect actual remote output,
fix concrete failures, and then coordinate all remaining focused, canonical,
history, release5, native and final exact-head checks above. Main remains fixed
until the original-source capture/baseline proof succeeds.

Remaining Gate4 work stays binding: Grapple/Escape/dragging, unarmed equipment
allowances, opportunity/Ready Shove, other mastery/spell displacement, teleport,
mounts, full zone contacts, improvised object/body/self actions, natural tactical
intent, typed consent, and all other twelve-family/eighteen-mechanism obligations.
No next gate begins and no scope reduction is implied by this bounded slice.

### First remote verification finding — 2026-09-30

Draft head `d8e1c7290fdf3787fa2e34eedd71bdc7d5c93bbc` failed strict Clippy on
both Linux and Windows stable before Rust workspace tests ran. The finding was
`collapsible_if` in the Shove save-parent validator. Collapse that nested guard
without changing either parent/occurrence predicate, evaluation order or error.
The strict lint remains enabled. Both MSRV checks, architecture and genericity
checks passed; both Windows frontend runs passed 122 tests across 19 files with
zero Svelte errors or warnings. No Rust runtime or package acceptance follows
from those partial results.

All six original job logs and the failed source/tree identity are preserved in
root's external `tooling/shove-d8e1c72-ci/failure-evidence.json`. The correction
requires a fresh remote run; local heavy verification remains reserved for the
original-source baseline. All remaining acceptance obligations above still apply.

The next head `7adbf99f33be00aa98f728190f12b1657f0450b0` cleared that validator
lint. Linux runtime job 109934925107 then stopped at strict `replace_box` in the
table test's wrong-stage command mutation, before Rust tests. Replace the existing
box's inner value directly; the mutated Save body and full-store refusal assertion
are unchanged. The complete failed log is preserved externally under
`tooling/shove-7adbf99-ci/`. Formatting and diff checks pass for this correction;
fresh exact-head CI is required. No assertion or lint is weakened.

### Source-control setup correction — 2026-09-30

Head `5c70954f9a9b765a4154470b0873d452fe4e654c` completed the Linux table-loop
suite with 56 passing tests and one failure in 6901.05 seconds. Job 109938078211
failed the real Air Shove case at the shared legacy typed-command helper:
`Refresh this legacy request before submitting new input.` The actual complete
job log is preserved externally at `tooling/shove-5c70954-ci/job-109938078211.log`.
The four Magic Missile cases and the separate ordinary Shove case passed in that
run; the Air scenario did not pass, and this is not a complete workspace result.
Windows stable job 109938328359 independently completed with the same failure at
`table_loop.rs:227:56`: 56 table tests passed and one failed in 8238.51 seconds.
The ordinary Shove case and all four Magic Missile cases passed there too. Its
complete log is `tooling/shove-5c70954-ci/job-109938328359.log`. Both jobs ended
normally with failed conclusions; neither was cancelled or replaced by this fix.

The Air setup had enabled source control but then submitted PrepareEquipment,
PrepareBattlefield, Begin, and both initiative faces through old typed helpers.
Use the current presented transport and each current opaque initiative handle
for those five steps. Preserve the original actors, source pin, geometry, dice
values, and all scenario assertions. These accepted setup steps now also use the
existing file close/reopen, independent restore, exact retry, and changed-body
no-write helper. The production rejection of fresh legacy commands after source
control activation remains unchanged. Explicitly assert normal single-d20
initiative before reporting the original physical faces.

The branch also contains the independently reviewed Air manifest/generator
correction from `021421c4990c686e31a6ad8da0b6ef3571f087ae`, brought in by normal
merge `158256ac2b3fe82a3069890344f05ef1c4122cb5`. That corrects the known later
six-definition expectation without changing source content or runtime behavior.
Only formatting and diff checks ran locally for the setup correction;
original-source baseline verification owned the heavy slot at that checkpoint.
A fresh exact head needs runtime CI, canonical verification, release5 reconciliation, and the
native and historical acceptance already required above.

Independent static review of the corrected Air test reports no blocking finding.
It traced current transport admission, PC/Host ownership, Normal initiative,
initial placement, clear Hover Push, private-wall Unsupported refusal, and the
unchanged cold helper. That helper uses an independent in-memory restore plus
two reopenings of the real file; it does not claim a second cold file. The review
is external `tooling/gate4-shove-air-transport-correction-independent-review-2026-09-30.md`,
SHA256 `eec36ddb72a41f10a8d5b254011b12aa29810b0192b1f543662889b82cf5a4f2`.
The review is static only and does not substitute for the required execution.

### Corrected dependent draft publication — 2026-09-30

The complete six-job failure record for `5c70954` is preserved externally in
`tooling/shove-5c70954-ci/completed-failure-evidence.json`, SHA256
`9c00fbae11cfdc38b527b7e84e28d0de2665fd8a4dc31cba6a5d2abde4526f00`.
Both runtime jobs stopped after eight Rust result groups (114 passed, one failed)
at the table suite, before the remaining workspace suites and package build.
Both MSRV checks, architecture and genericity checks passed. Both Windows
frontend runs passed 122 tests across 19 files, with zero Svelte errors or warnings.
The artifact listing is empty. This is partial evidence on a failed old head.

The Linux full-log SHA256 is
`7a8d3cc925cbe7d7c4746aa964e4acc36a29116a7f69325e40e1761bddccce98`;
the Windows full-log SHA256 is
`f701ca521ef735bc1f8b72d7af157d598b4b8021677ac5e427a9078397db2995`.
The Linux synthetic merge `4170357bd8f5e3b90450a4c2afa2b98bc192b5ec`
has parents Air `1a0a5e75bfbde98c7621c421fbdc9d5b4750ab92` and Shove
`5c70954f9a9b765a4154470b0873d452fe4e654c`; its complete tree equals the
failed Shove source tree `df97653892ff882f5250fad8451b0d1bc9341665`.
Windows checked out that exact Shove source head directly.

The setup correction is commit `3979863457def26a841d63059291467562051c09`.
Its test blob remains `e4ec07ff043540449d45995af7ff8c1007e9045e`, with
working-file SHA256
`9db909346b25c53f370c9f4b6185b16bf2710f4969a96c0454cccf26f49fd270`,
identical to the independently reviewed correction above. Normal merge
`6c5095174542213d0c3ea54c2b0097069b974199` then incorporated Air's
documentation checkpoint `e915bbb9568f14a21c253bebf75d05ad2e91476e`;
that merge changed only the Air plan. The current evidence checkpoint changes
only this Shove plan. No production or test assertion changes accompany it.

Root authorized normal publication of this corrected dependent draft to PR #51,
still based on `codex/gate4-air-source-admission`. Do not merge it into the Air
branch. Freeze the published head for fresh six-check CI and inspect actual
completed outputs. No compiler or test execution has yet validated the corrected
setup, and no local Cargo/npm/database/native operation ran for this publication.
Root retains the local heavy slot and must coordinate canonical `./scripts/verify`,
verified dependency/main and release5 reconciliation, historical continuation,
independent final review, and genuine native play/restart on the integrated head.
Only after those criteria and all six final-head checks pass may the retargeted
main PR receive a protected merge, followed by literal merged-main verification.
Gate 4 remains active; the earlier failures and remaining obligations are retained.


### Planned Air/release integration checkpoint � 2026-09-30

Read root AGENTS and this full plan, then freshly fetched main and the published
Shove/Air branches. The clean starting head is
`c1124004dfb3fcc84d105b512a5fe4f9502a7727`, tree
`e97d1eca51a84332af5b26ed8d96333c8f35550a`; current fetched main is
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`. Root authorizes a normal merge of
independently reviewed local Air head
`3f3e3590e7977eca529168fa9069e8e1bca72434`, tree
`75350e552cac0ad404c2e130b8c595d396d13d50`. Its Air/release inputs are
`e915bbb9568f14a21c253bebf75d05ad2e91476e` and
`8c03f9fb0058610fd37c0cfe7762e8b96d658f38`; both remain unaccepted. Their union
contains the reviewed UI and current main ancestry. Air's independent static
review is external `tooling/air-3f3e359-independent-integration-review-2026-09-30.md`,
SHA256 `cbf2eed9c6fbb028ef3e4dffe2ce118d19716f1cb48dcc7d4ded6774a2ed9161`.
That review is not executable verification or dependency acceptance.

Preserve Shove paid work, exact source/authority, selected-work history and immune
Prone no-effect behavior. Preserve release Finish, Finished identity/history and
full-source/high-water rules; preserve Air full-pin admission before Finished
creation and coherent source placement on the staged candidate. All source
payloads, picker histories, manifests and their strict expectations must retain
Air bytes. All 29 fixture Git blobs, five old-execution receiving suites and
protected raw capture/log bytes must retain the reviewed dependency unchanged.
No historical fixture is to be recreated or edited for the integration.

Reconcile current test Begin/DTO producers with flow5 while preserving every
assertion. In particular, the corrected Air opaque-transport test must retain
its setup, actual source pins, cold/retry helper and scenario assertions; any
current Begin literal update must be recorded as an explicit one-line delta,
not claimed as identical bytes. Shove/release UI and optional DTO additions must
form a coherent union, including the existing prompt identity and retry paths.

A concrete execution seam was surfaced to root before source authoring:
`shove::require_execution` currently accepts only flow4 and also guards retained
validation, so new flow5 Shove cannot run/restore after a plain merge. The newer
central live guard also lacks the three paid Shove choice commands in its old-
flow continuation list. Proposed bounded reconciliation is an internal explicit
flow4/flow5 guard plus those exact authenticated, stage-bound continuation
commands; fresh flow4 Shove remains blocked by the central live gate. Root must
review this concrete proposal before this semantic change is authored. Focused
controls must cover current5 and retained paid4 continuation versus fresh4
rejection without rewriting a captured history or claiming historical producer
provenance for a reducer fixture. No execution tag/roll-role change is proposed.

This checkpoint advances the same production/persistence/agency requirements
and defers none of their acceptance obligations. It imports no MR, corrected Hag,
classifier, Grapple, or expiry child. No unrelated gameplay, source content, DTO,
or historical-compatibility expansion is authorized. Surface any additional
semantic conflict before extending scope.

Validation for this delegated checkpoint is static: inspect the full remerge,
compare combined files against both parents, audit immutable bytes, run direct
Rust formatting/parse and whitespace checks, then commit a clean reviewable
checkpoint with actual evidence and return exact head/tree/diff to root for
independent review. Cargo/npm/build/database/native execution and publication
are reserved to root. The published c112400 CI must finish normally and is not
cancelled or replaced by this local work. No old-head CI result validates this
integration. Exact integrated focused/canonical verification, six required
checks, old-history continuation and native play/restart remain acceptance
blockers; PR #51 must never be merged into Air. Root owns accepted dependency/main
reconciliation, later publication/retargeting, exact-head review and protected
merge. Next action: resolve the surfaced execution seam with root, normally merge
the pinned reviewed dependency, and report any further concrete conflict before
editing outside this bounded union.
