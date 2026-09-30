# Gate 4 — Timed effect expiry at turn boundaries

Status: **All six CI checks pass on corrected source 41d034ce. Development
integration with release8c is planned below; final acceptance remains pending.**

## Objective, ownership and baseline

Repair the supported case where a still-live timed tactical effect prevents normal
turn advancement at its deadline. Preserve every previously accepted history's
operation stamps, expiry tickets, work ordering and durable continuation.

Branch: `codex/gate4-timed-expiry-boundary`. Sole writer: root after the expiry-repair
agent's completed handback. A draft PR targets main. Base is freshly fetched `origin/main`
`c4d8c34c19b5c92eca789f292f99632a0107d861`. The clean, unused historical turn-core
checkout was reused; `codex/gate4-table-turn-core` remains preserved at
`25be7f6c5f31e60a065f009c52ab44c852639aed`. No reset or force operation was used.

Root retains the single build/verification slot and the original-source capture,
baseline, UI, Air, Shove and encounter-release integration work. This branch must
not modify those checkouts, captured histories or immutable content.

## Product and architecture contract

This advances the [product definition](../../product-definition.md)'s complete
tactical timing, player control, explainability/provenance, real production path,
and session start/end and exact suspension requirements. It addresses the
[Gate 4 checkpoint](../../checkpoints/gate-04-tactical-encounters.md)'s durations,
concentration, start/end triggers, ongoing saves and exact mid-combat recovery.
The [gate protocol](../../checkpoints/gate-execution-protocol.md) and
[active gate plan](gate-4-tactical-encounters.md) remain binding.

Relevant accepted contracts are [ADR 018](../../architecture/018-rules-kernel-and-semantic-replay.md)
(pure atomic resolution and semantic replay),
[ADR 020](../../architecture/020-rules-restore-history-validation.md)
(original-anchor and complete history preflight), and
[ADR 026](../../architecture/026-interruptible-encounter-resolution.md)
(durable interruptions, controller-owned simultaneous ordering and historical
semantics). The [effect lifecycle plan](gate-4-effect-lifecycle.md) explicitly
retains suppressed effects and their durations. SRD 5.2.1 timing remains the
pinned rules source; this repair adds no adjudication or house rule.

This bounded repair does not complete Gate 4. Other tactical families and packaged
encounter acceptance remain open. Broad noncombat elapsed-time scheduling belongs
to Gate 5; the already-recorded Gate 4 offstage/no-turn tactical deadline and
aftermath timing obligations remain in Gate 4 and must be preserved. Broad catalogs
belong to Gate 6, living-world simulation to Gate 7, autonomous
language/DM to Gate 9, and voice to Gate 10. No product scope is reduced.

## Source finding and reachability

At the base, `tactical/turns.rs::begin_boundary_from` performs existing Ready and
legacy expiry handling, then observes Time and immediately Turn, before collecting
effect work. `tactical_effects::observe(Time)` queues due AtTime group/target expiry
tickets. Turn refuses a nonempty pending list. Consequently the normal scheduler
cannot offer the newly due work. Domain `expiry_matches` already matches a due
AtTime under Turn as well as Time.

A genuine source path exists: create/control Cultist Fanatic and Mage through the
normal table catalog, cast Hold Person, fail its actual initial and repeat saves,
and retain concentration through ten natural round advances. The source installs
a 60-second deadline. The End-to-Start transition that would reach it queues expiry
under Time, then fails at Turn. EndTurn itself is available while Paralyzed. If an
accepted End was suspended on child work, the command completing that work can be
the rejected clock-crossing command instead. The resolver clone and application
transaction make rejection atomic; earlier accepted costs and raw rolls remain.

This is source analysis, not an executed reproduction. The independent external
memo `tooling/timed-expiry-boundary-independent-audit-2026-09-30.md` outside the
repository contains the reviewed source chain and alternatives. The same sequence
is present in release `44ae95053d5f94d16a64e5f0684c605a4a8b2c87` and integrated
Shove `3f8024bde89f8878bcc42197e42d60169182cceb`; their inclusion is not required
to repair the base.

## Bounded design

1. After the existing Ready/legacy handling, inspect **all raw stored groups and
   effects**, including suppressed entries, for AtTime due at authoritative now.
   Reuse the domain expiry predicate; do not infer expiry from visible conditions,
   active-overlap projections, source names or wall-clock time.
2. If no AtTime is due, execute the current **Time then Turn** sequence unchanged.
   Empty Time operations still carry an operation stamp; they are not removable.
3. If any AtTime is due, observe **Turn once**, omitting the preceding Time only on
   this formerly refused path. Existing Turn observation collects timed expiries
   and matching turn consequences together. Use existing Turn ticket causes and
   normal step allocation; do not fabricate a no-op Time receipt.
4. Keep pending guards strict. Then use the existing shared boundary work frame,
   `new_effect_work` and pump. Simultaneous work remains the current turn owner's
   explicit choice; singleton handling and cancellation stay unchanged. The raw
   roller may differ from that ordering owner and retains its own authority.
5. Preserve End effect work before the separate Legendary window, all End work
   before initiative/time advances, Start cursor/per-turn reset exactly once,
   Ready's current expiry order, and source recharge/death/recovery and nested
   concentration/fall continuation boundaries.

No persisted schema, command, observation or transport variant is planned. Do not
globally remove Time, reverse Time/Turn, resolve expiries automatically before
collecting other boundary work, relax pending guards or add a second scheduler.
A dedicated atomic boundary adapter is a larger alternative only if a concrete
counterexample defeats this design; stop and record that evidence before expanding
scope. No implementation of such an alternative is currently authorized.

## Replay proof obligation

Every previously accepted boundary must retain its complete operations and tickets.
The intended proof is that, after existing preliminary cleanup, any raw due AtTime
record would have queued at least one ticket under Time and failed Turn. The inner
observation does not remove records. Therefore every historically successful
boundary takes the unchanged no-due branch. Suppression does not exempt a record
from this proof.

Review must check this argument against queue deduplication, group/target membership,
pending guards and preceding Ready cleanup. Tests must verify actual historical
operation step/command stamps, ticket IDs/causes, work occurrences, request identities,
source pins, outcomes, projections and receipts, not only final HP/conditions.
Preserve successful history prefixes and all original capture bytes. If a previously
accepted history changes, this design has failed its acceptance criterion; fixture
regeneration or relaxed equality is not a repair.

Independent review correctly required composing that inner observation argument
with the adapter's fixed-point concentration cleanup. Root and the author inspected
the following preservation facts before resuming source work:

- Kernel entry validation invokes `validate_effect_attachment`, rejecting every
  grouped owner who is dead or Incapacitated. Legacy concentration pointers and
  disjoint identities prohibit a simultaneous legacy/grouped binding for an owner.
- `active_effect_views` suppression depends only on retained tactical target,
  overlap and establishment records, not time, pending tickets or legacy effects.
  Time changes no such records. Installing its identical groups/conditions cannot
  newly incapacitate an owner. Legacy expiry only removes conditions; the only
  subtractive condition rule is Petrified suppressing Poisoned, and Poisoned does
  not imply Incapacitated.
- Initial initiative completion only installs timing. At boundary entry, Ready
  expiry removes ordinary declarations; held spell expiry is explicitly unsupported.
  Dodge removal changes no condition. Adding an absent recovery attachment cannot
  introduce Incapacitated; it can only remove legacy HP-zero Unconscious for an
  immune actor. Round/turn advancement resets timing and budgets, not conditions.
- Internal End completion preserves the same invariant: lifecycle expiry/save
  acknowledgments run the fixed-point adapter before returning; vitality work
  immediately handles EndConcentration followups; failed concentration removes its
  group before pump resumes. EndOccupiedSpace adds only Prone. Recharge/Legendary
  hooks change source counters and windows; after-turn cleanup removes Disengage.
  Movement/fall can incapacitate only through the same vitality path. Pump drains
  these children before advancing to the next Start.

Consequently the hypothetical cleanup of all newly due tickets requires invalid
input or a missed invariant-breaking producer, not a discovered accepted history.
No reachable counterexample was found. Root accepted this composed proof and
reauthorized the bounded fix without clone preview or guard relaxation. Focused
tests must distinguish an invalid grouped owner (rejected unchanged at entry) from
actual accepted source histories; original-history replay remains a required check.

## Acceptance and planned slices

1. **Review this plan.** Confirm the due-only branch and unchanged accepted-path
   proof before source changes. Only this document may change at this checkpoint.
2. **Minimal resolver repair and meaningful rule regressions.** Use an actual
   installed timed spell and normal round advancement to cover both immediate End
   completion and completion after an accepted raw/owned interruption. Assert one
   clock advance and ordinary legal expiry work. Add focused invariant coverage
   for simultaneous timed expiry, owner-relative expiry and turn-trigger work in
   both selected orders, suppressed raw entries, group/target cancellation, ticket
   deduplication and unchanged rejection while prior work is pending. Any synthetic
   pure lifecycle fixture must be labeled as such and cannot substitute for the
   real-source path. Verify End effects still precede Legendary windows.
3. **Real persistent Hold Person regression.** Extend/reuse the existing genuine
   table creation/control/cast route in `table_missile_cases`, with file SQLite and
   normal physical failed saves, without injected effects, manual time changes,
   manufactured actors or database edits. Keep the effect alive to the natural
   deadline. Cold reopen before crossing and at the resulting expiry choice; also
   restore an independent export and continue through the same commands. Exercise
   exact accepted retries, changed-body/stale/foreign no-write rejection, selected
   expiry completion and final restore. Compare the original cast cost, all prior
   raw roll objects and durable journal/projection/binding prefixes; concentration
   and Paralyzed clear without unrelated damage or resource changes. Check the
   ordering owner's capabilities separately from target raw-save ownership.
4. **Historical compatibility.** Run unchanged no-due boundary controls, including
   Shield owner-start expiry, Ready, recharge and End/Legendary work. Replay genuine
   original-source histories once root's producer/baseline work is accepted, including
   the seven flow4 captures. Root coordinates their integration; this branch neither
   generates them with the repaired source nor rewrites their fixtures or producers.
5. **Independent full-diff review and exact-head verification.** Resolve findings,
   run the appropriate focused checks and canonical verification in the assigned
   serial slot, update actual evidence here, then let root coordinate exact-head CI,
   protected merge and merged-main verification. No old branch pass is acceptance
   for this repair. Gate 4 remains active after this bounded slice.

Planned focused commands, only after root releases the verification slot:

- `cargo test --locked -p dmd-rules --test tactical_turns`
- `cargo test --locked -p dmd-rules --test tactical_effects`
- `cargo test --locked -p dmd-rules --test tactical_attacks`
- `cargo test --locked -p dmd-app --test table_loop` with the new case's focused
  filter during iteration, then the relevant existing controls.
- Root's unchanged original-source baseline/compatibility target after integration.
- `./scripts/verify-fast`, then `./scripts/verify` on the complete reviewed source.

Use the root-configured toolchain and one compiler. Do not start another build,
change test stack limits, weaken source/persistence assertions or edit immutable
captures to obtain a pass.

## Validation, risks and exact next action

Completed: source-only audit, product/ADR/checkpoint review, clean unused-checkout
check, fresh main fetch and exact-base branch creation, root plan/proof review,
minimal due-only selection in `turns.rs`, and seven authored regressions:

- Five explicitly synthetic turn/lifecycle cases cover no-due stamps and exact
  ticket/raw-request identity, group-only deadlines, suppressed raw deadlines,
  simultaneous timed/owner expiry and suppressed damage in both selected orders,
  and unchanged pending guards.
- One source-built Hold Person rules case uses two actual initiative orders to
  cross 60 seconds either on EndTurn or the final owned repeat-save result. It
  preserves paid use/old dice, checks target versus ordering authority and group
  cancellation, and rejects an explicitly forged incapacitated group owner before
  restoring the genuine test image. The existing source setup helper is factored
  before initiative; its historical default call sequence remains unchanged.
- One real file-SQLite case uses unchanged normal catalog creation/control helpers,
  a real Cultist casting on the real Mage, actual +4 Wisdom saves with physical
  ones, and ten ordinary round crossings. Authored cold steps restore independent
  files at initial save, last repeat, deadline boundary and expiry selection, with
  accepted retries and changed-body/no-write controls. Earlier journal/audit and
  projection prefixes, each UUID-sorted binding, paid source counters and raw rolls
  are compared; foreign player, wrong controlled actor and stale input are refused.

Owned Rust files passed rustfmt parsing/formatting, including recursive module-path
resolution, and `git diff --check` passes. These are static checks only. No compiler,
test, npm, build, UI, database inspection, push, PR or merge was run for this repair.
The original-source capture/baseline checkouts and histories remain untouched.

Risks: a raw due record missed by the predicate would preserve the bug; unconditional
Time removal would change accepted provenance; expiry-first automatic cleanup would
steal a simultaneous choice; an imprecise app fixture could end Hold Person early
and never exercise the deadline. Rule-only success would not establish file-SQLite
retry/restore correctness. Source analysis does not establish full Gate 4 acceptance.

### Reviewed draft verification checkpoint — 2026-09-30

Root and an independent reviewer inspected the complete eight-file diff at
`6e601fef2eea066d057d57bd029ab661d1a42f4b`, tree
`b323fd74cfa7f6bbcaa0721c7f0fe5ee26b2d16b`. Both found no actionable static
defect in the due-only change, the composed accepted-history proof or the seven
authored regressions. External review evidence is retained as
`tooling/timed-expiry-6e601fef-independent-review-2026-09-30.md`, SHA256
`43eaf0dfdf553465fea17e825b32bf1b0371c155e98f0a7f18cfc91be0528248`, and root's
separate review. This does not establish compilation or runtime behavior.

Root authorizes publishing the reviewed source plus this documentation checkpoint
as a draft so remote CI can run while the original-source baseline retains the
local heavy slot. This supersedes the earlier push hold only. No local compile,
test, npm or build may overlap that baseline, and main remains fixed until its
complete proof succeeds. Original captures are neither regenerated nor imported
before their baseline acceptance.

**Next action:** inspect actual remote results and fix concrete failures, then
assign the local serial verification slot for the focused and canonical checks
above. Original-source replay, final exact-head CI, protected merge and separate
merged-main proof remain acceptance prerequisites. Draft publication, source review
and authored tests are not acceptance or Gate 4 completion.

### First executable finding — 2026-09-30

Linux runtime job 109938945831 on published `95e41edf58517c1770224d2a78aedafb1b4cd932`
passed fast verification, strict Clippy and compilation, then failed all five
unchanged `legacy_reactions_v1_replay` tests at restore sequence 12 with
`missing effect attachment`. The full log is preserved externally under
`tooling/expiry-95e41ed-ci/`. The suite stopped there; neither the new regressions
nor full compatibility passed. The initial source reviews missed this boundary.

The new predicate called `effects(state)?` before the historical Time operation
could initialize an absent legacy attachment. Read the optional attachment without
mutating it instead: absence contains no due records, so it follows the exact old
Time-then-Turn path. The existing adapter's `unwrap_or_default` still owns the
initialization and original stamps. Do not eagerly insert an attachment, change old
fixtures or bypass restore checks. The no-due focused case now explicitly asserts
its initial attachment is absent before checking the original two operation stamps.

The corrected source needs independent review and a fresh full run. All five
genuine historical failures must clear unchanged; formatting or static review
alone is not recovery evidence. The original flow4 baseline remains untouched.

### Corrected-source results and integration plan — 2026-09-30

All six checks completed successfully on corrected source
`41d034ce12eb2861c811441cbe031f20488827ae`, tree
`80044b602be5b8b0417182cda804d91f72ac5f70`. Linux runtime job 109941663863
in [run 36731400969](https://github.com/idiotswill/DMd/actions/runs/36731400969)
passed 754 Rust tests across 55 result groups, including all 55 table cases in
5535.55s. Windows stable job 109941770012 in
[run 36731401000](https://github.com/idiotswill/DMd/actions/runs/36731401000)
passed 756 Rust tests across 55 groups and all 55 table cases in 6358.63s.
Both passed the unchanged five Reactions histories, the actual timed Hold Person
case and all four Magic Missile cases. Windows also passed 99 frontend tests in
17 files, zero Svelte errors/warnings and a 140-module build.

The original missing-attachment failure and corrective independent review remain
preserved. Full corrected logs and API/tree evidence are retained externally in
`tooling/expiry-41d034c-ci/final-evidence.json`, SHA256
`ed84a1d1b552b61cd0fca38d3f467cd82aedff8746fa6aa3e307716560a02b49`.
Linux synthetic `4b11f802f09ef8a9f6bd55f71f48bdeb0c71205d` has parents
c4d8/41d034c and exactly the corrected source tree. Windows used literal 41d034c.
Artifact 11112627704, SHA256
`efdaff1499d85aff7318b3177ef80ac016e1ac26094b96085d4c4f59efa5503e`, was uploaded;
it has not been played. These results do not validate a later combined head.

The genuine original flow4 baseline has now completed all eight cases. UI PR49
merged at fetched main `d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`.
Root owns this bounded integration and commits this plan before changing code:

1. Normally merge release `8c03f9fb0058610fd37c0cfe7762e8b96d658f38`, which
   already includes main/UI and the exact preserved seven-export, eight-case
   flow4 corpus. Release remains an **unaccepted development dependency** while
   its canonical and native acceptance continue. Its first shared-target compile
   failure is preserved; the fresh isolated check and Clippy passed on unchanged
   source, and its runtime tests remain in progress.
2. Preserve the due-only repair and exact no-due initialization/stamps, release
   completion/highwater and all current flow5 admission and old-pause semantics.
   Retain every original fixture and replay suite exactly. Update only newly
   authored current test setup if the combined current executor requires it.
3. Inspect all conflicts and compare combined production/test changes against
   both reviewed parents. Obtain an independent static review before publishing.
   No second local heavy run, database/native action or evidence regeneration is
   part of this integration.
4. Publish the reviewed candidate for new exact-head CI. Schedule canonical and
   receiving original-flow compatibility in the serial verification slot. Resolve
   actual failures without weakening assertions; protected merge and literal-main
   checks still require final dependency and slice acceptance.

Offstage/no-turn tactical deadlines and all other Gate 4 work remain open. This
integration does not add a second scheduler, alter wire formats, or advance a
coverage-ledger family merely because prior checks passed.

### Local integration checkpoint

Plan `e554f96` preceded normal merge `af87b98` of exact release8c. The only
textual conflict was the `tactical_turns.rs` module list; it retains both release
and timed-expiry suites. The auto-merged missile parent retains the timed-expiry
child and release's current Begin. The one additional current rules-test Begin
in `casting_timed_expiry.rs` now requests `EncounterReleaseV1`; all actors, raw
faces, costs and assertions are retained. No captured historical body changes.

The sole production difference from release8c remains the exact corrected
`turns.rs` implementation from 41d034ce. Release code, UI and all genuine fixture
bytes/suites remain inherited without alteration. This records an integration
candidate, **UNCOMPILED/UNRUN** on the combined source. Independent review, fresh
checks, local canonical/receiving replay and dependency acceptance remain the
next actions; the corrected parent's CI is not evidence for this new tree.
