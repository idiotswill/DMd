# Gate 4 — Source creature control at the table

Status: implemented in draft PR43; verification in progress. Source `26241a9`
passes native Windows CI, 693 Rust tests and packaging. Its Linux PR merge with
main `d82d7b2` passes all checks and 694 Rust tests. The local default-stack and
source-control confirmation also passes. Verified main `2798b6b` and its genuine
PR42 corpus are now integrated. Final combined verification is pending; this is
not acceptance.
Sole writer: source_control_recovery, taking over environment_audit's preserved
branch. Branch: `codex/gate4-source-creature-control`.
Authorized base: `100c7dabe07b07b7430bcb721b1dd7f48e4792cf`.
Fetch on 2026-09-25 found main `a84c5a1`; the owner of integration deliberately
requested this frozen prerequisite base. No unreviewed main merge is implied.

## Objective and authority

Give an attending player genuine control of an authenticated source creature through
the actual desktop, including actor selection, physical tactical dice and the Mage's
own-turn Mage Armor. Preserve old accepted table requests, historical presentation
hashes and existing player-controlled source data. Never fabricate a Character row.

This advances the product definition's actual application, physical dice, private
identity, save/restart and recovery requirements within Gate 04. ADR026 governs the
shared tactical flow; ADR027 governs audience revisions and exact transport recovery;
ADR028 separates actor control from execution-version and ordering ownership.
`gate4-live-reaction-responses.md` remains root-owned and required Gate 4 work.

No reaction windows, new execution version, spell-grant expansion, NPC autonomy,
full class catalog, or natural-language NPC declarations are implemented by this
slice. Existing own-turn source mechanics and PC conversation remain authoritative.
Builds, frontend runs and integration use the shared heavy-process slot coordinated
with root. Draft PR43 is authorized and open; merging remains root-owned.

## Audited starting boundaries

- Table creation makes source creatures Autonomous. Internal source construction and
  SetContext already support Player ownership; absence of such historical data cannot
  be assumed. CreatureRuntime.controller/control_origin remains the sole owner truth.
- Table transport v1 selects a CharacterId; table tactical admission, projected owned
  actors, pending-roll visibility and desktop controls currently assume PCs.
- Kernel PC sheet/owns logic must not be widened to level-zero source creatures.
- Mage Armor is executable, but frozen table_casting options omit Caster target rules.
  Add its self-target option only under the explicit new presentation semantics.
- SQL0011 accepts JSON and protects immutable rows; it has no record-version check.
  Rust load/insert/portable validation has six version-one checks to version explicitly.
- Existing table.action@2 and table.conversation@2 retain transport-v1 meaning forever.
  Old UpgradeExecution remains Legacy-to-ReactionsV1. Control is not inferred from
  encounter flow version, and source control persists outside encounters.

## Planned coherent slices

1. Capture/locate genuine pre-change presentation and exact-retry evidence before
   changing projectors. Prefer an authentic old Player-controlled source history;
   if unavailable, state that limit and distinguish typed compatibility fixtures
   from genuine old binary output. Do not manufacture or bless old hashes.
2. Add a default/skip-none TableState source-access marker and explicit host table
   activation/adoption. Authenticate all existing Player-controlled source actors,
   their immutable profiles and actual control origins at activation. Retain the
   original host command and adoption evidence; this is provenance, not a new owner
   map. Add journaled controller assignment/revocation using current source ownership
   and preserving lair/context semantics. Reject pending or held work, including Ready.
3. Add deliberately versioned player actor transport, canonical audit interpretation,
   and projection dispatch. Freeze v1 DTO bytes and projectors. New player selection
   is Character or SourceCreature, separately authenticated from player identity.
   A source-only attending player needs no fake PC binding. Exact accepted nonce/body
   recovery precedes current version, actor, session and revision checks. Never silently
   normalize an unaccepted old envelope or rewrite an accepted v1 body.
4. Bind new projection records to the historical marker; preserve before-v1/after-v2
   activation and bootstrap in one BEGIN IMMEDIATE transaction. Authenticate audit
   markers, record/binding versions, origins, capabilities and exact responses during
   replay before restore writes. Legacy schema/export inputs reject future authority.
   Keep SQL0011, schema4 and export3 unless a concrete compatibility reason requires
   a separately documented change; an old binary already rejects unknown record2.
5. Expose host activation/assignment and player actor selection, genuine source-aware
   own-turn options, self-target Mage Armor and actual owned tactical roll forms.
   Preserve the full original outbox across restart/retry and discard stale query
   replies. Text declarations remain tied to actual supported PCs. No implicit host
   reaction control or change to source monster/mixed initiative tie rules.
6. Run focused source/control/transport/restore/UI tests in the granted slot, obtain
   independent exact-diff review, then root integrates and runs canonical/package
   verification. Report limitations and failures accurately; this is not Gate4 acceptance.

## Compatibility and admission decisions

Activation is a new host TableAction, independent of executor upgrade. A future
explicit targeted upgrade could include it only by retaining the table semantic
target and writing the same marker; old upgrade actions cannot acquire this meaning.
The activation boundary must be settled: no table decision/roll, raw rules request,
shared resolution, source routine/recharge ticket or held Ready/cast. Assignment uses
the same restriction, rather than relying on SetContext's narrower local guard.

The activated projector uses source-authenticated actors and current runtime owners.
Other audiences do not receive controller lists, private actor names or hidden roll
work. Source roll admission still requires the current owned opaque capability and
actual roller; secret/host-only source rolls do not become public. Do not globally
change legacy kernel ownership or sheet formulas.

Use explicit transport v2 and projection-record v2, with a distinct canonical audit
envelope version for new requests. Version-one records, responses and numeric legacy
recovery keep their original decoder. Newly submitted v1 requests after activation
must refresh, while genuinely accepted v1 requests still recover exactly. The marker
is mandatory evidence even without source assignments; removing it, its activation
event or the new ledger cannot trigger a legacy fallback.

## Acceptance criteria

- Real host catalog/CreateCreature creates Mage unchanged; explicit activation and
  assignment give one present player the actual actor, without Character/profile/slot
  fabrication or a free change to physical equipment/source uses.
- The player selects Mage, rolls real initiative through an opaque capability, and
  on its own turn selects/executes source Mage Armor with genuine material and source
  cost. Other player and foreign actor/channel/raw identifiers reject without writes.
- Source-only session attendance works under the activated semantics; old PC-only
  session and text paths replay unchanged. A source NPC remains TacticalSource::Creature.
- Activation/adoption retains every genuine prior Player-controlled source owner;
  no old control is automatically projected by merely opening a campaign with the
  new binary. Existing v1 hashes and original request/response bytes stay unchanged.
- Pending/held work blocks transfer; a settled reassignment/revocation is journaled,
  rotates only changed audience presentations and survives reopen. Previously accepted
  exact requests still recover after ownership/session changes; fresh stale/foreign
  requests reject atomically.
- Cold file SQLite at assignment, owned raw pause and casting completion; independent
  portable restore agrees. Tampering actor/source/control_origin/adoption/marker,
  record version, audience, capability, binding body or response fails before writes.
- Stripped new records/markers and legacy envelopes carrying new authority fail closed.
  A genuine old corpus validates byte-preserving compatibility; synthetic tests are
  clearly labelled and never replace that evidence.
- Desktop tests drive actual host assignment and actor selection, then uncertain
  delivery/restart with the full original version/channel/actor/revision/body retained.
- Independent review plus focused Rust/app/persistence/UI checks on the exact final
  source; canonical checks and native evidence remain root-owned and explicit.

## Verification commands and status

Planned focused commands (only with the shared slot): source-control table/transport
tests, persistence protocol/schema tests, all dmd-app tests, strict Clippy for domain,
rules, persistence and app; desktop npm check/test/build. Run repository fmt/diff
checks before source checkpoints. Root runs scripts/verify on the combined candidate.

At plan creation: AGENTS, product definition, Gate4 execution documents and ADR027/028
read; base clean; fetch completed. No implementation/tests/builds have run for this
slice. Prior branch checks are not evidence for it.

## Risks and next action

The transport/history change crosses several strict serializers. Preserve old variant
shapes and version-dispatch each acceptance path, including trusted internal table
commits after presentation initialization. Restore origin allowlists must be narrow,
with semantic action replay proving the actual change, not broad table-event permission.

The original 100c7da route audit is complete: there is no InitializeTable action.
`create_table_campaign` constructs an empty sequence-zero table; its same-ID retry
only accepts matching initial creation. `CreateCreature` is the only table source
constructor and fixes its controller to Autonomous. Raw `execute_rules` and
`execute_tactical` reject table campaigns. `RulesAction::Initialize` accepts only
mechanical entities and explicitly initializes `tactical_creatures` to None; it
cannot carry source profiles or controller provenance. Pure `build_creature` and
`SetContext` can produce Player-owned typed source state but no shipped accepted
table route imports that state. Restore also rejects source-bearing first anchors.
No genuine old Player-source table corpus is claimed or fabricated. The pure
SetContext projection/adoption test is expressly qualified, while both checked-in
pre-change table corpora are independently restored and their original legacy
acceptances and exported historical bytes are compared unchanged. Both old captures
have zero modern accepted bindings, so they do not prove historical `@2` retry bytes.
The genuine new Mage scenario separately accepts a v1 modern request before activation
and recovers that exact response afterward; it is not labelled a pre-change capture.

Implementation now drafts transport v2 with an additive SourceCreature channel.
Host and Player serialized variants remain unchanged. Activation itself uses v2
with the genuine preceding v1 Host revision; only the explicit activation action
may cross that boundary. Its audit@3, TableState marker, projection2 and binding2
commit together. Every subsequent fresh action/observation requires v2, including
trusted legacy entry points; accepted old requests recover before current checks.
The v1 projector stays PC-only. V2 adds authenticated source ownership, tactical
raw requests and self-target spell choices. Old CreateCreature and execution
versions are unchanged. Source-only attendance/map admission requires a real
present controller and an assigned source actor, rather than a fabricated PC.

SQL0011 has no version constraint to migrate. Its six Rust loader/insert/portable
guards now recognize 1 or 2; historical app replay determines which version belongs
to each state. Export3 is unchanged; older readers reject record2/new TableState
authority, and older export/state envelopes explicitly reject the new field.

The implementation covers real Mage activation/assignment, source-only session,
owned initiative, material-backed Mage Armor, held-work transfer rejection,
revocation/exact retry, file reopen and hostile portable restore. UI tests exercise
actual review/activation, assignment, actor selection, self-cast and raw-roll
submissions with retained retries. The first checkpoint was deliberately uncompiled;
the executed results and subsequent corrections are recorded below.

## PR43 review and first CI correction

Draft PR43 began at source `67156c1` and ancestry-only merge `bca4f10`. The frozen
`100c7da` base and merged main `d5d1db7` have identical tree
`801db9204a1418303ffada37d46cabf073fef159`; the ancestry merge changed no source.

Initial Linux run 36177838852/job 108212680221 passed format, MSRV and strict lint,
then overflowed the default stack in the existing normal table scenario. The new
optional access marker is now boxed, preserving identical serialized bytes while
removing its inline CommandMeta/Vec cost from every CampaignState async frame.
There is no stack-limit increase. Initial Windows run 36177838848/job 108212679995
passed 71 frontend tests and failed the new Mage selector scenario. Its logged DOM
shows no selected source actor; the fixture now waits for the actual actor selector
to be enabled, selects it and requires persisted actor selection before casting.
The later local/CI runs below distinguish the successful UI correction from the
marker boxing, which was insufficient to fix the Windows default-stack failure.

Root review also found generic trusted Host tactical authorization could substitute
for the new player's source decisions. An activated-table-only admission guard now
derives the responsible actor for each supported action and rejects Host substitution
for owned declarations, public dice and optional choices. Host initiative setup,
explicit frame ordering and secret raw-dice handling remain unchanged. The tests use
the Host's own current revision and roll capability, assert the ownership rejection
itself, and retain the later legitimate owner acceptance. The UI hides the same owned
controls, preserves old views without the marker and discards stale source-control
query successes/errors after campaign, channel or revision changes.

The existing CreateCreature setup still requires earlier real PC creation; this slice
adds source-only attendance and encounter participation after that existing setup,
not a replacement character-creation path. Independent source review preceded the
authorized local batch and fresh CI; no earlier checks are attributed to later source.

Local verification at `9638f64`: Svelte check 0 errors/0 warnings, all 76 frontend
tests and production build (137 modules) passed. The first default-worker Vitest
attempt was deliberately interrupted after observing parallel workers; its partial
results are not counted. The successful rerun used `--maxWorkers=1`. Logs are
`tooling/source-control-ui-check.log`, `source-control-ui-test-serial.log` and
`source-control-ui-build.log` outside the repository.

The default-stack `normal_scene` still overflowed locally after the marker boxing.
Temporary markers and a debugger trace located real CreateCharacter source-definition
deserialization after campaign creation/AddPlayer; they have been removed. The
public execute future occupied 19,248 bytes inside parent futures. The next candidate
heap-pins the unchanged legacy transaction futures at their existing execute/observe
public wrappers. It changes neither transaction semantics nor stack limits. This
candidate needs execution; logs retain both failed diagnostics and the backtrace.

Linux run 36179064153 reached 44 passing table cases and a hostile-restore assertion
failure. Static investigation found the fixture used random-CommandId binding order
as acceptance order: its downgrade could select an already-v1 row and do nothing.
Historical rows are now compared by immutable ID; the version downgrade selects an
actual v2 Host binding, while audience/channel/response changes select the Mage
owner's accepted SourceCreature raw-roll binding and token changes select its player
Roll capability. Every named corruption must differ before the unchanged zero-write
rejection assertions. A fresh run must still prove every negative case.

Existing identical-monster initiative grouping remains a documented Host initiative
authority exception. Groups and their representative roll retain the established
source semantics even when an assigned actor shares that group; the new guard owns
individual public requests and voluntary actor decisions, not a retrospective rewrite
of grouped initiative. Source-control activation does not change execution versions.

The failure corrections are committed as `26241a97ab9aef19161564a15f5acf2baea11a45`.
Independent exact-head review confirms the unchanged transaction semantics and the
deterministic Host/source-owner/capability corruption selectors.

Linux CI run 36181359111 is green for this PR head: genericity, architecture,
MSRV, check, strict Clippy and 694 Rust tests, with zero failures or ignored tests.
Its standard PR checkout is merge `95f1a61c9559bf5dbef7028bf66003251c4f554e`,
combining branch `26241a9` with main `d82d7b2`; it is not a literal branch-tree run.
Actual rust-job 108224247410 logs include all 46 table cases passing: the original
normal-scene regression, genuine Mage ownership/raw/self-casting/cold retry and all
named hostile restores, plus the qualified old-v1 corpus. The table suite took
1179.34 seconds.
Windows run 36181359026 has passed both frontend steps and MSRV; its completed
MSRV log confirms 76 frontend tests, Svelte 0 errors/0 warnings and 137 build modules.
Stable job 108224247419 checked out literal `26241a9`, passed check/lint and all
693 Rust tests (zero failed/ignored), including 45 table cases. Actual logs confirm
the normal-scene default-stack test and both source-control scenarios passed. The
native table suite took 987.14 seconds. The Linux count additionally includes three
main Night Hag cases and one Unix manifest test; Windows has three desktop host tests.
Release compilation finished at 20:14:40 UTC and the NSIS bundle at 20:16:49 UTC.
Artifact 10885613281 is `dmd-windows-26241a97ab9aef19161564a15f5acf2baea11a45`,
231628602 bytes, SHA256
`050fdbd8cb0012b14dc0ef1b5699aae1b522bcb5c8987ad2b2d3b1f486761eb5`.
All six workflow jobs are green; native gameplay acceptance beyond the tests is not
claimed.

Root released the heavy slot after PR42 canonical succeeded. Local session 50690
then passed the normal-scene regression (1 test, 7.27 seconds) and both source-control
cases (2 tests, 123.94 seconds), with zero failures/ignored. It ran serially with
jobs1 and RUST_MIN_STACK removed, after refreshing the shared crate roots; no stack
override or assertions were removed. Logs are
`tooling/source-control-262-default-stack.log` and
`tooling/source-control-262-cases.log`. The process exited zero and the heavy slot was
explicitly released to the next worker.

Next: integrate the genuine PR42 pre-change @2 corpora only after their reviewed
prerequisite merges, preserve their captured bytes, add the exhaustive source-channel
test-helper match, and run the combined compatibility/canonical checks in the next
authorized slot. Do not conflate this source-control boundary with later live reaction
execution or claim Gate 4 completion.

Resource recovery continuation: the preserved branch was clean at `daf8534`, one
evidence-only commit above verified production `26241a9`. A fresh fetch confirmed
remote PR43 remains at `26241a9` and main at `d82d7b2`. No unknown branch movement
or prior work was discarded. Wait for the parent's verified PR42 merge, then merge
that exact main and adapt only the new corpus helper's exhaustive channel match.
No heavy local checks run until the parent releases the shared slot; use jobs1,
the default thread stack and CARGO_INCREMENTAL=0 after the disk recovery.

## Genuine corpus integration candidate

Merged verified Night Hag main `d82d7b2` normally, then fetched and merged PR42 main
`2798b6b1d6263b5e321a1903d9fb4f2331b73895`. The parent verified the latter's complete
tree parity with reviewed source `cbe9575`; its six CI jobs passed before protected
merge. This branch preserves all four captured JSON files exactly. The sole test
adaptation is an exhaustive SourceCreature-to-player-view arm in the new corpus
request helper; its wire requests remain literal v1 because those tables have not
activated source access. No previously verified production source changed during
these integrations, and no new Shield executor is part of this branch.

Publish the coherent integration candidate for fresh Linux/native CI while waiting
for the shared heavy slot. Canonical `./scripts/verify`, focused genuine corpus
acceptance, and final review must still succeed on the combined source. Earlier
green heads are retained evidence, not a substitute for this candidate's checks.
Integration format and diff checks pass. The four JSON files have no diff against
verified main, and all four Rust production source trees have no diff against
`26241a9`; inherited Night Hag content and test changes remain part of the candidate.

## Attendance review correction

Review of integrated candidate `6b06597` found a reachable source-owner deadlock:
an absent player's source creature could enter initiative, or be assigned during
a settled encounter, while its subsequent actions require actual attendance and
Host substitution is correctly denied. Pending initiative also blocks transfer.
Add activated-table admission requiring every selected source controller to be
Present before Begin, and requiring a new controller to be Present when assigning
an actor already in an encounter flow. Outside-flow assignments remain valid for
preparing future sessions. Extend the actual source-only Mage scenario with
zero-write absent-owner Begin/transfer rejections and the subsequent valid owner
continuation. Existing unactivated v1 replay retains its admission semantics.

Activation of an already running, settled encounter receives the same attendance
protection for every adopted actor in that encounter. A labelled pure typed-state
test uses the genuine settled upgrade capture as a baseline, synthesizes source
ownership only through SetContext, then proves absent-owner refusal and present-
owner activation. It does not claim a genuine historic Player-source capture.

Independent review also found that choosing a source actor left a PC declaration's
Correct/Withdraw buttons active. Their actions now require the current PC channel;
the source selection instead explains how to switch back. The UI regression selects
the actual Mage, checks that neither control can create an outbox request, switches
back to the PC, and exercises correction focus plus a lost-ack withdrawal/restart
whose exact original PC channel overrides a later local actor preference.

The review corrections were published as `fe76131`. Its exact-head Windows MSRV
job 108250211419 passes all 77 frontend tests, including the new PC selection and
restart case, Svelte check with zero errors/warnings, and the 137-module production
build. Linux compilation, strict lint, MSRV and guards also pass; full Rust acceptance
and packaging are still pending. No local runtime success is claimed yet.

A final static correction keeps the Begin attendance check behind the existing
Admin/System issuer gate. Nonprivileged Begin attempts must reach the established
rules authorization refusal without learning a private source's attendance from
this new guard. Assignment and activation already authenticate Host first. Desktop
source is unchanged from the verified frontend head. The next heavy slot runs
focused Rust/source/adoption and genuine corpus cases, then releases briefly for
the parent's Shield rules cases before full canonical verification.
The actual SQLite scenario also assigns the prepared Mage to an absent player,
then submits Begin through the other attending PC's valid channel. It requires
the ordinary unauthorized-action message and unchanged complete normalized export,
before the Host returns control to the legitimate present owner.
