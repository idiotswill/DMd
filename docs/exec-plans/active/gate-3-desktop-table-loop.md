# Gate 3 — first playable desktop table loop

Status: **Active — owner authorized continuation after Gate 2 on 2026-09-24.**

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

PR #21 starts Windows packaging and typed native IPC; the gameplay interface is being
integrated on its separate writer branch. No packaged desktop acceptance is claimed yet.

The accepted Tauri/Svelte path will build on Windows CI using MSVC because the local
Rust toolchain is GNU. Portable Node tooling stays outside the repository. WebView2 is
available on this host; packaging must also support an offline prerequisite installer.
ADR 024 records the current table authority/visibility decisions pending verification.

Inherited TD-001/004 affect this gate directly: new events need semantic replay and desktop
entrypoints must use the runtime instead of raw persistence. TD-002/005/006/007 measured
performance and TD-003/008 recovery/trust retain their recorded receiving gates unless a
minimal prerequisite is demonstrated. Foundation verification is not desktop gate acceptance.

## Exact next action

Integrate the two anchor-review fixes, review the exact foundation diff, rerun full local
verification and CI, and merge #20 with expected-head protection. Reconcile dependent #21
with refreshed main, finish frontend/native checks, run the packaged scene and restart
scenario, then record checkpoint/ledger/debt evidence and verify final merged main.
No owner blocker is known; Gate 4 remains outside current authorization.
