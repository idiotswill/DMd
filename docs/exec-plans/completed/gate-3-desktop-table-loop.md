# Gate 3 — first playable desktop table loop

Status: **Completed — Gate 3 technical implementation and native acceptance.**

## Completion and handoff

Foundation PR #20 merged at `ac900fb5c5603f26bd3f3108aecf81bf597eadae`; desktop PR #21
merged at `998eedea92da8aab288e1c586eb4a3e406c74b4c`, matching reviewed/tested head
`05ff272d16ab7d893e0c39950f07bd5176c82e3e`. Full verification passes 207 local Windows/GNU
tests and 208 Linux tests. Windows run `36025008694` passes MSRV/stable checks, native
host/frontend/notice tests and fresh packaging. All 1,068 final artifact checksums and
the dependency/SRD notice audit pass. Independent source review found no remaining blockers.

The packaged native scene covered creation, agreement, character/attendance, questions,
correction, raw dice, ambiguity, graceful/crash restart, duplicate launch, campaign isolation,
session end and new-session continuation. Final build reopened the pending Second Wind roll,
resolved raw 4 + 1 once with 1/2 uses retained, closed/saved/reopened, and started Beyond the
ridge with Alex/Mira, the original agreement, two accepted outcomes and unchanged resources.
The checkpoint distinguishes intermediate-build scene work from final-build continuation.
No manual save edits or developer gameplay bypass was used.

See [Gate 3 acceptance](../../checkpoints/gate-03-desktop-table-loop.md) for the complete
matrix, exact artifact identities, limits and debt. PR #22 records its final reviewed head,
checks and post-merge main verification. Gate 4 is not started. After final main verification,
pause for owner review; the next action is owner-authorized Gate 4 planning, not another
Gate 3 implementation slice. Human playability and reference-hardware/endurance acceptance
remain assigned to their existing later checkpoints.

## Historical execution notes

The implementation notes below preserve intermediate findings and next actions as history.
Their pending integration/build work is superseded by the completion evidence above.


## Objective and baseline

Deliver a distributable Windows desktop application through which a normal user can
create/open a campaign, establish Session Zero, create/select supported PCs, bind players,
start a session, enter ordinary text declarations/questions, report physical dice, inspect
a player-safe transcript/recap, correct uncommitted intent and restart at the exact pending
decision. Stop at Gate 3 completion; do not begin Gate 4.

Fetched and verified main `397f5bbd1deee6d93f8b303e582cb2b7b99a5c88`; Gate 2 is accepted,
the tree is clean and no active plan supersedes this task. Root starts on
`codex/gate3-table-foundation`; one writer per branch/worktree remains mandatory.

## Contract and scope

Product clauses: normal-user access, recognizable sheets, Session Zero/table contract,
trusted speaker/player/character identity, natural input, agency, questions versus actions,
corrections before resolution, hidden information, transcript/recap, exact suspension,
recoverable failures and offline core. Gate 3 checkpoint and requirements traceability
govern acceptance; ADR 001 specifies Tauri 2 with Svelte/TypeScript, while ADRs 002–020
preserve typed proposals, source fidelity, atomic persistence and semantic replay.

Full tactical encounters remain Gate 4, noncombat breadth Gate 5, complete catalogs/world
templates/adventure Gate 6, broad autonomous language/DM interpretation Gate 9 and voice
Gate 10. A supported creation subset must still derive faithful legal characters; a blank
mechanical record or developer JSON form is not character creation. A bounded text adapter
must preserve unknown/material decisions rather than inventing authority or world facts.

## Planned slices

1. Durable table/session/character metadata, transcript and pending-intent model; compatible
   state migration; atomic session persistence and composed replay/restore for table+rules.
2. Source-derived supported character creation, sheet queries and bounded local text
   interpretation; runtime session/player authority, correction, physical dice and recap.
3. Tauri/Svelte desktop shell, trusted IPC boundary, offline packaged resources, keyboard
   interaction, recoverable errors and normal-user campaign/session flows.
4. Packaged production-path scenario/restart and negative acceptance, independent integrated
   review, exact-head CI/merges, ledger/checkpoint/debt updates and final-main verification.

Each PR-sized objective has a plan, independent review where practical, complete exact-head
verification and expected-head-protected merge. Refresh main for dependent integration.

## Acceptance evidence

- User flows require neither development tools nor internal JSON/SQL edits.
- Table contract and supported creation choices persist with explicit content/version pins.
- Input identity comes from the trusted session channel; another PC's name cannot transfer it.
- Queries/unsupported interpretations never become authoritative mechanical actions.
- Pending decisions and rolls survive close/restart; correction cannot silently rewrite an
  accepted outcome. Transcript/recap does not reveal secret state or create competing truth.
- Session start/end, binding, mechanics and journal recover consistently after failure.
- Real desktop scene test covers campaign/PC/session setup, ordinary declaration/question,
  correction, physical dice, pending-decision quit/restart, end/resume and a second campaign.
- Full verification, frontend/build checks, appropriate Windows packaging checks and CI pass
  on exact reviewed heads; final merged main is verified before the owner summary.

## Decisions, risks and current evidence

PR #20 integrates schema-3/session/observation persistence, licensed Human/Fighter/Soldier
creation, application table commands, safe views, bounded text interpretation and composed
rules/table replay. Exact head `b9dbf7c18ec2fc3f1e2b173072c111e7b92b047f` passed full local
`./scripts/verify` and all four jobs in CI run `36013408111`. Seven application table-loop
tests cover corrections, queries, ownership/absence, player privacy, raw dice, retries,
export/replay, independent campaigns and real-file database reopen with pending Second Wind.

Independent review fixed negated/third-party/composite intent, query-retry actor identity
and missing source grants. A subsequent source/restore review found two additional anchor
boundaries: pending Second Wind must retain turn/issuer authority; retained profiles must
constrain all immutable mechanical grants, not just abilities and feature flags. Regression
fixes are being integrated and require a fresh exact-head verification before merge.
The source kernel rebuilds the profile's complete sheet and permits only explicitly mutable
play state; future equipment/advancement work must extend that boundary deliberately.
Those anchor fixes passed full local verification together at `f6d705a`. Follow-up review
preserves legal concentration state and verifies it through a real kernel effect/replay.
Native integration found that generic table errors could clear uncertain requests after
a storage failure. `TableRejected` now denotes only proven input rejection; stored-state,
receipt/observation readback and persistence errors retain the original request. Two
application regressions inject an observation write failure and an unavailable accepted
receipt lookup, then recover with the same ID exactly once. All nine table-loop tests and
strict app Clippy pass; the final full-workspace/CI rerun is still required on this change.
At `8be04e5`, full local verification passed 206 tests and all four jobs in CI run
`36015592974` passed. A final language-boundary review then found that one recognized goal
could hide an unsupported alternative/additional action. The bounded adapter now asks for
one action when conjunctions or separate clauses remain, including unsupported remainders.
The runtime regression proves host adjudication cannot spend resources or create a roll
until the player clarifies. This follow-up requires fresh exact-head verification.

PR #21 starts Windows packaging and typed native IPC; the gameplay interface is being
integrated on its separate writer branch. Ten gameplay UI tests now pass, including
source-choice forms, raw dice, durable retries, malformed local storage and player privacy.
No packaged desktop acceptance is claimed yet.

The accepted Tauri/Svelte path will build on Windows CI using MSVC because the local
Rust toolchain is GNU. Portable Node tooling stays outside the repository. WebView2 is
available on this host; packaging must also support an offline prerequisite installer.
ADR 024 records the current table authority/visibility decisions pending verification.

Inherited TD-001/004 affect this gate directly: new events need semantic replay and desktop
entrypoints must use the runtime instead of raw persistence. TD-002/005/006/007 measured
performance and TD-003/008 recovery/trust retain their recorded receiving gates unless a
minimal prerequisite is demonstrated. Foundation verification is not desktop gate acceptance.

## Exact next action

Foundation #20 is now merged at `ac900fb5c5603f26bd3f3108aecf81bf597eadae`, matching
reviewed head `52ffab23c5e67db7d6ee40a622533b08d44f10fd`. Full verification passed
207 Windows/GNU tests and CI run `36016090350` passed all four jobs (208 Linux tests).
The foundation findings above are resolved, not outstanding merge prerequisites.

Desktop #21 is reconciled with main. Head `51d2962` passes the local core checks,
Linux CI, native MSRV/stable checks, strict desktop lint, three host tests and twelve
frontend tests. Windows run `36017036314` built the executable and NSIS installer,
but the clean-source packaging guard failed. Investigate the exact generated change,
preserve the guard, and fix confirmed dependency-notice omissions before packaging again.

The remaining work is tracked by `gate3-integration-review.md` on PR #22: verify a clean
exact-head package; run the ordinary desktop scene, graceful restart and abrupt crash
recovery; reconcile checkpoint/ledger/debt evidence; merge and verify final main.
No owner blocker is known; Gate 4 remains outside current authorization.
