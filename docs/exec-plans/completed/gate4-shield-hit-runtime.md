# Gate 4 — Source Shield at an accepted attack hit

Status: bounded hit-Shield slice complete and archived on 2026-09-27 after
canonical verification, all six final-head checks, protected merge and separate
literal post-main proof. This does not complete Shield, reactions or Gate 4.
Final writer: root. Branch: codex/gate4-shield-hit-runtime, based on verified main
d82d7b2c28be3b74e6e2b0b8c85ffe8c042b1278. Verified PR42 main
2798b6b1d6263b5e321a1903d9fb4f2331b73895 and verified source-control PR43 main
e813e3a13911497902a3d4a55aec3c70653afb2a are integrated.

## Final acceptance evidence

Final reviewed head is `bfe4aac67ed80a5aec5165cfdd198acdbbfec9c5` in
[PR45](https://github.com/idiotswill/DMd/pull/45). Independent full implementation,
integration and final six-file documentation/ledger review are clear. Its final
delta changes no runtime source, test logic or captured bytes; complete crates,
apps and content trees match canonical source
`74c2cabf8fd8af44cdc2a58fc4bb7a5c6e9acfcb`.

That exact canonical source passes `./scripts/verify`: 723 GNU Rust tests across
54 suites, zero failures/ignores, all 50 table cases in 1724.91 seconds, all 121
tactical attack cases and five genuine ReactionsV1 histories. Formatting, all-target
check, strict Clippy and both guards pass with one build job, incremental off and
the default Windows stack. The genuine closed flow 2 export remains byte-exact;
its owner restart, upgrade, complete-state preservation and original armor deadline
pass focused and canonical continuation. Detailed historical evidence follows below.

All six final-head jobs pass. [Linux run 36307320564](https://github.com/idiotswill/DMd/actions/runs/36307320564)
records 724 Rust tests/54 suites, zero failures/ignores, 50 table cases in 3032.16
seconds and five histories in 136.63 seconds. Synthetic checkout
`840b151ccbce476777089dfa72158a0522c6b83f` and literal bfe share complete tree
`1954f41fa0f2fd7c2e61540f328ef8546e0bce0b`.
[Windows run 36307320510](https://github.com/idiotswill/DMd/actions/runs/36307320510)
uses literal bfe and records 726 native Rust tests/54 suites, zero failures/ignores,
50 table cases in 3181.12 seconds, five histories in 159.80 seconds, 89 UI tests in
16 files, zero static diagnostics and 139 modules. Fresh EXE/NSIS artifact
10928817066 is 231938230 bytes, SHA256
`c6900221a938b33c65a99e18535c2bf23be91a4b810b2901effe81aa062cf136`.
Actual upload logs and artifact API agree.

Protected squash main is `a0b12d2d0144a744e3419c1ba69e2d7aac64fd79`, whose
fresh fetched complete tree equals bfe with no file delta. Separate literal main
[Linux run 36311244326](https://github.com/idiotswill/DMd/actions/runs/36311244326)
and [Windows run 36311244361](https://github.com/idiotswill/DMd/actions/runs/36311244361)
pass all six jobs: 724 Linux and 726 native Rust tests across 54 suites, zero
failures/ignores. All 50 table cases pass in 2831.22/3175.88 seconds respectively;
five histories pass in 131.83/154.99 seconds. Native checks also record 89 UI tests,
16 files, zero static diagnostics, 139 modules and fresh EXE/NSIS artifact
10929844206, 231948830 bytes, SHA256
`1d8c3934077f7ff3a8fa926093dfd14933997c786bd0fe52a388f4010f93c06a`.
Actual upload logs and artifact API agree. Source CI is not substituted for this
separate main proof.

No work remains in this bounded hit-only slice or its integrated support plans.
The [Magic Missile plan](../active/gate4-shield-missile-runtime.md) and
[reaction umbrella](../active/gate4-reaction-ready-runtime.md) remain active,
including Counterspell, Ready and off-turn producers. Ledger families remain
implementing. The remaining sections preserve earlier decisions and source-qualified
checkpoints; their pending next actions are historical, superseded by this evidence.

## Objective and authority

Implement the first real source reaction through the existing table, one tactical
queue, physical dice and durable recovery: an accepted hit offers its target Shield,
the actual controller chooses it, and the defense applies before damage. Follow
AGENTS, product-definition's production/agency/recovery invariants, Gate04,
ADR026/027/028 and pinned SRD5.2.1 pp161–162 (Shield) and p187 (simultaneous ordering).
The full reaction umbrella and twelve-family Gate4 ledger remain active.

## Scope and non-goals

- Explicit ShieldHitV1 execution boundary. Preserve all genuine Legacy/ReactionsV1
  histories and old unit UpgradeExecution1-to2. New targeted forward upgrade requires
  settled Active state, no raw work/resolution and no paid Ready declaration.
- Uniform current-turn ordering stage for every attack hit, independent of private
  source eligibility. Source-safe private intent, explicit ordering or exact-trigger
  delegation, then a fresh command from the selected controller to cast or decline.
- Genuine source Mage Protective Magic, current components/shared uses/Reaction,
  retained original roll and source damage, current defense and owner-Start expiry.
- Physical, intrinsic and spell attacks, including an opportunity attack inside a
  retained movement path. Child Shield uses the existing cast/frame stack; it cannot
  replace the attack, movement, area or parent casting records.
- Actual private desktop controls and file-SQLite cold retry/independent replay.

This bounded version does not introduce Counterspell, Magic Missile target windows,
Ready release/held spells, or child physical attacks. Those mandatory Gate4 mechanisms
need subsequent explicit semantic boundaries if they change previously accepted
execution. This PR must not be described as complete Shield or reaction coverage.
Do not add a general historical-fact bypass to make later mechanisms fit prematurely.

## Implementation decisions before code

Freeze the existing attack facts at the actual accepted AttackRoll. Keep the original
modifier, mode, AC, critical rule, damage and accepted raw history. A typed hit record
binds the exact work, cause and parent attack. Reconstruct the original source against
current unchanged geometry/equipment/conditions, excluding only defense effects
derived from committed Shield child casts authenticated to this exact window.
Re-evaluate same-source suppression on that read-only defense set: subtracting five
from live AC is incorrect when an earlier Shield was already active. Final hit uses
the live effective defense plus retained/source-checked cover; natural20 still hits.
No client-supplied effect IDs or numeric permissions are admitted.

An independent response starts its own ordering authority scope. Each action's work
remains in the existing frames and ancestry. The bounded Shield child does not need
to overwrite the current physical cursor; future Ready attacks require an explicit
suspended-cursor policy. Damage request issued_by remains the original accepted
attack roll's accepted_by, even when the defender's response resumes it.

Private acceptance is nonpaying intent. The selected caster must issue a fresh
current-head command. Revalidate source grant/components/budgets before costs; stale
or invalid requests write nothing. Never retime a prior command, fabricate System
authority or attribute the response cost to the player who ordered the trigger.

Existing tests of fresh live actions must follow the new current executor and make
explicit controller choices. Do not convert them all to historical replay to evade
the new pause. Untouched genuine old fixtures continue under their own versions.

## Acceptance and verification

1. Real Mage hit/Shield/miss and natural20/damage, later attack and own-Start expiry;
   shared-use exhaustion, spent Reaction and blocked V/S refusal without partial cost.
2. Genuine player-controlled Mage after PR43: owner cast/decline, wrong player and
   host substitution refused, actual attendance and opaque capabilities enforced.
3. Uniform zero/one private Shield offer ordering surface, unrelated complete DTO,
   revision and transcript invariance; selected intent and exact-trigger delegation
   cannot transfer to a child occurrence or another attack. Two eligible controllers
   cannot both cast self-only Shield against a single hit target; retain the general
   two-respondent ordering tests, and require actual multiple-respondent privacy
   acceptance with the later Counterspell/Magic Missile trigger family.
4. Retained attack roll/cost/ammunition and OA movement cursor, original damage
   issuance cause, source spell/physical parent preservation, no second queue.
5. Cold file reopen at each decision; exact accepted retries; independently continued
   and restored histories; changed fact/cause/source/effect/ordering anchors rejected.
6. All four genuine PR42 fixtures unchanged and passing. Strict canonical verification,
   desktop checks/tests/build and all six final-head CI checks; full independent
   review; protected merge, fetched tree parity and final main evidence.

## Historical verification checkpoint

On 2026-09-27, literal74c2cab passed canonical `./scripts/verify`:723 GNU Rust
tests across54 result suites, zero failed/ignored, all50 table scenarios in1724.91s,
formatting, workspace/all-target check, strict all-target Clippy and both guards.
The run used one build job, incremental0 and the default Windows stack. Its complete
log is `tooling/shield-74c2cab-canonical.log`, outside the repository. All121
tactical attack cases pass. The genuine source-only aftermath continuation first
passed separately1/1 in132.70s, then the complete five-case ReactionsV1 corpus passed
5/5 in123.05s within canonical verification. Original captured bytes remain frozen.
Independent full review and the actual main-reconciliation/whole-state-test delta
are clear. The support test plans are integrated; their old unrun notes below are
historical authoring checkpoints, not the current result.

At this documentation checkpoint, literal74 Windows run36303904222 has passed
MSRV job108576655855, with89 UI tests/16files, zero Svelte errors/warnings and a
139-module build. Its stable runtime/packaging job and Linux36303904234 runtime
job remain in progress; Linux MSRV and both guards pass. These are not final success
claims. Linux's fetched synthetic merge2c208f659d935854f524134a4c6ab186de4b7fdf
has parentsf441aded/74c2cab; both complete trees equal
c19661883f2ffa0be207c893e778715c5eb96150. Inspect completed logs and final artifacts,
and obtain all six checks on the final documentation head before expected-head merge.
Record that head, protected merge/tree parity and literal post-main results in PR45;
do not attribute earlier source checks to a later SHA.

PR44 post-main verification is separately complete on literal
f441adedcf490504b6f1e3db1a964c023c511e47. Linux36303644071 passes706 Rust tests
across54 suites/49table in2106.86s; Windows36303644066 passes708/54/49table
in2249.28s. All six checks pass, including81 UI tests/15files, zero static
errors/warnings,138modules and fresh EXE/NSIS. Main artifact10926879955 is
231661638bytes, SHA256
78a32c342205e1ad8426323053f0849b9098f263f0f7527afe4990340714c58a.
The actual upload log and artifact API agree. This evidence does not claim full
encounter release; preserved aftermath cadence and full release remain distinct.

Latest integration: PR44 protected-squash merged asf441aded after exactbe5442a
review,705 GNU Rust/54suites/49table canonical and all six checks706Linux/708native
Rust,81UI/15files,0/0,138modules and fresh EXE/NSIS. Artifact10900519221 is
231671321bytes, SHA256
4bb368f92c657abf96491a6eb49421e6a09b19d6b622562dc86cc6c9457f5ee1.
Fetched main's complete tree6b497034863a6bd79d250be98d2f3f6b2d02d7af equalsbe5442a.
Normal reconciliationc9189a116d09ef45069f17add83cfac37b80845f equals the prior
fixture commit3d4c83c whole tree8329d608a8b6e35ccd28e55ae667d77e0a4ea6ca. Seven
duplicate-squash textual conflicts retained the previously integrated source;
a duplicated auto-merged TableAftermathView was caught by whole-tree comparison
and removed before committing. No production change occurred in reconciliation.

The genuine source-only flow2 aftermath export is imported unchanged with pinned
provenance:98963bytes/SHA2564d9075b1566a0279717d9eb617ff4651576187fbbbbc07bf8a053916413d96b5,
15events/10projections/9bindings. Its exact-source official export/independent
restore/cold-resume/all-receipt baseline passes1/1 in31.55 seconds. A new fifth
genuine ReactionsV1 corpus exercises its actual session resume, explicit upgrade
and owner continuation under ShieldHitV1. Independent fixture/provenance review
is clear; review strengthens the test to compare the whole CampaignState while
normalizing only the intended session binding or execution version plus sequence,
and drops the recovered handle before later awaits. The focused/default-stack and
canonical results above now verify this continuation at74c2cab.

The full3090 integration is independently reviewed clear and has all six CI checks;
its Linux syntheticfa0a06a35210170805e9251fa74946027a91bb51 and literal3090 complete
trees match2366ca4e87c51207860c42e8279b1e4f4513a5cd. That evidence precedes the
genuine aftermath corpus addition and does not substitute for its final checks.

Earlier hit-only executable candidate with all six CI checks passing is
8b5cf52a250c5f381d22977391906cfedc53a630. Documentation successor7bc01af and
verified-main reconciliation98399da change no executable source. The latter's
whole tree equals7bc01af at926629f4aec461374b38e5ff57a93260e190d408.
No Gate4 criterion is waived; final integrated/canonical acceptance is pending.

- The fourteen Shield rules cases pass on the local default Windows GNU stack
  (57.29 seconds, jobs1/incremental0). The prior full tactical-attacks run passed
  all 107 existing cases and exposed a shared missing Mage-language fixture input
  in the fourteen new cases. Supplying the real required three languages corrected
  the fixture; production rules and assertions were unchanged. Logs are outside
  the repository: tooling/shield-hit-tactical-attacks.log and
  tooling/shield-hit-focused-r2.log.
- Linux run36193351652 passes all four jobs, including fast verification, strict
  Clippy, Rust1.88 and both guards. Actual runtime job108263431336 records715 Rust
  tests across54 result suites, zero failed/ignored, all121 attack cases and47
  table cases in1885.29 seconds. The owned Shield SQLite case passes. Checkout is
  synthetic mergee07b7db932aa1ab57280566e6c90a43406c2231a of8b5cf52 and
  main2798b6b. Fetched complete trees both equal
  cc35d233924c88344598dd8206688e34455fd6bb; their full diff is empty.
- Windows run36193351721 passes stable108263469783 and MSRV108263468702 at literal
  8b5cf52. Actual stable logs record717 Rust/54suites/47table in1950.51 seconds,
  zero failed/ignored,84 UI tests in15files, zero Svelte errors/warnings and138
  built modules. Fresh EXE/NSIS artifact10890038954 is231884106bytes; upload log
  and run-artifacts API agree on SHA256
  d2b4193963af65a9e814c55e2e73879be1c6ffcbb3c4e2497144fbeec393fc77.
- The genuine player-owned Mage SQLite case passes on Linux and native Windows.
  Updated source-control cases use fresh flow3; four genuine old exports retain
  their original bytes. Final integrated canonical and exact-head acceptance
  remain required after aftermath integration.
- Independent full production review and the actual source-control merge delta
  32d937b plus equivalent fixture correction8b5cf52 are clear. Review covered both
  parents, v1/v2 replay/projections, source authority, activation/attendance,
  role-specific opaque response routing, both UI test sets and actual scenario
  registration. Static review is not a runtime result.

Source-control's canonical verifier completed and released the heavy slot. Root
ran the otherwise identical source in an isolated diagnostic worktree,
06045b2 over7bc01af, with independently reviewed read-only capture hooks in the
Night Hag and owned Shield tests only. The hooks preserve genuine pre-change
flow3 exports for the next semantic boundary; they never merge into production.
The default-stack focused batch passes: Night Hag1/1 in500.88 seconds, source
units3/3 in0.13 seconds, actual source cases2/2 in177.07 seconds and the full owned
Shield case1/1 in1427.43 seconds. All four real flow3 captures are preserved.
Diagnostic successorf9698e1761ee0485c4cf0ff593f89a03f016302b adds only the exact
reviewed compatibility module and capture bytes for separate baseline restoration
and original-receipt retries. Its exact selected test passes1/1 in299.50 seconds
(compile25.99 seconds, exit0), covering all four byte/typed roundtrips, complete
restores, cold resumes, every original envelope retry and changed-body zero-write
refusal. Log: tooling/shield-baseline-f9698e1.log. This source-equivalent proof
does not replace final integrated canonical verification. The earlier disk and
committed-memory shortage is resolved; use jobs1, incremental0 and default stack.

### Integration and recovery decisions

The v2 source channel derives privileged-response authority from the actual
retained target. A fresh Begin3/UpgradeTo3 refuses unactivated player-owned
sources before they can become stranded; older Begin versions and unit upgrade
are unchanged. This additional guard runs behind privileged issuer checks so an
unauthorized Begin cannot disclose source-access state. The typed-state unit case
separately covers activation and historical admission.

The real SQLite fixture establishes both controllers as Present with accepted
EndSession/StartSession commands, creates the Mage through the content catalog,
assigns its owner, and supplies physical attack dice. Its table contract records
an agreed practice encounter; that prose does not prove typed PvP enforcement.
Cold independent continuation, exact retry, wrong-controller/Host theft, raw
canonical bypass, whole unrelated DTO invariance, original damage cause and
forged current/retired snapshots are all asserted. The prior hit snapshot is an
actual captured earlier image backfilled as a recovery anchor, not an old fixture.

Desktop tests retain both integration parents and exercise a selected source's
Shield/outbox retry across ownership changes. Host cannot substitute an owned
source response; an explicitly issued delegated ordering capability still works.
Every target acknowledges collection even without an available Shield. Continue
is free controller coordination, available when incapacitated; it is not a
fictional action or spent Reaction. This prevents eligibility-dependent public
ordering/timing from exposing private source information.

ResumeHit retains the exact accepted roll identity and authenticates its parent
AttackRoll occurrence, tactical role/purpose and issuance origin. It does not infer
that origin from the outer movement or casting resolution. Passing rules cases
cover an opportunity attack and a later Scorching Ray with distinct ancestry.

Early compiler diagnostics required the existing workspace UUID dependency, an
explicit Serde bound for optional opaque hit keys, a correct vector-removal test,
and strict-lint idioms. Final integrated32d937b passed compilation/MSRV; Clippy
identified a manual hand-membership scan, corrected equivalently in8b5cf52.
No lint exemption, assertion weakening or invented default capability was used.

### Historical prerequisite and next-action record

PR42 merged as2798b6b1d6263b5e321a1903d9fb4f2331b73895 with full-tree parity to
reviewed cbe9575 atddfe5b58c0c55ad2aab6e367de295bf7c1760e5e. All six post-main
checks pass: Linux36188457488 has695 Rust/44table cases; Windows36188457455 has697
Rust,67UI,zero static diagnostics,136modules and fresh EXE/NSIS. Artifact10888336544
is231464157bytes with SHA256
7cb2a446ffb122ca6aa1d78430c3c50b79eb8da78af0f77b1e0016747b79c19a.

PR43 is protected-squash merged as e813e3a13911497902a3d4a55aec3c70653afb2a.
Fetched whole-tree95ccff0f48d47f8a92e2a2eafab6f8385e9f7ce3 equals the independently
reviewed finala119d7f. Its exact canonical verification passes698Rust/54suites/
46table; all six final-head checks pass699Linux/701native Rust,77UI,0/0,137modules
and fresh EXE/NSIS. Artifact10890060494 is231627996bytes, SHA256
ed11f6b7fba78c8733a59cf524514c32e7de118d882011c51f0bd8dea9ffbc12.
Post-main runs36194898895/36194898980 separately pass all six checks on literal
e813e3a:699Linux/701native Rust,54suites/46table,77UI,0/0,137modules,7manifest
tests and fresh EXE/NSIS. Artifact10889754911 is231636930bytes, SHA256
5b1f74d35e3204bd16d8364c32e47d9967109bcc5979e7c2c6a1b23e735c52e6.
The bounded source-control plan is archived after that proof.

The planned remaining merge order is aftermath PR44, then this hit slice. PR44's
bounded conclusion currently validates ReactionsV1/flow2 only. Before final Shield
canonical acceptance, normally integrate its verified main, deliberately admit
the same retained aftermath cadence under flow3, and update fresh real session/
source scenarios to Begin3. Preserve already accepted flow2 images and source
timing; do not leave the new current executor unable to conclude hostilities.
If verification changes the merge order, perform that integration explicitly in
the later branch instead. Full encounter release remains separate Gate4 work.

For parallel development during PR44's final canonical run, integrate reviewed
candidatebe5442af8b24f8ee24d2c0549ea8dac452d44489 locally first. Its complete diff
from all-six-greenf798432 is only its execution plan; root's final delta review
and the independent full review are clear. This is development integration, not
PR44 acceptance. Preserve both parents' real application tests and controls,
admit retained aftermath validation under explicit versions2 and3, and change
fresh aftermath setup to Begin3. Exercise historical flow2 conclusion/upgrade
separately and preserve a genuine flow2 capture from PR44's unchanged real case
if the approved read-only backup succeeds. Final Shield canonical/merge still
requires normal reconciliation of verified PR44 main, full integration review,
all required checks and no discarded source or pending-work behavior.

Development merge8a1f2f5 retained both complete source/UI test sets. Three textual
conflicts joined the two independent TableApp tests, retained hit plus aftermath
DTO fields and retained all hit action arms plus conclusion. Auto-merged medicine
and source helper changes only expose the existing tested helpers to aftermath.
The follow-up admits validation only for explicit ReactionsV1/ShieldHitV1, changes
the two new actual aftermath Begins and fresh UI fixtures to ShieldHitV1, and adds
a qualified typed historical2 replay/upgrade invariant. Actual selected Shield and
post-Shield physical damage now both reject Host conclusion with the whole export
unchanged; the UI keeps conclusion disabled for either pending boundary. These
integration additions now pass exact3090 CI:723 Linux/725 native Rust across54
suites,50table cases,89UI, zero Svelte errors/warnings and fresh EXE/NSIS. Linux
run36225272787 and Windows36225272802 pass all six jobs. Artifact10901048468 is
231940930bytes with SHA256
f3b7102676bde66db075e55b04b6c56251ca398a2bb4ec46a96d665896494658.

The genuine source-only closed aftermath backup from PR44's unchanged canonical
case is available. Preserve its official flow2 export as a fifth genuine
ReactionsV1 corpus, with byte-preserving attributes/provenance. Acceptance extends
the existing file/mirror helper: original receipts and cold resume, absent source
owner refusal, real owner-only session start, explicit upgrade to3 preserving the
whole cadence/state, Host/foreign action refusal and the owner's actual next turn
with unchanged source uses and original armor deadline. Do not manufacture rows
or reinterpret the captured current-state/source/session history. The exact-source
export/restore baseline must pass before import; final canonical includes this
additional continuation under the integrated current executor.

Read any current CI failure before changing source. The registered player Shield
SQLite case, integrated aftermath and genuine historical continuation now pass the
complete canonical run on the default stack at74c2cab. Complete final all-six-head
CI with fresh packaging, full documentation-delta review, expected-head merge and
fetched tree parity. Record post-main evidence in the PR. The evidence/ledger
documentation changes do not modify runtime source, test logic or fixture bytes;
final-head CI must still verify the updated ledger and complete repository.
Continue the live-reaction umbrella inside Gate4;
Magic Missile, Counterspell, Ready release/held spells and all other open tactical
families remain required. Do not pause at this slice boundary or begin Gate5.
