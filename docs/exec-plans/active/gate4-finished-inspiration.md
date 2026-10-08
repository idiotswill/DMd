# Gate 4 — New Host Inspiration after authenticated encounter release

Status: plan before implementation, 2026-10-08. PR76 remains a draft and Gate4
remains active. Root owns `codex/gate4-family-receiver`; implementation may be
delegated to exactly one named writer, followed by independent root review.
Starting head is b62d8e719b4998a401745c41cc7cd9ee3fa681b5, tree
994183da202488546990114e3ff0019deb52c2c5. Fresh origin/main is
c101c202a9d7302e5ac60d791d3d6c5a4ed30e84. No Gate5 work is allocated.

## Objective and contract

Let a genuine new Host excess-Inspiration award after a completed encounter
produce its attending owner's durable choice. Reads, cold recovery and that
owner's decline/gift must remain available. Starting another encounter, changing
session, creating characters or other unrelated mutations must still wait for
the choice. Never erase a choice or pretend it was resolved before release.

This advances product-definition requirements for actual continued play,
player authority, meaningful retained resources and save/exit/resume. Relevant
authority: [product definition](../../product-definition.md), [Gate4](../../checkpoints/gate-04-tactical-encounters.md),
[gate protocol](../../checkpoints/gate-execution-protocol.md),
[Inspiration transfer](gate4-inspiration-transfer.md),
[encounter release](gate4-encounter-release.md),
[historical compatibility](gate4-physical-facts-historical-compatibility.md),
and ADR024/026/028. The transfer contract is session-wide: active session,
attending owner and settled mechanics; it requires no combat turn and spends
no action or time. `source_control::settled` explicitly admits Finished.

## Observed failure and diagnosis

Linux run37776869118/job113310121255 checked synthetic c50daad9 with the exact
b62 tree and failed in `physical_history_decline`: five earlier targets36pass,
then one binding control passed and the genuine continuation failed. All seven
original decline cuts and24 hostile-copy checks passed first, including the
corrected owner receipt and all five post-mass checks. Ground/Recipient and later
remainder targets/doctests were unrun. This is37passes/one failure, not timeout.
Original ZIP SHA256:
1d516e129212fd6631382c53edb6c560c165f0cdb343b34676b5bbe63a50e8fd.
Original full job log SHA256:
eec4068e2cf59b5447aa861b8d029fbbc5d32e0008d2e6928f3cf92485dc3b28.
Independent terminal audit SHA256:
5a33599850ed89ecb33958ba3d265b714ee40afc32b1265147c326abc2c93a19.

The terminal error at shared driver292 says outstanding table/dice/Inspiration
choices must be resolved before finishing. The original log does not print the
request, so it alone does not prove a FinishEncounter or award caller. Root and
independent source tracing identify the later `new_version` excess award after
Finished/mass: it creates the valid Host transfer, then Finished history's strict
release-readiness scan rejects its new flag. The Host Finished projection also
uses that scan. Moving the award into combat would hide the composition defect;
preserve the historical producer order and original corpus.

Root deliberately superseded the separately frozen local b62 canonical run
after preserving this actual hosted failure. The reviewed control stopped only
its identity-bound owned test; the unchanged runner finalized15:58:40 UTC with
stage exit127, exact clean source and self-released lock. Check/strict Clippy and
normal test compilation passed, six whole targets44cases passed, and decline's
binding control passed. Its continuation was interrupted, not a local assertion
reproduction. Later stages are unrun. Terminal result SHA256:
0736697f9a836cb9edd512246756f130073a2c759351749707eb55d0ca9672b9.
Original log SHA256:
a140542369a03954ebedc3866faf86ce9470d54009cb489a64d4f00393dea5de.
All1,008 source working/blob pins remained exact after termination. Preserve
every old log, target, runner, manifest and failure without replacing evidence.

## Bounded correction

Separate Finished-state validation/presentation from admission to a transition.
Only Finished read/validation may admit the single actor from a structurally
valid Host transfer companion whose origin sequence is strictly later than the
latest authenticated release. Reuse existing companion validation, including
actual owner, attendance/session, unique matching flag and no pending table/raw
work. Original owned reduction and journal replay remain the authenticity proof;
state shape alone cannot authenticate a forged award.

Keep `retained_encounter_dependencies` strict. Keep release preflight and
replacement-initiative admission strict; the existing initiative exception
does not admit Inspiration. Add a clearly named read query for the Finished
projection and reuse its exact narrow predicate in Finished history validation.
`require_finished_encounter` must explicitly retain the strict dependency scan,
so its setup/session/creation callers gain no mutation exception. Every other
pending, permission, grip, routine, recharge, effect, recovery, timing, receipt,
epoch and closed-space guard remains intact. Do not clone/clear state for a scan.

Allocated production files: `crates/dmd-rules/src/tactical/release.rs`, its export
in `tactical.rs`, and `crates/dmd-app/src/table_tactical.rs`. Add focused genuine
application regressions through the existing public Grapple/Inspiration fixture;
a separate support module and its registration are allowed. Add narrowly useful
rules controls if needed. Improve the historical shared acceptance panic to
include the actual request/error; change no success or refusal semantics.
Update this plan and receiver/historical status. No schema, wire, execution,
source-content, migration, replay-normalization, workflow, test-runner/profile,
capacity, MR or Offstage change is allocated.

## Acceptance and verification

- Genuine Finished award, owner decline and gift work through actual table
  requests, cold reopen, portable restore, replay and exact retries; original
  completion, clock, resources and unrelated audience privacy remain intact.
- Host and owner views remain readable while pending; another player/Host
  cannot substitute for the owner. Missing companion, stale/pre-release origin,
  extra flag and unrelated pending work remain invalid. Hostile saved copies
  fail without destination writes; valid originals remain recoverable.
- Strict release/replacement/session/creation/raw admission still refuses the
  unresolved choice, then admits the supported next boundary after real owner
  settlement. Preserve all prior test bodies and historical/corpus bytes apart
  from the explicitly allocated failure diagnostic.
- Perform changed-file format checks, independent complete-delta/preservation
  review, then a focused regression run with ordinary GNU1.98.1/jobs1/
  incremental0 settings in the exclusive local slot. This focused result does
  not replace unfiltered canonical verification. Preserve actual failures.
- Freeze the corrected source and prepare/review a dedicated-target canonical
  runner/manifest. Run unchanged `./scripts/verify`, all25 CI controls and ordinary
  npm ci/check/test/build. Reconcile new test counts from actual discovery.
  Preserve ordinary profiles, default stack/threads and existing platform flags.
- Publish the reviewed head and require fresh complete Linux/MSVC unions,
  prerequisites, aggregates and ordinary Windows packaging. Older partial
  successes do not pass it. Combined native, exact-head merge, literal-main
  verification and separate human Gate4 obligations remain mandatory.

## Next action

Implement the bounded production/read-path correction and genuine regression,
independently review all changes and preservation, then verify the actual frozen
head. The historical route still must complete; this diagnosis accepts neither
the family nor Gate4. PR77 retains its own unaccepted head and must later receive
the accepted parent through normal ancestry reconciliation.
