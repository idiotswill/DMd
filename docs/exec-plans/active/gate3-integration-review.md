# Gate 3 integrated desktop acceptance and closeout

Status: active. Sole writer: root on `codex/gate3-integration-review`.

## Objective and baseline

Verify the real Windows desktop table flow, resolve integration findings, reconcile the
coverage/debt/checkpoint evidence, and pause at the end of Gate 3. Do not enter Gate 4.
Main `ac900fb5c5603f26bd3f3108aecf81bf597eadae` contains foundation PR #20; its tree matches
reviewed head `52ffab23c5e67db7d6ee40a622533b08d44f10fd`. That exact head passed full local
verification (207 Windows tests) and CI run `36016090350` (208 Linux tests, Rust 1.88,
strict Clippy, formatting and both guards). PR #21 supplies desktop/UI integration.

Read AGENTS, the active Gate 3 plan/checkpoint, product-definition normal-user/recovery
requirements and ADRs 001/021–024. The native package must exercise the production runtime;
frontend mocks and kernel tests alone cannot satisfy the gate.

## Acceptance scenario

1. Download the exact CI-built Windows package, verify its commit/checksums, and launch the
   portable application with bundled resources. Record the build and host prerequisite.
2. Create a campaign through the UI and save all table-contract settings. Add a player,
   create a source-derived Human Fighter Soldier with chosen equipment, inspect the sheet,
   and bind the player/character when starting a session.
3. Establish a generic host-described scene with one ability-check challenge. Switch to
   the player, ask a question, submit ordinary text, correct it before adjudication, then
   request/report physical dice through the appropriate local channels. Check the outcome,
   sheet, transcript and recap without exposing the host's hidden check context.
4. Propose an unsupported or alternative action; preserve it unresolved across quit/restart.
   Withdraw or clarify it explicitly. Quit/restart again with a pending Second Wind roll,
   report the raw face, and verify the resource count and accepted history survive.
5. End the session, quit/reopen the campaign, and resume with correct bindings/history.
   Create/open a second independent campaign and verify state does not leak between them.
6. Check recoverable invalid input, keyboard operation, and duplicate launch behavior. All
   acceptance uses ordinary application controls; no developer tools or manual save edits.

Record observed behavior and any limitations in the checkpoint; fix defects before claiming
acceptance. A build failure or UI failure is evidence to investigate, not grounds to waive a
criterion. Human enjoyment/vertical-slice acceptance remains the mandatory post-Gate-6 gate.

## Remaining review and verification

- Finish the independent host/package findings: single native instance protects the pending
  request slot; packaging must bind binaries/resources to the actual verified build commit.
- Verify frontend checks/tests/build, Windows MSVC 1.88/stable, native host tests, packaging
  and complete repository verification on the exact reviewed PR #21 head before merging.
- Refresh main after merge, update coverage/traceability/debt and checkpoint evidence, archive
  completed plans, and run required local/CI checks on final merged main.
- Return the handoff's evidence-based gate summary, including remaining future-gate scope.

## Current evidence and exact next action

The integrated frontend at `363a968` passed Svelte checks (zero errors/warnings), all 12
tests and a real mounted production bundle build. Windows CI at `277d705` reached native
compilation after frontend preparation. Those are intermediate heads, not final acceptance.
Root has confirmed native desktop automation is available. Next: integrate/verify the two
package fixes, obtain a successful exact-head package, and execute the scenario above.
