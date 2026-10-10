# Gate 4: complete verification within runner capacity

Status: proposed 2026-10-10, before implementation. Root is the sole writer on
`codex/gate4-family-receiver`. Baseline is
`63b48d50a29407ac3e7197f49aaf106f70714fe6`, tree
`d0c540f81568017dcc89ff02fb4c96b7c3beb60c`. Gate 4 remains active; Gate 5 is not
allocated. Independent review precedes the change and new runtime allocation.

## Problem and governing scope

The corrected family still cannot complete its current hosted verification.
Linux run 37813727243 and Windows run 37813727264 have nine cancelled runtime
jobs after approximately six hours. Their downstream union/package guards
failed. Five complete runtime jobs succeeded; those jobs do not complete either
platform. The original Linux Grapple log also records an actual failed case,
`roll_details::mass::actual_m_g3_source_save_and_escape_details_preserve_owned_source_and_retry`.
It contains 82 named passes, that failure and nine unresolved cases; cancellation
prevented the final panic summary. This is a functional blocker, not merely a
capacity problem. Windows Grapple has 81 named passes and 11 unresolved cases.
Original job logs and artifacts are being preserved and audited under
`tooling/ci-oct10`, outside the repository. Timing alone does not prove the
cancelling actor, an assertion failure, or successful completion of partial tests.
The subsequently recovered Ground annotation explicitly reports the six-hour
maximum execution time. Ground and Recipient each finish their small control
and remain inside a single long historical case, so case-level sharding would
not resolve those individual cases.

The local GNU attempt on the same head completed the three new Finished
Inspiration cases, the existing Finished/Mass case, and the intrinsic-attack
case. That last case took 1,805.54 seconds. The next source-roll case has no
terminal result. Windows subsequently booted on October 10; the original process
tree is absent. Classify that attempt as abruptly interrupted, not failed by an
assertion or passed. Keep its target, logs, executables and original receipts.
Recovery audit SHA256:
`284463dac04d021dab488a77fa6f0fee8c7e670783cdc12f365f237008170a9d`.
The audit checks all 1,012 source entries, 16 executors, 133 prior evidence pins,
configuration and completed focused outcomes. The stale lock can be archived
only after independently verifying its exact identity and absent processes.

This slice advances trustworthy verification under the [product
definition](../../product-definition.md), [Gate 4](../../checkpoints/gate-04-tactical-encounters.md)
and [gate protocol](../../checkpoints/gate-execution-protocol.md). It supersedes
only the earlier CI/family allocation's prohibition on changing the ordinary
test profile. It does not reduce coverage, change tactical behavior or satisfy
native/human acceptance. The [whole-harness allocation](gate4-physical-history-ci-allocation.md)
and [CI union requirements](gate4-ci-runtime-partitions.md) remain in force.

## Bounded candidate

Configure the workspace's built-in test profile in root `Cargo.toml`:

```toml
[profile.test]
opt-level = 1
debug-assertions = true
overflow-checks = true
```

Use basic optimization while explicitly preserving debug assertions and integer
overflow checks. Leave debug information, unwind behavior, features, target,
default test threads/stack and every test/oracle/scenario/corpus byte unchanged.
Leave the development and release profiles unchanged. There is no environment
profile override, release-mode substitute, timeout increase, case sharding,
cross-request cache, skipped history validation or stored-state shortcut.

Cargo documents that `cargo test` selects the built-in test profile, which
inherits development settings unless changed at the workspace root. Optimization,
debug assertions and overflow checks are separate settings. Source:
[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html), read
2026-10-10. This is a candidate performance improvement, not a measured claim.
Optimization can increase compilation time/memory and affect debugging. Dependency
build scripts can observe the changed optimization setting; full normal compiler
graphs and fresh runtime execution must establish the actual result.

Keep `scripts/verify-fast`, `scripts/verify`, both hosted workflows, seven runtime
allocations, 27 CI controls and frontend commands intact. The committed manifest
is the visible configuration for all future ordinary test runs on this head.
Hosted discovery already records actual compiler profiles and compares all three
canonical compiler graphs; preserve that check and inspect optimization/debug
settings explicitly in new artifacts. Previous-head passes do not verify it.

## Verification and acceptance

1. Independently review this plan and the complete manifest delta. Prove that all
   preexisting source/test/corpus/lock/workflow/script entries remain byte-exact;
   only this plan, the root profile and current receiver status may change.
2. Freeze a clean head. Prepare and review a fresh dedicated-target local runner
   using the previous seven focused stages and unchanged full canonical/Python/
   frontend stages. Preserve all original attempts; do not resume or reuse their
   targets. Keep GNU 1.98.1, jobs=1 and incremental=0 as previously allocated.
   Run the existing M+G3 source case first to obtain its terminal panic/outcome;
   then run the other six original stages. A failure stops the sequence and
   requires a diagnosed source correction before a new frozen-head allocation.
3. Compare the exact intrinsic case with the original 1,805.54-second outcome,
   clearly separating compilation from test duration. The earlier observation
   is not a controlled benchmark. Record actual configuration, resources and
   outcomes; do not promise a speedup before execution. Stop for an actual failure
   and diagnose it normally, without changing oracles or retrying until green.
4. Require unchanged complete `bash ./scripts/verify`, all 27 ordinary CI controls,
   and frontend install/check/test/build on the frozen head. The expected GNU
   catalogue remains 1,150 cases, 58 executable targets plus six doctest groups;
   reconcile actual discovery, not just expected counts.
5. Publish the reviewed candidate for fresh exact-head Linux/MSVC prerequisites,
   all seven allocations and full unions. Inspect the original graphs for the
   declared test profile, unchanged full case catalogues and actual named outcomes.
   Windows release/installer packaging and the original native ledger/addendum
   remain required before merge. Human Gate 4 encounter evidence remains separate.
6. If optimization is insufficient or compilation exceeds available resources,
   record the actual result and design a separately reviewed correction. Do not
   silently expand flags, alter histories or treat cancellation as success.

The separately observed duplicate full authentication in `table_view`,
`replay_rules` and `query_rules` remains a potential production improvement, not
part of this change. Its existence does not establish its share of total runtime.
Avoid mixing an authority-path refactor into this configuration experiment.

## Exact next action

Independent plan review, then apply only the three explicit test-profile settings,
review/freeze the delta, and allocate fresh verification. No new run or profile
performance result is claimed by this plan.

Independent source assessment found repeated full replay in the actual fixture
and read paths, with bounded outer loops. Removing the duplicate replay/query
authentication would save only two of fourteen authentication calls inside a
successful cold helper, before additional audience reads. That is a call-count
observation, not a wall-time attribution. The test-profile experiment leaves
those real restore/retry/privacy paths intact. Development-build and packaged
application latency still require separate measurement; faster tests cannot be
reported as a production performance fix.
