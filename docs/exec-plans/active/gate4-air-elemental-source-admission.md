# Gate 4 Air Elemental immutable source admission

October5 receiving status: PR50 is accepted at main
`32c0c682c4dbb235e1f9a119643c5d8626d5cb71` after its final checks and review.
Separate literal-main32 runtime verification remains pending. The dated source
evidence and then-pending publication steps below retain their historical meaning.

Status, 2026-10-04: integrated `f932c73` passes canonical verification (782 GNU
Rust tests) and all six CI jobs (783 Linux / 785 MSVC tests). Independent full
review and the completion-log audit are clear. All 411 non-document files match
`3f3e359`, whose verified Windows package completed the bounded native
creation/placement/initiative/flight route on 2026-10-01. Release prerequisite
PR48 and accepted main `dbf1d63` are reconciled. This evidence update still needs
final documentation review, all six checks on its published head, protected merge
and literal merged-main verification. This is not complete Air Elemental gameplay.
Branch: `codex/gate4-air-source-admission`, based on `c4d8c34`.
Writer: root, sole writer after the reviewed integration handback. Historical
checkpoints below retain the evidence status at their original source heads.

## Objective and authority

Admit the genuine SRD 5.2.1 Air Elemental through current creature creation while
preserving every historical source identity, command and presentation. This is a
bounded prerequisite for the reviewed Shove Prone-immunity case, not Shove or full
Air Elemental gameplay. Product-definition source fidelity, controlled entities,
durable play and complete tactical timing remain binding, as do Gate04 and
ADR026/ADR028. The accepted immutable-registry direction is recorded in
[Counterspell source preflight](gate4-counterspell-source-preflight.md).
Magic Missile has merged; that preflight's ordering prerequisite is satisfied.

## Accepted scope and non-goals

- Freeze `tactical.json` and `table_creature_catalog_v1.json` byte-for-byte. Add one
  complete immutable Air Elemental definition in separately installed content.
  Resolve profiles by the full ruleset/version/definition/fingerprint pin; freeze
  ID-only/profileless fallback to V1. Existing source and spell tuple fingerprints
  retain their meaning. Do not reconstruct V1 by removing new fields.
- Add optional, omitted-when-absent creation/source-option pins. Missing pins mean
  V1 forever, and are accepted only for historical replay. New live creation must
  supply an exact current admission pin, including for unchanged V1 definitions.
  Normalization, accepted retries, old requests, exports and the V1 picker never
  acquire a default pin. No existing profile upgrades or general migration API.
- Use pinned SRD 5.2.1 pp258–259 (PDF SHA256
  `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`). Preserve
  the complete statistics, languages, Fly/hover, resistances, damage/condition/
  Exhaustion immunities, Air Form, both attacks and recharge details. Air Form
  authorizes entering/stopping in another creature's space and one-inch passage;
  it does not authorize crossing solid walls or ignoring all difficult terrain.
- Represent Air Form and Whirlwind faithfully as typed source data. Ordinary
  open-space movement may use the existing footprint. Operations that need Air
  Form's shared-space or narrow-passage exception must fail explicitly as
  unsupported before an ordinary-body rule can falsely decide them. No global
  footprint shrinkage, grid rounding, teleport or forced-movement wall bypass.
  Placement must not create an unsupported overlapping Air Form state.
- Refuse Whirlwind and Air Elemental Multiattack at authoritative scheduling
  boundaries before spending action/recharge. Expose execution limits to the Host
  through the current source catalog, without modifying historical presentation.
  Thunderous Slam may use the existing intrinsic attack executor only if source
  identity and implement resolution are exact; otherwise it is explicitly refused.
- Audit profile creation/validation, statistics/saves/initiative, policy,
  equipment/weapons/opportunities, spell planning/components/reconstruction and
  actor-to-program source equality. New full-pin actors must not encounter an
  ID-only V1 lookup or borrow another revision's spell tuple.

No Counterspell, Magic Resistance, corrected Hag, Shove continuation, paid-Shove
ruling, shared-space/narrow-gap executor, Whirlwind executor, general registry
framework, or Gate4 acceptance reduction is included. Source representation and
an explicit unsupported operation are not feature completion. Opponent-facing
refusals must not reveal hidden traits or immunities.

## Planned slices

1. Complete source payload, small immutable registry, explicit current admission
   and full-pin profile/build/equipment resolution. Keep legacy helpers frozen.
2. Audit and repair all source resolver boundaries; add spatial/action guards.
3. Current Host DTO/UI exact pins and execution-limit copy; live versus replay
   admission without normalization or receipt changes.
4. Meaningful tests for immutable V1, source fidelity, forged/missing pins,
   legacy absence/retry, normal Air creation/initiative/save/reopen, and unsupported
   action/spatial atomic refusal. Static review, then root-owned verification.

## Acceptance and validation

Required before acceptance: exact-head source/content tests and application tests;
canonical verification and independent review; genuine historical export/retry
continuation unchanged; new admitted Air Elemental survives normal persistence and
reopen with exact statistics, hover and immunities; unsupported actions/geometry
cannot silently execute or consume resources. Future Shove must separately prove
paid Prone against this source completes without effect, including retry/reopen.

No tests/builds may run on this branch while another lane owns the
heavy slot. Root coordinates verification, publication and protected merge. `rustfmt`,
static inspection and `git diff --check` are permitted. Authored tests are not
passing evidence. Preserve source PDFs, historical exports, capture producers and
the root's baseline. No other worktree is writable by this agent.

## Decisions, risks and next action

- Full V1 definitions remain immutable current admissions; this additive source
  does not supersede any V1 revision. Missing creation pins do not select current.
- New source installation is verified separately against its declared bytes;
  compiled content alone cannot authorize a missing or changed installed entry.
- The narrow spatial guard intentionally leaves the Air Form execution obligation
  open. Its exact scope and privacy will receive independent review.
- Added [ADR029](../../architecture/029-immutable-creature-source-admission.md).
  The coordinating writer approved package compatibility: older installed bundles
  lacking Air content receive a recoverable content error until normal package
  update, with no database mutation. Old campaigns replay using current verified
  content with byte-identical V1 definitions, not a save migration.
- Shared-space guard refinement: ordinary permitted allied/size-based intermediate
  transit and ordinary involuntary overlap need no Air Form permission and retain
  normal behavior. New placement and voluntary final sharing remain unsupported.
  Bounds and a destination wholly enclosed by authored solid remain ordinary
  obstruction; partial body/sweep/corner contact is conservatively unsupported,
  not proof that a one-inch passage exists. Opponent errors name no hidden source.
- Thunderous Slam uses the existing intrinsic backend attack/implement path; this
  does not add a new desktop own-turn intrinsic action form or claim full creature
  play. Multiattack and Whirlwind are refused before payment. Source page anchors
  were independently checked: traits/first two actions p258, Whirlwind p259.
- Resolver audit: profiled statistics, saves, initiative, attack/Pack Tactics,
  weapon/equipment planning, policy and opportunity labels now use exact source;
  spell planning/components/program reconstruction, retained feature activation
  and actor binding compare the complete tuple. Frozen ID-only profileless/build/
  equipment helpers remain V1. Normalization and request receipts are unchanged.
- Authored tests cover Air source/build/immunity/hover/saves, forged pins and V1
  absence, pre-cost unsupported action refusal, open movement/allied transit/
  forced overlap/special geometry, current owned transport creation with live
  missing/forged pin refusal, cold reopen/exact retry/restore, installed missing/
  undeclared/rehashed source no-write refusal, frozen picker serialization and UI
  pin/limit handling. Existing live constructors now carry current pins; these are
  new producers, never replacement historical evidence. Genuine exports and old
  producer/baseline files remain untouched.
- Static checks: direct rustfmt parse/format and `git diff --check` pass. Static
  Python manifest inspection confirms all six declared lengths/FNV checksums.
  Frozen tactical SHA256 remains
  `5cb3e2e6965489c4c8e4f2f0ca643e0e3634ee84e2aed455260f1589fd330548`;
  frozen picker SHA256 remains
  `adca5f6c4e7d5f316f9a8958c93e35a411493d201fac95f6938734e385e63dac`.
  Portable script and Tauri resource configuration already include the entire
  source directory. No Rust/JS compilation, test, packaging or runtime execution
  has run on this branch. Source tests are authored, not passing evidence.
- Recovery review on 2026-09-30 rechecked the complete source block against the
  pinned extracted SRD text. It corrected a non-UTF-8 byte in the new distribution
  notice and refreshed its declared checksum; expanded authored assertions for
  the actual Slam/Multiattack clauses and healthy airborne Hover; and recorded
  qualified scope in the Rules Coverage Ledger. No evidence status was advanced.

Next action: coordinating writer reviews the exact unpublished commit, then owns
focused and canonical verification after the existing heavy/capture slot is free.
Run rules source/profile/schedule/spell/spatial tests, application content-integrity
and table-loop source creation tests, frontend tests/check/build, and unchanged
historical replay/retry suites. Resolve actual failures without weakening evidence.
New Air tabletop placement/initiative and the eventual genuine Shove immune
no-effect retry/reopen proof remain required before claiming those gameplay paths.
No merge or Gate4 completion is implied by this checkpoint.

### Draft verification checkpoint — 2026-09-30

Root reviewed the independent complete 51-file static review of source
`4a27cb0776f1ba5efe0bc2c9fa4a3e782093039e`, full tree
`e1e90f9fa046505cd79dfd0275cc00acface1e00`. No actionable static defect was found;
source fidelity, all six installed manifest lengths/checksums, unchanged V1 Git
blobs, exact source resolution, live/replay admission and explicit unsupported
boundaries were checked. Formatting and whitespace checks pass. This adds no
passing Rust, frontend, application, native or compatibility result.

Root now authorizes draft publication to run remote CI while the local heavy slot
continues the original flow 4 capture and baseline. This supersedes the earlier
agent publication hold only. Main remains fixed at verified c4d8 until that
original-source proof completes; neither this draft nor a green CI result can
waive the remaining acceptance above. This checkpoint changes documentation only.

Next action: inspect actual draft CI output and fix concrete failures; run local
focused/canonical and frontend checks when the heavy slot is available. Preserve
the exact old exports and prove their continuations under this source before
acceptance. Reconcile accepted dependency movement, review the final full diff,
and complete native source/placement evidence before any protected merge.

### Manifest tooling correction — 2026-09-30

The later same-ID creature revision work found two omissions in the Air change:
the manifest regeneration script still enumerated only the five legacy files,
and the existing distribution test expected that same five-file set. Running
the old script would drop the required Air entry. The script and exact-set
assertion now include the installed Air payload; the assertion remains strict.
No source payload, current manifest, legacy picker or historical fixture changes.

Root ran the corrected actual script against an isolated copy of all six declared
files. Its output is byte-identical to the installed manifest, SHA256
`63196eccc85324e248973fb65741ada9b02404ce9e0b74939ddbc8dc0fd0a5bd`.
The script SHA256 is
`792f232ed554a9fe649338bd8c9147e278f33c7ff16d4806b02ce64c419a613e`.
External evidence is `tooling/air-manifest-regeneration-2026-09-30/result.json`.
This verifies regeneration only; the Rust assertion has not executed locally.
Original 1a0a5e7 runtime CI remains separately under audit, and no green result on
that head verifies this correction. The corrected head requires fresh checks.
The original-flow baseline still owns the local heavy slot. Preserve the prior
remote runs until normal completion before publishing the corrected source.

### Actual original-head CI and correction publication — 2026-09-30

Both original 1a0a5e7 runtime jobs completed normally, without cancellation:
Linux job 109923678299 in run
[36726225309](https://github.com/idiotswill/DMd/actions/runs/36726225309)
at 16:07:47Z, and Windows stable job 109923674475 in run
[36726225269](https://github.com/idiotswill/DMd/actions/runs/36726225269)
at 16:18:14Z. Both fail `distributed_rules_pack.rs:23` because the actual six-file
manifest includes `air-elemental-v1.json` while the test still expects five.
Linux ends with 153 passed and one failed test across 17 emitted result groups;
Windows ends with 155 passed and one failed. No packaged artifact was produced.

Before that failure, all 55 table cases pass on both systems (7086.79s Linux,
7460.13s Windows), including actual current-catalog Air creation/cold restore and
all four Magic Missile cases. The installed-source integrity refusal case, five
legacy Reactions histories and five Shield histories also pass. The other four
CI jobs pass, including both MSRV checks, all eight architecture guard tests and
genericity. Both Windows jobs pass 101 frontend tests in 17 files, zero Svelte
errors/warnings and a 140-module build. These are bounded results on the failed
original head, not full workspace or corrected-head verification.

Linux checks out synthetic merge `bcd9d3c85e31503e91ce592bd34fcbcbf4935c24`,
whose parents are c4d8 main and 1a0a5e7. Its complete tree equals source tree
`589dfa92d70b1f63f2195de78df010133d8bf5b2`; Windows checks out literal 1a0a5e7.
The actual six logs, tree comparison and empty artifact API are preserved in
`tooling/air-1a0a5e7-ci/completed-failure-evidence.json` outside the repository,
SHA256 `cf85ab2ccc4a8cf7530190db47252e6e899ca8fb95608a1352080e6e85b10d62`.
Runtime log SHA256 values are
`b373cddc6a4e1f659b9ad75abcdf1833d1383e886c1950e9a35ea17a01bf6a8a`
and `c6514177b2ff73f5146ed7d5ade841fcc9176d457142f5fd90e98b4cd81caf35`.

The already independently reviewed correction is source checkpoint
`021421c4990c686e31a6ad8da0b6ef3571f087ae`: add Air to the exact expected set
and to the manifest generator, preserving the installed manifest and every
immutable payload. Root now publishes that correction with this evidence update.
No assertion is removed or generalized. Fresh exact-head CI, local canonical
verification, genuine flow4 compatibility and packaged source placement/initiative
remain required; the original baseline continues to own the local heavy slot.

### Corrected-head completed CI and reviewed integration plan — 2026-09-30

All six actual jobs completed successfully on corrected source
`e915bbb9568f14a21c253bebf75d05ad2e91476e`, tree
`37db98204f1f91d0ebe44573b5d556fc2f58a55c`. Linux run
[36743556839](https://github.com/idiotswill/DMd/actions/runs/36743556839)
passed Rust, MSRV, architecture and genericity jobs; Windows run
[36743556655](https://github.com/idiotswill/DMd/actions/runs/36743556655)
passed stable and 1.88.0 jobs. Linux's actual checkout
`908baf757b33e39bd0bbe89179a14f8c08307261` has c4d8/e915 parents and the exact
source tree; Windows checked out literal e915. Linux Rust completed at 17:24:42Z
with 754 passed and zero failed across 55 result groups; Windows stable completed
at 17:48:24Z with 756 passed and zero failed across 55 groups. Both include the
strict distribution test, all 55 table cases, original Reactions/Shield histories
and source/content tests. Windows frontend reports 101 tests in 17 files,
zero Svelte errors/warnings and a 140-module build. These are real corrected-head
CI results, not evidence for the forthcoming integrated tree.

The Windows artifact API and upload log agree on artifact `11116031577`, name
`dmd-windows-e915bbb9568f14a21c253bebf75d05ad2e91476e`, 232,377,773 bytes,
archive SHA256 `c1065e0ccaca3780bb66ab1f36058ac50103dbc7664fd7ab5158227ba376ae23`.
Artifact creation is not native GUI acceptance. Completed evidence is external
`tooling/air-e915bbb-ci/final-evidence.json`, SHA256
`56a85dc7a9cc81eefb855a5d4ce5207b3aa7807f5fd1a4230a0356f5f02d50f5`.
Root independently rehashed all six logs and recalculated all Rust result groups
in `root-completed-log-audit.json`, SHA256
`d660c46393d49276ece089c54ba7c7fcaf2e229566d02cc0d94ab3b3777b1ea6`.
The integration writer also rehashed each preserved job log against that completed
evidence before this plan commit. No old failed or successful evidence is removed.

Root now authorizes a bounded normal local merge of exact encounter release
`8c03f9fb0058610fd37c0cfe7762e8b96d658f38` into clean e915 after committing this
plan. Release8c already contains UI65 and fetched main
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`; it is an **UNACCEPTED release
development dependency**. Its first local canonical attempt failed compilation;
root owns the fresh-target retry and all acceptance. The existing no-merge
restriction is superseded only for this explicit development merge. No Cargo/npm/
build/test, database, native UI, source capture, push or job cancellation is
authorized for this writer. Do not import MR, corrected Hag or its coexistence
child into Air.

Resolve the complete source/release union: exact live full-pin admission plus the
authenticated Finished creation exception; absent historical pins and accepted
retry precedence; staged coherent replacement scene/actor locations before Air
placement validation; flow5/completion history and turn high-water plus full
source lookup. Retain the complete source-pin UI tests and lasting-actor tests.
Update only newly authored current test constructors/Begin fixtures where needed;
keep original source bytes, all genuine historical fixtures and the imported
Flow4 corpus exactly unchanged. Inspect all conflicts semantically and the complete
integrated delta, then return a clean checkpoint for root's independent review.

Before acceptance, root must reconcile final main/dependency movement, verify the
exact integrated head with focused/canonical checks and fresh required CI, execute
the genuine old-history/Flow4 continuation suites, inspect the final full diff and
complete packaged Air source placement/initiative and persistence evidence. Release
acceptance remains separate. The future paid Shove Prone-immunity/no-effect path
remains open. Neither e915 green CI nor this local development merge closes those
obligations or any Gate 4 requirement.

### Local release integration checkpoint — 2026-09-30

Plan commit `9cf3d3f37a2064bbda810be7f19dded415f3632c` preceded normal merge
`e6c44a129cc9ccdda4ad30bafc798c4212ffee6e`, whose parents are that plan and exact
release8c. The merge tree is `2776a15915134f520e8a87d389da81e54de39694`. This is
local development ancestry, not publication or prerequisite acceptance.

There were three conflicted files. `Creature.test.ts` retains both current-pin/
execution-limit tests and release's required-lasting-actor placement test.
`tactical.rs` retains authenticated replacement/dependency/identity preflight,
stages participant locations into `next`, runs Air source placement validation
against that coherent candidate, then validates/activates the new scene. The
coverage ledger retains both Air and release obligations, with corrected e915 CI
distinguished from unexecuted integrated source. Auto-merged admission keeps the
exact current full-pin check before the Finished creation exception, and historical
missing pins/retries are unchanged. Optional source and release DTO fields, exact
source initiative lookup, release high-water/history validation and both UI routes
remain present.

The sole additional current-fixture edit supplies exact Mage/adult-red-dragon pins
in `table_release_recharge_cases.rs`; no assertions, resource costs or historical
inputs change. The existing six-file Air manifest, generator, strict distribution
expectation, every content payload and frozen V1 picker remain exact e915 bytes.
Release fixtures and all legacy replay suites, including the original Flow4 corpus
and binary-protection attributes, remain exact release8c. No MR source, corrected
Hag, private save classifier or coexistence child is imported.

Direct Rustfmt parsing/format checks and Git diff whitespace checks pass. These
are static evidence only; no integrated compilation, test, GUI or native result
is claimed. Next action is root's independent review of the clean exact checkpoint,
followed by accepted dependency/main reconciliation and the outstanding exact-head
canonical, required CI, genuine replay and native obligations above. This writer
performed no push, cancellation, build/test, database, UI or source capture.

### Exact integrated CI and native continuation — 2026-10-01

Independent full integration review found no actionable defect at
`3f3e3590e7977eca529168fa9069e8e1bca72434`, tree
`75350e552cac0ad404c2e130b8c595d396d13d50`. The review covers the normal release
union, source admission, staged placement, exact source consumers and unchanged
history; its independent byte audit is SHA256
`c00b57ae03d7c7f39181ee1350f54357d09e2d3a1ca3681aeff17c74ed8cf47f`.
All29 fixture blobs, five original receiving suites and21 protected raw artifacts
remain exact release8c; Air payloads/manifest/picker remain exact corrected e915.

All six CI jobs passed on3f. Linux run
[36758101295](https://github.com/idiotswill/DMd/actions/runs/36758101295), runtime
job110033398903, executes synthetic merge
`b0da5b9857c699b912f09ce5cd7ed4341a9e5ed9` with d88/3f parents and exactly the
3f tree. Windows run
[36758101277](https://github.com/idiotswill/DMd/actions/runs/36758101277), runtime
job110033398912, executes literal3f. Actual full logs report783/785 passed,
zero failed/ignored/filtered,56 result groups; all60 table tests, all8 original
Flow4 continuations, four Magic Missile and five release cases pass. The source
creation/cold restore and installed-source refusal controls pass. Windows frontend
reports118 tests in18 files, zero diagnostics and141 modules. Both MSRV and the
architecture/genericity guards pass. External completed bundle
`tooling/air-3f3e359-ci/final-evidence.json` has SHA256
`4b20ce165a53a570ff9c9253b1dc1ba29d1850431ed84de39180044d11045b57`.
These results verify3f; they do not become executions on a later documentation head.

The actual Windows artifact11130047457 was downloaded,232467920 bytes, archive
SHA256 `bcf9eb2441b7965657156a4de43d9e91d2f6d7406c3743461da41a49b9317090`.
Independent verification checked all1071 payloads against the archive/extraction,
manifest and source. The portable executable SHA256 is
`f177c0a05ba445f86256d0f6794b025ad67a72ee7abc79a58d9cfb1f8d61cf45`;
the independent package audit SHA256 is
`30bcba73bab75aa227cc061944c0c877d479b038ba4fd1a6340e393260b5ee43`.
The installer was verified as a payload but was not installed.

Root exercised that portable build through actual native controls in the existing
QA campaign after the release route's genuine Finished66. Read-only persisted
cuts independently establish the following bounded observations:

- Actual current-catalog creation67 admits a Large90HP Air with full pin
  `srd-5.2/5.2.1/air-elemental/b1b6e8fbf0f250cf`, empty loadout and no item/ammo
  grants. Its catalog shows the documented unavailable actions/geometry. Actual
  assignment68 gives Host control while preserving Arin's Rook/Mage bindings.
- A submitted Air/Rook overlap at5/5 is visibly refused. The captured campaign
  state and all scoped table rows remain exactly68. Correcting Air to10/5 on a
  bright50-foot square map succeeds69, with10-foot body height and floor elevation0.
- Normal initiative70 creates four separate groups. Labeled QA faces entered
  through ordinary owned dice forms yield Rook21, Hag6 and Mage12. Air's actual
  normal1d20/+5 request is pending73. A normal close, process exit and launch of
  the same hashed executable preserve that exact request, state and captured rows.
  The reopened input has its initial accessibility value0; no Air face was accepted
  before closing. The labeled QA face20 is then accepted once74, yielding25 and
  first turnAir/global9. These test inputs are not claimed physical die tosses.
- Host's normal `fly 5 feet up` route succeeds75 at10/5/elevation5feet. Movement
  spent is5feet; Action and Reaction remain available. No OA, falling work or raw
  request is produced. A second full process close/reopen retains the identical
  saved state and captured rows,90HP, airborne position and movement cost. Normal
  End turn76 reaches Rook/global10. No round wraps; world clock remains6.
- Existing item identities/custody, source identities, resources,11/81/93HP,
  Mage Armor's absolute deadline28806, earlier release histories and session state
  are preserved. The app then closes normally. No database mutation or debug/API
  path was used to manufacture these states.

The completed native observation log is external
`tooling/ui-air-3f3e359-native-observations-2026-10-01.txt`, SHA256
`815794887d924af4e9e7aebd2f88aaa7e244736b9e7b08e00414e13247ee99cf`.
The snapshots are campaign-scoped read-only evidence, not a full SQLite backup;
tables without campaign_id, including sessionparticipants, are outside that
capture. Native observations and saved-state audits serve different purposes.
This route does not prove native Slam/Multiattack/Whirlwind/Air Form execution,
all special Hover condition transitions, ammunition/thrown use, or later Shove.
Those declared limits and receiving obligations remain unchanged.

### Accepted-main reconciliation plan — 2026-10-01

Release PR48 merged as `dbf1d633460473183324b4ec519e8d1980884b5c` after expected-head
verification of d4 and its exact all-six checks. Its full tree equals reviewed d4;
compared with the already integrated release8c, only five documentation paths
change. Root freshly fetched main and the unchanged published Air3f and will
commit this plan before a normal merge of dbf. Preserve both Air and accepted
release evidence in any documentation conflict; compare every non-document blob
with3f afterward. Any unexpected source difference needs fresh semantic review.
Do not import MR, corrected Hag, Shove, Grapple, counts or Ogre into this slice.

After that reconciliation, inspect the complete delta and independently review
the candidate. Run the repository's canonical `./scripts/verify` on its frozen
receiving head with one local heavy slot, private target, normal stack/profile
and full original history suites. The corrected core focused runner currently
owns that slot; no parallel local build/native/database operation is authorized.
Update this plan with actual results, run all six checks on the final published
head and merge only with expected-head protection. Verify literal merged main
separately. Prior successful3f CI/package/native evidence remains attributed to3f,
with byte equivalence established explicitly if only documents change.

At this historical checkpoint local canonical and receiving-head CI were unmet. Air acceptance,
Shove acceptance and Gate4 completion are not implied by this checkpoint.

### Accepted-main union recorded — 2026-10-01

Plan3593c1 preceded normal merge
`5b45c7f5646027ba059bd1a1ea086946ae09ad43`, tree
`6620c91d5c3e31a92fcc046bf70ff3200bff31ed`, with accepted dbf main as its second
parent. Squash ancestry exposed eight conflicts. Root inspected each: the three
source/test conflicts were already-reviewed Air additions (source-pin UI controls,
current Mage/dragon creation pins, and source placement against the staged release
candidate). Their exact3f blobs were retained. Five documentation conflicts combine
the accepted release checkpoint and qualified Air evidence; no source union was
silently discarded or replaced by the older release-only version.

The complete merge differs from3f only in six documentation files. A fresh root
audit checks all411 non-document tracked blobs,29 fixture blobs and21 protected
raw worktree artifacts against the previously reviewed3f; every one is exact.
Audit SHA256 is `541631139a58c1ec5ff71c2009caf10bbde43a11dc3bcc52baab8a8a7da9642f`.
The accepted main is now an ancestor. This proves byte equivalence, not new-head
execution. The next frozen documentation checkpoint receives independent review
and local canonical verification after the guarded-core focused run returns the
heavy slot. No additional source edits, fixture recapture or parallel native run
are needed for this reconciliation. Canonical results and final-head CI remain
to be recorded before protected merge.

### Canonical completion and final publication — 2026-10-04

The exact frozen head `f932c73bf1f79cd0c5600431839a6ecece7877f7`, tree
`b654f37272f42b5115ee2c0ebdcc550f191f091e`, completed `./scripts/verify`
normally from 13:05:16 through 16:15:55 UTC. The clean source remained unchanged.
This used rustc1.98.1 on Windows GNU, one build job, incremental disabled, a fresh
private target, and default test profile, stack and harness parallelism. Actual
results are 782 Rust passes in 56 complete result groups (50 executed and six
empty doctest groups), zero failed/ignored/measured/filtered. All60 table cases,
all8 original Flow4 continuations, four Magic Missile and five release cases pass.
Formatting, check, strict Clippy and genericity pass. Architecture tests report
eight run: seven pass and one Windows symlink-privilege skip. The skip is not
counted as a pass. The final script exits0 with `Full verification passed.`

The preserved canonical log SHA256 is
`a43a9604b12494644fff028a0ceef620992126d34d2db93626db1798dc383ef0`;
metadata SHA256 is
`e09d7b755814e30d9a6139d13c9caf154f77e4fb4c0136a864619dee5143fd60`.
The independent completion audit, SHA256
`e27697778e16733246d16bab40e8746e4c916aaf0cfd1b624741828bf9cfff26`,
recounts every harness/name and binds the exact source, command, script hashes,
normal completion, clean before/after state and platform-specific test inventory.
A separate reviewer read the actual log and metadata and independently confirmed
the totals and completion marker. These are completed results, not a running-log
estimate or timeout.

All six checks also pass on this same f932 head: Linux run
[36840950338](https://github.com/idiotswill/DMd/actions/runs/36840950338), runtime
job110299669063, and Windows run
[36840950425](https://github.com/idiotswill/DMd/actions/runs/36840950425), runtime
job110299669656. Linux runs synthetic merge
`b69d7ca936a3d9b1b903edc4498e24ec55e4dbfa` with dbf/f932 parents and the exact
f932 tree; Windows runs literal f932. Actual783 Linux /785 MSVC passes share the
same56 groups and60 table cases. The count differences are the one Unix symlink
test and three MSVC host tests, not missing common tests. Frontend118 tests in18
files, zero Svelte errors/warnings,141 Vite modules; both MSRV jobs and guards pass.
Completed CI bundle SHA256 is
`9332687847ed00297c5708b5ccd2137412849664d7db21c408eb85be0acec23d`.

The complete independent acceptance review at f932 covers all58 changed paths
against accepted dbf, source consumers, compatibility and bounded native evidence;
memo SHA256 is
`b7d32e641397b50a2e979fdbf5fd51a25a96608f875699ac6cee208550d5b573`.
All411 non-document blobs,29 original fixtures, five receiving suites and21 raw
artifacts retain their audited identities. Native/package evidence remains
attributed to actual3f execution above, supported by exact source equivalence.
It is not presented as a second native execution on f932 or this evidence update.

Next: independently review this documentation-only delta and recheck non-document
identity, publish the final head, inspect all six completed exact-head checks,
merge PR50 with expected-head protection, fetch main and verify the literal merged
head separately. Do not silently borrow the f932 checks for a later head. No new
feature branch enters Air. Shove, Air Form geometry, Multiattack, Whirlwind and
the rest of Gate4 retain their separate acceptance obligations.
