# Gate 4 table falling and recovery

## Objective and ownership

Branch `codex/gate4-table-falling`, based on parent integration `df75fb8` in the
`gate4-table-falling` worktree. Sole writer: rules-architecture agent. Parent owns
integration/release and the serialized Rust compiler. The area author owns subsequent
shared tactical queue work. This slice owns app/projection/desktop/tests and only the
required missing `falls` constructor on source integration.

Advance Gate 4 and product requirements for a playable local table, player agency,
authority separation, durable exact decisions, privacy and replay. Read root AGENTS,
Gate 4, ADRs for typed table/tactical commands, dice authority, journal replay and
privacy. Do not claim source helpers or scripted unit fixtures complete application
acceptance; do not narrow the owner product definition or start another gate.

## Scope

Integrate reviewed falling leaf/domain/queue checkpoints `dd8730c`, `8c6f271`,
`3bc7e36`, `4cdae2c`, `14d0aeb`, `eec8f01`, `ebbff59` without replacing newer parent
source/application changes. Add missing empty fall-vector constructors. Source author
checks cover final12 falling+43 spatial and strict all-target domain/rules Clippy;
parent/application checks still follow here.

Expose the real, actor-owned liquid-landing decision through the existing table
command path: Athletics or Acrobatics spends that actor's Reaction; decline does not.
Use the actual retained source identity, not a client-authored target or difficulty.
Present safe labels for the actual fall/check raw requests. Keep surface IDs, hidden
landing geometry and another actor's private work out of unauthorized projections.
Reset pending form choices when viewer/actor/decision changes.

Collect every persisted fall CommandMeta in export validation: cause/origin, original
Move where present, accepted liquid choice and actual completion. Preserve earliest
live-tactical-anchor rejection and exact composed journal replay. Add real SQLite
quit/reopen/export/restore/retry tests through table commands and compare the complete
state across the same accepted continuation. Include wrong controller, stale identity,
forged fall/provenance exports and no partial restore writes.

## Acceptance and validation

- Source checkpoint integrates with parent NPC/casting state without losing existing
  action variants, audit collectors, decoder guards or fixture fields.
- Only the falling actor's bound player or authorized host receives usable choices;
  declined/accepted choice is durable and disappears after resolution.
- Physical raw input uses the existing table dice boundary and returns
  the same recorded result on same-request retry, including after process restart.
- Actual landing/Fell receipt/vitality/concentration state and journal match after
  fresh SQLite import and deterministic continuation; tampering fails before writes.
- Focused frontend tests, Svelte check/build, app/table recovery tests and strict lint
  run on the actual resulting source. Compiler use is globally serialized; obtain an
  explicit handoff before starting a Rust build.
- Parent performs combined full verification, exact-head review and PR integration.
  Gate 4 remains active; this slice does not claim complete combat or gate closure.

## Current status and next action

Source integration and the owned table/UI/origin implementation are committed through
`27c458e`. Both independent app reviews found no concrete blocker. Frontend checks pass.
The focused real SQLite scenario and all 62 app tests now pass on the unchanged default
Windows stack. Strict three-crate all-target Clippy also passes. The
test-only follow-up splits the long async scenario into heap-pinned phases while
retaining every authority/recovery assertion; it also proves malformed exports pass
generic structure checks before semantic rejection and checks all five actual restore
tables for zero writes. Parent combined verification and PR integration remain next.


## Implementation checkpoint (backend verification pending)

The seven reviewed source commits are integrated through `f81922b`. Conflicts were
historical execution-plan overlap and additive `CreatureAttack`/`ChooseLiquidLanding`
action/dispatch arms; both source actions and the newer parent evidence were preserved.
The sole subsequent rules change adds `falls: vec![]` to the newer source creature
attack constructor, preserving the same transparent wire representation.

`TableTacticalView.liquid_landing` exposes only the controlled falling actor to that
player or the host. The stateless form offers Athletics, Acrobatics, or decline, with
no selected default, geometry, source IDs, target or difficulty supplied by the client.
The authoritative saved stage derives the choice owner and Reaction. Concurrent-work
labels remain generic for another actor; ordering buttons stay suppressed while an
owned liquid choice is selected. Typed raw labels identify the real skill or falling
damage without changing persisted requests or source replay inputs.

Restore collects original fall/movement, accepted landing-choice, retained successful
check and completed-fall metadata. Existing earliest tactical-anchor rejection and
semantic replay remain unchanged. The new real SQLite scenario creates a legal source
PC, materializes equipment, authors a ledge and water through host setup, accepts real
movement, then exercises all three choices. It closes and reopens the actual database
at decision/raw boundaries, independently restores the export, submits identical
commands/raw faces on both branches and compares full state, audit and typed journal
payloads. Generated event-row IDs are intentionally independent. Malformed current and
historical fall images must reject before campaign rows are installed. At the original
`27c458e` checkpoint these Rust scenarios had not yet compiled or passed; the subsequent
verification results and fixture corrections are recorded below.

Frontend evidence on the implementation source tree: all 37 tests passed (including
both new landing interaction/privacy/disabled-state cases), Svelte check reported zero
errors/warnings, and Vite production build passed. The initial offline dependency
installation lacked one lockfile-pinned cache tarball; ordinary `npm ci --ignore-scripts`
succeeded without changing the lockfile. Rust formatting and diff checks pass.

The original next step was the serialized backend run. Native desktop, complete combat
and Gate 4 acceptance remain outside this bounded checkpoint.


### Interrupted backend verification and fixture corrections

The first focused build completed successfully, then the single long async test
exhausted the default Windows test stack. The follow-up separates initialization,
movement/privacy and landing phases and heap-pins their futures; no thread stack limit
or product behavior was changed. A rerun could not map Rust dependency metadata or
allocate 1 MB because system commit capacity was exhausted. No cache deletion or user
process termination was performed. A later rerun with recovered capacity compiled and
executed the test on the unchanged default stack.

That run reached semantic export-rejection checks and exposed a fixture mistake:
`campaigns` is not a persistence table. The corrected assertion checks the real
`campaign_state_current`, `campaign_lifecycle`, `event_journal`, `command_audit` and
`campaign_snapshots` tables for zero rows. Forged exports must now first pass generic
portable structure validation, ensuring the failure exercises tactical semantics.
These corrections subsequently passed the focused SQLite scenario and full app suite,
after the parent explicitly released the compiler following PR31 canonical verification.

Saved logs outside the repository preserve all outcomes:
`../tooling/logs/gate4-table-falling-tests-20260925.log` (compile then stack overflow),
`...-r2.log` (allocation/mmap exhaustion) and `...-r3.log` (default-stack execution,
then diagnosed fixture table-name error). No failed run is counted as acceptance.

### Backend follow-up evidence

The corrected focused SQLite test passed on 2026-09-25 in 15.88 seconds, exercising
Athletics, Acrobatics and decline through actual closed/reopened database files and
independent portable restoration. All 62 `dmd-app` tests then passed, including all
19 table-loop scenarios and the composed export/recovery tests. No stack-size override
was used. No production code changed after `27c458e` for these results.

The environment agent independently reviewed the complete test refactor and found no
lost authority, privacy, replay, raw-input or persistence assertions. Its review also
confirmed that the generic-upgrade assertion strengthens semantic rejection coverage
and the five real table counts replace the invalid fixture table name.

Logs outside the repository:
`tooling/logs/gate4-table-falling-tests-20260925-r4.log` and
`tooling/logs/gate4-table-falling-all-app-20260925.log` and
`tooling/logs/gate4-table-falling-clippy-20260925.log`.
`cargo clippy --locked --offline -p dmd-domain -p dmd-rules -p dmd-app --all-targets
-- -D warnings` passed. Formatting and diff checks pass. The compiler was explicitly
released before recording this checkpoint. Parent combined canonical verification and
exact-head integration follow separately; these checks cover this bounded branch,
not later area/shield source changes or a packaged encounter acceptance run.
