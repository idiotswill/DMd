# Gate 4 — Exhaustive Windows CI partition for slow recovery coverage

Status: **Bounded implementation authorized; verification not yet run.**
Sole writer: `gate4_ci_oct4`, explicitly assigned by root on 2026-10-04.
Review amendment: root and independent peer requested workspace-preserving
selection and native combined-output capture; incorporated below on 2026-10-04.
Root reviewed the complete amended plan at `fdb106f5d830203e2ca2f4b9fec2431014366d99`
and explicitly authorized this writer's helper/parser/tests/workflow work.
External root review: `tooling/mr-fdb106f-root-plan-review-2026-10-04.md`, SHA256
`3cd51872eceebe3a7317dcf5072ac4bd00264b3b45419412d370f2cc96111477`.
Return a clean static checkpoint before any helper-test or native execution.
Branch: `codex/gate4-magic-resistance-source`; checkout `gate4-shield-missile-runtime`.
PR: draft [53](https://github.com/idiotswill/DMd/pull/53), published source
`486ce8ba6ba8fb24f9d9f247f18c07370e8f6c4f`, tree
`5dde4b72324a0911512ee38ddfc337becdd28cab`. Fresh fetch confirms the same remote
head and clean checkout before this plan. This is not an accepted main branch.

## Objective, authority and boundaries

Complete every existing native Windows workspace regression within the hosted
job duration limit, retaining the original tests and an explicit proof that the
parallel jobs execute the entire discovered suite exactly once between them.
Package only after that proof and all relevant Windows checks succeed.

This serves Gate4's real persistence, replay, source fidelity and tactical
continuation acceptance under the [MR prerequisite plan](gate4-magic-resistance-source.md),
[Gate04](../../checkpoints/gate-04-tactical-encounters.md), [product definition](../../product-definition.md),
ADR026/028/029 and the [gate protocol](../../checkpoints/gate-execution-protocol.md).
It repairs CI scheduling; it does not reduce product or gate acceptance.

Permitted implementation after plan review: `.github/workflows/desktop.yml`,
small verification/evidence helpers under `scripts/`, meaningful helper tests,
and the directly relevant plan/runbook evidence. Linux CI, `scripts/verify` and
`scripts/verify-fast` remain unchanged. No Rust/application code, test bodies,
fixtures, content, source pins, cold-step assertions, raw histories, replay or
save-format behavior changes. No test/profile optimization, ignored tests,
reduced scenarios, thread/stack override, extra permissive feature, raised
six-hour cap or third unchanged retry. Existing failed attempts stay preserved.

Root owns heavy/local execution, publication, prerequisite reconciliation,
protected merge and final acceptance. Until further transfer, this writer may
perform only implementation/static work: no Cargo/npm/build/test, DB/native operation,
push, merge, CI retry or cancellation. Do not edit another checkout.

## Observed failure and evidence

Exact486 Linux job110067180361 succeeds:791 Rust tests,56 result groups,
table62, the complete original eight flow4 cases and all three separately
audited MR source cases. Windows stable job110067179524 (attempt1) and
job110281107694 (attempt2) are cancelled after six hours; actual check annotations
state the maximum execution time was exceeded. Both finish eight preceding
Rust groups with69 passing tests, then61 of the same62 table cases. No assertion
failure is observed and no Windows package was built. Copied attempt2 MSRV
successes retain their original execution timestamps; they are not fresh runs.

The sole unfinished table case in both attempts is the complete three-history
test, which must remain one unmodified function:

`table_missile_cases::missile_zero_one_two_eligible_sources_keep_uniform_ordering_and_private_acknowledgments`

Attempt2 begins the table binary at11:00:47UTC after its job starts08:16:32UTC.
The preceding genuine old/current Hag coexistence suite takes4093.38s and the
unchanged original eight flow4 suite5268.67s. The table then has11753.05s before
timeout. Its other61 cases pass; the final such completion is12:54:05UTC. The
entire exact Linux table suite completes in10335.39s. Neither a deadlock nor a
profiled production hotspot follows from these logs.

Saved outside-repository evidence under `tooling/ci-recovery-2026-10-04/`:

- `mr-486ce8b-final-evidence-v2.json` SHA256
  `9282a832019c0415a2aec21bbf5b9252a29c530695b13677c0ed84e8e3b7bac0`;
- `mr-timeout-diagnosis.json` SHA256
  `e893d3a4b93a56b8d16a288b61ad99f7da94d6ea23ecf881fc51009e73b21ea1`;
- `review-and-mr-recovery-recommendation.md` SHA256
  `8a207db90381f4e0340579922f439aa5fa295bd28df46a5fb108f87e03cd028d`;
- actual attempt2 log SHA256
  `c33bcbf2b9659ff59120bdd8a32f65bf0ac1589cd1b9f1c2bc74efde5f2f2189`.

The live PR merge ref has recomputed after its Air base moved. Historical Linux
executed synthetic `b80f9dcf646a8bcff7e464aa6afba69fd47fbd61`, whose tree equals
literal486; it did not execute newer synthetic7803f73. New checks must identify
their literal/synthetic execution separately. PR53 must never merge into Air;
accepted-main reconciliation and any retarget remain root-owned.

## Concrete partition and discovery contract

Define one exact partition key: the full test name above, in package `dmd-app`,
integration target `table_loop`. Preserve Cargo's existing default workspace test
selection rather than hand-maintaining all targets.

Both stable regression jobs first discover the complete same-head workspace test
inventory with the existing native target and locked dependency selection:

`cargo test --locked --workspace --target x86_64-pc-windows-msvc -- --list`

Discovery compiles/loads test harnesses but does not substitute for running the
tests. Record each package/target/harness identity, test name, kind and complete
summary, including doc-test groups. Use Cargo metadata/artifact information and
the actual harness boundaries to distinguish equal function names in different
crates; executable filename hashes alone are not portable identities. Map every
observed harness exactly once. Fail closed on incomplete discovery, command
failure, ambiguous identity, duplicate target/test identity, unknown result format
or unexplained missing/extra harness. Legitimately empty harness/doc-test groups
are recorded as empty; they cannot satisfy the dedicated required test.

Require the partition key to occur exactly once in the workspace and only in the
expected package/target. Since `--skip` is a filter, also require that no other
discovered test name contains that full key. An absent/renamed key or substring
collision must fail before declaring an exhaustive partition.

The remainder job executes:

`cargo test --locked --workspace --target x86_64-pc-windows-msvc -- --skip <exact-key>`

The isolated job executes:

`cargo test --locked --workspace --target x86_64-pc-windows-msvc <exact-key> -- --exact`

Both execution commands retain the canonical workspace package selection and
feature resolution. Prior full-workspace discovery alone would not preserve the
feature graph of a subsequent package-only command. The isolated invocation
therefore visits every selected workspace harness, including doc-tests, while
the globally unique exact key permits just one real test to run.

Use structured argument arrays and native exit-code checks. Neither command
changes test concurrency or profile. No ignore/retry/capture switch is introduced.
Capture the child process's stderr into stdout at native process creation
(`stderr=STDOUT`), then read that single pipe into an exclusive raw evidence file
before streaming it to GitHub. Do not reconstruct test ordering from GitHub's
separately timestamped stdout/stderr logs: existing saved logs demonstrate that
a later Cargo harness announcement can otherwise precede the prior summary.
Preserve complete bytes and the actual exit status even through logging.
Keep successful and failed evidence separately; an incomplete process has no
successful result manifest.

For each harness, map literal named outcomes and its summary to its discovered
inventory. Require normal command completion, no failed/ignored/measured cases,
no missing or duplicated outcomes, and exact pass/filter counts. The remainder
must pass every discovered test except the partition key and filter exactly that
one test, in table_loop. On exact486, table_loop is61 passed/1 filtered. The
isolated job must actually pass exactly the named one test across the workspace;
on exact486 its other61 table tests and every discovered test in other harnesses
are explicitly filtered. Every harness must still complete. Derive future
counts from inventory rather than silently accepting61 forever. Extra filtered
tests outside this explicit complement are failure. Summaries alone and global
zero-test success are insufficient.

### Parser feasibility and required controls

Separate compilation identity from test completion. Saved actual Cargo JSON
`compiler-artifact` messages expose package ID, target kind/name/source path,
test profile and executable. Use versioned metadata and those artifacts for the
identity map; canonicalize workspace package/source paths relative to the actual
checkout and match each executable only within its own run. Do not compare
absolute checkout paths or executable hashes across runners. Cargo's
`build-finished` success reports compilation, not completed tests. Test result
JSON is unstable; do not introduce nightly flags or `RUSTC_BOOTSTRAP` to obtain
it. Keep stable libtest discovery/results and the native command exit status.

Recognize anchored Cargo test-harness and `Doc-tests` boundaries, not arbitrary
lines containing `Running` (saved packaging logs also contain unrelated build
commands). Strip known ANSI styling and normalize BOM/CRLF only in the parser's
view; retain raw bytes/hashes unchanged. Reject unknown terminal controls or
ambiguous/incomplete records. Concurrent long-running warnings are progress,
never pass results; completion order need not equal discovery order. Recognize
libtest's optional `should panic`, `compile fail` and `compile` result labels
without incorporating those display labels into the discovered test identity.
Discovery lists ignored tests as ordinary names, so only actual passing results
can discharge coverage; an ignored result, including one with a reason, fails.
Benchmark or custom-harness output needs explicit reconciliation, never omission.

One doc-test crate boundary may contain several valid libtest subgroups: Rust
2024 can execute merged bundles and a standalone group separately. Accumulate
their unique test names under the package/doc-target identity; validate every
list summary and every run-start/result pair, then reconcile aggregate pass and
filter counts with that crate's complete inventory. Retain subgroup counts and
summaries so an extra summary cannot impersonate a new group. A second summary
without its own run start, duplicate named outcome, incomplete subgroup or
unexplained subgroup-count change fails. Do not reject a valid crate merely for
having multiple summaries, and do not flatten away subgroup completion checks.
Current successful Windows evidence contains six empty doc-test crate groups;
each still requires its explicit empty discovery and completed runtime summary.
Do not add `--show-output` merely to flatten docs: it changes rustdoc's merging.

These semantics were checked against actual saved Windows/Cargo output and
primary Rust sources because no saved `--list` transcript or installed HTML docs
were available locally. Rustdoc forwards test arguments into each merged runner
and executes standalone cases separately. See the [rustdoc dispatcher](https://raw.githubusercontent.com/rust-lang/rust/1.98.0/src/librustdoc/doctest.rs),
[merged runner](https://raw.githubusercontent.com/rust-lang/rust/1.98.0/src/librustdoc/doctest/runner.rs),
[libtest formatter](https://raw.githubusercontent.com/rust-lang/rust/1.98.0/library/test/src/formatters/pretty.rs),
[discovery implementation](https://raw.githubusercontent.com/rust-lang/rust/1.98.0/library/test/src/console.rs)
and [Cargo external-tool protocol](https://doc.rust-lang.org/cargo/reference/external-tools.html).
This read-only investigation establishes implementation constraints, not a new
executed discovery result or acceptance evidence.

Keep `legacy_shield_missile_v1_replay` as one unchanged remainder target, with all
eight original named tests required to pass unfiltered. Preserve their current
source/fixture bytes, full cold restore/retry scenarios and existing receipt
assertions. No original test is moved into a replacement module or rewritten.

## Cross-job evidence and packaging topology

Retain the existing Windows1.88.0 and stable frontend checks, all-target workspace
checks, stable format and desktop Clippy checks. The stable matrix entry executes
the remainder. A separate stable job executes the isolated case, installing the
same Rust target and preparing any required desktop assets for full discovery.
Both jobs check out `${{ github.event.pull_request.head.sha || github.sha }}`
explicitly and record the actual HEAD/tree. Preserve cache/profile settings.

Each regression job saves a versioned manifest containing actual HEAD/tree,
workflow run ID/attempt, job identity, commands, toolchain/target, inventory,
literal test outcomes, summaries and full-log hashes. Upload successful evidence
only after local validation. Failure logs remain available but cannot masquerade
as successful manifests. Artifact names must distinguish partition and execution
attempt; do not accept evidence from another SHA, run, cached build result or
earlier failed attempt merely because a filename matches.

Add a separate package job with explicit `needs` on the existing Windows matrix
(including MSRV/stable checks and remainder) and the isolated regression job.
Do not use an unconditional/always packaging condition. Before building, a small
coverage check loads both current-run manifests and verifies:

1. The source HEAD/tree, native target, run and attempt identities match the
   package job's actual checkout and required successful executions. Require both
   regression manifests from this same workflow attempt; mixed-attempt partial
   reruns fail closed and require a coherent fresh pair before packaging.
2. Complete discovered inventories match, including all doc-test/empty groups;
   both inventories independently prove the unique partition key and skip safety.
3. The sets of actual successful test identities are disjoint; their union equals
   the complete discovered workspace inventory. The sole remainder omission is
   the isolated real passing case. Every other isolated filter is proven passed
   in the remainder. No unexpected result, missing test or duplicate pass remains.
4. Every expected harness completed; the original eight cases passed together
   without filtering. Each manifest's recorded counts agree with literal logs.

Fail before packaging on any mismatch or absent evidence. The package job then
runs the existing preparation/package path and uploads the existing source-named
Windows package with unchanged payload/notice/inventory validation. The other
Linux workflow remains independently required for merge; cross-workflow status
is not invented as a `needs` dependency. Do not publish a package before the
Windows partition coverage proof and all listed Windows checks pass.

## Planned slices and validation

1. Commit this plan and refreshed MR current status. Return the clean head/tree
   for root review; no helper/workflow implementation before that review.
2. Implement the small inventory/results validator and evidence orchestration,
   then wire the workflow topology. Keep production/test/fixture bytes unchanged.
   Explain actual Cargo/doc-test output handling rather than assuming its shape.
3. Add meaningful validator controls using representative captured output and
   manifest cases: valid exhaustive union; absent/duplicate/substring-colliding
   key; zero dedicated pass; missing/unexpected/ignored/failed/duplicate result;
   incomplete harness/doc discovery; wrong SHA/tree/target/run/attempt; mismatched
   inventories; tampered log/hash; missing original case; evidence from failure.
   Include ANSI/CRLF and Windows paths; reordered concurrent passes/long-test
   warnings; all-filtered isolated harnesses; empty doctests; valid multiple
   doctest subgroups; ignored-with-reason/mode labels; duplicate or missing
   subgroup summaries; and a Cargo build success without completed tests.
   These verify the new CI guard, not copied assertions about untouched gameplay.
4. Independently review the complete delta, argument/filter semantics, parser and
   topology. Root schedules any helper tests and exact-head CI after static
   review, preserving all actual failures. Verify every new job's actual logs,
   original-eight outcomes, inventory/pass union, packaging dependency and actual
   uploaded artifact source/hash. No execution result transfers from486.
5. Root still runs unchanged local canonical `./scripts/verify`, reconciles
   accepted dependencies, verifies the receiving head and native obligations,
   then updates the MR plan with qualified evidence. This CI repair alone does
   not accept the source prerequisite or complete Gate4.

Acceptance requires real successful completion of both native partitions and
the final coverage/package job, alongside existing required checks, with no test
or product weakening. New job names must be reconciled with required-check
configuration by root before protected merge; inability to inspect/change that
configuration is reported, not bypassed. GitHub's job limit remains unchanged.

## Risks, unresolved obligations and next action

The single case could still exceed its own job limit; the successful Linux full
table duration and repeated Windows progress support the partition but do not
prove success. If a new exact run fails, read the actual failure and collect
bounded diagnostic evidence before changing production behavior or CI strategy.
Cargo/doc-test discovery and parsing must fail closed without discarding tests.
Future renamed tests, new harness kinds or output changes must require explicit
reconciliation rather than silent green checks. Additional compilation/discovery
is intentional; no faster runtime is claimed.

Actual positive Counterspell/repeated-save Magic Resistance gameplay, remaining
canonical/native evidence, accepted-main integration and the wider Gate4 finish
line remain open under the MR plan. No obligation moves to Gate5.

Next action: implement only the approved CI helpers, their pure tests and workflow
topology; return the complete clean static checkpoint for independent review.
Root schedules helper tests and native evidence afterward. The writer retains
sole branch ownership until explicitly transferred back.
