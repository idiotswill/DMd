# Gate 4 — Authenticated encounter release and the next battlefield

Current status, 2026-10-04: PR48 is accepted at main
`dbf1d633460473183324b4ec519e8d1980884b5c`. Separate literal-main CI runs
[36834974479](https://github.com/idiotswill/DMd/actions/runs/36834974479) and
[36834974592](https://github.com/idiotswill/DMd/actions/runs/36834974592) completed
all six checks successfully (776 Linux / 778 Windows Rust tests). Native and local
canonical evidence below remains attributed to source8c; no new native execution
of the main package is claimed. Gate4 and its other slices remain unaccepted.

Historical receiving status: integrated source `8c03f9fb0058610fd37c0cfe7762e8b96d658f38` passes
canonical verification, all six CI jobs and the actual packaged two-encounter
continuation with Finished and pending-attack cold restarts. Original-source flow 4
capture/baseline and all eight receiving continuations pass. Independent source,
package and saved-state reviews are clear. Final documentation head d4 passed
exact-head review and all six checks, then PR48 merged as dbf1d63. Separate literal
merged-main runtime verification remains pending. Gate4 remains active; this
bounded release is not gate completion. Historical checkpoints below retain their
original evidence status.
Writer: root, sole writer of `codex/gate4-encounter-release` for the
2026-09-30 corpus import and parent integration. `gate4_release_verify` authored
the preceding 2026-09-28 evidence checkpoint.
The prior implementation writer was `aftermath_finish`.
Development base: `e0adf8071cc3cbbba50dc1a43952d0ac001a0f4b`, the reviewed
Magic Missile candidate. Fetched main at branch creation:
`f441adedcf490504b6f1e3db1a964c023c511e47` (verified PR44).
Root allocated the next semantic executor, **EncounterReleaseV1, flow 5**, to
this work on 2026-09-27. Counterspell must use a later execution boundary.
Flow 5 is now authored across rules, application and desktop. Its presence in source
is not runtime acceptance; the original executions 1–4 remain separately preserved.

Root authorized this plan on a fresh branch in the free former aftermath worktree;
the old `codex/gate4-encounter-finish` branch remains preserved at its verified head.
Root reviewed and approved plan commit `fa99f371725df5e5c8e54ca8282015c707a03c41`.
The first checkpoint is the domain records and pure rules, reviewed before app/UI
expansion. Static implementation may run alongside the unchanged parent's capture
work; the complete frozen-corpus proof remains a hard final acceptance blocker.
Root initially reserved the heavy verification slot for prerequisites. After cache
619 completed canonical verification, root assigned it to this branch for focused
release checks with one build job, incremental compilation off and the default stack.
Those focused checks are complete and the heavy slot has returned to root for
the prerequisite and UI work; no further local build is running here.

## Objective and product traceability

Let the actual table release a fully settled, explicitly concluded encounter's
initiative timing, preserve its physical and rules consequences, close/reopen the
session, and prepare a second encounter with the same characters, source creatures
and items. Players must not recreate actors, reset a campaign, repair a save, erase
effects or refresh resources to continue ordinary play.

This advances the product-definition requirements for a real production gameplay
loop, durable consequences, source-derived mechanics, controller authority,
player-safe information and exact save/exit/resume across sessions. It advances
[Gate 4](../../checkpoints/gate-04-tactical-encounters.md) encounter completion,
timing, physical inventory, death/recovery and production integration acceptance.
It does not complete Gate 4 or authorize Gate 5.

Authority and design references:

- Root `AGENTS.md`, [product definition](../../product-definition.md),
  [gate execution protocol](../../checkpoints/gate-execution-protocol.md).
- [ADR024](../../architecture/024-table-application-authority-and-transcript.md),
  [ADR026](../../architecture/026-interruptible-encounter-resolution.md),
  [ADR028](../../architecture/028-reaction-execution-and-work-ownership.md).
- The accepted first aftermath slice and approved next-release proposal in
  [the aftermath plan](gate4-encounter-finish.md), plus the independently reviewed
  2026-09-27 release audit. Its important conclusions are incorporated here so this
  plan does not depend on an external scratch file.
- [The Magic Missile plan](gate4-shield-missile-runtime.md) for inherited flow 4
  history, owned nested work, immutable committed darts and parent prerequisites.

Pinned source: SRD 5.2.1, SHA256
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Relevant anchors are rounds/turns (p.13), dying/stable recovery (pp.17–18), duration
(p.106), Hold Person target-end saves (p.141), concentration (p.179), and Ready plus
same-time choices (pp.186–187). These do not prescribe a digital end-encounter
clock or authorize a host to dismiss another controller's outstanding work.

## Why development starts before the parent is accepted

The reviewed Magic Missile candidate already supplies the source shapes that a
correct release scan must understand: retained missile respondents, casts, individual
amounts/impacts and nested children. Planning against it avoids implementing a scan
against an obsolete flow 3 shape and then omitting the new obligations during a
merge. The current parent is an explicitly authorized **development dependency**.
Its source review and focused evidence do not substitute for accepted merged code.

After independent review of the release checkpoint and its equipment correction,
root authorized a normal development merge of published cache checkpoint
`619ada0219e4a53f30bea28b3b099314b0c01405`. Merge
`c878f9cb51db8364aa1b5b633a0d2373dffd9411` is conflict-free and retains both
histories. The cache parent includes Shield main
`a0b12d2d0144a744e3419c1ba69e2d7aac64fd79` through reviewed identical-tree
reconciliation `8da8e3c6f250a6f166b652530b36af2f47a164ff`; 619ada0 adds only
reviewed evidence documentation to that reconciliation. Cache PR47 is now merged
as `046109cdc849c16100c42588e771f8abe710c787`, whose full tree equals 619ada0.
This branch still needs normal reconciliation of that literal main and the final
verified Magic Missile prerequisite; parent evidence does not verify release.

Final acceptance and merge of this branch require:

1. Fetching and normally reconciling verified PR45 and PR46 main, including every
   intervening source correction; no unknown force pushes or discarded histories.
2. Integrating a cache prerequisite if root has merged it. Cache behavior must
   preserve the same strict semantic restore, audience and receipt authority.
3. Completing the genuine flow 4 compatibility capture gate below under its actual
   original source, then keeping those frozen bytes and old behavior unchanged.
4. Independent review, canonical verification and all six checks on the exact
   final integrated head, followed by expected-head protected merge and separate
   verification of the merged main.

## Scope and non-goals

Deliver one production vertical slice: authenticated timing release from settled
aftermath, durable completion and turn highwater, retained old scene item positions,
Finished session/setup admission, atomic battlefield replacement, real physical
initiative and an actual action in the second encounter.

Release is an explicit GM timing decision with a typed command and receipt. Any
freeform explanation is narrative only: it does not establish victory, surrender,
consent, loot, XP, an elapsed interval, healing, a rest, a pickup, an effect dismissal
or a source resource reset. It performs no synthetic Start/End turn and changes no
world time. Source outcomes continue to come from their existing real actions.

This first bounded release refuses situations for which no supported subsequent
timing route exists. It does not silently end or rebase them. Offstage/no-turn
absolute work, free elapsed time, safe relative-effect rebasing, old-scene pickup
or revisit, broader carried Reaction expenditure, Counterspell and Ready release
remain named required workstreams **inside Gate 4**. Their absence is not permission
to waive full gate acceptance.

## Durable authority and execution version

### Preserve executions 1–4

Introduce `EncounterReleaseV1` as flow 5 and explicitly inherit the accepted hit,
missile and aftermath behaviors of the verified parent. Existing flows 1–4 retain
their meanings, source programs, wire bytes and historical continuation choices.
Numeric flow 0 remains invalid. An absent old execution field stays absent.
The original unit `UpgradeExecution` remains 1→2 forever; accepted typed upgrades
to 3 or 4 replay under their original admission rules. Never fill an old request's
missing field from today's current executor or change a source fingerprint.

Fresh ordinary play requires flow 5. A forward upgrade is an explicit accepted
command at a settled boundary, preserving HP, resources, timing, geometry, inventory
and effects. Pending old hit/missile/raw/Ready work must finish under its original
executor before upgrading. No release or setup can smuggle an upgrade through
an otherwise incompatible pending state.

Root approved a narrowly admitted host `UpgradeExecutionTo` 5
with no active session, permitted only when the same complete release preflight
already proves no owed work or stranded dependency. It changes only execution
authority and journals the accepted upgrade, followed by a separate release command.
This prevents an already closed flow 4 save from requiring a fabricated attendance
session merely to reach a safe administrative boundary. Preserve existing admission
for all other upgrades and commands. This route requires already concluded legacy
aftermath, identical complete release preflight, and exact source/issuer/replay
guards. It cannot change costs, elapsed time or resource state.

### Authenticated completion and highwater

Add an omitted-when-absent completion record outside the replaceable live flow.
Retain the exact release metadata, old encounter and scene identity, original
conclusion command, unchanged clock, final actor/global turn, and execution version.
Strict replay reconstructs every field from its original command and prior state.
The record is historical evidence and a highwater authority, not another mutable
copy of HP, inventory, concentration or resource pools.

The first campaign initiative still starts at global turn 1. After authenticated
release, the next actual initiative starts at checked `previous_final_turn + 1`,
while its new round starts at 1. Reject overflow, a reused encounter identity and
a forged/reused predecessor. Do not rewrite existing scalar Savage Attacker turns,
raw roll identities, recharge records or historical event payloads. Retained receipt
and scene identity validation plus journal replay must authenticate identity reuse
across more than the immediately preceding encounter, without adding a second
gameplay-state ledger. The chosen representation is an ordered immutable list of
compact completion receipts: encounter/scene/location IDs, setup/initiative/
conclusion/release command metadata, predecessor release ID, release instant and
final global turn/actor. Every retained encounter ID is unique across that list;
each predecessor and strictly increasing highwater follows the previous receipt.
Scene spaces are separate spatial attachments keyed to those receipts. Neither
store duplicates mutable HP, resources, concentration, equipment or item custody.
Record this representation in the ADR update.

### Old scene custody

Add an omitted-when-empty scene-space attachment retaining old encounter, scene and
location IDs, its battlefield geometry and accepted release origin. Atomically move
the live loose-item spatial records into that space, preserving each item's exact
position and original drop command. The later release authenticates the transfer;
it does not replace the earlier drop cause. Item ownership, custody, quantities,
physical state, starting-grant receipts and equipped identities remain authoritative
in their existing stores.

Release marks the old scene Closed while retaining its complete presence evidence.
Otherwise a later active scene duplicates participant presence or moves an actor
away from a still-active old location, violating existing world invariants. The
Finished encounter remains attached to that closed scene until atomic replacement.
Every retained scene remains Closed and a replacement must use a fresh scene ID;
reusing a location is permitted but overwriting retired scene history is not. A
surviving offstage dependency already participating in a different Active scene
blocks release, because this bounded route cannot close that other scene by proxy.

An item has exactly one live placement across current and retired scene spaces.
Validate origin, scene/location, bounds and uniqueness even when the live flow is
Finished. Reusing the same location for new geometry does not move dropped gear to
the new map. Keep the Finished encounter attached until one atomic accepted setup
installs its replacement; no intermediate empty-encounter state may loosen the
legacy guards or discard current inventory/effect/recovery attachments.

## Derived release preflight

Use a pure shared scan for live command admission, capability derivation and strict
replay. No client-provided eligibility flag, narrative ruling, visible prompt count
or saved offer can replace it. Evaluate the whole authoritative campaign, including
offstage actors and suppressed effect records, before any mutation.

1. Require an explicit existing conclusion and a settled active flow. Reject table
   pending decisions/roll context, rules raw dice, the central resolution or any
   selected/queued child, hit/missile/area/cast/movement/falling continuation, effect
   ticket, source routine/recharge/opportunity window and global
   `CharacterFeatures.inspiration_transfer_pending`. Paid attack grants or remaining
   attack-window choices cannot be discarded. Selected-but-not-yet-cast Shield and
   committed missile impacts remain owed work even when their eventual damage is zero.
2. Reject paid Ready/held spells, Dodge, Disengage, live movement progress and any
   legacy `AtTurn` expiry. Scan all effects and groups for owner-relative expiry,
   every Turn trigger and every `OncePerTargetPerTurn` rule, including Damage and
   ZoneContact rules and suppressed overlaps. Unsupported retained zone/spatial
   lifecycle dependencies block release until their old-scene route exists.
3. Reject any living zero-HP unstable death-save actor, unreported stable recovery
   die, pending recovery choice and already-due absolute effect/group/recovery
   deadline. Preserve valid future stable/knockout/rest records and actual raw
   recovery dice. Advancing neither clock nor source boundaries grants no healing
   or completed rest.
4. Require central `reactions_spent` to be empty in this first bounded slice. A real
   owner's Start may clear that owner's marker before release; release cannot clear
   it to satisfy its own check. Preserve per-rest uses, source recharge availability,
   legendary resistance, own-turn/legendary expenditure and past recharge receipts.
5. Derive every surviving timing dependency from retained effect/group sources,
   targets, concentration owners and recovery records. Before releasing, prove the
   required actors can enter the supported later setup and initiative route using
   its actual pure eligibility checks. Require the same dependency set at replacement.
   A dead Mage with surviving nonconcentration Mage Armor blocks this first release:
   fresh initiative rejects the corpse and no offstage expiry route exists. Reject
   while the old cadence, geometry and session remain usable; do not release first
   and discover the dead end during setup.

Only after the full scan succeeds may the transition retire the central timing,
invoke the existing effect/source `LeaveCombat` operations, preserve permitted spent
turn history, reset only retired flow budgets and mark Finished. Those helpers clear
cursors and can clear a source routine; they must never be invoked to make a failed
preflight appear quiescent. No source resource becomes available until its existing
real own-Start/rest rule says so.

## Application, replay and desktop integration

- Admit release through the trusted host channel both during an actual active
  session and after an already quiescent aftermath session has closed. For the latter,
  require an explicit no-active-session envelope with no invented binding or actor.
  Apply the same owed-work/dependency scan in both cases.
- Current tactical table dispatch requires `active(...)`, and semantic recovery
  separately rejects tactical table events without a session. Both need the same
  narrow typed release/approved upgrade exception. Ordinary actor decisions retain
  their current session/controller requirements. Preserve the exact nested event,
  command audit, transport handle and trusted issuer checks.
- Active aftermath continues to require all retained owners for normal resume.
  `require_aftermath_attendance` currently selects every marked flow regardless of
  phase, while retained dead-PC attendance is admitted only while Active. Give
  authenticated Finished a separate session/start/setup policy based on actual
  remaining dependencies and new placements. A dead owner with no owed work need
  not return solely for host timing release or a different settled encounter.
- Finished validation must not take the current inactive early return before
  authenticating retired ground items, origins and source attachments. Collect the
  new release, predecessor and original drop origins through codec, preflight,
  snapshot and semantic history validation, including retired images.
- Permit source/equipment preparation against authenticated Finished while keeping
  one-time starting-grant protections. Reuse real surviving source and item identities.
  New placement requires genuine controller attendance and current source eligibility;
  dead actors do not enter initiative. Geometry is an explicit accepted GM placement
  ruling, not elapsed travel, rest or automatic item transfer.
- The actual host desktop offers release only from its real capability, explains the
  retained consequences, and exposes fresh battlefield/session controls after success.
  Keep hidden dependency reasons host-only: unrelated player DTOs, revisions and
  transcripts cannot reveal hidden actors or blocker counts. Uncertain outbox retries
  retain exact accepted command bodies across restart and later setup.

## Genuine flow 4 compatibility gate

The original MM run at `0f43823a848a2aeb055bc9c59677843b6e60a012` produced
real file-SQLite candidates from source creation, owned casting and physical inputs
before its authorized interruption. None of its four scenarios completed.
Root authorized consistent read-only online backups outside the repository before
test cleanup. They are **raw candidates**, not official exports, accepted captures,
passing runtime evidence, or fabricated checkpoint history. Their per-file manifests
record actual original path, source binary, copied sequence/stage, hash/size and UTC.
Later checkout `e0adf80` does not relabel the binary that generated those states.

Before flow 5 behavior changes are accepted, complete original scenario verification
and the official original-source export/baseline procedure for:

- selected missile Shield before payment, including a second selection after the
  first real Shield; private collection and exact original receipts;
- partial amounts and the complete face barrier before any impact;
- partial impact and a genuine pending concentration child before the final singleton;
- a flow 4 ordinary hit-Shield pause and an ordinary committed spell's first raw
  request. The existing flow 3 corpus does not prove these flow 4 boundaries.

Use `export_campaign`/`CampaignExport::to_json` under unchanged verified source on
the preserved database. Write each official JSON once; freeze its exact bytes/hash,
pre-tactical anchor, original bindings and provenance. Under that original source,
restore to an independent file, close/reopen/resume, retry every original request
with exact response bytes, and reject changed bodies with complete no-write checks.
Then prove the appropriate old continuation under flow 5 without inserting new
reaction windows, reordering old work, relabeling execution versions or rewriting
source programs. No capture hooks enter production and no event/state JSON is
hand-built to replace an absent genuine stage. Missing cases remain blockers to
that compatibility claim, not permission to fabricate them.

## Required production acceptance

1. **Two genuine encounters.** From actual source Mage Armor, spent limited uses,
   Second Wind and ammunition, conclude and satisfy the real retained cadence.
   Release, close/reopen, prepare a second scene, physically roll initiative and
   perform an actual action. HP, inventory/equipment identities, concentration,
   source counters and the original eight-hour deadline remain correct. Use the
   same source creatures rather than creating replacements.
2. **Turn and Reaction boundaries.** Use Savage Attacker in the first encounter,
   then exercise a distinct global turn including an off-turn attack before its
   owner's next own Start. Reject forged/reused epochs and overflow. A spent
   Reaction blocks release until the real owner Start. Real source recharge runs
   only at the proper new own-Start boundary, with actual physical raw input when
   required; release, setup and another actor's turn grant no refresh.
3. **Physical custody.** Actually throw/drop equipped items and trigger an
   unconscious held-item drop. Preserve old scene positions and original causes
   through release, replacement, restart and portable restore. Reject duplicate
   placements, altered old geometry/custody and automatic movement onto the new map.
4. **Denied release remains playable.** Cover every scanner family, including global
   Inspiration transfer, paid attacks/Ready, raw/selected/missile children, source
   pending work, active and suppressed relative/Turn/per-turn rules, dying/stable
   recovery, spent Reaction and dead-Mage Armor. Compare the complete durable store
   before/after rejection and then continue the genuine outstanding work normally.
5. **Closed session and authority.** Release an actually closed settled aftermath
   with no invented attendance. Start the next session without an unneeded dead-PC
   owner. The same action must reject a closed state with an owed choice/dependency.
   Cover foreign actor, player-issued host action, stolen role/handle, stale revision,
   changed command body and missing prerequisite admission with no writes.
6. **Recovery at each material boundary.** At upgrade, release, Finished session
   rollover, replacement, physical initiative/source-start pauses and second action,
   use actual file SQLite, independent portable continuation, cold public resume,
   original exact retries and later ownership/session changes. Mutate both current
   and authentic earlier snapshots, retained origins, predecessor highwater,
   scene-space records, projection handles and accepted bindings. Failed imports
   leave all application tables at their original empty baseline.
7. **Actual UI and release build.** Desktop controls/outbox tests and a packaged
   session/restart path must demonstrate that the user can reach this route. Schema
   definitions, pure reducers or isolated scripted mechanics do not substitute for
   this production evidence.

## Planned slices and verification

### Remaining real application scenarios

The existing release case already authors Mage Armor, Second Wind, thrown-dagger
custody, reduced attendance, physical second initiative and source movement. Its
dagger is purchased in real character creation by the focused follow-up
`02715793fa365c3d9d56a9036072c31d241414c0`. Do not substitute that case for
the following additional witnesses. All four are authored; their source-qualified
runtime results and outstanding reruns are recorded below:

1. **Savage across the encounter boundary.** Reuse
   `table_savage_cases::prepare` and its actual `SubmitSavageAttacker` construction,
   based on `table_attack_cases::prepare_at`. Purchase the dagger normally. After
   the first genuine Savage roll, conclude and release; give the same Goblin the
   first turn in the next encounter. Reuse the ordinary opportunity-decision path
   from `table_oa_concentration_cases` and `table_hit_driver` when that Goblin moves
   out of the PC's reach. Prove Savage is available on this distinct global turn
   before the PC's own Start, its marker advances, and the actual spent Reaction
   blocks release with complete store equality. End the real Goblin turn, prove
   the owner's Start resets only its Reaction, then use Savage on the owner's new
   turn. Keep the source target alive until the needed witness, using actual low
   physical results and the ordinary knockout decision if lethal damage is reached.
2. **Spent ammunition and unconscious custody.** Reuse source creation from
   `table_attack_cases::prepare_at` plus the real DoffShield and two-handed
   `bow_action` pattern in `table_shield_cases`. Fire a genuine source shortbow
   shot, proving the same ammunition stack falls from 20 to 19; leave the bow held.
   Reuse the actual melee/knockout path from `table_attack_cases::check_knockout`,
   including any real movement/turns needed to reach the target. At release preserve
   the unconscious source's bow drop position and original damage cause, spent
   ammunition, item identities and recovery/rest evidence. A second setup must
   retain the required living source while leaving that bow on its retired scene;
   initiative may interrupt the source's rest only through its existing real rule.
3. **Dead owner omitted after release.** Reuse
   `table_medicine_cases::prepare(false)` and the actual death-save sequence in
   `table_aftermath_cases::dying_scenario`, with physical 1 then 2 and real intervening
   turns. Factor only necessary setup/commands rather than repeating unrelated
   session-rollover assertions. Before death, release refusal must leave the whole
   store unchanged and the actual death-save route usable. After death and settled
   timing, finish without reviving or clearing evidence, close the session, start
   with the surviving owner only, and prepare/act with survivors. Reject adding the
   dead PC to initiative; retain its death rolls and corpse/item consequences.
4. **Recharge and the dead-Mage deadline trap.** Adapt the genuine source creation,
   area targeting/raw save workflow and legendary declines already exercised by
   `table_area_cases` and `table_casting_cases`. Use a PC, Mage and Adult Red Dragon;
   the pinned catalog gives the dragon a 17d6 fire breath and the Mage 81 HP. Cast
   real Mage Armor before the breath. From one genuine pre-breath export, use two
   independently restored file branches: low physical damage keeps the Mage alive
   for the successful release/recharge witness; 17 physical sixes and a failed
   actual save kill it while its nonconcentration armor deadline remains. The latter
   must refuse release before retiring timing, preserve the complete store, and
   still permit its old cadence. On the successful branch, reach a real later
   dragon Start, report a failed recharge die, and release while availability is
   false. Release, replacement and another actor's first turn must retain that
   false value and its complete prior RecordedRoll proof. Only the dragon's new
   own Start may request the next physical recharge die and refresh on success.

Use the existing `durable_step` pattern at material release/setup/initiative/source
raw boundaries: independent file export/restore, cold public resume, original exact
response retries, changed-body refusal and complete store equality. Share bounded
helpers where this avoids duplicating setup; never edit state or event JSON to
manufacture HP, Savage markers, pending work, recharge or custody.

The current supported table route does not appear to remove a participating actor
with a surviving effect into an offstage state. A genuine omission attempt at
replacement is already authored for the living Mage and must write nothing.
Keep raw offstage/suppressed scanning in explicitly isolated rule tests; do not
relabel those as real app history. Investigate any supported offstage-producing
route before adding that positive-history claim. This limitation leaves the named
offstage timing workstream inside Gate 4; it does not waive the global scan or
permit unsupported release.

### Verification sequence

1. Root reviews this plan and its narrow closed-upgrade proposal (approved). Recheck source
   pins and parent movement, finalize the additive records/version contract, and
   update the relevant ADR/coverage ledger with explicit compatibility rationale.
2. In parallel with static implementation, complete the genuine flow 4 export and baseline gate under the parent source;
   import frozen artifacts byte-exact with provenance and meaningful continuation
   cases. Missing ordinary hit/spell states require genuine original-source runs.
3. Implement pure release/dependency validation plus authenticated flow 5 records
   and a real release transition. Add focused rule tests for every actual failure
   mode, highwater and ownership, with no broad copied implementation tests.
4. Integrate narrow closed-session admission and replay, independent Finished
   validation, scene-space custody and atomic replacement. Add real two-encounter
   application/recovery scenarios and preserve prior genuine corpora.
5. Implement actual host session/setup controls and exact outbox behavior. Run
   focused Rust/application and desktop checks only in root's assigned heavy slot;
   use one build job, disabled incremental compilation and default Rust stacks.
6. Normally reconcile verified parents/cache prerequisite. Review the full exact
   diff, run `./scripts/verify-fast` while iterating and canonical `./scripts/verify`
   before completion, then all six final-head CI checks including native Windows,
   declared MSRV, frontend checks/tests/build and fresh executable/NSIS packaging.
   Read actual failures, fix their cause and repeat only checks justified by changes.
7. Update durable evidence and PR, perform expected-head protected merge, verify
   full-tree parity and separate post-merge main logs/artifact. Continue remaining
   Gate 4 work; pause only at the accepted gate boundary.

## Current evidence, risks and next action

Planning and read-only audits are complete. Root reviewed the first implementation
at `a5928c2052ec4ba7726d7f93a306cc616afd13a2`. It adds omitted completion/scene records, whole-campaign
release/dependency preflight, highwater-aware initiative/Savage validation and
retained recharge-roll proof outside Active. Focused isolated rule tests cover
conclusion/paid Ready/Reaction boundaries, global hidden work, raw defense effects,
suppressed timing clauses, orphaned Casting, recovery dice/deadlines, old-wire
omission, Finished/replacement source proof, scene custody and setup feasibility.
Those 14 tests were authored, not executed. They are explicitly isolated fixtures,
not genuine app captures or proof of an authenticated release transition.

Root then authorized the authenticated vertical expansion. The new checkpoint
adds `FinishEncounter`, complete preflight before existing LeaveCombat helpers,
immutable completion and item transfer, closed-scene retirement, a narrow host-only
closed-session finish/upgrade route, strict nested-event/origin replay, Finished
session/preparation admission and atomic replacement. Replacement stages its new
scene closed, then moves the selected actors and activates that scene only inside
the accepted rules transition. This preserves the old Finished placement during
prevalidation even when the new location differs. No intermediate state is persisted.

The desktop now offers derived host-only finish capability, closed-session host
administration, Finished setup/session controls and retained-actor placement checks.
That new capability is omitted entirely from flows 1–4 and player DTOs. Fresh Begin
and targeted upgrades now require flow 5; old execution meanings and typed upgrades
to 3/4 replay unchanged, including every old missile continuation. Authored live
test commands use the current executor; frozen JSON histories and original request
bodies were not edited. The mutation-7 regression now directly asserts the exact
latest-completion domain error, independent of old initiative-roll rejection.

Additional authored tests cover the actual rule release/replacement, a genuine
table two-encounter path with Mage Armor, Second Wind, thrown-item custody, reduced
attendance, physical initiative, source movement, independent file restore/cold
resume, exact retries and current/retired semantic forgeries. A separate test uses
the unchanged genuine flow 2 closed aftermath corpus for the narrowly approved
closed upgrade and release, preserving its original response bytes and sessions.
UI tests cover capability/privacy, required placements and uncertain finish retries
after another encounter appears. At this initial source checkpoint the tests had
not run; later source-qualified evidence follows below. This was not final slice
acceptance.

Root and an independent reviewer inspected the complete authenticated checkpoint
`ffc36f89580ac2f2b3551887d7d32de5e0d8d787` and focused dagger correction
`02715793fa365c3d9d56a9036072c31d241414c0`; both reviews are clear at source
level. They cover closed host admission, historical replay, independent durable
retries, atomic closed-scene staging, custody/origins, privacy and UI controls.
Runtime and default-stack safety were unverified at that source-review checkpoint.

Root also reviewed the complete 0271579→c878f9c development integration. Its delta
is the known 11-file cache source/test patch and seven evidence documents, with no
conflicts. The ten cache source/test files not independently changed by release
match 619ada0 byte-for-byte; `tactical.rs` combines only the reviewed immutable
accessor with the existing release changes. The explicit warm-cache tamper tests
are retained unchanged. Catalog/content, frozen history JSON and Cargo.lock are
unchanged by integration. Formatting and staged/working diff checks pass. Parent
CI or measurements do not prove this newly integrated release head.

Early draft PR48 was published at `49895680630470e217b304707ea2e5fcf54d12ac`.
Its actual Linux stable/MSRV and Windows logs found a boxed/unboxed reference
mismatch in the new completion-origin closure; Rust runtime tests never started.
Windows job 108603592771 nevertheless passed 103 UI tests in 17 files, with zero
static errors/warnings and a 140-module build before that compiler failure. This
frontend result belongs to 4989568 only. The isolated root-reviewed correction
`4f38c1d079c43fa12a9a58021e22dc4d0869b3e1` explicitly borrows the boxed
Tactical event with `as_ref()` and was published for fresh CI. No local build had run.

Actual native Windows stable job 108604354546 at literal 4f38c1d compiled and passed
lint, 103 UI tests in 17 files, zero static diagnostics and a 140-module build.
The separate MSRV/frontend job 108604354339 passed its declared compiler check.
It passed all 55 table scenarios in 3012.53 seconds, including the original actual
two-encounter release path, and the genuine closed flow 2 upgrade/finish case.
It then failed `completed_receipt_survives_turn_reset_and_rejects_incoherent_anchor_claims`
at `tactical_movement.rs:994`: that old isolated fixture manually fabricated current
Finished with no authenticated completion history. The production refusal is the
intended new invariant. Packaging did not run after the failure. Linux runtime job
108604355059 in run 36313714772 subsequently completed with the identical fixture
failure (30 movement tests passed, one failed). Before that failure, all 55 table
cases passed in 4207.79 seconds, six Reactions histories including the closed
flow 2 upgrade passed in 37.25 seconds, and five Shield histories passed in
82.26 seconds. Its architecture, genericity and MSRV jobs passed separately;
neither full runtime result is a whole-head pass.

The focused test correction concludes and finishes through actual rules commands,
preserving all nine movement-proof corruption checks and the inactive movement
proof assertion. It also explicitly verifies that the old unauthenticated current
Finished fabrication rejects for its missing completion before taking the genuine
route. The initial genuine-route correction passed 1/1 on the default GNU test
stack (0.05 seconds after 2m06s compilation). At reviewed exact checkpoint
`96b4faf3423da32edb3fe6cb4c9d498db82290be`, all 31 movement tests passed in
2.07 seconds and all 52 turn tests, including 15 release cases, passed in
6.54 seconds. The explicit refusal assertion therefore also ran. Logs and exact
source metadata are retained outside the repository as
`tooling/release-96b4faf-movement-turns.{log,json}`. No production guard was weakened.
Root released the heavy slot after cache 619 canonical completion for focused local
release verification with jobs 1, incremental 0 and RUST_MIN_STACK unset. The four
new application families compiled in 3m00s and ran in a separate focused serial
table-loop invocation from exact 96b4faf. Custody, recharge/dead-Mage Armor and
Savage/Reaction cases pass. The death case reached the real second raw save, then exposed
a test-only assertion expecting three retained failures after death; existing
`tactical_damage::kill` correctly clears both counters, and authoritative validation
requires that settled representation. Reviewed correction
`703e1ff1dcbedd96176ec46e5df15d4fdd24a331` asserts HP zero, both counters zero,
dead mechanics and Dead world/character state while retaining the actual natural
1→2 roll history, two-failure intermediate state, provenance and all later guards.
The complete original run remains recorded as three passes and one failure in
2233.27 seconds, with zero ignored and 55 filtered tests. Its original binary SHA
was recorded before rebuilding. No passing result is substituted for that failure.

The corrected death family then passed 1/1 in 495.09 seconds on clean exact
703e1ff, after 40.51 seconds of compilation, with zero failures/ignores and 58
filtered tests. This executed the complete survivor-only session, dead placement
refusal, new global turn 10 and surviving actor movement, corpse/item preservation,
both original death-save response retries and changed-body whole-store refusals.
The only source change from 96 to 703 is the reviewed test expectation above.
At that focused checkpoint all four additional families had actual passes, but no
complete canonical or CI pass. They were absent from then-published 4f38c1d.
The subsequent publication and complete CI results are recorded below. No
production rule or refusal assertion was weakened.

External source-qualified logs/metadata are
`tooling/release-96b4faf-four-app-families.{log,json}` and
`tooling/release-703e1ff-death-r2.{log,json}`. Both runs used one build job,
incremental compilation off, serial test execution and RUST_MIN_STACK unset.
The earlier Linux/native draft CI results are separately summarized in
`tooling/release-4f38c1d-ci-runtime-results.json`.

The additional Savage checkpoint authors the first real scenario above.
It reuses source creation and the initial genuine critical-hit helper, purchases
one dagger, reports two physical ones for the first Savage damage, and retains
the actual Goblin's 10→5 HP change. The first receipt ends at global turn 2; the
same Goblin starts the next encounter at turn 3. A real movement crossing offers
the PC's opportunity attack before its own Start, with physical Savage damage
leaving the Goblin at 1 HP. Its actual host Dash using the ordinary speed grant
settles retained walking continuity while leaving the PC Reaction spent. Independent
review found the missing continuity step; the test now checks both facts rather
than weakening the exact blocker assertion. The Reaction blocker is asserted after explicit
conclusion and with no central work remaining. Only the real next Start at turn 4
clears that Reaction, retaining the prior Savage marker until the next actual
Savage roll. Its lethal melee result is resolved by the normal knockout choice.
Independent file restore/resume surrounds material boundaries; full-store refusals,
old receipt retry bytes and retained raw roll history are checked. This source
initially had no passing runtime result; its focused pass is recorded above and
its two complete published-head runtime passes below.

The following local custody checkpoint also authors a genuine file-SQLite path.
The source Goblin doffs its actual shield, waits for its next real turn, and fires
its source shortbow with a physical natural 1, reducing the original arrow stack
from 20 to 19 while leaving the bow held. A purchased PC dagger's real critical
hit and two physical fours reach the ordinary knockout choice against the source
10 HP. Choosing knockout leaves 1 HP and drops the bow at its actual old position,
with the accepted knockout command as the original cause. Release preserves all
items, mechanics, knockout/rest records and clock; omission of that living recovery
dependency from replacement is refused with complete store equality.

The second real Begin interrupts the source's rest through the existing rule,
retains the original knockout cause, and requests its actual disadvantaged
initiative dice. After physical initiative, a real PC Medicine action and physical
20 end the unconscious condition at 1 HP. The source remains prone until its own
accepted StandProne. An attempted attack with the retired bow refuses without
writes; the awake source instead draws its still-owned scimitar and makes an actual
attack. At every material boundary the original arrow stack remains 19 and the
same bow remains at its old scene position, absent from the new map and loadout.
Independent file restore/cold resume, original bow-response retry bytes and a
changed-body whole-store refusal are included. This test/helper-only checkpoint
initially had not compiled or run; the source-qualified focused and complete CI
passes are now recorded in this plan. It changes no production rule.

The third local family reuses the real source attack from
`table_medicine_cases::prepare(false)`: a Goblin critical with physical 5 and 5
reduces the real 12-HP PC to zero after the normal-damage choice. Both live and
already-closed aftermath release refuse with the precise dying-actor preflight
reason and complete store equality. The closed state still resumes normally with
both owners; actual turns and owner-submitted death saves of 1 then 2 produce
a third failure and a real dead character, with settled death counters cleared.
Release also refuses during the actual
pending raw save. The dead participant's bound owner ends its retained turn at
the proper cadence, leaving the old encounter settled at global turn 9.

After closing that session, survivor-only attendance still refuses until the
host performs the explicit no-session finish. That command leaves both session
tables unchanged. A new real session then binds only the survivor; replacement
with the dead PC explicitly refuses at the dead-character guard, while the actual
survivor/Goblin setup proceeds. Physical initiative starts at global turn 10 and
the surviving PC takes a real movement action. The corpse's world/mechanical and
character records, owned/carried items, old scene presences and complete raw-roll
history remain unchanged. Both original death-save response bytes are retried
after the owner is absent and the session/encounter has changed; changed physical
bodies refuse without any durable mutation. Material boundaries use independent
file restore and cold public resume. This was test/helper source only at its
authoring checkpoint; its first runtime failure and correction are recorded above.
The corrected scenario is included in the published-head passes below.

The fourth local family creates a real Mage and Adult Red Dragon, casts actual
Mage Armor, and exports the settled dragon turn before its first breath. Two
independently restored file databases continue those same original commands.
The dragon's source initiative modifier is +12; physical faces explicitly avoid
ties while putting the Mage first in encounter one and the PC first in encounter
two. The accepted cone geometry reaches only the Mage. Its source 17d6 damage and
actual failed Dexterity save produce either 64 HP from seventeen ones or death
from seventeen sixes; neither branch edits HP, dice, effects or source counters.

The dead-Mage branch retains the exact nonconcentration, defense-only armor effect
and original eight-hour deadline. Both live and closed-session finish refuse with
the actual unplaceable timing-dependency error and complete export equality. A
real session resumes the same old encounter and advances to the surviving PC's
next turn, preserving the corpse and its raw effect. The surviving branch reaches
the dragon's real turn 5 Start, reports an actual failed recharge die of 1, and
releases with breath availability false. Finish, replacement, requested initiative
and the PC's first global turn 6 preserve that source state and the complete old
RecordedRoll. Only the dragon's real Start at turn 7 requests a new die; physical
6 then refreshes the breath while retaining the old failure's full proof. Any
actual source legendary windows are individually declined through their normal
commands. Material transitions use independent file restore/cold public resume,
original response retries and changed-body full-store refusals. This additional
test/helper source initially had only formatting and whitespace checks. Its full
scenario passes in the original 96b4faf focused run and the two published-head
runtime jobs below.

### Published 75991dd verification checkpoint — 2026-09-28 review

Root published the reviewed test/evidence checkpoint as
`75991dd6c1e782b8fe42257bcb65c26a8d0f3997`. Fresh inspection of the actual
job logs, workflow metadata and artifact API confirms all six CI jobs passed.
Linux run [36320964281](https://github.com/idiotswill/DMd/actions/runs/36320964281)
checked out synthetic merge `514e5ad4912295d6e6401260c90a89477e267486`.
That fetched commit has parents cache main
`046109cdc849c16100c42588e771f8abe710c787` and the literal 75991dd source.
Both source and synthetic merge have full tree
`5e5f36c03a22e36fc25e7cc9622b1819fd138953`; their complete file delta is empty.
This establishes the source tested by Linux without calling its synthetic SHA
the literal branch checkout or treating it as normal parent reconciliation.

- Linux runtime job **108624568916** passes fast verification, strict all-target
  Clippy and the complete workspace: **768 Rust tests across 55 result groups**,
  zero failures or ignored tests. All **59 table scenarios** pass in **6232.06s**.
  The six Reactions histories pass in 47.28s, including the genuine closed flow 2
  upgrade/finish; five frozen Shield histories pass in 105.30s. All 31 movement
  tests pass in 0.52s and all 52 turn tests in 1.82s.
- Linux architecture job **108624568731** passes all eight guard tests and the
  boundary scan. Genericity job **108624568912** passes. MSRV job
  **108624568855** passes the complete all-target check on Rust 1.88.0 in 50.43s.
- Windows run [36320964277](https://github.com/idiotswill/DMd/actions/runs/36320964277)
  checks out literal source 75991dd in both jobs. Stable job **108624568286**
  passes formatting, strict desktop lint and the native workspace: **770 Rust
  tests across 55 result groups**, zero failures or ignored tests. All **59 table
  scenarios** pass in **3541.41s**; six Reactions histories in 26.75s, five Shield
  histories in 88.27s, 31 movement tests in 0.32s and 52 turn tests in 1.06s.
  Windows MSRV job **108624568472** passes the Rust 1.88.0 workspace/all-target
  check in 28.55s. Both jobs pass **103 UI tests in 17 files**, Svelte checking
  with zero errors/warnings and a **140-module** frontend build.
- Both runtime logs contain literal passing results for the original two-encounter
  release case and all four new real file-SQLite families: Savage/off-turn Reaction,
  spent arrows/unconscious bow custody/first aid, actual death/survivor-only play,
  and actual failed-then-successful recharge with the separate dead-Mage Armor
  refusal history. The genuine closed flow 2 case passes separately. These results
  replace neither the recorded earlier failures nor the missing flow 4 corpus.
- Windows stable completes fresh optimized application and offline NSIS packaging.
  Artifact **10933931692**, named
  `dmd-windows-75991dd6c1e782b8fe42257bcb65c26a8d0f3997`, is **232261406
  bytes**, created 2026-09-27T14:14:14Z. The upload log and artifact API agree on
  SHA256 `9f7a2baeecc09dc39af9e575069842cc9dbce19d00d4e6f45a823a1f28cc6468`.
  This checkpoint verifies creation and metadata, not a local archive download,
  internal file-manifest check, installation or packaged release play.

The independent review covers the complete 4f38c1d→75991dd source delta:
eight test/helper files and this evidence plan. No production code, dependency,
catalog/content, frozen JSON history or original request body changes in that
delta. The movement correction retains all nine corruption checks and the
post-Finished corruption check, and adds the explicit unauthenticated-Finished
refusal before genuine Conclude/Finish commands. The death correction matches
the unchanged `tactical_damage::kill` representation while retaining physical
1→2 saves, their accepted provenance and the intermediate two failures.
Review of all four complete families finds actual creation/actions/physical dice,
independent file restore plus cold public resume, exact response retries and
complete export equality on changed-body refusal (only export time normalized).
No source defect was found in this delta; no local build was started for this
read-only CI/source review. The heavy slot remains assigned to root.

External evidence is retained as `tooling/release-75991dd-ci-<job-id>.log`,
`tooling/release-75991dd-ci-evidence.json` and
`tooling/release-75991dd-review.md`. This is a successful **development
checkpoint**, not final release or Gate 4 acceptance. The final integrated head
still requires canonical verification, all six checks, actual packaged play,
protected merge and separate post-main evidence after the verified prerequisites
and genuine flow 4 original-source baselines are incorporated.

The original uncached `0f43823` capture producer was stopped by root's instruction
for controlled cache measurements before any of its four scenarios completed.
Nine read-only raw SQLite candidates were preserved; the fifth pending concentration
child was never reached. The watcher was stopped by exact PID/path. Those immutable
files retain their true source identity and candidate-only status. Final official
corpus proof must use a fully verified current/merged flow 4 producer; no interrupted
run or relabelled file substitutes for an original-source baseline.

The old PR44 source is
verified; its main checks and artifact are separately documented in that PR.
PR45 main `a0b12d2d0144a744e3419c1ba69e2d7aac64fd79` is now integrated through
the reviewed cache development parent. Cache main 046109c now has all six separate
post-main checks and fresh packaging passing; its normal reconciliation here remains
required. PR46 acceptance, final parent reconciliation, flow 4 official exports and
original-source baselines, final integrated canonical/CI tests, UI evidence and packaged play for
this slice are still outstanding. No evidence on a parent or older head substitutes
for this branch's exact final verification.

Primary risks are freezing the wrong source version, clearing future/hidden work,
closed-session recovery admission drifting from live authority, dead/offstage timing
stranding, numeric turn reuse, silent Reaction refunds, retired item placement loss,
and a partial setup transaction that abandons the prior encounter. The design and
acceptance matrix above explicitly constrain each one. Any necessary receiving work
stays inside Gate 4; no product requirement is reduced.

First-slice review must explicitly cover orphaned Casting concentration groups,
legacy effect source/target/owner dependencies, due absolute work and the retained
source recharge `last_roll` ↔ `RecordedRoll` proof even outside Active phase.
The first independent review found and fixed a missing unconditional Finished-to-
latest-receipt check: removing both optional history and conclusion, or attaching a
Finished flow to a fresh encounter/newer origin, must still reject without relying
on a Savage marker. Focused negative fixtures cover both paths, including a fresh
scene and later setup origin that pass the older predecessor guards. Replacement
setup/no-flow/initiative structural validation now requires every retained actor in
the actual new placements; only its own validated initiative raw request is exempt
from the pre-release no-raw condition. Aggregate feasibility reserves the route's
required character/owned-source seat when all required dependencies are host
creatures; 100 required host creatures cannot consume all 100 placements. The
authenticated app setup/upgrade/release routes must use these same checks during
the next slice, with real controller/session and semantic recovery evidence.
User-facing controls use “Finish encounter” and “Prepare battlefield”; they
do not expose executor or schema internals.

### Original-source compatibility completion — 2026-09-30

PR46's verified original production main is
`c4d8c34c19b5c92eca789f292f99632a0107d861`. Test-only direct-child capture
`2b839d7fc7c8caa8574304ea6f99020ed491e849` completed all three original
scenarios normally, including continuations and cleanup. Seven official exports
cover selected ordinary-hit Shield, selected missile Shield, three physical faces,
all faces before impact, the first and fifth concentration children, and the first
save of an already committed ordinary spell. Original baseline direct child
`05b43bddade5c400e4efa12fadec1d798d5fc0c6` completed all eight tests,
unfiltered and serial: **8 passed, 0 failed, 0 ignored**, 10897.39 seconds in
the test harness (10899.172 seconds recorded command wall time). Both source
checkouts stayed clean and the producer main stayed pinned until completion.

Root and independent completion audits separately verified all seven export
hashes, clean source/tree, normal exit, exact compiled and preserved executables,
full logs and source provenance. Their SHA256 values are respectively
`782846d644f789d758e703ab4101cfd491b01ec2148903f630396ccce656dc2b`
and `ae536fe85b444588441c28b2fb4ff72429c41523d7a5c12591ad2f31deb2fd59`.
The original baseline log SHA256 is
`2cbe4fd66585ec3daecb3d80934afec93191bf719c45579036582701a6dcbd52`.
Both diagnostic commits are published under `codex/gate4-flow4-capture-c4d8`
and `codex/gate4-flow4-baseline-c4d8` solely for reproduction; their diagnostic
hooks must never merge into production. The earlier interrupted candidates remain
unaccepted historical evidence and are not the origin of this corpus.

The receiving import preserves all seven raw export bytes and protects them with
Git attributes. `fixtures/flow4-c4d8-provenance.json` records source identities,
per-file hashes, commands, normal completion, binary hashes and original versus
workspace-normalized display-log hashes. The companion Markdown explains the
seven boundaries and reproduction. `legacy_shield_missile_v1_replay.rs` preserves
every original test/helper assertion; only its module comment and fixture loader
change. No receiving-head runtime pass is claimed by this import.

UI PR49 subsequently merged with expected-head protection as
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`; fetched main has the complete
tree of reviewed source `65b7606bd486120a1c650e44028e4195763a5b6c`.
Its canonical and all-six source CI results and the qualified, source-equivalent
7289c5e packaged UI play are separately recorded in the UI plan. Literal d88a692
main checks are running; neither a source result nor older native play is relabelled
as a post-main pass.

The bounded import passed independent review (SHA256
`50d2831641a11ae479d1504dd20bc1a2ae01e88d54c79ef6165af6bbdc697a7f`)
and was committed as `134aad3050c4bc09d080c4486f5d42e9883d1dfc`.
The source-verification log attribute permits only the harness's original trailing
blank line while preserving every log byte and all other whitespace checks.
Normal UI merge `4e6c58a80874133bf84896182ee50343ce06be44` combines
release Finished/Aftermath rendering with the UI encounter focus boundary. It
changes no backend or old release test/helper. Normal literal-main ancestry merge
`2e4deb7748e00440b4ff0ba11ef85b687fb46751` retains that complete tree,
`379acf1dc05784a59287aa84b2919a6f6e012897`; its repeated squash-history
EncounterPanel conflict keeps the already combined file byte-for-byte.

Root found one integration-only test fixture mismatch: new UI `TablePosition`
cases still advertised flow4 for current EndTurn/Cast controls, which flow5
correctly hides behind the legacy-upgrade boundary. Only that synthetic current
DTO's executor becomes `EncounterReleaseV1`; commands/assertions and every frozen
old export/request remain unchanged. Independent review confirms the distinction.
No production guard is relaxed. Updated UI, gate and closure plans record the
actual source/native/merge evidence and separate pending post-main results.

The next action at the import checkpoint was receiving-head verification and native
play. Those results are recorded below; the original chronology and prior failed
attempt remain historical evidence, not pending work silently erased from the plan.

### Integrated verification and packaged continuation — 2026-09-30

Verified source is `8c03f9fb0058610fd37c0cfe7762e8b96d658f38`, tree
`642290cd2a38d91a9913c12cde42c4df91c28a65`. Normal parent reconciliation,
the byte-exact flow 4 import and the one synthetic current-DTO correction passed
independent review; the review fingerprint is
`810f49b3fe251d0c2d2be7bdf1937ff569ae3232293396048a8b5dce5a57b45f`.
All twenty inherited backend production files and five genuine release histories
retain their previously reviewed release bytes. The receiving eight-test suite
retains every original assertion. No capture hook entered production.

The unchanged clean source completed `./scripts/verify` normally on Windows GNU
Rust 1.98.1, 17:38:29–20:18:50 UTC: **775 Rust tests, 56 result groups, zero
failures/ignored**, including all eight original flow 4 continuations (2208.75s)
and all 59 table cases (5755.73s). Formatting, workspace/all-target checking,
strict all-target Clippy, genericity and architecture checks pass; architecture
has eight Python tests with one explicit skip, separate from Rust counts. It used
one build job, incremental off, default stack/profiles and a fresh isolated target.
The earlier shared-target attempt failed with 19 missing-interface errors and
exit101 before runtime tests; its log remains preserved. The exact cause is not
established merely because the unchanged source passes in a fresh target.

All six actual CI logs pass at this source:

- [Linux run 36752377825](https://github.com/idiotswill/DMd/actions/runs/36752377825),
  runtime job110013954892: **776 Rust tests/56 groups**, original eight in2388.14s,
  59 table cases in4265.45s. Checkout `8304b42be266726df7d87d178a5e02bc800c7b2a`
  is the synthetic merge of literal d88 main and8c, with the exact8c full tree.
- [Windows run 36752378019](https://github.com/idiotswill/DMd/actions/runs/36752378019),
  stable job110013955697: literal8c, **778 Rust tests/56 groups**, original eight
  in2245.06s and59 table cases in3974.87s. Both Windows jobs pass116 UI tests in18
  files, zero Svelte errors/warnings and141-module builds. Both MSRV1.88 checks,
  genericity and architecture jobs pass.
- Named outputs explicitly include all five real release histories, the closed
  legacy upgrade, all original compatibility suites and all four missile cases.
  The count differences are source-verified platform guards: Linux adds the Unix
  symlink test; MSVC adds three desktop host tests. Interleaved Cargo banners are
  not used to invent per-crate totals.

The separate literal UI main d88 also completed all six checks:
[Linux36749962892](https://github.com/idiotswill/DMd/actions/runs/36749962892)
has747 Rust tests; [Windows36749962753](https://github.com/idiotswill/DMd/actions/runs/36749962753)
has749. These are prerequisite evidence, not post-release-main results.

Artifact11120862111 from the literal8c Windows job was downloaded and independently
verified: archive232430711bytes, SHA256
`95e6af6caef3119f401e7a01c0b4df4b357565f72898e3e0f3cf265775942562`.
All1070 checksummed payloads, manifest and exact six installed source-content files
match. Its portable executable SHA256 is
`2fc692c37e5eef3da3188183ade3142886597abc00a9699978b3ba3baa1bc39e`.
The installer was verified but not installed. Native acceptance below used that
portable executable; it is not an installer or reproducible-build claim.

Root operated the actual existing `Prompt focus QA Sep28` campaign through the
desktop, without editing game state. The observed chronology is:

| Accepted sequence | Actual action and retained consequence |
| --- | --- |
| 46→47 | Open the existing flow4 save; explicitly continue it into flow5. Same actors, HP, geometry, items and spent Mage protective-magic use. |
| 48→49 | Host concludes hostilities. Finish is visibly disabled for the spent Mage Reaction; the player view omits that private reason/ruling. Actual Hag EndTurn reaches Mage Start5, expiring Shield and clearing only that Reaction. |
| 50→53 | Player-controlled Mage casts actual Mage Armor using its original cured-leather component. Its deadline is28806. Actual Rook Bonus Action pays Second Wind2→1; submitted raw d10 face4 resolves with level1 modifier and capped HP11/11. |
| 54→55 | End and save the session, then Host Finish with a real null-session envelope. Old turn controls disappear; first completion ends at global turn6. Normal exit, process absence, same-hash relaunch: exact saved state and every captured table row remain equal at55. |
| 56→57 | Start a new session with the same player/Rook binding and owned Mage. Prepare a bright50×50-foot battlefield with the same Rook, Hag and Mage at(5,5), (10,5), (25,20). Only the accepted new location changes their world placement; old scene remains Closed. |
| 58→61 | Request fresh initiative. Actual prompt order is Rook/Hag/Mage, with raw faces20/1/10 and totals22/6/12. Round1 starts with Rook at global turn7, then Mage, then Hag. The player view omits the Host creature's private score. |
| 62→63 | Rook pays its Action for an Unarmed attack against adjacent Hag. Normal exit/relaunch preserves request `b39ff042-230e-50ce-b301-79c6c9826739`, modifier+5, origin/window `fc1d2556-3f2f-412f-ba7c-48988bb66ccb`, spent Action and exact saved state/tables. Raw1 resolves the same request as a miss, with no damage request or refund. |
| 64→66 | Actual Rook EndTurn clears its attack window at Mage Start8. Host concludes and Finishes during the active session, appending a second receipt linked to the first, final turn8. Player Finished view has no host controls or private ruling. |

World time stays6 throughout. Hag remains93/112 HP, Mage81/81 and Rook11/11.
Second Wind remains1; Mage protective-magic spent1, original source/character
grants, all three physical item identities/quantities/custody/loadouts and the
original Mage Armor deadline remain intact. The first receipt, old scene spaces,
raw-roll prefixes and prior journal/audit/transport/projection-history rows are
retained. Normal app shutdown at21:00:32 UTC ends this native run with no DMd
process left running. The second session remains active with its encounter Finished.

Read-only campaign snapshots corroborate actual native actions; no SQL mutation,
synthetic command injection or state repair was used. Snapshots cover only tables
with `campaign_id`, excluding the separate session-participant table. Attendance
is checked through the accepted StartSession payload, active-session projection
and unchanged character/controller mapping. This is not a whole-database backup or
an independent replay of the native campaign. The separate real file-SQLite tests
provide portable/replay, hostile-state, exact-retry and whole-store refusal proof.
This native campaign has no ammunition or throwable weapon: those additional
custody claims come from the genuine executed release families, not this UI run.
The disabled Finish inspection is not described as a submitted no-write refusal;
no pending-Second-Wind cold restart or pending-die Host Finish check is claimed.

Evidence retained outside the repository under `tooling/`:

| Record | SHA256 |
| --- | --- |
| `release-8c03f9f-canonical-r2-2026-09-30.json` | `3d59ebe027ab619e5e3862c7445360bf5953c2d4d31c254b1d36ae019b10f305` |
| Canonical full log | `7dd6aad70cd4e6e9c19661aada80a2a2c1c24fbc296f8d9df40320f966d907e0` |
| Independent canonical completion audit | `cc584eb892812172ab970cd09b89bd117a45a57678d154437a2152b881017144` |
| `release-8c03f9f-ci/final-evidence.json` | `f813d641bc062471d8ff8393879dd9c549cabd807874aa7e4bf1df473345ed77` |
| Independent package byte audit | `02878f1b129775708d0b3a1c8f339adee6064aec37463c0a1c1c3a659e6a6fa3` |
| Native accessibility observation log, final | `24c85c97b06cf1857c0fdd704858462b4a6549e5081dea93ba4c148c414746f8` |
| Independent20-cut retention audit through66 | `5bc7cd3faa122ed48130eeb92440639fe568ab14389eacb24ff96d6d8a966dea` |
| Independent second-Finished detailed audit | `f47c8dce335a8077aa3a7468838961a2e8bc4fac53097d05ec6a95b04ad6a278` |

Next action: review this documentation-only checkpoint and prove every executable,
test, source-content and fixture blob still equals8c. Verify all required checks
on its published exact head before expected-head protected merge; then fetch and
separately verify literal merged main. Do not relabel8c canonical/native evidence
as a different binary/head's execution. Keep the bounded release limitations and
remaining Gate4 families; do not begin Gate5.

### Protected merge and separate main verification — 2026-10-01

The final documentation head `d4adb1b1a959ea1c4e2032b6512bd33496177afa`, tree
`a0a350cf695cce9cc7aa228931d5b10014bc66ce`, passed complete independent review.
All410 non-document blobs remain exact8c. Its actual six CI jobs succeeded:
[Linux36778662703](https://github.com/idiotswill/DMd/actions/runs/36778662703)
and [Windows36778662617](https://github.com/idiotswill/DMd/actions/runs/36778662617).
Actual totals are776/778 Rust tests,56groups,59table cases, all eight original
Flow4 continuations, all five release histories and all four Magic Missile cases.
The final evidence bundle SHA256 is
`6d7c79180e26e89db813a60ac85a1a5726c9f2bf4a9a937bcc0ed17b5b8f083a`.
Root independently rehashed all six logs before merging.

[PR48](https://github.com/idiotswill/DMd/pull/48) merged with expected-head
protection as `dbf1d633460473183324b4ec519e8d1980884b5c`. Fresh main and the
candidate have exactly equal full trees; no post-review source delta was accepted.
The prior8c canonical run and native package remain attributed to8c, with complete
non-document equivalence established. No native execution of the d4/dbf artifact
is claimed. Separate literal-main runs
[36834974479](https://github.com/idiotswill/DMd/actions/runs/36834974479) and
[36834974592](https://github.com/idiotswill/DMd/actions/runs/36834974592) have four
quick checks passed and both runtime jobs pending at this checkpoint.

Next: audit actual completed main logs, then record that separate result. Continue
the approved Gate4 workstreams; preserve release's limits and remaining gate debt.
