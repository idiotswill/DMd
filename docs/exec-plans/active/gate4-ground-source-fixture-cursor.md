# Ground private source fixture turn-cursor correction

Status: planned before source correction, 2026-10-06. Root is sole writer on
`codex/gate4-ground-production`, starting at exact f9e466430cd753a9056054394f71137fd205b5e8.
This corrects the shared Ground test fixture before normal receipt by its
Consequences, Physical, Mage, Graze and Grapple integration descendants.

## Actual failure and scope

Graze436 Linux job112141975137 finished with four failing private source cases
at attack_equipment_access_tests.rs:478. All fail during the new initiative's
second physical roll, before their equipment assertions, with
`duplicate/out-of-order creature turn hook`. The full normalized log is retained
outside the repo with SHA256
7e0696ba13a61f6391b94878525e3c0a602b20a6fd4b070a1e0d41cf836c6ee1.
This helper is byte-identical on Groundf9 and Graze436.

The explicitly constructed mechanism fixture clears central timing, rolls and
effect-turn state, then starts fresh initiative. It retains the old Mage target's
observed creature turn from the frozen Shield image. The authoritative source
scheduler correctly refuses to observe the restarted turn again. This is a
fixture inconsistency, not permission to weaken source idempotence validation.

After central timing is removed, use the existing privileged LeaveCombat source
operation for that retained target, as the production encounter-release path
does. Assert no cost/request/activation/feature, the exact cleared cursor and
unchanged profile and limited resources, then continue the existing setup. The
new Goblin already has no observed turn. Do not edit the original capture, test
bodies, production scheduler, source pin, assertions or actual attack producers.
These cases remain private mechanism tests, not accepted application histories.

## Verification and acceptance

Commit this plan first, apply the single helper correction, inspect its complete
diff and prove every other tracked file and all test bodies remain unchanged.
Direct edited-file formatting/check is allowed; no competing Cargo/runtime run
while root's MSVC prerequisite/package work owns the local heavy slot. Publish
for fresh exact-head CI, retain the old failed run, and normally receive the
complete corrected Ground ancestry into affected descendants. Each receiving
head needs its own verification; no prior pass transfers. All four named failed
cases and the full existing suite remain required. This changes no product or
Gate 4 acceptance criteria; native and remaining integration work remain due.

Exact next action: commit this plan, correct the retained target's setup through
the existing source operation, then review and publish the bounded correction.
