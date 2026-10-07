# Gate 4 — Preserve full verification within hosted job capacity

Status: source implementation allocated on 2026-10-07. Sole writer after this
plan commit: ci_oct7. Root owns independent review, publication and merge.
Branch: `codex/gate4-ci-runtime-partitions`.
Baseline: main `4cf815bd0f0d9b128612867ba829c9ac1c2549f7`.

## Problem and scope

The full canonical suite has become expensive enough that a single hosted job
cannot be assumed to finish reliably. Actual PR66 Linux job112274454208 ended
cancelled after6h00m39s, with an incomplete table_loop harness; Windows on the
same PR finished its full951 tests and package in5h31m. The cancelling actor is
not proved. Timing is consistent with GitHub's documented six-hour hosted-job
limit, rather than proof of a test failure. Preserve that distinction and every
original log. Other pending jobs still need their actual terminal outcomes.

The reviewed external duration inventory rehashes and reconciles15 terminal
runtime logs. In13 of14 completed runs, table_loop plus
legacy_shield_missile_v1_replay take94.7–98.2 percent of summed harness time.
The Grapple suite is also growing. These are source/platform-specific historical
observations, not a controlled benchmark or a prediction that all jobs will fail.
Evidence: tooling/ci-oct7/ci-capacity-evidence-v2-2026-10-07.json, SHA256
`578906e399bc6642e095a804a0f2c26eab54ece1886f2414877277d7fcd36081`;
proposal SHA256
`899b2605b425078bdc9f44ccad3bbe70ac36996b70e34f9717851470f5d7350e`.
The earlier v1 timing parser is invalid for harness attribution and is not used.

Binding sources are AGENTS, the product definition, Gate4, the gate execution
protocol, the current development runbook and canonical scripts/verify. This
slice advances trustworthy evidence for tactical development; it changes no
game behavior, source content, runtime authority, persistence or acceptance.
Canonical local full verification remains available and unchanged.

## Selected boundaries

Split hosted execution at complete test-executable boundaries. Start with
separate allocations for table_loop, legacy_shield_missile_v1_replay and
table_grapple_public, plus every remaining target and documentation test.
Retain each whole test case and its full scenario/restore loops. A target absent
from an older baseline is recorded as absent in the discovered manifest; no
required target present in the build may silently disappear or be unassigned.

Use the identical workspace, dependency lock, ordinary test profile, selected
platform target, default features and default harness threading. No release
test profile, raised timeout, stack/thread override, test deletion, new ignore,
removed assertion, reduced fixture matrix, filtered incomplete pass or restored
state shortcut is allocated. Do not alter existing test files or cargo locks.
Case-level sharding is outside this first implementation; if a complete harness
still cannot fit, record the actual evidence for a separately reviewed design.

Generate a complete execution inventory from the canonical platform-specific
Cargo build and actual test listings. Bind source head/tree, package and target
identities, features/configuration, toolchain/target, executable identity and
all named cases. Preserve canonical unit, integration, binary, example and
documentation coverage, including compile-only and zero-case targets. Dynamic
discovery must fail closed on unsupported target semantics, duplicate or missing
allocation, unexplained changes, ignored cases or an incomplete result.

Before implementation of invocation, record its exact Cargo equivalence here.
Prefer ordinary Cargo execution where it preserves the canonical workspace
feature graph and target semantics. Direct binary execution, if necessary,
must preserve Cargo's original package working directory, runtime environment,
required binaries/libraries and complete harness arguments, with proof rather
than assumption. Building an isolated package/target must not change resolver2
feature unification. A source manifest is not a compiler/execution result.

Every allocation must record complete outcomes and reconcile them against its
expected inventory. A required aggregate must verify the complete disjoint union
and source/platform identity. Missing, failed, cancelled, timed-out, skipped,
partial or duplicate execution never passes the aggregate. Log parsing cannot
associate interleaved stdout/stderr with the wrong target; execute/log boundaries
must be explicit. Preserve complete original logs beside compact receipts.

Keep Windows release packaging required after its full runtime aggregate.
Retain exact source checkout selection and all existing frontend, static build,
MSRV, formatting, strict lint, source-boundary and genericity checks. Preserve
normal installed-content/icon/notice/NSIS packaging and artifact naming.
No successful package may replace a failed or missing runtime allocation.
Required final check names must remain clear; normal GitHub protections still
apply, with no admin bypass or accepting skipped prerequisites as green.

## Implementation and review

1. Inspect complete workflows, Cargo target/configuration metadata sources,
   packaging scripts and the official Cargo semantics. Record concrete discovery,
   execution, aggregation and artifact transport decisions here before code.
2. Implement a small deterministic runner/inventory validator and complete-job
   workflow separation. Avoid introducing an unrelated dependency or deployment.
3. Add meaningful lightweight runner tests for missing/duplicate/wrong-source
   allocations, incomplete outcomes, zero-case targets, malformed listings,
   discovery changes and aggregation gating. These must exercise behavior,
   rather than mirror a static YAML text string.
4. Freeze coherent source and full preservation inventory for root and a fresh
   independent reviewer. The same author must not self-certify the CI replacement.
5. Publish a draft only after review preparation, then collect actual complete
   platform evidence. Existing unmodified main4cf full jobs keep running as the
   canonical comparison; do not cancel them because this branch exists.

## Acceptance

- Every original source, content, test body, fixture, dependency lock and local
  verification command remains exact baseline.
- Exact-head source review and meaningful runner-negative tests pass.
- On both Linux and Windows, discovered and executed targets/cases equal the
  ordinary canonical workspace scope at the same production/test source. Account
  explicitly for platform-only targets and docs; a lower passing count is not
  sufficient. Use independently audited completed canonical evidence and one
  controlled source-equivalent comparison, with all differences attributed.
- All new exact-head jobs complete and required aggregation checks pass only
  after every allocation. The actual built target/configuration identity is
  checked, not inferred from matching filenames.
- Windows frontend/MSRV/lint and ordinary release/installer packaging remain
  successful. No native gameplay claim is created by this tooling-only change.
- Record evidence, risks and next actions in this plan and PR. Merge normally
  with expected-head protection only after the full replacement is proved.

## Allocation and current status

Root created this separate checkout from freshly fetched main4cf after the app
worktree helper could not resolve that commit in the outer wrapper repository.
No existing checkout or active run was reset. This plan precedes implementation.
All implementation, runner tests and new CI are currently UNRUN.

ci_oct7 may edit only this branch and external CI evidence. Static Git/file reads
and lightweight Python runner unit tests are allocated. Local Cargo, Rust
compilation, npm, project gameplay tests, native execution and database work are
not allocated: capture969 currently owns the single local heavy slot. Cloud CI
publication/dispatch remains root's responsibility. Continue preserving current
terminal CI outcomes while developing this bounded tooling slice; report a real
failure promptly. Gate4 stays active, and no Gate5 work or product waiver begins.

Official references checked on2026-10-07:
[GitHub Actions limits](https://docs.github.com/en/actions/reference/limits) and
[Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html).
