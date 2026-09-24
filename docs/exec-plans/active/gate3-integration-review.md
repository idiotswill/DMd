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
   Withdraw or clarify it explicitly. Terminate only the tested application's process
   with a pending Second Wind roll to simulate a crash; relaunch through its normal UI,
   report the raw face, and verify the resource count, bindings and accepted history survive
   without duplicate expenditure or outcomes. Process termination is fault injection,
   not a gameplay or save-state bypass.
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

Follow-up at desktop head `51d2962d1a9bd7e1bae792ade7bc203e44e3d262`: both package findings
passed an independent source review. The missing project LICENSE was added consistently
with the existing workspace MIT declaration; rules/dependency notices retain their own
terms. Full local `./scripts/verify` passes with `CARGO_BUILD_JOBS=1`; the prior parallel
attempt exhausted host compiler memory and is not a passing result. Linux CI run
`36017036464` is green. Windows run `36017036314` passed the full Rust 1.88 native check;
stable lint/host tests and packaging remain in progress. No native UI acceptance yet.

At desktop head `f4f1ad2e2049aecbc2323252d290dfbf2893de02`, full local verification
again passes 207 Windows/GNU tests, and Linux CI run `36020516060` is green. The prior
Windows run completed native lint/three host tests and built executable/installer, but
packaging failed because Tauri rewrote its manifest's checkout line endings. A narrow
LF attribute preserves a clean fresh checkout. Independent review also fixed real
dependency-notice omissions (underscore filenames, LICENSES directories and declared
license files); all three copying/containment regressions pass, including a Windows
junction escape. New Windows run `36020516064` must produce the tested artifact.

Closeout review strengthened the mechanical boundary guard to parse Cargo package
identities, including renamed, inherited and target-specific dependencies. Seven
regressions and both foundation/desktop tree checks pass under Python 3.11.9; the
eighth fixture needs symbolic-link privilege unavailable locally and passes in Linux
CI run `36021191797` at `cc1b948`. CI now
provisions Python 3.11 explicitly; local verification can set `PYTHON` to a working
3.11+ executable when Windows exposes a nonfunctional python3 Store alias.

## Packaged native observations (partial; gate remains active)

Windows run `36020516064` produced artifact `10816249118` for exact head
`f4f1ad2e2049aecbc2323252d290dfbf2893de02`. Its ZIP SHA-256 is
`c9f03483a6c03b0bc458649b91b7ad57594345bfaf888c453cc2f6f04fcd0018`.
All 3,318 manifest checksums passed before launching the portable MSVC release executable
outside the repository with the installed WebView2 runtime. Ordinary native UI controls
were used throughout; no developer tools or save edits were used.

Observed through that executable:

- Created Ridge Crossing QA and saved a custom Session Zero tone; created Alex and
  Mira (Human Fighter 1 Soldier/Skilled), purchased/wore Leather Armor, and obtained
  source-derived HP 11/11, AC 14, proficiency +2 and Second Wind 2/2. Started Across
  the ridge with Alex present and bound to Mira.
- Established The narrow ledge, a Strength (Athletics) DC 15 check. The Alex view hid
  the host DC and unrevealed consequences. A character question returned saved HP/AC
  without spending resources or becoming a recap outcome.
- Corrected an uncommitted Second Wind declaration to a climb; the resource remained
  2/2. The host requested dice. Native validation rejected 21 on a d20; raw 12 resolved
  as total 17 and the selected success appeared once in the transcript and recap.
- An alternative action, "I climb or drink a potion", remained a material clarification
  across normal quit/restart with the binding, history, resources and recap unchanged.
  Explicit withdrawal reported no world outcome.
- A subsequent Second Wind request reserved one use (1/2) and a d10. Duplicate launch
  focused the same single window. Forced termination of only the verified test executable,
  followed by normal relaunch, retained the pending request, binding and 1/2 resource.

The pending d10 is deliberately retained for a continuity check against the final package.
Created Harbor QA with a separate default agreement and verified empty characters, scene,
transcript and recap. Switching back preserved Ridge Crossing QA's pending roll, history
and resource count. Remaining native steps: resolve the saved d10, then end/save/reopen/
continue on the final build. Reference-hardware performance and human enjoyment are not
claimed by this test.

Actual package inspection found recursive self-copying of generated notices and three
published WebView2 crates without license files. Narrow pinned MIT supplements and
generated-output exclusion now have seven copying/containment regressions. Failed Windows
runs `36023230952` and `36024054085` exposed, respectively, temporary-root aliases and a
vendoring-only checksum-file assumption; neither is acceptance evidence. The collector now
uses the exact Cargo.lock registry record after locked metadata, with revision/text checks.
Native testing also found debug-format roll labels; the application projection now emits
readable labels without changing stored requests or private context. Independent review
cleared these fixes; all ten real application table-loop tests pass. Final candidate
`05ff272d16ab7d893e0c39950f07bd5176c82e3e` passes full local verification (207 tests,
strict Clippy, formatting and guards). Linux run `36025008608`, Windows run
`36025008694`, artifact inspection and native completion remain pending.
