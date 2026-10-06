# Public Grapple: actual first full-run failures

Status: diagnosis/plan before source edits, 2026-10-06. Root is sole writer on
codex/gate4-grapple-public-completion, exact clean/published 4d8f5bcf029b56443bb67d0b03e93f8ae8560323.
Gate 4 and the full public-completion acceptance contract remain active.

Linux CI run 37430605731, job 112160323555 completed the public application
harness: 21 passed, four failed, none ignored/filtered. Full actual normalized
log is retained externally at tooling/ci-oct6/pr67-4d8-job-112160323555-2026-10-06.log.
This is failure evidence, not acceptance. Windows is still pending.

## Diagnosis and bounded changes

Both activated_unarmed_opportunity_keeps_empty_occurrence_read_and_spends_one_reaction
and corrections::unarmed_opportunity_seals_full_holder_hands_separately_and_keeps_them_after_release
fail with "window cut lacks actual opportunity ancestry". The movement pump
leaves the selected MovementOpportunity work before accepting the later player
response. The attack adapter currently allocates AttackRoll while no parent is
entered. The strict validator correctly rejects the missing causal edge.
For the modern owned Grapple path, reenter the actual latest retained
MovementOpportunity for the already-validated reactor while allocating its
AttackRoll, then restore the previous active work before capture/pump. Use the
existing exact prior_work/enter/leave functions; keep historical non-Grapple
allocation unchanged and retain every strict ancestry/source/window validator.
No raw state or serialized permission may create a read owner.

The three-ray test reaches no casting options after PC EndTurn: the actual Adult
Red Dragon's end-of-other-turn legendary window must be declined through its
offered Host command before its own ordinary turn. Insert that real setup
response with actual window/actor assertions; preserve all ray assertions.

The Chimera test attempts EndTurn while its real upward movement has an unanswered
PC opportunity. Settle that actual crossing by a normal PC decline, assert the
real mover/reactor and completed flight position, then retain all original
Grapple/fall/release/raw/history assertions. Do not skip or fabricate the crossing.
Confirm both setup findings against the source and actual harness diagnostics;
runtime follow-up may expose further distinct failures.

## Verification and preservation

Review complete diffs; old failing cases remain enabled with original identities
and assertions. Add only the two necessary producer sequences and assertions.
The opportunity correction must also preserve old captures/non-Grapple source
behavior, selected-window hand proofs, foreign source/reforged ancestry rejection,
and the combined physical Glaive test. Run direct formatting, fresh exact-head
canonical CI and later focused regressions once the sole local package slot is
free. Receive the entire published correction normally into dependent branches.
No runtime/native/Gate 4 acceptance follows from this plan or source edits.
