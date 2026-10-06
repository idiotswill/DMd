# Gate 4 â€” Complete public Grapple application path

Status: **TYPE AND STRICT LINT CORRECTIONS AUTHORED; NEW SOURCE UNVERIFIED**.
Date: 2026-10-06. Sole writer for this receiving freeze: private_grapple_oct6, explicitly allocated by root.
Branch: `codex/gate4-grapple-public-completion`; checkout `gate4-grapple-public-completion`.
Exact original parent: `ade8e93b450d6e02027afc7615a555db16b2915d`, tree
`a6097d485244a2ed5443765fb4ee40c34625892b`.

## Allocation, authority and evidence

### Application audit-version lint correction (2026-10-06)

Published `513fd011a930922e2a81eb3f04221a1103761cf8` passes fresh minimum-version,
architecture, genericity and fast checks. Stable job `112156450474` now reaches
the application crate and reports nine equivalent numeric-pattern lints across
rules_restore, table_presentation_history, table_transport_runtime and
table_runtime. Root read every diagnostic and each surrounding reader. Commit
this plan before replacing only those nine `2 | 3 | 4` patterns with `2..=4`.
The accepted historical action/conversation schema set stays exactly 2,3,4;
no schema5/transportv4 support is added on this public baseline. Preserve every
original test, validation branch, decoder and audit join.

Root allocates a brief direct formatting pass on only the changed public Rust
files, followed by its check; no compiler, tests, npm, database or native task
runs alongside Offstage. Review any resulting formatting delta separately.
This supersedes the earlier formatter scheduling hold only. Independent source
review and fresh exact-head CI remain required; no lint suppression is permitted.

Plan `5cd1ecb` preceded the nine substitutions. Direct Rustfmt and its check on
the four changed files passed. The only additional formatting reflows the same
presentation-history iterator closure; its expression and evaluation remain
unchanged. Root read the full diff and whitespace checks passed. Tests, decoder
branches and the allowed version set are unchanged. This source is ready for
independent review; no local compiler or runtime has verified it.

### Strict lint correction after the first successful compile (2026-10-06)

Root owns the public branch from reviewed/published
`2aa9c597b7c73b8b3a34a024e258ab887d5a9bc9`, tree
`b5541197f6e82e2d2b92a8843ff447b578b58816`. Fresh Linux CI run
`37428585505` passed the Rust 1.88 check, architecture and genericity guards;
its stable Rust job `112153838423` passed fast verification and failed strict
Clippy before tests. The actual complete failure output was read and preserved.
This confirms the earlier type errors no longer block compilation, not runtime
acceptance. Current Windows verification remains pending.

Commit this plan before the narrow lint correction. Replace six numeric
`1 | 2 | 3` persistence version patterns with exactly equivalent `1..=3`.
Remove only four unused private ordinary forwarding wrappers:
`intrinsic::plan`, `planning::weapon_plan`, `planning::hit_facts`, and
`planning::hit_facts_for`. Each only forwards to its existing `_with_read`
implementation with `None`; compiler diagnostics and complete source searches
confirm no callers. Keep every live `_with_read` implementation and ordinary
None branch unchanged, retaining the common-facts documentation on its live
implementation. Collapse the nested OutOfRange check in modern_lifecycle into
an equivalent let-chain with the same parenthesized condition, short-circuit
evaluation and error. Do not suppress warnings or change any assertion, test
body, live authority check, version set or public API.

The complete resulting source diff requires an independent review and fresh
exact-head CI. No local formatter/runtime is allocated during the active Offstage
run. Old CI failure logs and unrun preparations stay bound to their original
heads. All public Grapple and Gate4 acceptance obligations remain unchanged.

The checkpoint was committed at `33a1888` before editing the four source files.
The authored delta is exactly the six equivalent range patterns, removal of the
four unused forwarding wrappers, and the equivalent OutOfRange let-chain above.
Root read the full source delta; `git diff --check` passed. No existing test body
or live `_with_read` implementation was edited. No formatter or local runtime
has run for this correction. Next: independent full-diff review, then fresh CI
on the published correction; do not credit the prior successful compile as
verification of this head.

The completed exact-2aa Windows stable log `112153838994` additionally reports
an unused `crate::table_engine::table` import in `table_tactical.rs:5`. Its
frontend check/build and all135 tests passed before the same strict-lint failure.
Root read the import's complete module context and confirmed no uses; the next
narrow correction removes only that import. Commit this evidence amendment
before removal. Do not change any live table helper or tests. This adds one
source path to the independent lint review and avoids knowingly carrying the
compiler's additional warning into the next strict check.

### Actual b6 CI failures and approved narrow corrections (2026-10-06)

Root allocated sole writing to private_grapple_oct6 from clean published
`b6ef6207d37f99bf3edb35349cd59b59cf4c81d3`, tree
`dfc4a0b18e760f661d4f66321fc51fb5fa480849`. A specific fetch of
`origin/codex/gate4-grapple-public-completion` confirmed that exact remote head.
Commit this checkpoint before the source changes below.

The actual PR67 Linux jobs `112148663982` (MSRV) and `112148664436` (Rust)
failed with the same four compiler diagnostics. Both logs identify synthetic
merge `83e1f68b04cf75122e9c49cbaed774def48426bc` of b6 into main5af, not literal
b6. The errors are two `Option<&Box<TacticalEncounterHistory>>` callback type
mismatches in `grapple/execution.rs:330/761`, and two owned `String` arguments
passed to `invalid(&str)` in `grapple/modern_lifecycle.rs:100/240`.

Windows jobs `112148663642` (stable) and `112148663907` (1.88) checked out
literal b6. Each completed Svelte check with zero errors and warnings, then
reported 134 passed / one failed across 22 frontend test files. The sole failure
is the old unknown-envelope-version specimen at `table-transport.test.ts:81`:
numeric version 3 is now explicitly supported. Each of the three new Grapple
frontend files passed its three cases. The failed test stopped normal desktop
preparation; these runs do not establish its later build/icons/notices or native
Rust/package steps. The four complete failed logs are preserved externally.
No local b6 runtime occurred; the frozen afb and b6 426-case preparations remain
UNRUN and must be preserved unchanged.

Approved scope is exactly:

- Change the two `encounter_history.as_ref()` calls immediately before
  `.and_then(TacticalEncounterHistory::last)` to `.as_deref()`. Keep the same
  borrowed last receipt, all chronology/provenance checks and all predicates.
- Borrow the two temporary diagnostic strings as `invalid(&e.to_string())`.
  The existing helper immediately owns the same message; change no diagnostic
  text, error type, geometry comparison or acceptance condition.
- In the original transport rejection test, replace only numeric `version:3`
  in the unknown-version specimen with `version:99`. This deliberately distant
  unsupported integer avoids repeating this fixture migration during planned
  version-4 dragging work. Keep the string-version, mixed-head and empty-revision
  specimens, the test name, all assertions and zero-invoke check byte-exact.

This unknown-version specimen migration is the only approved original test-body
exception in this correction.
The test still rejects an unsupported version rather than rejecting newly legal
version 3. Frontend saved-request validation strictly admits numeric 1/2/3;
TableApp selects 3 after Grapple activation, and the server's version-3 admission
and PC text dispatch agree. Do not change production frontend validation.
Preserve every current Rust test body, all nine Grapple frontend cases and every
other frontend test byte. No new type-only tests or predicate changes are needed.

Freeze a clean correction for independent root review with a complete static
delta and preservation audit. No formatter, compiler, npm, database, native run,
push or runtime-preparation retarget is allocated to this author. Root retains
the heavy slot. Only after root review may fresh exact-head verification be
prepared and allocated; the previous failed logs and UNRUN preparations remain
evidence. Group 3, dragging/carrying, canonical/CI, independent-review and
packaged-native acceptance requirements remain unchanged.

The checkpoint above was committed at
`e787c489c099a2334ac50a63c11cf65612b3ee6f` before editing the three source files.
The authored delta is exactly two `as_deref()` substitutions, two diagnostic
borrows and the one old transport specimen's `3` to `99` substitution. Static
whole-file comparison and `git diff --check` passed. All other 616 Git entries,
all 919 current Rust test bodies with attributes and occurrences, all 28 inline
test modules, the other 21 frontend test files and all nine new Grapple frontend
cases retain their exact source. The old transport file is otherwise byte-exact.
All 426 selected Rust bodies, names, attributes and lines are unchanged; the
previous afb/b6 preparations remain UNRUN and unchanged, and the four failed CI
logs remain preserved. No formatter or runtime has verified these corrections.
Next: independent root review of the complete clean freeze, then separately
allocated formatting and fresh exact-head verification. Do not run or retarget
the obsolete b6 preparation as evidence for this correction.

### Early draft CI and current-main reconciliation (2026-10-06)

The owner handoff requires opening slices early enough for CI. Root therefore
supersedes the earlier publication restriction below: after complete review of
the coherent groups 1–2 source and private-history receipt, this branch may be
published as an explicitly unaccepted draft for exact-head CI. This changes
verification scheduling only. Group 3 positive dependencies, ordinary drag/carry,
canonical verification, independent review and packaged native acceptance remain
mandatory before acceptance or merge. A draft or green partial suite is not
feature or Gate 4 completion.

The reviewed public receiver is `afbaf9b15b5a86872993d9f1f2ae0da0a81e9ddc`,
tree `e2a11fa9a1b9494dda6ac8dc7a787666091dfe88`. Before publication, normally
receive freshly fetched main `5afc992e62bb967aceec69db53f19b6347b70855`.
Its common ancestor with this branch is
`f9c0f3df7c6c7ed8c53c41adb95480e634a15c20`; the complete incoming delta changes
seven documentation paths only. Preserve all production, content, frontend and
test bytes. Review any documentation union, then freeze and retarget the prepared
426-case selection with unchanged test identities and command arguments. Preserve
the unrun afb preparation; no older runtime pass verifies the new public head.

### Approved whole private correction receipt (2026-10-06)

Root allocated sole receiving ownership to private_grapple_oct6 from clean
`9fcb0bbe610372fc25799c21877129047654c40a`, tree
`795ee80e60dc947b644533d1ec921b8dbd34bb37`. Commit this checkpoint before a
normal `git merge --no-ff` of the complete reviewed private history at
`de6b9c52a6187e2f947e660a9cffbf11615caa3d`, tree
`ea9359d6ba23fdfab90aa2f937538c2cd16296b1`. The common ancestor is
`ade8e93b450d6e02027afc7615a555db16b2915d`. No copying or cherry-picking.

The prerequisite private run completed normally at 2026-10-06T06:39:30Z:
whole formatting check, strict domain/rules/app all-target Clippy and all 82
selected cases in ten harnesses passed. This receiver independently read all
12 complete logs and checked exact names, actual log order, summaries, step
arguments, clean source bindings, runner hashes, all 328 mapped source hashes
and all 597 Git/working files. The immutable evidence report SHA256 is
`2ac4a2c138b933059fb1f9084d9add3cd2a6d487fab489bf36f63e8389ca2b73`.
These passes apply only to private de6; the receiving public union is unrun.

The whole incoming delta has four paths: the private execution plan; one
unchanged-predicate reserved-hand diagnostic in `grapple/admission.rs`; and
the two private execution test files. The three explicitly reviewed test-body
corrections retain the owed AfterEquipment choice after withdrawal, use genuine
shared initiative for two canonical Host Cultists in the concentration fixture,
and assert the actual HP1 knockout/recovery/rest outcome. Receive those complete
files and their complete plan history. Preserve all other original bodies,
helpers and names; preserve every public production/UI correction, all 25 public
application cases and nine frontend cases. Only admission.rs overlaps public
production, with its accepted public behavior retained and the incoming message
composed without changing its predicate or error type.

Review the actual merge delta and report any semantic conflict before resolving
it. Audit exact incoming test blobs and all unaffected public/source bytes,
then freeze the clean receiving head for root review. No formatter, compiler,
test, database, native run, push, wrapper preparation or group 3 receipt is
allocated here. Root must separately authorize fresh receiving verification;
all public, ordinary-replay, group 3, dragging/carrying, canonical/CI and native
obligations remain open.

The plan was committed at `2dbc48ea32d86883e9e23c1e8eec1ce964499506` before
normal merge `b0c7e954888e7855cf09c966fdd993bd512cfc6d`, tree
`6522472e9bab3a955431c69f00365ba22eb34165`. Git merged all four incoming paths
without conflicts or manual source adaptation. The incoming private test files
and private plan are exact de6 blobs; the complete public admission file differs
from 9fcb only by the reserved-hand diagnostic. All other 615 public Git entries
outside these four paths and this receiving plan are exact 9fcb. The 23 reviewed
public production paths, 25 public app cases, nine frontend cases, eight content
files and 28 original inline modules remain exact. Of 894 original Rust bodies,
891 retain exact bytes/counts and only the three explicitly approved corrections
are replaced by their exact verified de6 bodies. All 82 selected bodies match
the private run; this does not transfer that run to the different public source.
Static source and whitespace audits passed. Next: root reviews this complete
receiving freeze and separately allocates fresh public verification. No new
formatter or runtime ran, and no group 3 dependency was received.

### Reviewed formatting freeze (2026-10-06)

Root formatted the changed Rust paths and reported direct rustfmt/check PASS,
then transferred sole writing for a format-only freeze from exact
`3684a3738276eae8218739b5179753b7afb1575d` (tree
`5bd90ae00c1ad1841050074ca221c578b7b5a1fe`). This reviewer read all 15 complete
formatting diffs. Changes are layout, trailing commas, transparent closure/match
blocks and semicolons on diverging statements; literals, comments and all other
tokens are exact. No source behavior or imports changed. The formatting-only
patch SHA256 is `6092fd473b37e061a47060c74278a8b78d156bf2e3cc0c382bfb2a37acde2255`.

The independent stdlib/Git audit preserves all 154 original protected files and
all 8 content files as exact Git bytes, all 28 original inline modules and all 894
original Rust test bodies with occurrence counts, and all 25 new public test
names and meaningful tokens. The complete punctuation changes were also read;
token comparison alone is not the semantic review. No compiler, test, runtime,
dependency intake or publication occurred in this formatting continuation.

The proposed first verification inventory contains 426 Rust cases: the full 25
public app cases, preserved 82 private selection, 22 original replay cases and 297
ordinary attack/movement/turn/table/recovery cases, followed by the normal full
frontend check/test/build. The public branch still has three original private
test bodies superseded on the pending private correction branch; root must
authorize normal whole-history receipt and review the union before preparing its
exact-head runner. No wrapper has been prepared here. Full group 3, canonical/CI,
dragging/carrying and packaged native obligations remain open. Next: root review
of this clean format freeze, then separately allocated receipt and verification.

### Approved identical-source initiative fixture correction (2026-10-06)

Root reviewed frozen `0f6e176a766be15b5944b73285c6c4200e20e92f` and allocated
one fixture-only correction to public_grapple_oct6. This plan amendment is
committed before its source edit. The unchanged initiative validator requires
identical creature sources with identical surprise/circumstances to share one
physical roll. `Fixture::with_opponent_geometry` currently places its two current,
unsurprised Goblins in separate groups, so those genuine producer paths cannot
reach the intended Grapple assertions.

Only that helper may change: when the original source is `goblin-warrior` and the
optional Goblin opponent exists, declare exact groups `[PC]` and
`[goblin, opponent]`, submit the actual PC18 and shared Host2, and resolve the
real source-creature tie by accepted Host `ProposeInitiativeTie` with
`[goblin, opponent]`. Assert the retained groups/request actors, one shared raw,
and actual order/totals `PC20 -> goblin4 -> opponent4`. Keep distinct-source and
no-opponent setup semantics, all25 complete application test bodies, production,
frontend and every protected old body unchanged. Do not change source profiles,
controllers, surprise, map geometry, costs or expected Grapple mechanics.

This is source-only and will remain unformatted/uncompiled/unrun. Root retains
formatter/compiler/runtime/native/publication scheduling and all group3 receipt.
Freeze a clean head with a complete source diff and original-body preservation
report for root review. The full receiving checks and existing Gate4 obligations,
including ordinary dragging/carrying and integrated/native evidence, remain open.

The plan-only amendment was committed at `a5254cb` before the helper edit. The
specified fixture correction and intermediate request/raw/tie assertions are now
authored. No formatter/compiler/test ran; the clean source freeze will carry the
separate stdlib/Git preservation audit, which is not runtime evidence.

Root resumed the already approved correction on 2026-10-06 at plan-first
`10fcc38`, transferring the original writer's complete 23-path production/UI
draft to public_grapple_oct6. The prior writer was stopped; no concurrent source
writer remains. The receiving draft and both complete independent 716 reviews
were read before adding source-only coverage. No private successor or group3
dependency has been received. This continuation is unformatted, uncompiled and
unrun; root retains the sole heavy slot and all publication authority.

The five approved production corrections and precise lint fixes are retained.
The new public test target now has four additional genuine application cases in
`tests/support/table_grapple_public_corrections.rs`: isolated live-grip Finish
refusal with actual pause/attendance/resume/release/replacement; historical
Player-target voluntary save across controller transfer with current-owner new
input; third-actor unarmed OA with a separate outgoing-hand window proof that
survives release; explicit in-range movement and suspended last-grip release
with exact old-request retries. New hostile imports compare every destination
typed cell, including matching current/latest images. Ray coverage explicitly
distinguishes the paid cast origin from the first SelectWork admission and
later advancing commands and adds wrong origin/cast/target/missing-cut cases.
The previous unrun self-only case now sends the explicit action and preserves
its original assertions after rejecting ordinary Move without writes.

Three separate frontend tests in `components/GrappleMovement.test.ts` exercise
the explicit button, absent-field ordinary behavior, and original saved action
after a lost reply. Existing Movement.test.ts is untouched. The new fixture's
third-actor geometry variant is selected only by the new OA case; its default
producer inputs remain unchanged. The common new-fixture rejection helper now
compares all tables/typed rows in addition to the existing export comparison.

The 2026-10-06 pre-freeze stdlib/Git audit passed: all154 protected files,
all28 original inline test modules, all894 original Rust test bodies with their
occurrence counts, and all8 content files are unchanged against ade8e93. The
inventory is25 application cases and9 frontend cases, all UNRUN. `git diff
--check` was clear. These are source-preservation results only; no formatter,
compiler or executable validation ran in this correction continuation.

This is authorship only. The decisive purchased Greatsword menu exclusion,
Glaive refresh/reach, group3 interactions, full runtime/frontend checks and
packaged native evidence remain required. Ordinary dragging/carrying remains
open Gate4 scope. No newly authored scenario is credited as passed.

Root approved the complete independent-review correction plan on 2026-10-05
and transferred sole source ownership back to this writer from clean71673d7.
The [five-finding correction plan](gate4-grapple-public-corrections.md) is
committed before source changes. Source/tests/docs only are allocated; no
formatter, runtime, dependency receipt or publication is allocated. The source
handback will remain uncompiled/unrun for full independent review. Ordinary
dragging/carrying remains an open Gate4 obligation; explicit self-only movement
is an intermediate boundary, never an acceptance waiver. No private418 receipt.

Implementation progress (2026-10-05, source only): the single typed reducer,
owned prepared/applied images, fixed complete-history verifier, authenticated
live/query/replay/open/restore/observation paths, occurrence-bound ray/OA reads,
lifecycle hooks, explicit activation/version3 and desktop opaque choices are
authored. Nested kernel and battlefield mutations use the same fixed candidate.
Twenty-one additive genuine creation/equipment/source file-SQLite application
cases and six desktop cases are authored. They have not been compiled or run;
the complete acceptance matrix and every group3 dependency remain outstanding.
Nothing here is a compilation or feature-acceptance result.

Five root-authorized direct rustfmt passes finished with exit0 on edited files
only (26,34,73,78,78 Rust paths, edition2024, skip_children=true). The first two caught
authoring syntax issues corrected before their successful reruns. The most
recent broader static audit found all154 original test/fixture/frontend control
files, all28 original inline cfg(test) modules, all894 original Rust test bodies
and all8 original content files exact against ade8e93 after line-ending
normalization. Final formatting also completed with exit0 and `git diff --check`
was clear; its repeated protection audit accompanies the frozen source handback.
No Cargo, compiler, project runtime, SQLite or native run occurred.

Root fully reviewed and approved the complete 387-line proposal and 446-line
resolved ownership design, reproduced as repository appendices below/in the
linked design. This allocation supersedes their historical request for source
permission. Root explicitly permits implementation groups 1â€“2 on this sibling
before the frozen ade baseline82 result. Root reports that separate run stopped:
fmt and strict Clippy passed, the corrected-opportunity case passed, and the
owner-ten group had nine passes and one failure at
`typed_pending_withdrawal_preserves_exact_cancel_and_eligible_export`
(`grapple/execution/tests.rs:195`, expected `resolution.is_none()`). Root is
investigating the actual lifecycle; no descendant correction, acceptance waiver
or original test-body change is made here. The full82 did not complete.
Prior lint/compile failures remain recorded in the inherited attack-read plan.
No prior result is a pass for this new branch. Frozen original checkout is untouched.

Root owns heavy scheduling, all compilation/Cargo/npm/database/native execution,
dependency receipt, publication, independent review, CI and exact-head merge.
This writer may edit source/docs/tests only. Request a direct configured-rustfmt
window before formatting. No fetch/merge/dependency intake/push/PR by this writer.
The next implementation commits are intermediate production work, not a new
private checkpoint or a claim that Grapple/Gate4 is complete.

Root AGENTS, product-definition real production play/physical dice/local-first
recovery/continued campaign obligations, Gate04, gate-execution protocol, and
ADRs 009/011/012/025/026/027/029 remain binding. No gate/scope waiver.

## Current implementation groups and complete acceptance

1. Complete six-family occurrence/cut readers and actual holder-break,
   self-only movement/refresh/flight/lifecycle prerequisites. Implement the
   reviewed whole-state owner plus one typed mutation reducer and every live,
   query, restore, historical presentation consumer. Keep activation closed
   until that coherent path supports its real lifecycle waits.
2. Add explicit optional origin-bound Grapple activation and presentation/
   transport version3, opaque owned choices, original-envelope replay, genuine
   PC/Goblin/old-Mage incoming-grip application tests, file-SQLite portability,
   privacy and desktop controls. No positive synthetic source/HP/item/proof edits.
3. Root must later receive whole reviewed Physical creation and Ground/Ogre
   histories normally, reconcile this plan, review the union and finish genuine
   Graze/Glaive reach/LR, source caster-hands revision and Inspiration positives.
   All group3 obligations in the full proposal remain acceptance requirements.
4. Independent exact-head source/test-body review, root-scheduled strict affected
   all-target Clippy/tests, protected captures/replay, frontend check/test/build,
   canonical verify-fast/verify, required CI and packaged native physical-dice,
   save/exit/reopen evidence are still required. Source authorship is not evidence
   of runtime correctness. Only root determines exact-head acceptance/publication.

## Required authority and compatibility implementation

The [resolved ownership design](gate4-grapple-public-completion-design.md) is
approved. Rules owns whole CampaignState in a non-Clone/non-Deserialize
CampaignExecution. Prepared steps keep the original image unchanged; app checks
original TableEvent equality before committing a pure step. Owner-issued borrowed
before/after TableRead values preserve exact historical query authority.
No arbitrary state setter, nonmechanical rebase, mutation callback, ambient bool,
serialized proof or crate dependency exception is permitted.

The single rules mutation reducer covers all17 current outer operations:
UpdateContract, AddPlayer, CreateCharacter, PrepareEquipment, CreateCreature,
EnableSourceActorAccess, SetSourceCreatureController, Tactical,
PrepareBattlefield, StartSession, EndSession, SetSituation, Declare, Correct,
CancelDecision, Adjudicate, SubmitPhysical; plus explicit EnableGrappleAccess.
App keeps original text interpretation, TableAction/TableEvent composition and
historical equality. No rules->conversation/app/persistence dependency. Existing
pure creation/setup input types are re-exported unchanged; pure session deltas
convert to the unchanged persistence SessionChange.

One complete original-anchor walk owns semantic, every-snapshot equality/full
validation, audit/command/causal joins, sessions, presentation/bootstrap/revisions,
capabilities and request bindings. No latest-snapshot shortcut. Preserve full
staged-export revalidation before every SQL commit and preflight before restore.
Every consumer listed in the design must use the owned read path. Existing
RunnableCampaign remains a read DTO, never a cache of write authority.

## Implementation details recorded during source work

### Open and resume must select history from the full export

Independent review found `open_campaign` chose original replay only when the
mutable current state still contained a Grapple marker or grip records. A
genuine just-activated journal with its current marker removed could therefore
return an ordinary Runnable DTO without checking the retained activation event.
Command, presented-query and restore consumers already authenticate full history;
this finding is a read-path defect, not an observed write-authority bypass.
Root approved this correction before source: open a consistent portable export,
use the existing `has_rules_history` detector across its current state, original
snapshots, audits, events and observations, and authenticate that original history
before returning a table/rules campaign. Construct the returned lifecycle/state
from that same export. Preserve ordinary non-rules catalog/validation behavior.
Add genuine accepted activation then hostile SQL current-marker and matching
snapshot removal controls. Rejected open/resume must preserve every typed cell
in every table; hostile portable restore retains its independent destination
control. These negative mutations are never positive gameplay setup.

### Preserved TableView transcript privacy correction

Independent preliminary review found that `CampaignRuntime::table_view`
authenticated complete original history but then discarded its historical
visibility and passed `visible_events=None` to the preserved DTO projection.
That suppresses the Player transcript filter and becomes reachable with enabled
Grapple through the new owned read. Root approved this narrow correction before
source: retain the authenticated PresentationHistory and call its existing
`raw_view` projection helper (made crate-visible), which uses accepted historical
event and observation audiences. Do not recompute visibility from current sight,
alter Host transcript content, change saved projections or amend old test bodies.
Add a genuine private attempt/save/live/release test comparing the unrelated
Player DTO transcript/recap with its pre-action value and the presented route;
the Host must still retain the accepted events. This is an unrun privacy repair,
not proof of full boundary acceptance.

### Approved marker-specific empty read shape boundary

Before the following source change, root read the full-state caller, `ids()`,
the full cut/inheritance/end validator and actual modern producers and approved
this bounded amendment. The legacy public
`TacticalGrappleResolution::validate_shape(resolution, live)` and `ids()` retain
their nonempty behavior and every original body/control. One private validator
implementation receives a closed enum selecting legacy or modern local shape;
the crate-internal modern entry is called only by
`validate_tactical_grapple_shapes(CampaignState)` after explicitly validating
that state's `TableGrappleAccess`. There is no externally supplied permission
boolean and no new rules authority constructor.

The modern entry permits an empty `AttackAdmission` or `RequestIssue` list:
these are real occurrence-bound reads proving there was no relevant grip at
admission/issuance. `FlightLoss` still requires a nonempty list with a retained
proof supporting that target. Nonempty lists always use unchanged `ids()`.
Every duplicate/key/ancestry/source/chronology/retained-proof check remains shared.
This validates shape only; the nonserializable rules owner still reconstructs
the original producers, exact complete cuts, all requests and every snapshot.
Missing/forged marker or cut cannot create accepted authority, and raw ordinary
execution remains closed even with a structurally valid marker.

Caller inventory: domain full-state validation reaches this function through
`CampaignState` validation/codec; rules validation and the complete app restore
walk validate that same state but additionally require their owned original
history. The legacy direct resolution validator remains available to all old
tests/consumers with its original stricter behavior. No ordinary consumer may
directly select the crate-internal modern entry. In continuations, the actual
pending work is installed first, the exact raw `PendingRoll` is installed next,
and only then does `capture_issue` record/validate its occurrence read. This
ordering is required by modern cut validation and is retained.

Additive acceptance cases must use actual activated no-grip attacks, same-cast
multiple rays and a real OA; negative controls cover markerless empty reads,
empty FlightLoss, omitted/forged cut/marker imports and raw execution. Existing
synthetic/private validator controls and their test bodies remain untouched.

### Actual completed encounter retirement

The existing Establish transition can retire an already finished flow. The
owned execution observer records the exact prior finished encounter/completion,
replacement origin and unchanged accepted raw history at that actual reducer
site. Its delta check permits only that observed old-flow decision retirement;
no generic state reset or serialized evidence grants permission. Original
accepted journal replay still proves the retired decisions after the new flow
is established. New live grips and raw/cancellation suffixes remain independently
checked. This source is uncompiled and requires the full original-history tests.

### Paused Knockout and Graze completion settlement finding

Static review of `attacks::choice_scope` found that its actual re-entered
AttackDamage/FinishAttack parent ran the paid consequence, then checked live
grip constraints before ending a holder incapacitated by that consequence.
This would reject the legitimate accepted Knockout/Graze transition. The ordinary
continuation start/finish sites already settle that same effect before checking
constraints and queuing falls. Add the same owned `settle_work` call inside this
existing exact-parent scope, before `check_live_constraints` and `queue_losses`.
Do not create a work node, a replacement state, a callback authority or permission
for an unowned context. The hook is a no-op for all legacy ordinary/private paths.
The real Goblin critical-hit and damage-choice app case must prove holder
incapacity ends the grip while preserving issued/accepted dice. Genuine Graze
acquisition remains a required later whole-dependency positive. Preserve every
original attack/private test body. No execution result is claimed.

The writer initially added this one hook under the already approved lifecycle
scope before this specific finding note. On root's explicit reminder it was
withdrawn, this note recorded, and only then reapplied; no runtime occurred in
between. Future concrete boundary amendments remain plan-first.

The nonserializable TableOperation owns its ordinary input values and is borrowed
as `&TableOperation` for execution, rather than borrowing every individual field.
This keeps the relocated mutation match mechanically identical and ensures a
parsed declaration proposal cannot borrow the owner across its mutable step.
It is a typed proposal only: no state, capability, derived mechanical total or
serde implementation is added. Command/event payloads remain the original app
owned types. Historical mode is represented by `Some(ExpectedNested)` even when
both child events are absent, preserving old source-creation replay semantics.

## Original control and migration inventory before code

Preserve the exact original bodies in grapple/execution/tests.rs (10 owner) and
execution/attack_tests.rs (15 attack), every inherited Grapple core/condition/
hands/lifecycle test, source coexistence capture, all raw source fixtures and
legacy v1/v2 request/projection/history guards. Preserve old raw-state public
Grapple refusal: the modern route is original-history owned, not raw resolve.
Preserve GuardedGrappleExecution::new's private ordinary-baseline meaning and
its original bounded profile for those controls; do not silently reinterpret
old82 as a newly broadened suite. Modern full-profile execution is separately
admitted by explicit accepted activation and complete replay.

No original test body modification is preauthorized by this initial inventory.
The optional marker uses TableState's existing constructor; no existing test
must acquire a forged marker. The old source-control seven-command denial still
applies before modern activation. Add new producer positives and old-flow/
missing/malformed/foreign/refreshed-handle negatives. If an actual old temporary
staging assertion is necessarily superseded, first record its exact file/name/
assertion and paired replacement evidence here before changing only that assertion.
Never remove unrelated assertions or weaken old import/raw/history guards.

## Durable progress and immediate next action

- Root design approved; exact sibling parent/tree and clean status verified.
- Original frozen baseline82: root reported a stopped owner-group failure above. New
  groups1-2 source/UI and additive tests are authored but UNCOMPILED/UNRUN.
- No runtime, database or compiler execution allocated or performed here.
- Initial plan/design committed as `c096f31` before implementation. Preliminary
  independent bridge review found two read-routing gaps, both corrected as
  documented above and independently re-read as statically clear. Scoped final
  rustfmt completed. Next: commit the coherent source, repeat the exact-head
  byte/body audit and hand it to root as unrun for full independent review.
  Root must review the complete frozen diff and actual test bodies before any
  execution/publication. No result from another head verifies this source.
- Risks: every retained-history reader must keep its proof; finished/replaced
  encounter history cannot require an active flow; source control changes cannot
  rewrite historical source identity; read ownership must not alter old digests.

## Authored application and desktop evidence inventory (all UNRUN)

`crates/dmd-app/tests/table_grapple_public.rs` is a new target; no original test
file or body is edited. It creates genuine characters, purchased physical items,
source creatures from the current full-pin catalog, sessions, host-authored
battlefields and physical initiative. `cold` exports, independently restores,
closes/reopens the file and executes the same original next request on both;
it compares complete canonical state/outcome, exact local retries and full rules
replay. Independent random audience handles/revisions are compared within their
own stores. Hostile restore compares every table, row and typed cell (including
hex bytes for text/BLOB) against an unrelated genuine destination campaign.

The 21 authored test names are:

- `activation_uses_original_history_and_refuses_legacy_raw_foreign_and_unsettled_input`
- `real_grapple_save_release_and_after_equipment_cold_replay_keep_exact_raw_and_private_handles`
- `withdraw_cancels_only_its_pending_request_and_never_refunds_the_paid_attack`
- `forged_activation_snapshot_audit_and_original_anchor_leave_every_destination_row_unchanged`
- `activated_no_grip_attack_records_real_empty_reads_and_refuses_markerless_or_flight_substitution`
- `activation_rejects_an_actual_pending_attack_then_accepts_the_settled_same_encounter`
- `activated_unarmed_opportunity_keeps_empty_occurrence_read_and_spends_one_reaction`
- `activated_source_three_rays_keep_distinct_admission_ancestry_and_raw_ids_across_cold_replay`
- `real_goblin_escape_checks_use_current_source_skills_and_off_turn_release_cancels_only_escape`
- `player_owned_goblin_genuine_save_and_after_equipment_retry_survive_controller_transfer`
- `current_old_mage_accepts_incoming_pc_grip_but_never_gains_a_grappling_anatomy_grant`
- `actual_escape_success_and_failure_keep_paid_action_and_physical_evidence`
- `self_only_move_retains_grip_until_actual_range_crossing_and_never_moves_the_target`
- `actual_source_critical_knockout_ends_the_incapacitated_holders_grip_after_damage_choice`
- `actual_chimera_flight_loss_keeps_its_fall_and_issued_dice_after_owner_release`
- `unrelated_audience_whole_projection_stays_exact_through_private_attempt_save_and_release`
- `pc_physical_attack_keeps_admitted_hand_read_after_release_before_the_reported_attack`
- `genuine_intrinsic_attack_retains_the_target_relation_after_off_turn_owner_release`
- `activation_codec_preserves_omission_and_null_but_rejects_old_future_duplicate_and_unknown_authority`
- `actual_after_equip_then_before_stow_support_two_private_hands_without_fabricated_items`
- `open_and_resume_replay_original_activation_when_current_and_latest_marker_are_removed`

The six new frontend cases in `grapple-transport.test.ts` and
`components/GrapplePanel.test.ts` cover opaque source retries after lost reply,
v3 Host activation plus old v1/v2 retries, source administration remaining v3,
selected-actor controls/opaque identity, pending/expired controls and Host-only
settled-encounter activation. UI wiring uses the ordinary saved-request workflow.
None is native or runtime evidence.

Remaining acceptance is explicit: root's full source review and exact-head
runtime/strict lint/old-history/frontend checks; actual finite all-form Ogre,
genuine physical Graze/Glaive OA and source caster-hands dependency receipt;
full retained Shield/concentration/Savage/LR/Inspiration, target-only break,
refreshed unselected OA and all broader matrix intersections from the retained
contract; artifact/native physical dice and save/exit/reopen. Authored common
hooks or a passing subset may not be substituted for these required positives.
The original 82 private controls retain their original meaning and exact bodies.

## Approved full completion contract (historical proposal)

The following retained proposal is the complete acceptance contract. Its earlier
wording requesting allocation is historical; the allocation above is current.

# Proposed complete ordinary Grapple production path

Status: EXTERNAL PLAN ONLY, 2026-10-05. This is a proposal for root design review,
not an accepted execution plan, source allocation, test result or activation.
No source/branch/worktree/dependency/publication changes and no Cargo/compiler,
npm, database, formatter or native execution occurred during this mapping.

## Exact input and objective

Locally verified clean frozen source:
`C:/Users/jadra/Documents/ChatGPT/DMD/gate4-grapple-attack-read-context`,
`ade8e93b450d6e02027afc7615a555db16b2915d`, tree
`a6097d485244a2ed5443765fb4ee40c34625892b`.
Root has reviewed its focused82 source/runner; that receiving-head run remains
queued and UNRUN. Earlier failed0df/58b lint/compile attempts are retained evidence,
not passes. This memo does not repeat their independent review.

The complete earlier map was read and its hash recomputed:
`tooling/grapple-0df1754-next-completion-read-only-map-2026-10-05.md`, SHA256
`e539c6c4ecd79f57f830cecb7fa6dbd73ae2f452cee0dff1cd6d0d199b260ef2`.
Current source confirms its unfinished seams. The two later corrections and core
fixture receipt do not supply public Grapple or alter that completion boundary.

Recommended next implementation objective: **complete ordinary source-backed
Grapple, Escape, free release and their existing tactical interactions through
accepted application commands, opaque owned controls, actual file-SQLite recovery
and packaged play**. Bounded commits may support review, but a new private-only
producer checkpoint is not the feature's completion boundary.

If allocated, use a fresh root-created sibling from exactade8 after root reviews
this design and receiving82 evidence. Commit the reconciled repository plan
before source, name one writer, retain ade8 frozen. Root owns dependency receipt,
heavy scheduling, exact-head CI/publication/merge and native verification.

Authority read: root AGENTS; product definition's real-application, physical-dice,
local-first/restart and feature-completion requirements; Gate04 tactical encounter
acceptance; ADR025 spatial/compatibility, ADR026 shared resolution, ADR027 immutable
audience/transport history, ADR029 immutable creature source admission; full Grapple
lifecycle and its reviewed follow-on/condition/core contracts. Existing dragging,
special body parts, mutual-player consent and wider reaction/Ready replacements
remain named Gate4 obligations. They are not silently moved to Gate5 or represented
by this ordinary own-turn slice. The existing self-only choice is explicit.

## What the frozen source actually supplies

| Prerequisite | Present at ade8 | Completion work that remains |
| --- | --- | --- |
| Source/anatomy | Strict Human reconstruction; immutable GoblinWarriorV2 ordinary hands; old Goblin, raw captures, pins and physical Items retained | Real current table Goblin ownership/coexistence; normal Ogre/physical-creation receipt; reviewed caster anatomy positive |
| Paid core | Seven typed Grapple actions, actual Begin/Save/AfterEquipment/Escape work, role19/20, exact raw/automatic/voluntary/LR structures; Attempt hand reservation and own-Attempt exclusion | Public accepted command authority and complete original replay, actual LR/Inspiration producers |
| Current hands | EffectiveHands combines actual physical slots and live/provisional reservation without fake ItemIds | Every current/sealed hand caller in newly admitted spell/OA/movement/after-equipment paths |
| Condition lifecycle | Live source-specific Grappled projection; speed zero; actual establishment ends Dodge; final save validation retains accepted issue facts | Causal incapacitated/dead/range ends; genuine flight loss and last-consumer retirement |
| Continuity owner | GuardedGrappleExecution owns one validated ordinary baseline, immutable predecessor, candidate-bound ExecutionContext and actual raw/decision/cancellation observations | Safe reconstruction from original semantic replay and continued ownership across the real table pipeline |
| Attack reads | Four own-turn families and direct requests/children; actual Goblin source damage, Shield, Savage, Graze/Knockout under the bounded private owner | Spell rays, selected/unanswered OA, Multiattack occurrence ancestry, all suspended consumer classes |
| Public boundary | Both public policies, raw history, restore anchors and source-control entry deny new Grapple authority | Deliberate activation after every prerequisite below, while retaining unauthenticated/old-executor refusal |

Exact current limits are visible in `tactical/grapple.rs::require_execution`
(requires EncounterReleaseV1/flow5), `guard_action`, `execution::admit`,
`admission::supported_context/live_constraints`, and `profile::validate`.
The old lifecycle document's earlier flow4 language is superseded by the current
core's permanent flow5 requirement. Do not allocate or reuse flow6. Offstage7 is
not in this source; later compatibility is an explicit root-owned normal intake,
not `version >= 5` admission.

## Decisions proposed for root review before source

### 1. Replay-bound execution authority

Do not turn the private owner into a constructor accepting a current imported
grip image. Do not add an ambient boolean, caller-selected policy, stripped clone,
generic state setter, lifetime hash ledger, or an API granting chosen cut masks.
Keep the single tactical dispatcher, common validation graph and ordered actual
producer observations. The public raw `resolve_tactical/replay_tactical` APIs must
continue refusing a new-authority image without the complete continuity proof.

Recommended bridge: an owned, non-Clone/non-Deserialize rules execution context,
constructed only from the normally validated no-Grapple predecessor reached by
the application's original-anchor replay, and a crate-private application
`AuthenticatedTableHistory` holding the reconstructed head and that context.
`visit_rules_history` becomes the single constructor of the application value;
the live writer consumes that exact value at its canonical campaign/head. Reads,
queries, presentation and final candidate validation borrow its authority rather
than recreate it from JSON. Snapshot/raw current images never construct it.

The rules context already owns CampaignState. The bridge must address ordinary
table commands after first Grapple explicitly: do not export then re-import an
owned state after each accepted event. Extend the owner with narrow production
operations for its actual Rules/Tactical producers and explicitly enumerated
nonmechanical table-context changes; never accept an arbitrary replacement image.
The pure table reducer remains the sole owner of outer session/membership/channel
validation, and its actual accepted transition supplies those typed changes.
Examples requiring an exhaustive match audit are session binding, table intent/
situation, source controller, supported character/source creation/equipment,
new battlefield and table contract changes. Mechanical changes must call their
real rules producers; source-control changes cannot use a context update to alter
a live grip or paid work. Unhandled changes fail before mutation. These operations
must compare exact predecessor/campaign/head, retain all prior raw/decisions/cancel
entries and validate the entire resulting context. A generic callback allowed to
edit CampaignState would recreate the forbidden setter and is unacceptable.

This API ownership split is a design review point: verify the exhaustive typed
bridge is smaller and safer than relocating the pure composed reducer before
committing to signatures. Do not solve the crate-cycle by moving app/persistence
dependencies into rules or by exposing unrestricted guarded constructors. The
first source review must show every caller and constructor, including current
read queries and both replay policies. No activation can precede that proof.

`rules_restore::visit_rules_history` currently calls `validate_table` on current
and every snapshot before replay. Split structural/source-safe image checks from
authority-bearing semantic checks: decode/shape/source/identity first, reject any
Grapple authority in the original anchor, then replay from it and compare every
snapshot/current image to the authenticated result. Replay-derived validation
must replace the guarded semantic checks at those exact sites; it must not omit
them. Preserve observations, audit/event joins, envelope-child equality, session
ledger, presentation-history visiting and final whole-state equality.

`command_origins` must enumerate every declaration, finalizer, save decision/LR,
equipment choice, end cause, issue cut, refresh, self-only origin and flight-loss
cause. Enumeration helps joins but never substitutes for replay. Standalone raw
Rules/Tactical events inside a table campaign remain rejected. The normal live
writer must use the reconstructed capability from `protocol_pack`/history;
accepted saved-binding lookup retains precedence over current ownership,
attendance, action admission and audience revision. The existing final staged
export/replay before transaction commit remains mandatory.

### 2. Explicit presentation activation, preserving old menus

Initial Grapple options cannot simply be appended to historical v1/v2 projection
menus: they would alter existing digests even before any grip. Proposed concrete
boundary: optional omitted `TableGrappleAccessV1` with the exact activation
CommandMeta, a Host-only `EnableGrappleAccess` action at a settled flow5 boundary,
and additive presentation/transport version3. Do not conflate this marker with
anatomy, source ownership, consent, a flow upgrade or replay authority.

On activation, produce the new projection/capability set through the normal
ledger. Preserve v1/v2 projectors and recorded versions byte-for-byte. Existing
v1/v2 accepted requests still retry exactly after activation. Fresh v3 requests
require the actual marker; malformed versions do not fall back. SourceCreature
channels additionally require the existing source-access activation/owner checks.
Normal v3 action audit markers become table.action@4 by the existing version+1
convention; update supported-version joins deliberately, including observations,
portable validation, persistence capability records and UI outbox decoding.
No SQL table replacement or export-format change is presumed necessary; confirm
the current format3's strict typed version checks refuse unsupported records.

This additive version/marker is proposed, not allocated by this memo. Root must
confirm no concurrent assignment before the checked-in plan. An alternate current
options-query design is acceptable only if it supplies genuinely durable opaque
handles without rewriting existing projections, a mutable query-side ledger or
an unauthenticated client cache. Do not invent a deterministic public grip ID
encoding as an opaque capability.

## Occurrence-specific and causal rules completion

1. Refactor `grapple/reads.rs::{capture_admission,capture_issue,attack_root,
   validate_cuts}` around the exact AttackRoll work occurrence and causal ancestry.
   Current single parentless root/resolution.origin assumptions are insufficient.
   Preserve complete relevant incoming-attacker/incoming-target/outgoing-physical
   grip membership, unique cuts and actual issuing CommandMeta. Reuse existing
   GrappleCutKey/work fields; add serialized fields only if an actual ambiguity
   cannot be represented and document its schema consequence first.
2. Spell `attacks/spell.rs::begin_spell_attack/fill_facts/validate_admission`:
   the advancing command is attack.origin while raw origin is casting_origin.
   Each real ray gets a fresh admission at its own AttackRoll occurrence; only
   its request and descendants inherit it. Later rays read current conditions.
   Preserve the actual paid BoundSpell, targets, source program, components and
   material choice. No per-cast blanket cut or invented focus after release.
3. OA `attacks/opportunity.rs::validate_admission`: reconstruct the actual paid
   Reaction/crossing/equipment before-image using that selected attack's sealed
   hand context. Keep recorded answer and source equality. Weapon/unarmed/intrinsic
   OA are supported; printed Actions/spells are not thereby OA sources. No new
   equip/pickup/Attack allowance is granted by a Reaction.
4. Unanswered `movement::{options,validate_opportunity,advance_segment}` keeps
   exact current options. Real endings record GrappleOpportunityRefresh with
   original work/window/movement/step/actor and exact prior/resulting vectors.
   Preserve existing order, unanswered ownership, answered Attack/Declined and
   later rescan of prior automatic Unavailable. Do not auto-answer or freeze the
   actor roster. Source Multiattack/ordinary repeated attacks need their own
   occurrence admission, not action-name whitelist expansion.
5. Add one causal reconciliation helper, called after actual vitality/effect/
   position commits and before subsequent reads/work. End outgoing grips on
   actual Incapacitated or direct death even when both physical slots are empty;
   keep target-only incapacity/death occupied by a living holder. Keep dead-target
   Escape forbidden. `tactical_vitality_adapter::drop_held` cannot own this, since
   its empty-Item early return is legitimate and must not suppress grip cleanup.
6. After each committed movement/Push/landing position, end only relations beyond
   their established body range, never recost prior segments or move the other
   body. A route leaving and re-entering already broke the grip. Blocked Push and
   pre-departure OA keep their existing semantics and private geometry decisions.
7. Add explicit `MoveSelfOnly { path }` consuming existing GrappleSelfOnlyAdmission.
   Outgoing holders choose it even for an in-range path. Preserve original route,
   prefix/cost/cursor/actor and stationary target if the last grip is released
   while an OA waits. Old Move for actors without new grip authority is unchanged.
   Dragging is neither silently inferred nor implemented by this action.
8. Establishment on the actual airborne non-Hover Chimera produces the existing
   proposed GrappleFlightLost cause and FlightLoss cut under its establishing
   work. Complete `falling/validation.rs`'s currently refused new branch with
   exact source/geometry/work proof. Keep old FlightLost validation unchanged.
   Release before landing cannot erase/restart an initiated fall or child dice.
9. Thread candidate-bound reads through casting, area, missile, fall, effect,
   concentration, hit coordination, simultaneous work and every actual pending
   validator. Already issued source/request facts remain; new work uses current
   facts. Only obsolete own Escape/owned withdrawal cancels its unfinished raw;
   unrelated waits are byte-identical. Preserve accepted faces, source LR choice,
   direct-choice parent and last-consumer proof validation before retirement.
10. Integrate whole-campaign Finish/replacement/source-transfer checks. Never drop
    a grip to finish an encounter. The proposed default is explicit release or a
    genuine causal ending before Finished/replacement; preserve live grip state
    during valid aftermath/session pause and require every actual controller on
    resume. Audit current settled() and retained_encounter_dependencies: neither
    currently enumerates live new grips, and their old no-authority path stays
    unchanged. Raw-only past authority must remain replayable after all grips end.

## Genuine source producers and normal dependency intake

| Positive | Concrete producer and receipt requirement |
| --- | --- |
| Standard PC/Goblin | Accepted Human creation, PrepareEquipment, current GoblinV2 full-pin creation, exact finite gear, source-access activation and actual controller assignment. Use empty hands by lawful before/after actions, never loadout setters. Preserve original GoblinV1 capture and new/old coexistence. |
| Inspiration | Add a legitimate explicit Host award with a bounded source ruling and actual intended recipient, before the pending Attempt/Escape. Reuse/extract the existing source grant rules without relaxing the kernel tactical-inventory guard. The table/tactical command records provenance and enforces no stacking/eligible recipient. No synthetic flags, GrantInspiration raw API into a table, or rest state injection. Validate actual accepted original_result, reroll faces and once-only consumption. |
| Graze/Glaive reach | Normally receive root-verified Physical creation PR62 (currently authored e75/tree0169, unrun at preparation), including source pin, complete creation/restore/UI changes, not selected catalog files. Actual current purchaseâ†’one-time materialization supplies Greatsword/Glaive; no purchased-definition/mastery/paid-state injection. Parent sourcecatalog's original16 and old actions remain exact. |
| Size-legal LR | Normally receive root-verified Ground/Ogre production union (PR61 lineage includes exact3387 Ogre source). The earlier registry-only dependency by itself is not real creation admission. Actual Large Ogre creation and ordinary_hands can attempt the actual Huge Adult Red Dragon within the one-size bound. Unequip lawful gear for the chosen hand; use true source resistance counter/owner, accept/decline/exhaustion; Escape must never receive LR. Keep Human/Goblin oversized refusal. |
| Caster maintaining an outgoing grip | Existing Mage source page305 lacks ordinary_hands. Plan a narrowly reviewed immutable Mage revision with current exact source/gear/spells unchanged apart from accepted typed anatomy normalization; preserve old Mage pin/captures and installed manifest identity. Do not infer two hands from Humanoid, a staff slot or spell label. Root/source review must approve the original pinned-SRD evidence and interpretation before that asset is authored/admitted. Actual caster create/control/materializationâ†’gripâ†’S/M spell/Shield sequence is required. Incoming Grappled on old Mage already supplies spell attack coverage but does not prove outgoing-hand components. |
| Fall | Supported Human on real platform beside actual Large Chimera (source p273), legal airborne placement accepted by PrepareBattlefield. Establish by actual failed save; preserve Air's actual paid immune no-effect. No size/immunity/HP/condition changes in fixture setup. |

Dependency receipt is plan-first, whole-history normal merge after root separately
clears the exact incoming head; reconcile active plans and full diff, preserve
source/raw histories, then reverify the receiving union. Do not cherry-pick tests
or files, assume earlier branch green applies, or import during this read-only
allocation. Physical creation can support acquisition without Ground; the Ground
receipt is separately needed for its actual BeforeAttack custody and selected
AfterAttack compatibility, all three Ogre forms/three individual Javelins and
ordinary other-actor/same-Item semantics. Avoid duplicate schema/handle designs.

## Application, opaque controls and desktop

Add a focused `table_grapple.rs` projection/options layer using the same actual
admission queries, not copied hand/size arithmetic. New durable capabilities bind
audience + actor + target/hand option or exact grip/work/stage. Proposed variants:
GrappleStart, GrappleSaveChoice, GrappleAfterEquipment, GrappleWithdraw,
GrappleEscapeChoice, GrappleRelease. The capability stores canonical data only
inside persistence; the presented DTO carries random handles and allowed source
choices. Hand/ability/skill enums are ordinary user choices, not provenance input.
Target handles should reveal only the actor's already-located legal contacts;
own release remains reachable even after the target is no longer visible.

Add `TableTransportInput::GrappleDecision { handle, decision }` and reject raw
canonical Grapple/grip/work input through Action. Derive the canonical command
from the owned exact live handle, then recheck real rules/session/controller.
Use existing opaque Roll capabilities for physical dice/Inspiration and existing
Work capabilities for permitted ordering. Source actor, grappler, roller, turn
actor and issuer are independently resolved. Host channel cannot substitute for
a player-owned source; autonomous actors do not become Host-owned. Preserve the
existing pairwise harmful-attempt consent refusal, including same-player PC/source
pairs; release/Escape does not rerun new-harm consent.

Likely paths: `table_protocol.rs`, `table_grapple.rs`, `table_tactical.rs`,
`table_tactical_choices.rs`, `table_attacks.rs`, `table_movement.rs`,
`table_equipment.rs`, `table_source_control.rs`, `table_engine.rs`,
`table_transport.rs`, `table_transport_runtime.rs`, `table_projection.rs`,
`table_presentation_history.rs`, `rules_restore.rs`; persistence
`table_projection_store.rs` and schema/export preflight as actually required.
Desktop: `table-api.ts`, `TableApp.svelte`, Table controls and new Grapple form,
MovementForm's explicit self-only choice; use existing focus/remount and durable
outbox, not a second pending-request store. Add the precise desktop command adapter
only if new queries are needed; no database logic in renderer or rules.

Owned controls must coexist during other actors' roll/non-roll waits. A new
actor/stage/key resets draft dice, selection and focus; same surviving request
keeps its exact request and original faces. Uncertain acknowledgement and app
restart retry the original envelope/handle, never new source/nonce/head. UI labels
are ordinary gameplay, with source IDs, canonical UUIDs, sequence/cut ordinals,
private DC/HP/other actors' hands absent. Hidden-only release and source choices
must leave unrelated DTOs, revisions, transcripts and errors indistinguishable.

## Acceptance contract: authored tests must use actual producers

New app cases belong under `tests/support/table_grapple_*` with one reusable
file-SQLite/portable driver and small coherent scenario files. Every positive
starts by normal campaign/players/source creation/equipment/access/scene/initiative;
then accepted commands create conditions, grips, movement, raw and aftermath.
No SQL/state replacement, manually constructed grip/cut, paid budget, inserted
Item, size/HP/feature edit or authored "expected accepted" event is a producer.
Keep synthetic pure controls explicitly labeled and byte-protect their bodies.

Required integrated matrices:

- Both PC/Host-source directions, independently assigned source player, Host/Host,
  and before-cost foreign/absent/autonomous/PvP/unknown anatomy/size/contact refusal.
  STR/DEX save choice, real automatic/voluntary failure, source proficiency, armor,
  Dodge/cover/conditions and paid immune Air no-effect; two hands/two targets and
  multiple incoming sources; exact withdrawal before/after request and no refund.
- Both Escape skills and correct source modifiers versus established DC, success/
  failure, one paid Action, real Inspiration, no save proficiency/LR/fail-save.
  Holder off-turn release during Escape cancels only obsolete pending work and
  retains paid Action and any accepted original faces. Real LR accept/decline/
  exhaustion and withdrawal-after-accepted-evidence through the genuine Ogre route.
- All six attack families with admission/request/accepted-hit/damage boundaries;
  actual Goblin Prone+Grappled cancellation retains Normal/base damage after release,
  later attack gets current Advantage/printed extra damage. Per-ray and per-routine
  cuts reject sibling substitution; genuine Graze/decline, Shield/material and
  concentration child retains cause after ancestor attack record retires.
- Real Glaive acquired from creation: occupied hand initially removes its two-hand
  reach option, lawful release refreshes an unanswered menu, previously automatic
  Unavailable participant is reconsidered before crossing, no automatic choice.
  Selected OA release preserves actual source, crossing and one paid Reaction.
  Verify unarmed/intrinsic alternatives and no pickup/equipment allowance.
- Itemless and armed outgoing holder incapacity/direct death via actual source
  spell/damage, target-only death/incapacity preservation, exact range boundary,
  one-body clear/blocked Push, route leave/reenter, explicit self-only suspended
  through OA and last release; no second body movement or prefix refund.
- Actual gripâ†’Chimera flight lossâ†’release before landingâ†’real landing/damage and
  any reached concentration children. Keep exact geometry/ancestry/issued dice;
  zero duplicate fall and old FlightLost controls unchanged.
- Cold close/reopen and independent portable restore before save choice/raw,
  live grip, Escape, after-equipment, accepted hit/Shield, each ray/OA decision,
  raw concentration, liquid/fall and valid session pause. Execute identical next
  request on both, compare complete authoritative state and outcome; retry accepted
  requests after later head/controller/session changes and prove exact response.
  A fresh former-controller request is independently refused without writes.
- Hostile import/current+matching-latest snapshot mutations: coherent fake source/
  anatomy/DC/grip, omitted extra relevant cut, wrong same-cast ray/parent, forged
  accepted raw/LR/cancellation/end/current owner, refreshed vector/flight cut,
  altered equipment/window/cost or original anchor. Destination contains an
  independently created real campaign; compare every table/typed cell/row before
  and after rejection. Counts or only current-state equality are insufficient.
- Whole unrelated presented DTO/history equality across 0/1/2 private grip/decision
  cases at equivalent public world state, normalized only independent random
  identities where strictly needed; same-campaign hidden release preserves exact
  revision. No private counts/UUIDs/error detail. Replay those projections too.
- All old receiving harnesses/captures, raw requests/roles/UUID domains, old source
  pins/16-row creation, paid receipts, generic legacy Grappled effects and old
  FlightLost cases exact. Missing/null optional authority preserves old bytes;
  old schema/executor, unknown/duplicate inputs and original-anchor authority
  continue to reject future semantics rather than silently ignore them.

Temporary closure controls require explicit plan migration, not deletion. Keep
the existing raw-state public/import-negative tests intact by using the separate
replay-bound public route. Where the direct private bounded profile refusals
(holder break, spell/OA/movement/fall) or source-control blanket seven-command
denial are intentionally superseded, enumerate exact affected names/assertions
before edits and pair real accepted-producer positives with unauthenticated,
foreign, stale, old-flow, missing/malformed and full-store refusal negatives.
Preserve original synthetic helpers and every unrelated assertion. Do not
reinterpret all82 historical cases as "the new suite" or erase earlier results.

## Manageable implementation sequence and release evidence

One active completion objective, with four reviewable implementation groups:

1. **Occurrence/lifecycle plus authority design**: complete six-family exact work,
   source breaks, self-only/refresh/flight and private context through all actual
   consumers; implement the narrow replay-owned bridge with source review of every
   constructor/caller. Keep public activation closed until this is coherent. These
   are intermediate commits, not another standalone private completion gate.
2. **Real accepted path**: versioned activation, opaque owned choices, source control,
   original-anchor replay and desktop, together with genuine PC/Goblin/old-Mage
   incoming-grip file-SQLite scenarios and all private/no-authority controls.
   Do not advertise a partial public activation missing required lifecycle waits.
3. **Actual positive dependencies**: plan-first normal physical-creation and
   Ground/Ogre receipt when root clears them, explicit source-reviewed caster
   revision, genuine Inspiration; finish all remaining Graze/reach/LR/material
   cases and combined before/after equipment/history invariants.
4. **Verification and native**: full exact-head independent diff/test-body review;
   root-scheduled default-stack focused rules/app/persistence plus strict affected
   all-target Clippy, all protected replay suites, frontend check/test/build,
   canonical verify-fast/verify and required exact-head CI. Inspect actual full
   logs/names and receiving union; fresh head after each fix. Package exact sources
   and record artifact hashes. Real desktop PC/source Grapple/save/Escape/off-turn
   release, source LR/Graze/Glaive refresh and landing, physical dice, save/exit/
   reopen must use that package. Automated widgets/SQLite tests do not substitute
   for native evidence. Merge only after root's exact-head requirements are met.

For a first allocation, authorize groups1â€“2 as one coherent writer assignment with
the full contract above retained and group3 dependencies explicitly waiting on
root verification. The early draft-CI amendment above permits publication for
verification; do not accept or merge until all required positive cases and native
evidence are complete. If the replay bridge or v3 boundary needs a broader
shared refactor than mapped here, resolve that concrete design before source,
without weakening guards or redefining the completion boundary.

Also update actual spatial witness/material-step accounting and memory/work
reservation for new relations, refreshes and falls. Existing candidate counts are
source-qualified estimates, not a global cap or a measured performance result.
All-observer samples remain at genuine material transitions.

Exact next action: root reviews this external proposal, especially the typed
continuity bridge, explicit projection3 activation and source-positive dependencies;
then reconciles/commits the approved plan on its selected fresh sibling and assigns
one source writer. No new gate is started. Gate4 remains open.
