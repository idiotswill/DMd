# Gate 4 — Exhaustive Windows CI partition for slow recovery coverage

Status: **Plan amendment only after the first real isolated job failed its
coverage guard.** The exact named test passed; the partition did not receive a
successful evidence manifest. The command-selection correction below is proposed
and has not been implemented or executed.
Sole PLAN-ONLY writer: `gate4_ci_oct4`, transferred by root on 2026-10-04 after
root read the complete failure diagnosis and exact Cargo source trace. Source,
helper tests, native work and publication require the next explicit root transfer.
Branch: `codex/gate4-magic-resistance-source`; checkout `gate4-shield-missile-runtime`.
Draft [PR53](https://github.com/idiotswill/DMd/pull/53) and freshly fetched branch
head: `ad3b82b7ba5741381ec6314b80c591f0d2f6251a`, tree
`f6ed6762420e92332435eeb6843529a981dd0cd6`. This is unaccepted development work.

Root approved the bounded diagnosis/proposal, external
`tooling/mr-ad3-isolated-diagnosis-2026-10-04/diagnosis-and-plan-proposal.md`,
SHA256 `10bde088845ab4c89a8ebe0d15086199e1e2ff732b3ec6605fbf5fe9898de1bd`.
This amendment must return as a clean docs-only checkpoint for root review before
source editing. It preserves the parser, doctest inventory and package topology.

Historical plan review: root and peer required full workspace selection and a
native combined output pipe, incorporated in `fdb106f5d830203e2ca2f4b9fec2431014366d99`.
Root review `tooling/mr-fdb106f-root-plan-review-2026-10-04.md` has SHA256
`3cd51872eceebe3a7317dcf5072ac4bd00264b3b45419412d370f2cc96111477`.
The subsequent source review and 29 pure helper passes remain attributed to their
old head. Actual CI disproved the original positional-filter target-selection
assumption; neither earlier review nor the synthetic controls certify this fix.

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
edit only this plan and directly related MR status: no source/helper implementation,
Cargo/npm/build/test, DB/native operation,
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

## First actual partition failure at ad3, 2026-10-04

Windows workflow `37206167960`, attempt1, isolated job `111447686335`
checked out literal `ad3b82b7ba5741381ec6314b80c591f0d2f6251a`, tree
`f6ed6762420e92332435eeb6843529a981dd0cd6`. The saved API records failure,
13:35:38Z start and 14:50:40Z completion. This was not the PR synthetic merge head.
All six native subprocess receipts exit 0 and match their literal raw log hashes.

Discovery found 793 named tests in 56 harnesses: 50 compiled executable harnesses
and six empty doctest crate groups. The isolated invocation completed the same
50 executable harnesses, with one named actual pass and 792 filtered tests.
Every executable pass/filter count reconciles with discovery. The unique
partition key in table_loop passed in 4116.01s with 61 peers filtered; its literal
pass appears 14:49:45.140Z. This qualifies the old target test only.

The execution omitted exactly Doc-tests dmd_app, dmd_conversation, dmd_core,
dmd_domain, dmd_persistence and dmd_rules. The helper rejected the incomplete
harness set at 14:49:45.508Z with `missing/extra runtime harness or docs`.
It produced failure.json and no successful manifest. Empty doctest discovery
is not evidence that those targets completed. The original eight remain one
unmodified remainder suite and are all filtered in isolated execution, as
intended; this failure supplies none of their required remainder passes.

Actual toolchain: rustc 1.99.0, commit b940084d7eb6a299eb4bfeb8e34901bc051e7ac4;
Cargo 1.99.0, commit 5f94df4789f005f9a352888e8355ffc645b7ed0e; native MSVC target.
The exact [Cargo command dispatcher](https://raw.githubusercontent.com/rust-lang/cargo/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/bin/cargo/commands/test.rs)
places the positional TESTNAME into test_args but also changes otherwise-default
target selection to all_test_targets. Arguments after -- reach the same test_args
without that switch. The [filter definition](https://raw.githubusercontent.com/rust-lang/cargo/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/compile_filter.rs)
uses an Only filter; the [unit generator](https://raw.githubusercontent.com/rust-lang/cargo/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/unit_generator.rs)
adds default library doctests in its Default/Test branch. The [test runner](https://raw.githubusercontent.com/rust-lang/cargo/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_test.rs)
forwards test_args to executables and through rustdoc test-args. This source trace
explains the actual omission and supports moving KEY after --; it is not a
success result for the proposed command.

Preserved original evidence in `tooling/ci-independent-four-heads-2026-10-04/`:

- `mr-ad3b82b-isolated-failed-11306751432.zip`, artifact 11306751432,
  SHA256 `303187c741f885cb67184b824ac96b5da3aa7755a2db01c77c732c8ab54c23c9`;
- `job-111447686335.log`, complete GitHub job log,
  SHA256 `c01402676398b17a5a88e52b3a42cafbd969318d1ca7152e55f83d8daeb50722`;
- original API snapshots, core artifact audit and both previous six-hour attempts.

Independent diagnosis directory `tooling/mr-ad3-isolated-diagnosis-2026-10-04/`
contains 13 inert decoded log/JSON entries and exact upstream source text. Its
`independent-audit.json` SHA256 is
`5b6c12aca772f93c6c19e9e0b631418d2be5384429545045e02cd63a65307ff6`.
It records all entry hashes, six receipts, exact identity, all 50 executable
reconciliations and the missing doc set. The literal execution.log SHA256 is
`7857a052c17b469a1012786e9421219b8fcbd96533e1dee6569da336254c4afe`;
discovery.log SHA256 is
`ca0cee11dc076a35bb6e84540a8371ec2dd7280ccca364642ac38391da06f26e`.
No artifact is repaired or promoted to successful evidence. The saved passing
test cannot be combined with another head/run/attempt to authorize packaging.

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

`cargo test --locked --workspace --target x86_64-pc-windows-msvc -- <exact-key> --exact`

Both commands must retain default workspace target selection and feature
resolution. Define BASE as the identical argument list through the native target;
use BASE + ["--", KEY, "--exact"] for isolated execution and
BASE + ["--", "--skip", KEY] for the remainder. Every filter is on libtest's
side of the separator. Prior discovery alone cannot preserve a later command's
selection. A Cargo-level positional TESTNAME changes default selection even with
--workspace; the failed ad3 attempt omitted all six doctest harnesses for that
reason. The old command's claimed doctest coverage was false. The corrected
command is required to visit every discovered harness, including doc-tests,
while the unique exact key permits just one real test to pass. This amended
behavior remains to be verified on an actual successor head.

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
Prior successful unfiltered Windows evidence contains six empty doc-test crate
groups. The ad3 default discovery also lists all six; its positional-key execution
omits them and is rejected. Each still requires its explicit empty discovery and
completed runtime summary. Do not relax this requirement for the failed command.
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

## Original implementation slices and validation

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

At ad3 the isolated case actually completed in 4116.01s, but the missing doctest
harnesses prevented partition success. This result does not verify the amended
command or a future source head. If a new exact run fails, read its actual failure
and collect bounded evidence before changing production behavior or CI strategy.
Cargo/doc-test discovery and parsing must fail closed without discarding tests.
Future renamed tests, new harness kinds or output changes must require explicit
reconciliation rather than silent green checks. Additional compilation/discovery
is intentional; no faster runtime is claimed.

Actual positive Counterspell/repeated-save Magic Resistance gameplay, remaining
canonical/native evidence, accepted-main integration and the wider Gate4 finish
line remain open under the MR plan. No obligation moves to Gate5.

## Historical implementation checkpoint, 2026-10-04

Authored `scripts/windows_test_partition.py`, its pure adversarial tests in
`scripts/tests/test_windows_test_partition.py`, and the bounded desktop workflow
change. The implementation first records Rust/Cargo versions and version1 Cargo
metadata, then uses the same locked workspace `cargo test` selection with
`--no-run --message-format json-render-diagnostics` solely to map compiled test
executables to package/target identities. This extra compile-only phase does
not replace either full discovery or actual execution. It reuses ordinary Cargo
outputs; profiles, features, concurrency and assertions are unchanged.

Both jobs write exclusive raw native combined-stream logs and separate command
receipts outside the checkout. A successful manifest appears only after every
command exits0, the source remains clean at the expected head/tree, discovery
is complete and every expected literal named pass is reconciled. Failure evidence
does not supply a successful manifest. The package job downloads evidence named
for both partitions and the current run/attempt, verifies hashes/receipts,
reparses the raw logs, compares toolchains and proves the disjoint actual-pass
union before invoking the existing preparation/package scripts. Its normal
`needs: [windows, isolated]` requires both Windows matrix entries and the isolated
job; it has no unconditional packaging condition.

The pure tests exercise synthetic transcripts shaped from the actual stable
Cargo/libtest protocol, including genuine original-eight names, variable totals,
empty binary/docs, multiple doctest subgroups, terminal formatting, serial and
concurrent progress, missing/extra/ignored/failed results, substring collisions,
incomplete/duplicate summaries, bad artifact identities, altered raw logs,
false manifest claims, command failures and cross-job provenance mismatches.
Subprocess calls are mocked in capture tests; these tests never invoke Cargo or
Rust gameplay. Workflow Python invocations use `-B` to avoid untracked bytecode
making the subsequent clean-source guards fail.

Static author checks: both Python files parsed with `ast.parse`; `git diff
--check` passed. The pure test suite has **29 authored test methods and has not
been executed** at this checkpoint. Neither actual discovery nor the native
partitions have run. No Rust source, gameplay test, fixture, content, Linux
workflow, `scripts/verify` or `scripts/verify-fast` change is included. Existing
failed Windows evidence remains unchanged. The artifact-download version and
current-run behavior were checked against the [official download-artifact documentation](https://github.com/actions/download-artifact#usage).

Historical next action at that implementation checkpoint: independently review
the complete clean static checkpoint, then root schedules
`python -B -m unittest discover -s scripts/tests -p test_windows_test_partition.py`
and any additional bounded guard checks. After
fixing actual findings, root schedules fresh exact-head native CI and the
unchanged canonical acceptance obligations. That implementation writer then
returned ownership to root; the current
plan-only assignment at the top supersedes this historical next action. Publication
and final acceptance remain root-owned.

## Root review and pure guard execution — October 4

The clean static implementation is
`aae8d47b7caef8a0a301d190f7921f8079ef45f8`, tree
`87860497663bf5b1250d73c47553deb24de0d782`. Root read all five changed files,
the entire helper and adversarial suite, workflow topology, preparation and
ignore context. No blocking finding was identified. Root review memo SHA256:
`1b6f6913a68f394a270a895f49021eb5d4fbca34f91f07a0f0d33f27928e32bb`.
Writer ownership returned to root; the source remains frozen for peer review.

Root explicitly allocated the lightweight stdlib Python suite while Air kept
the sole heavy/native slot. At clean exact aae8d47, the actual command was
`C:/Users/jadra/AppData/Local/Programs/Python/Python311/python.exe -I -B -m unittest discover -v -s scripts/tests -p test_windows_test_partition.py`.
All29 tests passed in0.983 seconds with process exit 0. Head and clean status
were unchanged afterward. Full log SHA256:
`9ff99a5292013727d6ac7c45e5cab37711b34333fb380ee9082589efc796180d`.
Its printed14-case proof is synthetic test output, not actual native regression
coverage. No Cargo, npm, database or native application command ran in this
allocation. First real native discovery/partition/coverage/package execution
and the unchanged MR canonical/native acceptance remain outstanding.

Independent complete implementation review is also CLEAR for fresh verification,
SHA256 `bef0be09986afd543c6949d66268386da0f36b5ac7656aa98f3575becd01e875`.
Root read that memo in full. The peer independently checked all changed files,
actual workspace manifests, original-eight names, native transcript semantics,
raw proof and packaging dependencies; it ran no executable checks. Root freshly
fetched the published MR branch and confirmed it remains486ce8b with no unexpected
movement. Root published the reviewed implementation plus result-only documents
at ad3
to draft PR53. Its first real isolated attempt is the failure recorded above.
Its development Air base stays unaccepted and must never receive a merge.

## Current bounded correction plan — argument placement only

Root read the complete external proposal and independent audit, then checked
exact Cargo argument/target/rustdoc forwarding sources. It approved this smallest
correction for a PLAN-ONLY handback first. Source editing has not started.

1. Commit these directly related plan/status changes and return the exact clean
   head/tree for root review. Preserve every old failure and qualified result.
2. Only after root reviews this amendment, change commands() isolated execution
   from BASE + [KEY, "--", "--exact"] to BASE + ["--", KEY, "--exact"]. Retain
   metadata/artifact/full-discovery commands and remainder argv exactly. Add a
   concise comment explaining Cargo's positional TESTNAME target-selection effect.
3. Strengthen the existing command-selection control: split discovery and both
   execution argv at their literal --, require the Cargo prefix to equal BASE,
   and require the exact discovery/remainder/isolated suffixes. Presence of
   --workspace alone is insufficient. Cargo-level positional names, -p, --tests,
   --all-targets or --doc cannot replace this default-selection contract.
4. Add a representative transcript with complete discovery, one named isolated
   pass and all executable summaries correct, but every doc runtime section absent.
   It must reject the same missing-harness failure. Also remove only an empty
   doc runtime group and require rejection; empty completion remains mandatory.
5. Add an otherwise valid isolated bundle with the old positional argv in both
   its manifest and separate execution receipt. It must fail exact command
   validation even with matching raw hashes and internally consistent metadata.
6. Add an orchestration negative whose six mocked subprocesses all exit 0 and
   hash-match but whose execution lacks doc sections. Require failure.json and
   the absence of manifest.json. No synthetic result substitutes for native CI.
7. Preserve all 29 existing pure controls. No parser, schema, inventory or union
   relaxation is authorized. Remainder must actually pass every discovered test
   except KEY; isolated must actually pass exactly KEY, and both must complete
   every executable/doc harness with exact named-pass/filter/subgroup checks.
   Both matching inventories, original eight unfiltered passes, native combined
   logs, exact receipts and source/tree/run/attempt/toolchain checks remain.
8. Source scope is commands(), meaningful pure controls and directly related
   result notes. Workflow/package dependencies, Linux, local canonical scripts,
   Rust/gameplay tests/fixtures/content, profiles and concurrency remain intact.
   A broader change requires evidence and another root plan review.
9. Return the coherent static source checkpoint for independent review. Root
   may then allocate the stdlib suite and a bounded list-only diagnostic using
   corrected argv before expensive fresh CI. Such a probe would only establish
   selection/boundaries and cannot replace actual named test passes. No probe,
   helper suite, Rust/native/DB or CI operation is authorized in this amendment.
10. Root preserves current remaining-job evidence and schedules publication.
    Acceptance still needs a fresh exact successor-head pair, complete coverage
    proof and package job plus all existing required checks. Never manufacture
    a success manifest for ad3 or mix its pass with a new head or attempt.

Separate executable/doc partition inventories are not justified: correcting
argument placement restores the approved complete-default-workspace contract.
The failed command's six absent doc summaries cannot be invented or ignored.
The present evidence schema already pins exact argv; this correction needs no
schema change solely to move an argument. Actual new output incompatibility
would be a fresh finding requiring explicit reconciliation.

Exact next action: root reviews the clean docs-only amendment and transfers
bounded source writing separately. Until then gate4_ci_oct4 owns only the two
plan files; Air87840 keeps root's sole heavy slot. Canonical/native/source
acceptance, accepted dependencies and full Gate4 closure remain outstanding.
