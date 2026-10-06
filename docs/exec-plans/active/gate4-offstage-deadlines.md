# Gate 4 — Released elapsed intervals and absolute deadlines

## October 6 preserved completion-history diagnostic

Exact `9cd146837945dce504aeb7f6fc266f7f9de52673` fails the existing
`completed_receipt_survives_turn_reset_and_rejects_incoherent_anchor_claims`
control on Linux111798689219 and Windows111798295472. The movement suite reports
30 passed and1 failed. Its unauthenticated Finished state is still refused,
but the new released proof constructor runs full domain structural validation
first and replaces the established missing-completion-history error with
`released candidate is structurally invalid`. The original assertion is correct
and remains unchanged; this is validation diagnostic precedence, not evidence
of an accepted unauthenticated state.

Commit this plan before implementation. Extract the existing missing-history
guard in `release::validate_history_with_released` into a small private pure
`require_completion_history` helper. Retain the exact predicate and error:
no encounter history, a release-capable execution, and Finished phase. Call that
helper at the original release validation boundary and at the start of
`ReleasedValidation::derive`, before general structural validation. The latter
must still perform every original structural, exact-flow, history and interval
check. This shares one diagnostic/invariant rather than duplicating it or
granting proof before validation. No state mutation, historical exception,
caller-supplied flag, test alteration or new public API is needed.

Root owns this bounded correction. Review the entire exact delta independently,
keep all old movement/legacy/elapsed bodies and installed content byte-exact,
then verify the complete31-case movement harness as well as the prepared426
selection on a freshly frozen head. Earlier426 preparation remains unrun and
must not be silently retargeted. Preserve both actual failed CI logs, verify
new-head required checks and retain all outstanding native/gate obligations.

## Normal receipt of the current Expiry base — 2026-10-05

Root allocates this branch's sole writer for a bounded normal merge of freshly
fetched Expiry3fa8e6c17f625098159d70689f9394bcc24050e3 into corrected Offstage
5a1650352fabb9796c689e17e525631311da76b5. The Git remote has advanced beyond
an older PR base snapshot; the actual current base is authoritative. Commit this
plan before the merge. The independent Expiry receiving memo and complete plan
were read, together with both documentary deltas and the prospective merge.

The actual merge base is accepted main32c0c682c4dbb235e1f9a119643c5d8626d5cb71.
The prospective merge changes no non-document entry relative to Offstage5a.
Four textual conflicts concern ADR028, the main Gate4 tactical plan, production
closure and the coverage ledger. Reconcile the equivalent successful PR48
literal-main evidence, retain both branches' bounded Air admission qualifications,
and keep candidate/checkpoint claims explicitly historical. The incoming Expiry
plan's accepted-Air receipt and exact c9 CI/package attribution must be retained
in full. Do not drop either parent's failure, verification, dependency or native
limitations. The encounter-release plan auto-union must preserve its existing
successful literal-dbf record.

This branch's source checkpoint contains accepted main32; later accepted main
movement is separate and is not silently imported by this receipt. Describe that
checkpoint accurately rather than calling main32 the latest global main. Keep all
remaining Gate4 scope and statuses unchanged; no pass on either older parent is
execution evidence for the new head. This merge neither accepts Expiry/Offstage
nor brings Shove or other development source into this branch.

Preserve every production/test/content entry exactly to5a, including the legal
third attendee, all three Host Mages, both unrelated observers,77 original
assertion snippets and all28 original controls. Recheck all29 fixture/five legacy
suite blobs and21 raw captures. A source delta or unexpected conflict requires
reporting and separate review before continuation. No source correction, formatter,
Cargo, build, test, database, native action or publication is allocated here.

Freeze the clean normal merge with full parent/tree and byte-preservation evidence
for root's independent review and normal push. The saved5a426 preparation is
UNRUN and becomes obsolete on a new head; preserve it and prepare a fresh exact
426 inventory/runner afterward. Root retains the single heavy execution slot;
this receipt allocates no local execution. New-head runtime and native outcomes
remain unknown. Root coordinates CI, canonical verification and acceptance.

Receipt outcome before root review: the four conflicts are documentation only.
Their resolution retains successful literal-dbf evidence, bounded Air admission,
explicit historical pending32 qualification and exact source/native attribution;
it removes a duplicated PR48 paragraph produced by the automatic merge. The
incoming full Expiry plan is retained exactly, and the release plan remains the
Offstage5a blob. All non-document entries are required to remain exactly5a in the
external full-tree audit. No new runtime/native pass or family status is asserted.

## Actual application setup failure and fixture correction plan — 2026-10-05

Windows stable run37297998875/job111723733666 on exact b5af47e completed
with130 Rust passes and2 failures across the9 reached harnesses. The table suite
reported63 passes/2 failures; both new Mage interval scenarios stopped in the
shared prepare_mages helper at PrepareBattlefield, before any elapsed command.
The actual refusal is "Choose a new encounter and scene with at least one character."
The original complete log is retained externally, SHA256
409dfb350c7b613dc3d0efcff12f84984316b054dc2422949b5eae9c38368c2b.

The helper supplies no character and three Host-controlled Mages. Existing
application admission permits source-only setup only with enabled source access
and an attending player's genuinely owned source. This fixture has neither.
Production admission is correct and must remain unchanged. Root transferred sole
writing of this plan and the new elapsed fixture to the independent diagnosing
reviewer; all other source and protected history remain frozen.

Before changing source, commit this plan. Repair only the new fixture by normally
ending its initial session, adding a distinct third player and ordinary Fighter
through existing table creation, preparing queried starting equipment and starting
a new session with the original observer entries plus that attending character.
Place that character beside the three unchanged Host Mages. Begin must include
all four participants: retain the original identical-Mage group and raw12; add
one character group and a genuine owner-channel physical raw1. Mage initiative14
precedes Fighter3, and the original Mage-only tie proposal retains their order.
All three Mage turns/casts therefore occur before any round wrap; existing clock0
and exact28800 deadlines remain assertions. No turn, clock, effect, owner, source
or accepted history is patched or invented.

The two original players keep their original characters outside the encounter and
remain unrelated observers. Preserve all their existing full-view/privacy and
hostile-input assertions. Return the third session participant from the setup
helper and retain it during the existing pending-interval EndSession/StartSession
route, in addition to its existing observer attendance. Preserve every current
scenario body assertion, old live5 behavior, strict fresh7 Begin, explicit5→7
upgrade, all28 original controls and protected29/five/21 fixture/receiver/raw bytes.
Add explicit setup checks for genuine four-actor order and unchanged clock; the
setup correction does not weaken any runtime expectation.

No production fix or source-access activation is justified. The previously passing
Medicine/d4 and genuine old-source scenarios remain byte-exact. Direct formatting
will use only a separately allocated edited-file window; no compiler, Cargo, tests,
npm, database, native execution or publication is allocated to this author. Freeze
the coherent corrected head for root's full review and fresh focused/CI execution;
both original failures and all later outcomes must retain their exact attribution.



The fixture correction is now authored after plan e3dcec8. The genuine third
participant receives its own one-d20 Physical initiative request on its owner
channel; the test asserts Mage/Mage/Mage/Fighter order with totals14/14/14/3,
clock0 and absence of both measured observer actors from the encounter. The
third attendee is retained through the existing pending-interval session rollover.
No assertion was removed or relaxed. The complete two other application scenarios
(Medicine/d4 and genuine original armor) and all other repository source are
unchanged. Direct configured rustfmt on this one file and git diff --check passed;
no compiler or runtime was invoked. Root must independently review this frozen
correction and run fresh verification; actual later outcomes remain unknown.

## Application source authored; execution outstanding — 2026-10-05

The normal tactical/TableAction route now dispatches released intervals and derives
candidate-bound validation; the old private wrapper is test-only. Host-only no-turn
presentation uses existing opaque Work capabilities. Replay inventories all nested
origins, requires exact accepted elapsed/upgrade commands and retains mandatory
original-anchor semantic replay for removed sources. Session pause preserves the
fixed pending interval; preparation still requires a quiescent Finished encounter.
Private ordering has Host-only transcript visibility. Desktop controls submit the
normal typed requests, retain the original uncertain request and hide preparation
while the interval is pending. Existing live5 continuation stays supported as below.

Authored current coverage in `table_released_time_cases.rs` uses real queried source
Mage creation/materials/casts, normal initiative, real physical critical damage,
Medicine and recorded d4. Four async scenarios cover three equal deadlines in both
orders with partial progress/session rollover, 0/1/2 hidden deadline privacy at the
same clock, exact stable wake and once-only recovery, and genuine captured old source
armor through5 release and7 upgrade. Important accepted steps independently restore
to a file mirror, reopen/retry the original file and verify changed-body refusal.
Current/earlier-image forgeries cover roots, removed source/time, completion/partition,
ruling and audit/envelope metadata; refusal compares all destination table row values
with an unrelated real campaign already present. Positive state is never patched.

The additional public rule test verifies actual returned event/replay equality and
rejects changed outcomes. Four new component tests cover Host-only input, opaque
selection, explicit upgrade and ordinary5 compatibility; a TableApp test preserves
the exact pending ordering request across restart/session change. Existing current
desktop fixtures deliberately select7. Original28 controls remain in the verification
inventory:19 private controls (only two staging validator errors become successes),
four original release bodies preserved byte-exact under internal Historical policy,
four unchanged wire tests and one unchanged codec test. The corresponding current
release integration scenarios also remain, using explicit7 Begin. All35 current
Begin expression changes are recorded in the external migration inventory.

This source is **uncompiled and unrun**. Direct GNU rustfmt with edition2024 and
skip_children plus whitespace checks are the only local tool verification permitted
to this writer. The original874 private28 pass is not transferred. Root retains the
sole heavy slot and publication ownership. Next action: independently review the
frozen full diff/preservation audit, run the original28 with their explicit inventory
moves, new rule/app cases, protected legacy receivers/current integrations, frontend
and canonical checks, then genuine native app evidence before acceptance. No Gate4
completion, public behavior pass, CI pass, PR or merge is claimed by this source work.

The independent Ground writer performed a read-only static pass over the new app
test types, module ancestry and producer sequencing. It found an audit-row field
typo (`id`, not `command_id`) and insufficient count-only hostile destination
comparison; both are corrected. The new comparison retains full typed cell bytes
and an unrelated genuine campaign. The peer found no further definite issue in
three-Mage timing, physical Medicine/d4 timing, session continuity or the original
source-capture bridge. This is static review only, not a compiler or behavior pass.

## Compatibility amendment — 2026-10-05

Source inspection found a real protected receiver contract: the genuine source
aftermath scenario in `legacy_reactions_v1_replay.rs` accepts2→5, proves exact5
state preservation, then accepts a new owner EndTurn on5. Root reviewed the actual
803–854 continuation and approved preserving the existing live ordinary5 behavior
as intentional public backward compatibility. Fresh Begin remains strictly7;
flow1–4 retain their exact pending/upgrade restrictions. AdvanceReleasedTime and
all new released records still require the explicit authorized5→7 upgrade and
full accepted-command provenance. No version-only authority or test bypass is added.

This supersedes the planned blanket ordinary5 closure and the two isolated-session
helper adaptations below: those helpers and extra upgrades are unnecessary and
are not part of the resulting patch. The protected receiver remains byte-exact.
Internal original5 release bodies remain meaningful producer/replay controls,
while contemporary integrations explicitly begin7. The original19 helper needs
Historical policy only for its old Begin5; its ordinary continuations and explicit
upgrades use the normal public producer. New controls cover denied
fresh Begin5, denied elapsed5, a genuine old5 release followed by session-bound7
upgrade, exact source timing, original journal bytes and literal old response retry.
The existing protected source-owner continuation remains the genuine ordinary5
positive control; it is neither copied nor relabeled as a new capture.

## Original-flow control migration plan — 2026-10-05

The two isolated legacy-upgrade mechanism scenarios in `tactical_turns/ready.rs`
and `tactical_turns/aftermath.rs` retain their exact2–4→5 comparisons, then explicitly
install a declared table/session fixture and accept the separate5→7 upgrade before
their final new action. This is isolated rule-fixture session setup, not application
history. The shared meta helper reads that session only when present; all other
original `None` metadata remains unchanged. No historical assertion is removed.

Before test-support edits, root approved explicit internal Historical producers
instead of widening live5 admission or exposing a public testing bypass. The
original19 private controls keep their genuine5 prefix through the existing
crate-private Historical reducer, retaining actual returned event/replay equality.
The only public-closure assertions removed remain the two validator errors named
below. All real authority, already-paused Advance, source, shape and clone-proof
refusals remain. These isolated mechanisms are not recaptured original app history.

The four original selected release controls move in the verification inventory
from integration `release::<name>` to unit `tactical::historical_release_tests::<name>`.
Their full function bodies are copied byte-exact from d093699's
`crates/dmd-rules/tests/tactical_turns/release.rs` to
`crates/dmd-rules/src/tactical/historical_release_tests.rs`:

| Function | Original full-function Git-byte SHA256 |
| --- | --- |
| accepted_release_preserves_source_resources_and_replacement_advances_global_turn | `72a53c57716686769e79aee52ca7ce8e78cb9dbb101c385b81d4b81df292a22d` |
| raw_recovery_die_and_due_wake_are_checked_across_the_campaign | `42a42b58c8f0ffb7847c3e38821ae28ffb892455dc5a73fd2c3b8c8dd7bd9123` |
| dependency_scan_uses_actual_collective_setup_capacity_and_scene_authority | `e29af9dca8139aff908a7fbb3c9968b0742e37c22cf89974061f291831defd90` |
| replacement_setup_and_pending_initiative_cannot_omit_retained_dependencies | `46e5cf83d6f2c6eab482ecbc1b840a8110ea2e0cb2f784e691dfffe534877a70` |

Only the needed original Fixture methods (`new`, `meta`, `rules`, `creature`,
`run`, `rejected`, `raw`, `begin`) and release setup helpers are copied. `run` and
`rejected` use the existing internal Historical policy; Begin stays5. The content
include path changes for the new file location. No alternate authoritative reducer
or fabricated accepted outcome is introduced. Corresponding contemporary public
integration scenarios stay present on7, with deliberate new-producer expectations;
they are not labeled as original5 verification. Original structural negative
fixtures that deliberately assemble5 receipt/state records keep that meaning.

Contemporary live Begin producers will explicitly select7 in these support paths:
`dmd-rules/tests/{tactical_attacks.rs,tactical_movement.rs,tactical_turns.rs}`;
`tactical_attacks/{areas.rs,missiles.rs,casting_timed_expiry.rs}`;
`tactical_turns/{release.rs,ready.rs}`; and `dmd-app/tests/support/` files
`table_{area,aftermath,casting,attack,falling,hit,medicine,night_hag,missile,oa_concentration,release,release_custody,release_death,release_recharge,release_savage,tactical,source_control}_cases.rs`
and `tactical_runtime_cases.rs`. Historical targeted upgrades2–4→5 and their
assertions stay explicit; new ordinary actions after that bridge require a separate
authorized5→7 operation. Current desktop fixtures follow the same explicit current
version. The final handback must enumerate actual migrated functions and any extra
helper adaptations, with preserved old bodies and current expectations separately.

All29 protected fixtures, five legacy replay receivers and21 original raw captures
remain byte-exact. No selected test is dropped from verification, no assertion is
weakened, and no passing result is claimed for moved or newly authored controls.

## Accepted-main receiving review — 2026-10-05

Plan7dce29a preceded normal merge `d093699cb9012db27979194393ac996ac9bc6b3d`,
tree `66957434ae8bc8dce9a35ef67fd212929bca46b1`, receiving exact accepted main32.
Root independently reviewed the full plan, all eight source overlaps against both
parents, five reconciled documents, both actual merge bases and every569 path
entry. Counts460 equal-parent/45 main-only/51 Offstage-only/13 overlap and protected
29 fixtures/five legacy suites/21 raw captures/four original control files agree.
The independent review is clear in external
`tooling/offstage-d093699-root-receiving-review-2026-10-05.md`. Root authorized the
already planned complete application implementation without another routine
permission checkpoint. Source ownership stays with core_recovery_oct5; root owns
all execution and publication. The receiving tree itself has no runtime evidence.

## Production application completion transfer — 2026-10-05

Current status: root's exact guarded checkpoint verification completed normally
at `87488ae1ba7b3b4fe13175a1563e41256d170a4f`, tree
`bc0942e51df053c80e9b7571addabce14bc380c8`, from09:08:48 to09:23:08 UTC.
Formatting, strict all-target Clippy for domain/rules/app/persistence and all28
selected controls passed:19 released rules,4 strict domain wire,1 old-schema
codec and4 original release controls. The initial, every post-command and final
head/tree were identical and clean. Root read/recounted the literal logs and
all28 exact arguments; this writer also read the four result logs and complete
run result. The independent CI peer audit is separately being completed.
These results qualify the private guarded874 tree only. Full public/application,
original journal/SQLite/privacy, canonical/native and gate acceptance remain open.

Evidence is preserved outside the repository in
`tooling/offstage-87488ae-focused-2026-10-05/`. The six log SHA256 values are:

| Log | SHA256 |
| --- | --- |
| 01-fmt | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 02-clippy | `21bd563b1c804226e73ab3e8ec5058c5c1a179f9cb15b5d2d82074c2e99b248c` |
| 03-released-private | `398df39169fd45ec1d1973f514fbe26f41487fe7f260179a9851f232a86e6f53` |
| 04-domain-wire | `e58754761c08b347ffa8e9cb3696c8a08e8e6612e5f7e4c8a3e8a1ac8cd596b2` |
| 05-old-schema-codec | `1f977644398227849de607126b06535f3321d22431fdba0202551c8dee6b8948` |
| 06-old-release-controls | `3f7e88f6c789626a71498de6c5adff794106ffb2668a3dc230ef9a4f70b06b87` |

The original b7085f0 derivable-Default and3c5272e collapsible-if strict-Clippy
failures remain preserved; neither ran its selected tests. Plan-first exact
corrections and independent reviews preceded874. No retry erases those failures.
The final run used a fresh private target, GNU1.98.1, jobs1/incremental0 and default
profiles, stack and test threads; no lowered acceptance or shared target is implied.

Root read and approved the full concrete completion map
`tooling/offstage-87488ae-next-application-completion-map-2026-10-05.md`, SHA256
`a2b55ddc709f9e0faa2a34b125163fb1d5dfc12e762085108fb06a896d8d04a7`, then explicitly
transferred sole source writing on this branch to `/root/core_recovery_oct5`.
This plan commit precedes receiving and application source changes. Root retains
publication, runtime/native slots and final independent review. No Cargo/compiler,
npm/database/native or project-runner execution is authorized to this writer;
direct rustfmt must be coordinated with root. No other checkout is writable.

### Receive actual accepted main before source

Fresh `git fetch origin main` still resolves accepted main to
`32c0c682c4dbb235e1f9a119643c5d8626d5cb71`, tree
`c189a2fc618569b270fa8f87f7cb0577ff1fc32e`. Receive that exact head through a normal
merge and preserve both histories. Its two actual merge bases with874 are
`dbf1d633460473183324b4ec519e8d1980884b5c` and
`8c03f9fb0058610fd37c0cfe7762e8b96d658f38`. Inspect every base and both parent
trees, the full receiving delta and all source overlaps. A single selected merge
base's58-path comparison cannot establish dropped changes. No force reset or
unrelated feature import is permitted. Return the clean receiving checkpoint
and preservation evidence to root for independent review before application edits.
If authoritative main changes before receiving, report the new head first.

### Concrete complete application scope

Use the existing deadline engine and normal `TableAction::Tactical` atomic
transition/event/replay route. Integrate actual upgrade, begin and choice producers
into the normal resolver; the private transition wrapper is not a second public
engine. Fresh Begin explicitly uses7; preserve historical1–5 behavior, unit1→2,
the live older-flow→5 bridge and only explicit settled5→7. Reserved6 stays refused.
Ensure ordinary active7 actions, Conclude and Finish also use the normal route.

Public kernel/tactical, release preflight/history and aftermath validators derive
the existing exact-candidate proof where the current live flow7 needs it. Do not
replace the proof with a caller flag. Completed elapsed history also survives
later encounter replacement; historical receipts must not require a current7
proof when no interval remains. Keep quiescent Finished checks separate from a
new exact paused-interval session boundary. Preserve the complete source/dependency,
physical support, condition-payload, legacy-time, stable-d4 and capacity checks.
A pure shared readiness query may expose Host eligibility without executing an
invented command or creating a second set of weaker admission rules.

Extend original-history audit with Finished upgrade origin, elapsed receipt root/
completion, live batch Time command, every completed/cancelled work command, Effect
source and establishment commands, Group source and Stable origin. Authenticate
exact upgrade/release and Advance duration/policy/ruling/start/target, selection
origin/occurrence and replayed Time/work stamps. Removed sources are reconstructed
from original accepted history, never authenticated by their own final receipt.
Keep mandatory pre-tactical anchors, strict old-schema/duplicate-safe codec,
original raw request envelopes and pre-write rules-enabled restore validation.

Add an omitted-None Host-only `released_time` DTO alongside the unchanged mandatory
actor Turn continuation. It presents eligible/blocker or actual retained interval
start/progress/target and current sibling choices. Reuse existing Work capabilities
and opaque SelectWork transport mapping, bound to audience/revision/origin/
occurrence; raw ChooseTurnWork stays refused. Add explicit ReleasedInterval Host
ownership in source-control routing without borrowing a Turn/area actor. Include7
in the correct source-access Begin/Upgrade gates while preserving old semantics.

Private interval ordering events are Host-only at acceptance, including transcript
visibility. The generic shared encounter-action message must not leak hidden
expiry counts. An Advance announcement may contain only admitted public intent.
Actual permitted visible HP/AC/time changes still update their audiences. Test
equal-visible-state/time zero/one/two hidden source sets for identical unrelated
DTOs, revisions/handles and transcripts, with only genuinely public changes allowed.

Preserve accepted-before-stale lookup and exact original response/envelope retry
through clock/selection/completion, process reopen and changed sessions. Add the
separate validated Host paused-interval EndSession capability; StartSession binds
genuine current attendance without altering interval origin or clock. New choices
need the active session, while original accepted retries retain their old request.
Keep old closed-session release/target5 exceptions exact. No preparation or new
battlefield can escape unresolved deadlines.

Update explicit desktop execution types, fresh Begin/current-executor controls
and proper old→5→7 bridges. Add the Host duration/ruling/HostSelect form, actual
progress/target, stable opaque choices, focus and disabled/in-flight behavior.
Retain session pause while blocking ordinary preparation during a pending interval.
Do not frame elapsed time as a PC turn, rest, cancellation, retargeting or free heal.

### Controls, review and acceptance still required

The only temporary staging assertions to migrate are the two public-validator
errors at `released_time/tests.rs:541–542` in
`guarded_interval_rejects_public_authority_and_candidate_splicing_atomically`.
Change them to acceptance of the genuine produced interval, or split that positive
into a clearly named public control. Its following Advance-on-already-paused error
is a real invariant and stays. Preserve pointer-bound clone refusal, all eight
candidate forgeries, actor/session negatives and whole-state equality. All other
rules/wire/codec and original release controls remain intact. Add normal public,
Historical and event-equality/replay controls, not just private transition calls.

Author accepted application play with actual catalog Mages and physical components,
same-round equal Mage Armor deadlines, both host orders and later-round recast.
Use three actual equal producers for a still-pending post-first-choice cut; two
correctly auto-drain their remaining singleton. Stable waking must start with an
accepted damaging attack, accepted Medicine and the actual owner's physical d4,
then valid release and before/exact/beyond wake with exactly1HP and no reroll/rest.
No fabricated spell/profile/paid state or injected clock/effect supplies positives.
Keep isolated legacy/group fixtures labeled and genuine old prefixes unchanged.

At admission, pre-deadline, original simultaneous choice, genuine post-first choice
and completed target, use real file SQLite plus independent portable mirror and
original exact request retries. Compare whole persisted exports (only export time
normalized), source/resource/raw/target/stamp/receipt invariants and actual next
encounter. Add current/earlier hostile snapshots, removed source/Time/receipt,
upgrade/release/audit/envelope corruptions and assert no writes across the whole
destination. Include old-schema SQL rollback, real session rollover and unauthorized
channel/stale/foreign/wrong-kind choice cases. Preserve all29 original fixtures,
five legacy receiver blobs and21 raw captures. Async phases stay boxed at default
stack/profiles/threads; do not weaken Windows or physical assertions.

Return the entire coherent source diff and all new test bodies for independent
review before root allocates focused/fast/canonical, frontend, exact-head CI and
the defined combined-package native two-Mage/recast/stable recovery acceptance.
No older package, private28 result or accepted-main run verifies the new union.
Full broader Gate4 offstage requirements and final literal merged-main verification
remain required. Exact next action: commit this plan, normally receive exact32,
audit all parents/bases and return its clean checkpoint for root review.

Status, 2026-10-04: **BOUNDED CORRECTION AUTHORED — INDEPENDENT REVIEW PENDING.**
The correction source and eight additional controls are authored. The original
16 controls remain unchanged; all 24 remain **UNCOMPILED/UNRUN**. Direct rustfmt
and static diff/preservation checks passed. The clean checkpoint is frozen at
handback and writer ownership returns to root. No self-review approval, public
activation, application/restore/native or gate acceptance is claimed.

Historical correction transfer: root transferred sole
source writing to `/root/gate4_ground_oct4` from clean
`9ffa19a56a09d7f73f735ee0187b05e02a758213`, tree
`86bb26338c4673e29f1353c3c465e79f6ce4e3ec`, after the full independent review
recorded three concrete P2 findings. This plan/status amendment is committed
before correction source. The reviewer now becomes the correction author; root
will independently review the complete correction and no self-approval is claimed.
The full external review is
`tooling/offstage-9ffa19a-full-independent-review-2026-10-04.md`, SHA256
`57e402cb047927125815f29922f4e27feb33a99cfae0fed58d88e71b4e65d96c`.

The correction scope is exact: (1) validate allowed source/work-kind and exact
Time-ticket shape for completed as well as live bindings after the source is
removed, without claiming journal authentication; (2) require the whole existing
retained-dependency union to have actual released participant positions before
elapsed time and at paused boundaries; (3) distinguish actual allowed ground/solid
support from `fall_destination(None)` for liquid suspension; and (4) replace the
new redundant field initializers with shorthand. Reuse the current records and
dependency scan; no new serialized fields, source grant, public admission or
shared falling-semantic change. Unsupported liquid suspension remains refused.

New controls must use otherwise valid predecessors and prove whole-state atomic
refusal: real post-choice completed-binding/trace forgeries (including ticket
command/step), omitted eligible legacy dependencies, and authored unsupported
liquid geometry with ordinary and real stable recovery. Preserve all original 16
controls and every protected source/history/capture byte; add genuine supported
ground/solid geometry controls. All new and original controls remain UNRUN.
Only direct rustfmt and static Git/file checks are authorized; no Cargo/compiler,
tests, helper execution, npm/build/native/database, push/PR/CI operation. Return
clean source and full correction evidence to root for independent review before
any verification allocation or broadening. The complete vertical-slice contract
below remains unchanged.

Correction implementation notes: elapsed admission uses the existing complete
retained-dependency scan, then checks actual participant membership; paused shared
release validation checks the same union through the candidate-bound proof. This
avoids recursively deriving a proof to scan a due batch. Every deadline binding,
including applied and cancelled records, now matches its allowed work kind and
legacy/stable ID, and every Effect ticket matches the batch's original Time
command/step/ordinal. This is structural consistency, not authentication of a
removed source. Physical eligibility requires floor or authored solid-top contact
and refuses liquid intersection, suspension and unsupported liquid-surface support;
the shared falling functions keep their existing semantics.

The eight added controls cover real post-choice completed Effect forgeries,
completed legacy ID and real stable-wake actor forgeries, cancelled group-child
forgeries, eligible omitted source/target/concentration owner, an omitted future
dependency at an actual paused batch, ordinary/stable liquid suspension and
surface refusal, and real elapsed sources on supported floor/solid tops. All
negative controls assert atomic refusal and use otherwise validated predecessors;
legacy/group/geometry fixture limits remain explicitly labeled. The next action
is root's independent full corrective-delta review and separately allocated
verification, with the public/restore guards still closed.

Historical status at 9ffa19a: **GUARDED CHECKPOINT AUTHORED — REVIEW PENDING.**
Domain/rules/codec/validation and 16 new controls are authored; every control is
**UNRUN**. Direct rustfmt and static diff checks passed. Public current/historical/
restore admission remains closed. See the full [consumer and evidence inventory](gate4-offstage-context-inventory.md).
The branch is frozen at handback and writer ownership returns to root; full
independent source review and allocated verification precede any broadening.
No application/UI/native or gate acceptance is claimed.

Historical writer transfer: root explicitly transferred sole
source writing to /root/gate4_ci_oct4 from clean corrected plan
`0f87dae259bc2d899a7dc1189337035f76b5b39b`, tree
`463f156d6eb7af9cf55ac413fd8aa244ae30ceec`, after full root and independent
review. Both complete reviews and this corrected plan were read before this
writer/status commit, which precedes source changes. Root review
`tooling/offstage-0f87dae-root-plan-review-2026-10-04.md` SHA256
`df7b197898c7e255e251ce5f45da290e6e2ec929ea8ddf5f2a8e306daf5a6576`; peer review
`tooling/offstage-0f87dae-complete-independent-review-2026-10-04.md` SHA256
`0d0219560d230e03a80833b73c94ef00ca2512c34a4d44ddd947e5fce720f871`.

The transfer authorized the coherent domain/rules/compatibility/validation and
actual producer controls checkpoint below, with every mandatory turn-context
consumer inventoried.
Keep public admission closed until the complete app/UI/restore path is coherent.
Return clean source for full independent review before broadening. The entire
production vertical slice, original history, application/recovery and native
acceptance remain the objective; this guarded checkpoint cannot replace them.
Direct rustfmt and static Git checks only are authorized. No compiler/Cargo/tests,
npm/build/native/database, push/PR/CI operation is allocated. Root retains the sole heavy/native slot (at source handback its current
focused run is session78392); no heavy execution was performed by this writer. No other checkout, source profile, protected capture
or raw artifact may be changed. A concrete caller contradiction requires a plan
amendment before scope expands. Gate 4 remains active.

Historical initial plan status: PLAN ONLY; no implementation was authorized by
that checkpoint before the explicit reviewed transfer above.

Root's review correction after initial plan `37bdd97`: retain the existing
automatic singleton dispatch. The two-Mage native recovery cut is the original
simultaneous choice, before either expiry is selected; after selection the second
expiry may drain automatically. A post-first-choice pending recovery control
requires three genuine independent equal-deadline producers. The new elapsed
ruling separately rejects control characters; old aftermath ruling semantics stay
exact. Both corrections require the final complete-plan review before source.

## Ownership and exact development dependency

Branch `codex/gate4-offstage-deadlines`, checkout `gate4-offstage-deadlines`.
Root created this fresh branch from remote-confirmed expiry candidate
`c9bbcb5f31a4b296bd88ce96464fda9d9ceffc27`, tree
`9a127388f5d6878412975371724c77465137d5f9`. Both identities and clean status were
read locally before planning. Original source author was `/root/gate4_ci_oct4`;
bounded correction author is `/root/gate4_ground_oct4`. Writer ownership returns to root with
the frozen checkpoint. Root retains publication, acceptance, dependency integration
and the sole heavy slot.
No other checkout is writable under this assignment.

Expiry PR52 is an **unaccepted development dependency**. Its previously verified
fcc source/CI/package and later documentation/main union do not verify this new
branch. Accepted release main is `dbf1d633460473183324b4ec519e8d1980884b5c`; this
plan must reconcile the actual accepted expiry/main before final acceptance.
Do not import Air, MR, Grapple, Shove, counts or Ogre merely to populate a test.
If root integrates a relevant accepted change, review its complete source union.

Planning inputs read: root `AGENTS.md`, [product definition](../../product-definition.md),
[Gate 4](../../checkpoints/gate-04-tactical-encounters.md),
[gate protocol](../../checkpoints/gate-execution-protocol.md),
[finish step 4](gate4-encounter-finish.md),
[release](gate4-encounter-release.md), [expiry](gate4-timed-expiry-boundary.md),
[ADR020](../../architecture/020-rules-restore-history-validation.md),
[ADR026](../../architecture/026-interruptible-encounter-resolution.md), and
[ADR028](../../architecture/028-reaction-execution-and-work-ownership.md).
The complete external recommendation
`tooling/offstage-dbf1d63-deadline-next-slice-recommendation-2026-10-04.md`
was read as source-grounded planning input, not execution evidence or independent
approval of this new plan. Its substantive decisions are made concrete below.
Its SHA256 is
`07d38930ac75ee0244db81786eb5e57b7225417ec7254444d8658fbdbab45cbb`.

## Product objective and bounded completeness

An ordinary host, in the real application, can request a positive elapsed interval
after a legitimately released encounter. The runtime stops at each earliest
absolute deadline, offers genuine simultaneous ordering to the host, resolves
the supported consequences through the existing stack and resumes the same
accepted interval. Save/exit/reopen, original retries, session suspension and
independent semantic restore preserve the exact target and remaining work.

This advances complete tactical durations/death recovery, causal time compression,
explainability, controller authority, private information and exact session
suspension. It does not complete broader travel/rest/world simulation, autonomous
DM or voice. Nor does this first checkpoint complete Gate 4 offstage timing:
dead/omitted dependencies, physical no-turn consequences, source-authorized rest
and owner-relative timing across changed encounters remain Gate 4 requirements.

The first slice admits only an already valid released dependency set and supported
condition-free tactical duration work plus genuine stable waking. It refuses
unsupported situations **before advancing any time**. These bounds must be visible
in capability explanations and the rules ledger; they must not become a claim
that the full finish/offstage contract was reduced.

## Source findings and exact caller decisions

| Current source seam | Required implementation decision |
| --- | --- |
| `kernel/engine.rs::resolve` and legacy `RulesAction::AdvanceTime` | Keep the public tactical-state guard. Legacy AdvanceTime and its old direct AtTime cleanup retain historical semantics. Add a distinct typed tactical action; do not grant access by removing the guard. |
| `tactical/turns.rs::begin_boundary_from` | Keep PR52's due-Time/Turn decision and every accepted operation stamp unchanged. Released elapsed time never calls this turn producer or makes synthetic Start/End observations. |
| `TacticalResolution::{turn_actor,turn_number,boundary}` in domain `tactical_resolution.rs` | Replace the mandatory internal turn triple with the sum context and strict compatibility wire adapter below. No fake actor, turn zero, copied final turn, Option fallback or default controller. |
| `turns::{pump,choose,new_effect_work,push_frame}`, `continuations::{start,finish}`, `work_trace` | Retain one resolution, occurrence allocator and frame stack. Context dispatch separates turn completion from elapsed-interval completion; actual effect/vitality reducers, work entry/leave and cancellation remain shared. |
| `turns::pump` calls `creature_bridge::after_turn`, movement pruning and End-to-Start advancement | Those turn-only calls must never run for a released interval. No recharge, legendary opportunity, movement allowance or six-second round increment is generated off turn. |
| `release::{retained_dependencies,validate_history,require_finished_encounter}`, `aftermath::validate`, `turn_validation::validate`, `validation::validate_tactical_state` | Preserve ordinary Finished validation. Add a narrowly authenticated in-progress interval validator accounting for each due record and pending work. Quiescent Finished admission for preparation/replacement stays separate and refuses a live interval. |
| `kernel/validation.rs` legacy `Expiry::AtTime` check | An expired legacy record remains invalid except while the exact admitted interval batch owns its unique unconsumed legacy-expiry work. Do not globally tolerate overdue legacy effects. |
| `tactical_effects::{observe,trigger_is_applicable,resolve_trigger}` | Observe authoritative Time once at each deadline; scan all raw groups/effects, including suppressed ones. Preserve individual group/child tickets and rechecks when group cleanup cancels siblings. |
| `tactical_damage::stable_wake_at`, `continuations::RecoverStable` | Derive the deadline from the actual retained physical d4 and stable origin; reuse RecoverStable for its ordinary 1 HP. The original zero-HP Unconscious/death state is positively supported, not confused with a forbidden tactical effect payload. |
| `turns::effect_operation`, `tactical_vitality_adapter::drop_held` | An expiry may reveal a suppressed Unconscious condition and drop equipment into `flow.ground_items`, which Finished forbids after custody moved to the retired space. The first slice refuses tactical condition payloads and airborne/physical consequence obligations before admission; it does not discover this failure after moving time. |
| `table_source_control`, `table_tactical_choices`, `table_transport`, `EncounterPanel.svelte` | Introduce explicit host interval controls and host-only interval ordering; preserve origin/occurrence capabilities. Do not pass a fake PC through the existing mandatory `TableTacticalContinuation.actor`. |
| `rules_restore` origin audit/replay, snapshot codec and SQL migration preflight | Add exact new action/context/receipt/upgrade bindings and duplicate-safe legacy rejection; original accepted history remains the authority, not a snapshot's own interval claim. |

Implementation must inventory every constructor and turn-context consumer before
changing the domain type. The footprint includes attacks and its creature/unarmed/
opportunity validators, casting, areas, movement, medicine, Second Wind, hit/missile
responses, creature bridge, turn validation, work ancestry, app hit/missile views,
source-control routing and tactical choices. Turn-only readers use a fallible
`turn_context()` accessor and reject the no-turn variant; no unwrap/default may
invent authority. The shared expiry/wake path uses the released context explicitly.
The five frozen legacy receiving suites currently contain no direct resolution
constructor requiring adaptation; preserve their complete blobs. Ordinary mutable
source-compiled helper constructors may be mechanically adapted and inventoried.

## Semantic allocation and historical wire contract

Propose **ReleasedTimeV1 = flow 7**. Flow 6 is already reserved for future
Counterspell in the independently developed Grapple lifecycle plan; this plan
preserves that reservation. Numeric 6 remains unknown/rejected until Counterspell
has its own reviewed implementation. This is a deliberate gap, not implicit
Counterspell support or permission to renumber existing events.
The reservation was checked at
`gate4-grapple-attack-read-context/docs/exec-plans/active/gate4-grapple-lifecycle.md`
under "Version, wire, ownership and recovery proof". Root confirmed the collision
during planning and directed this explicit7 proposal for review. Any later union
with Counterspell must deliberately combine capabilities; it cannot treat7 as
already implementing6 or downgrade a released interval to the reserved executor.

All feature/version switches use named predicates and explicit transitions:
ReleasedTimeV1 inherits release, ancestry, hit-Shield and missile behavior from 5;
flows 1–5 retain their previous meanings. Do not infer capabilities from `>= 5`,
an integer range or adjacency. Audit domain enum conversion, completion validation,
source version checks, live/historical policy, Begin/upgrade, aftermath/release,
app current execution, typed transport and desktop version lists together.
Include domain `CampaignState::validate`'s missing-completion guard and
`TacticalEncounterHistory::validate`; neither may retain a5-only hole for7.

Fresh Begin uses 7 after the complete production path is verified and activated.
The historical unit UpgradeExecution stays 1→2 forever; historical targeted
upgrades retain their old behavior. A live settled older flow can reach 5 through
the existing explicit bridge, then a separate **5→7** upgrade. No implicit
1–4→7, downgrade or unknown6 hop is introduced. Upgrade 5→7 requires host/system
provenance with no actor, current active-session authority and a settled Active
boundary or a fully valid, quiescent Finished state. It cannot reinterpret paid
Ready, raw work, a pending resolution or an in-progress interval. Existing closed-
session release/target5 exceptions remain exact; new time/target7 commands require
a real active session in this checkpoint.

For a Finished 5→7 upgrade, add omitted-None `flow.released_time_upgrade` containing
`{origin, release: CommandId, from: EncounterReleaseV1, to: ReleasedTimeV1}`.
Its accepted UpgradeExecutionTo action and exact last release authenticate it.
Do not rewrite the original completion receipt's execution5, release instant,
final actor/turn or predecessor. A fresh flow7 release instead records execution7
normally. Finished history permits exactly receipt5+authenticated5→7 or matching
receipt7; arbitrary mismatched flow/receipt versions remain invalid. No upgrade
record is emitted for any old action or accepted absent-field state.

Internally define `TacticalResolutionContext` with exactly:

- `Turn(TacticalTurnContext { actor, number, boundary })`;
- `ReleasedInterval(Box<ReleasedElapsedContext>)`.

Retain all existing shared resolution fields, including `origin`, frames, pending,
work_trace and next_occurrence. Implement a **strict explicit serialization adapter**
for TacticalResolution. A Turn context emits the exact old top-level
`turn_actor`, `turn_number`, `boundary` fields in their existing position/order;
every other old field keeps its original omission/null/order behavior. A released
context emits `released_interval` instead and omits all three turn fields. The
typed map decoder rejects duplicate/unknown members, both forms, partial triples,
explicit nulls claiming a context, missing context and future variants. Do not use
a Value map that silently collapses duplicate keys, or an untagged/flatten fallback
that ignores unknown authority. Old wire bytes and original event semantics must
be tested directly, not merely compared after deserialize/normalize.

Schema4 remains the optional tactical envelope. Add no SQL schema migration or
rewrite of shipped SQL checksums. Schema1/2/3 reject all new non-null authority
through strict typed codec and atomic database preflight, including duplicate
field/null shadows. New enum/action names fail closed in older binaries. The
existing tactical event envelope can retain version1 because the new action and
flow7 interpretation are explicit and old actions remain unchanged; any discovered
normalization ambiguity requires a reviewed envelope-version amendment first.
Do not silently fill execution on an accepted old `table.action@2` request.

## Concrete interval, batch and completion records

Add `TacticalAction::AdvanceReleasedTime { seconds: u32, ordering:
ReleasedTimeOrdering::HostSelect, ruling: String }`. The ordering enum has no
default. The bounded ruling explains elapsed-time adjudication; it does not grant
rest, healing, loot, consent, source admission or a particular consequence order.
Require positive seconds and checked i64 target addition. The new ruling must be
trimmed, nonempty and at most2000 bytes, matching the existing aftermath bounds,
and additionally reject control characters. The latter is new-action validation;
do not change aftermath's historical ruling rules or accepted bytes. Root command
metadata retains exact campaign/session/issuer and `actor=None`.

`ReleasedElapsedContext` contains the last release command ID, `started_at`,
`target_at`, `progress_at`, explicit ordering/ruling, and the optional prior
completed interval ID. `resolution.origin` is the accepted original advance
command; do not duplicate or replace it when a later choice resumes work. Context
progress must equal authoritative `state.clock.now`, lie within start/target and
advance only through actual accepted operations. Retain a bounded ordered list of
deadline batches as immutable causal evidence, not another executable queue.

Each batch records its ordinal, actual instant, Time observation operation stamp,
and the exact admitted deadline bindings paired with allocated work occurrences:
effect/group ID plus original source/establishment identity and expiry instant;
legacy effect ID plus original source/target/expiry; stable actor plus original
stabilization origin and recorded recovery-roll identity/deadline. Work membership
lives **only in existing frames/pending**. A batch manifest is non-executable
provenance used to validate the exhaustive partition. Its completion records bind
the actual entered work key, actual completing command and either Applied or
CancelledBy the precise completed group/source occurrence. A removed effect is
not invented anew to authenticate its own cleanup. Journal replay must reconstruct
the batch from its real preceding state and original accepted actions.

The existing ancestry allocator never resets within the interval. Its 32,768 work
and 128 nested-frame bounds remain. Bound this first producer to at most128 distinct
deadline batches; admission counts raw absolute/stable deadlines up to the target
and refuses excess before mutation. This first scope creates no new timed sources,
so that admission bound is conservative. Recheck actual capacity throughout and
reject the current transaction atomically on any mismatch, without partial time.

After all work and the target are reached, append an omitted-empty
`TacticalEncounterHistory.elapsed_intervals` receipt containing original origin,
release ID, predecessor interval ID, start/target, explicit policy/ruling and
actual completion metadata. `completed_at == target_at`. It references authority
and progress, not copies of HP/items/effects. The resolution is then cleared and
ordinary strict Finished validation must pass. Receipts remain linked to their
own release after a later encounter replaces the live flow. Validate unique
command IDs, chronological/nonoverlapping interval chains and release bounds;
replay authenticates them. A future interval starts at actual now, never at the
old release timestamp or a restarted eight-hour duration.

## Admission, deadlines and the one shared stack

Before any mutation, require full current-state validation, current flow7, a real
last completion/closed scene, no live resolution and ordinary release dependency
readiness. Every retained dependency must still satisfy the existing future
placement route and be present among the retained released participants. Preserve
the100-placement bound. A dead/omitted source or target remains outside this first
route; do not close another Active scene or synthesize a missing participant.

Additional first-slice eligibility is explicit and whole-campaign:

- No rules/table pending input, permissions, Inspiration transfer, source routine,
  recharge, reaction/Ready/turn budget/cursor, paid attack grant or existing work.
  Preserve existing bans on legacy AtTurn, owner-relative expiry, Turn/per-turn/
  zone triggers and unfinished casting. Scan suppressed records too.
- No ordinary `rules.rests`, knockout recovery or knockout-rest authorization,
  even if its unsupported deadline lies after this target. This chosen simple
  bound avoids entering a latent unsupported rest state; elapsed time grants no
  short/long rest, hit-die use, pools, recharge or spell-slot recovery.
- Every raw tactical effect has an empty `conditions` payload, including suppressed
  overlaps. Defense-only effects such as real Mage Armor are admitted. This does
  **not** require all actors to lack Unconscious: the actual zero-HP Stable recovery
  record, existing recovery projection and its accepted d4 remain supported.
  Stable records require a real noncancelled original d4 and a future derived wake;
  an unstable actor or missing/due d4 remains rejected before release/admission.
- Retained participant positions must be supported at ground level on their real
  saved battlefield, with no pending fall, airborne position, unsupported landing
  or flight-loss consequence. Refuse any required offstage geometry/custody route.
  No synthetic position, altitude, fall, dying save or die is permitted. Existing
  retired ground items remain exactly in their old scene spaces.

The condition restriction is based on actual `TacticalEffect.conditions`, not
spell names, visible effective conditions or actor HP. It prevents overlap expiry
from introducing a new unconsciousness/drop/fixed-point concentration consequence
that this checkpoint cannot carry through retired custody. Legacy effects still
use their actual old AtTime removal semantics; they are not silently ignored.
If source review finds any other admitted payload can produce an unsupported
physical consequence, identify it and amend this plan before broadening admission.

Compute the next stop from all raw tactical group/effect AtTime values, all legacy
AtTime effects and each real `stable_wake_at`, bounded by target. Never scan only
active projections, visible actors or the selected scene. With no deadline before
or at target, advance directly to target and complete. Otherwise:

1. Retain the exact batch/candidate bindings and advance only to its earliest
   instant on the resolver clone. Create its Time observation once using the actual
   executing command/step, retaining the interval's original root separately.
2. Collect every resulting expiry ticket through `new_effect_work`; add each due
   stable `RecoverStable` and new no-roll `ExpireLegacyEffect { effect }` occurrence
   to the same sibling frame. Deterministic allocation is bookkeeping, not a choice
   of fictional priority. Group and child expiry remain separate occurrences.
3. Reuse `pump` and `continuations::start`. A singleton follows current automatic
   dispatch; two or more material siblings require actual host ChooseTurnWork.
   Host selection is journaled against the same origin/occurrence and batch. No
   vector/UUID/network/default order substitutes for it, and the ruling text does
   not supply a choice. No current-turn or area consent is borrowed.
4. Shared effect cleanup rechecks applicability. If group cleanup removes a sibling
   ticket, mark that exact occurrence cancelled by the actual parent cleanup before
   pruning its frame entry. Never delete unrelated same-name effects or count a
   cancelled occurrence as an applied one. Its provenance survives the interval.
5. RecoverStable calls the existing vitality producer once. Legacy expiry removes
   exactly its still-due retained effect and corresponding legacy concentration
   pointer by the existing legacy-removal semantics; factor a shared removal helper
   only if old operations/order remain exact. It emits no raw roll key. Unsupported
   work kinds, raw roles, save decisions, source hooks and child producers fail closed.
6. Drain every descendant before siblings. When frames are empty, dispatch to the
   released interval resume routine, recompute from resulting state and continue
   toward the original target. Never call advance_turn/begin_boundary/after_turn.
   Completion at a deadline equal to target occurs only after all its work settles.

The allowed first no-turn work set is exact Time-expiry Effect tickets,
ExpireLegacyEffect and RecoverStable. The shared architecture remains capable of
future children, but this plan does not claim no-turn damage/fall/raw execution.
No current source grants a released ongoing-save/concentration-group positive:
Hold Person's repeated target End saves prevent release. No Shield of Faith
table positive may be invented; its catalog entry is not an executable grant here.

## Validation and authentication at intermediate states

Use a bounded, pure interval validation context computed from the actual candidate
state; no thread-local, global historical trust flag or client-supplied exception.
Distinguish input admission (fully quiescent released state), an in-process batch
construction phase and a completed public transition (complete pending/finished
partition). Construction authorization must be scoped to the actual reducer call,
not serialized or accepted by public state validators.

At every persisted pause validate:

- exact flow7/Finished/release/upgrade/origin/session lineage, start/target/progress,
  prior receipts, bounded ruling/policy, checked batch order and actual now;
- no live turn/effect cursor, turn budget, Ready, attack/hit/missile/cast/area/
  movement/fall/source window, raw request or failed save in this initial route;
- every raw due record at now has exactly one admitted batch binding and exact
  still-live work/ticket, or a source-valid completed/cancelled receipt; no later
  deadline has been admitted early, no earlier due record is omitted, and no work
  belongs to another interval or prior batch;
- exact occurrence uniqueness, ancestry, source and observation stamps, completed
  work disjoint from remaining work, no entry marker serialized as active authority;
- the unchanged completion/scene/highwater/custody invariants and continued bounded
  eligibility after each result. No state may become persistently invalid simply
  because its due dependencies were not yet allowed to enter the shared stack.

The exception for due legacy effects and release dependencies consults this exact
proof. Ordinary state validation cannot accept a fake elapsed record merely because
it names the due object. Kernel/public resolution validates prior state; application
restore reconstructs its creation from the original journal. New context/origin/
receipt introduced only in a later snapshot, even consistently forged across the
final snapshot and current state, must fail before destination writes. Extend
ADR020 origin audit to the original advance, actual choices/completions, Finished
upgrade and interval receipts. Original pre-tactical anchors remain mandatory.

`require_finished_encounter` remains the **quiescent** prerequisite for new setup,
source/equipment preparation and replacement. It must not return success for a
pending interval. Add a separate exact session-suspension check for a valid paused
host interval. EndSession may preserve that host choice; reopening rebinds genuine
attendance and the host's new command session while preserving the original
interval origin. No clock advancement occurs at session end/start. The interval
cannot be cancelled, replaced or retargeted to escape owed work in this checkpoint.

## Application, transport and desktop completion

The new action uses the normal authenticated TableAction::Tactical path, atomic
state/event/observation commit and accepted-before-stale retry lookup. Preserve
raw original request bodies and immutable accepted responses. The application
derives trusted host/system metadata with no actor; renderer/player payloads may
not supply it. ChooseTurnWork checks both current host authority and exact no-turn
context rather than the old turn actor or area delegation helper alone.

Add an omitted-None **host-only** `released_time` DTO on the tactical view, with
eligible/blocker, actual start/progress/requested target, and exact available work
choices. Preserve the existing mandatory-actor turn continuation DTO and its old
bytes; the host-only interval projection is a separate tagged context/view over
the same resolution, not a second scheduler. Never expose private dependency
names/counts, legacy source IDs or absence-sensitive pause cards to unrelated
players. Host handles reuse the existing capability binding to audience, current
revision, interval origin and occurrence; extend its projection mapping rather
than accept raw occurrence/authority from arbitrary clients.

EncounterPanel offers a plainly labeled elapsed-duration/ruling/explicit host-
ordering form only for this supported Finished state, reports the exact progress
and pending host choice, and resumes the retained target after selection. It
must not label the host as taking a PC turn or promise rest benefits. Block ordinary
setup/preparation while the interval is in progress. Source-controller selection,
current session identity and existing physical dice routing remain unchanged.

Presentation classification must treat private ordering steps as host-only at
acceptance. The generic shared "Encounter action recorded" path cannot leak one
observation per hidden expiry. A safe public interval announcement may report only
the host's admitted elapsed intent; no private count or target is included. Compare
unrelated audience DTOs, opaque revisions and transcripts across zero/one/two
hidden work sets while visible state/time/accepted public facts are held equal.
Where an actual visible deadline changes the public state, assert only that allowed
visible difference; do not falsely require time passage itself to be invisible.

## Meaningful controls and original-history gate

All following controls are planned, not authored/passing evidence:

1. Codec and version controls: exact legacy resolution/Begin/event bytes; reject
   mixed/missing/null/duplicate/unknown context, flow6, missing flow7 prerequisite,
   fake Finished upgrade, wrong release and old-schema future authority. Exercise
   real SQL migration rollback and rules-enabled export preflight, not just JSON
   decoding. Keep old table request normalization and accepted retries exact.
2. Genuine Mage route through actual catalog creation, physical component choice,
   source spell casting, turn progression, conclusion and legitimate release.
   Mage Armor is self-targeted here. One second before expiry preserves exact AC/
   effect; at expiry removes only the actual instance; beyond stops and then
   resumes to target without refund. Two Mages self-casting in the same round
   create two real equal deadlines and permit both actual host orders. A later-
   turn recast gives a genuine older suppressed instance and later survivor.
3. Genuine stable route: actual damage to zero, actual stabilization/First Aid,
   required owned physical d4 and original recorded wake, legitimate release,
   before/exact/beyond wake through the new command. Assert exactly1HP, retained
   raw face/origin, no reroll/rest/resources. This proves stable Unconscious remains
   admitted despite the tactical condition-payload restriction. Missing d4,
   unstable death saves, knockout and active rest are atomic negative cases.
4. Legacy AtTime route: a real accepted old producer carried through a valid
   current upgrade/release where reachable; exact targeted removal/concentration
   pointer and unrelated survival. If no current table producer can build that
   legacy state, use its genuine original accepted prefix for receiving coverage
   and label pure lifecycle fixtures separately. Never generate a supposed old
   positive capture with the new executor.
5. Lifecycle-level group/suppression controls: same-time group+child tickets,
   suppressed group/target expiry, cancellation after source removal, reverse
   host order, unrelated later instance and changed/missing/extra batch entries.
   These test shared mechanics; they do not claim an actual released Hold Person
   or Shield of Faith producer. No new spell/grant/profile is admitted for tests.
6. Real file SQLite and independent portable mirror at admission, before deadline,
   simultaneous choice, after first choice and completed target. With exactly two
   independent expiries, the post-first-choice cut may already be completed;
   never invent a second pause or suppress automatic singleton dispatch. If a
   post-first-choice pending recovery cut is tested, produce three genuine
   independent equal-deadline expiries. Close/reopen and
   replay the same accepted inputs; assert exact target/clock/receipts/operations/
   tickets/resources and accepted retry bytes. Box async phases and retain default
   Windows stack/profile/thread settings. Only request-time export timestamp may
   be normalized in independently equal saved-export comparisons.
7. Host, controlling player and unrelated player transport tests; forged issuer/
   actor, changed duration/ruling/policy, stale revision/handle, foreign interval,
   earlier occurrence and duplicate choice all reject without any durable rows,
   clock movement or cost. End/start session and process reopen preserve the exact
   host choice, with actual current-session checks. Failure during atomic commit
   preserves retry semantics, never half-applied time.
8. Strict restore negatives alter start/target/progress/batch instant/source/cause,
   omit an overdue object, duplicate a ticket, claim a future expiry, invent a
   completed/cancelled receipt, forge removed source identity or a completion at
   the wrong clock. Apply hostile edits to current and historical snapshots and
   accepted audit/event envelopes; actual preflight must reject before writes.
9. Admission negatives prove unsupported relative/zone/turn work, source/dead/omitted
   dependency, airborne geometry, tactical condition payload, rest/knockout and
   excess deadlines reject **before** changing the clock. Test group/overlap
   condition revelation explicitly rather than replacing it with a generic blocker.
10. All original29 frozen capture blobs, five receiving-suite blobs and21 protected
    raw artifacts remain exact. The five suites are legacy_reactions_v1_replay.rs,
    legacy_savage_replay.rs, legacy_shield_hit_v1_replay.rs,
    legacy_shield_missile_v1_replay.rs and legacy_tactical_replay.rs. Preserve all
    seven original flow4 exports/eight continuation cases, earlier flow histories,
    operation steps, roles, accepted dice/source pins, retry bodies, audience bytes
    and original baseline evidence. No regeneration, weakened assertion, filter,
    ignored test or generated positive history discharges compatibility.

## Native and final verification acceptance

After source review and root's verification-slot allocation: focused meaningful
controls, `./scripts/verify-fast`, canonical `./scripts/verify`, actual desktop
checks/tests/build and exact-head Linux/MSVC/MSRV/guard checks must pass. Parent
results and different-tree packages are not receiving-head evidence. Use default
stack/profiles, preserve failed attempts and inspect actual logs/counts/identities.

Run a fresh verified **combined implementation package** in a separate normal
campaign. Create two actual Mages, self-cast in one real round, conclude and
legitimately release at the unchanged clock. Advance to before their deadline,
close/reopen, request beyond it, observe the actual earliest two-way pause and
close/reopen at that original choice. Choose one host expiry and allow the
remaining singleton to resolve automatically to the original target; close/reopen
the completed result. Check exact armor/effects and unrelated HP/resources/items,
then enter a
genuine next encounter. Repeat later-turn recast/suppressed expiry and the real
stable-d4 waking route separately. Use ordinary controls and real submitted dice;
label any QA faces accurately. No injected clock/effect/JSON/SQLite or constructed
selected state substitutes for those paths.

Keep original native observations, read-only saved cuts, exports and receiving
replay separately attributed. The existing fcc74a9 expiry package lacks later Air
content and is not a combined package; never open newer incompatible QA saves
with it to manufacture acceptance. Package/hash identity, actual native interaction
and saved-state audit are distinct evidence.

Before merge: reconcile verified accepted parent/main, review the full exact union,
verify all required checks and this slice's actual production/native acceptance,
merge with expected-head protection and verify literal merged main separately.
No routine human merge approval is needed; root coordinates these actions.

## Checkpoints, risks and exact next action

1. **This plan checkpoint:** source/authority/caller map and version gap decisions;
   root plus independent peer review before any implementation.
2. On explicit writer transfer, first record the accepted reviews/ownership in
   this plan. Implement a coherent domain/rules/validation/compatibility checkpoint
   with actual interval/work/receipt producers and meaningful tests. Keep public
   activation closed until app/UI/restore are coherent; that checkpoint is not
   product completion and cannot merge as a substitute for this objective.
3. Complete app/transport/desktop and real producer recovery/privacy controls;
   independently review the entire source delta and every adapted turn consumer.
4. Root allocates execution, fixes actual failures through reviewed small changes,
   then conducts production package/native/integration acceptance above.

Concrete risks requiring review are the strict old-wire adapter across all
constructors, explicit flow7 with a reserved unsupported6 gap, Finished5→7 receipt
authentication, due-state partition validation without recursion/self-authentication,
operation stamps when a host choice reaches the next deadline, source-removal
cancellation provenance, and private observation classification. These are concrete
implementation obligations, not unspecified future design options. If the exact
producer/caller audit contradicts a bound, amend the plan before source expansion.

Remaining Gate4 work after this bounded slice: released dead/omitted source/target
dependencies with real saved geometry and authority; effects that reveal conditions
and cause drops/concentration/falls; no-turn damage/raw/player choices; source-
authorized knockout/rest completion; owner-relative obligations and explicit
rebasing/continuing cadence for changed initiative; source-safe pickup/revisit and
arbitrary later encounter replacement. Broad world simulation remains a later gate,
but these tactical obligations stay in Gate4 and remain acceptance blockers there.

Historical plan verification status: only read-only source/document analysis and Git
identity/status checks. No code, new tests, compiler, Cargo, npm, native process,
database, push, PR or CI operation ran for this plan. Air session87840 remains
root's sole heavy slot. That plan-review handoff is now complete as recorded above.
Exact next action is root's independent review of the complete bounded correction
and its added controls. All controls remain uncompiled/unrun until root separately
allocates execution; no further source work or public activation is authorized by
this frozen handback.
# October 5 strict-lint correction plan

The first focused attempt at b7085f010c1fef98297e2676d3f4a176f2ab0a7f passed
formatting, then failed strict Clippy before any of the 28 selected tests ran.
The actual diagnostic is clippy::derivable_impls for Present<T>'s manual Default
implementation in tactical_resolution_wire.rs. The existing implementation
returns the unit variant Missing; derive(Default) with #[default] on Missing
expresses that same behavior without imposing a default value on T. No wire
fields, omission, null rejection, serializer or decoder logic should change.

Commit this plan before that narrow source correction. Preserve every test body
and the failed attempt's logs. Independently review the delta, then rerun fmt,
strict four-package all-target Clippy and the same 28 exact controls on the new
frozen head. Any further diagnostics must be inspected and corrected without
lint suppression or acceptance reduction. App/restore/public/native integration
remains open; this is a compile/lint correction, not feature completion.

## October 5 second strict-lint correction

The exact 3c5272e focused attempt passed formatting, then strict Clippy failed
at released_time/validation.rs:286 with collapsible_if. No selected test ran.
The complete failed log is preserved externally with SHA256
face6100116a32d433a9f9e1fe9c81493ea506f70afac5174ee886e15bc8c1a5.
Commit this plan first, then combine only the nested Effect-pattern and pending
ticket predicate into the equivalent short-circuit let chain. Preserve the
fallible effects lookup, exact refusal text, every test, and all other source.
Review the complete small delta independently before rerunning the unchanged
28 exact controls with fmt and strict all-target Clippy. Earlier failures stay
recorded; public/application/native acceptance remains outstanding.
