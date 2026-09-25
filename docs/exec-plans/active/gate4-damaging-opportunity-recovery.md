# Gate 4 damaging opportunity and concentration recovery

Writer: bootstrap_audit. Branch: `codex/gate4-oa-concentration-recovery`, from
verified integration456272ed927149c52e16cfcdcd7e23504a65fe14. Root owns the eventual
main-based movement/attack/casting delivery. This branch adds a real application
regression only; the Ready writer owns shared source scheduling and the protocol
writer owns transport revisions/audiences.

## Objective and constraints

Close the explicitly recorded damaging-opportunity/concentration application
coverage gap in `gate4-table-movement.md`. Exercise the existing single queue and
real table command path against file SQLite, with source-derived gear, source
casting, controller-reported dice and semantic replay. Root AGENTS, Gate4, ADR026,
and product requirements for player authority, source fidelity, private views and
exact suspension/resume apply. This is bounded integration evidence, not native
or completed Gate4 acceptance.

Do not inject effects, fabricated prepared spells, held weapons, source counters,
budgets, positions after setup, or synthetic permissions. Both players are actually
present. Character0 buys a Dagger during supported creation, materializes real
equipment and equips it through a permitted ordinary Attack before its turn ends.
A genuine Cultist Fanatic casts Hold Person on Character1, whose real controller
reports a failed save. The Cultist then moves out of Character0's reach; the
controller chooses the offered Dagger opportunity, reports a hit and damage, and
the Host reports the Cultist's failed concentration save. Hold Person ends and the
original accepted movement resumes without borrowing the reactor's turn/Action.

## Acceptance and work

1. Build the complete source-valid setup using actual table commands, preserving
   the source spell's real material ItemId and all original command metadata.
2. Cold-close/reopen at opportunity, attack/damage and concentration suspension
   boundaries. Independently restore a mirror and send identical accepted inputs
   through each continuation. Exact retries must not spend or apply twice.
3. Reject stale and foreign-controller inputs without durable changes. Corrupt
   retained original/callback provenance in otherwise structurally valid exports
   and prove semantic restore rejects before writing any aggregate rows.
4. Verify reaction/Action/turn/movement budgets, original versus causal origins,
   concentration cleanup, final endpoint/cost and final semantic replay. Compare
   all durable export data exactly across unchanged retries/rejections in the same
   database; normalize only request-time `exported_at_utc`. Mirrored executions
   compare source states and semantic replay, since independent accepted journal
   rows correctly allocate distinct EventIds and acceptance timestamps.
5. Heap-pin the test's large phases from the outset; retain the default Windows
   stack and production code. Run the focused scenario after the protocol compiler
   handoff, then appropriate app/strict-lint checks. Obtain independent exact-head
   review before handing the coherent test slice to root.

## Status, risk and next action

The complete scenario is authored in
`crates/dmd-app/tests/support/table_oa_concentration_cases.rs`, including genuine
Hold Person origin/material assertions, four malformed-origin restore cases,
cold retries at each suspension, changed-body nonce rejection, and mirrored
continuation. Static review corrected Host visibility expectations: Host remains
privileged; another Player cannot take the reactor's decision or raw roll.
Formatting and whitespace checks pass. No production files are owned here.
Root's independently authored cold-round module is a separate
additive registration in `table_loop.rs`; preserve both during integration.
On exact code head `f13224ebfc9d73d7b8955c0f6596d5d777bd45db`, the first
`cargo test -p dmd-app --test table_loop table_oa_concentration_cases:: -- --nocapture`
compile and run passed: **1 scenario, 0 failures, 22 unrelated cases filtered**,
17.75 seconds execution, default Windows test-thread stack. Domain/rules/app source
timestamps were refreshed before compiling against the shared target cache; jobs=1.
The local log is `../research/gate4-oa-concentration/focused-first.log`.
No fixture/source correction or stack override was needed. Every boxed phase ran,
including all four structurally valid forged-export rejection cases with zero
aggregate writes. The global compiler was explicitly released to root for PR33's
combined canonical verification. This focused result does not claim full-workspace
verification or independent review; root must include this test in that exact-head
run. Read-only source/authority review is requested from the separate Ready writer;
Independent exact source/authority review subsequently cleared `f13224e` without
changes. Root integrated the reviewed scenario as `5e3f89a` into PR33; final combined
canonical evidence remains pending.
General privacy protocol, native combat acceptance, Ready/reactions and all other
open Gate4 requirements remain unchanged.
