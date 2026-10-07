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

## Invocation decision — before implementation, 2026-10-07

Use four fixed allocations: `table-loop`, `legacy-missile`, `grapple-public`,
and `remainder`. The first three select only the matching complete `dmd-app`
integration target; remainder owns every other executable and all doctests.
Discovery, not this list, determines the complete required universe. A genuinely
absent named allocation is explicit in every manifest and must still return a
valid empty receipt; no existing target may be omitted.

Each allocation builds the **whole** ordinary workspace first:
`cargo test --locked --workspace --no-run --message-format=json`, adding only
the existing `--target x86_64-pc-windows-msvc` on Windows. Cargo metadata and
compiler-artifact messages bind package/target identity, source, actual enabled
features, profile, all compile-only outputs and every test executable/hash.
There is no isolated package/target build, feature override or artifact transfer
between differently rooted jobs. Each job independently builds the same graph.

Execution uses `cargo test --locked --workspace --message-format=json` with
the same platform argument. The only execution configuration additions are a
`target.<host-triple>.runner` and a `RUSTDOC` delegate. Cargo still constructs
the commands, package working directory, library paths and runtime environment.
The runner receives Cargo's actual executable and original arguments. It checks
the exact inventoried executable/hash and actual package cwd, lists all/ignored
cases, and invokes selected complete harnesses with **unchanged** original
arguments and inherited cwd/environment. Unselected harnesses produce explicit
listed-only receipts, never passing execution receipts. No test filter or thread
argument is added to an executed harness. A custom/non-libtest harness, ignored
case, unsupported argument/listing or configuration is an explicit failure.

The rustdoc delegate forwards Cargo's original tool arguments/environment/cwd
to the actual toolchain rustdoc. It lists all/ignored doctests for inventory;
only remainder executes the original unmodified rustdoc test command. This
avoids substituting a separate `--doc` build graph. Non-test rustdoc queries are
delegated unchanged. Doctest identities, original invocation and full results
remain separately accounted for, including zero-case crates. Unsupported output
or semantics fail closed. Runtime Cargo artifact graphs must equal discovery;
all four jobs must agree on normalized source/configuration/build/listing scope.

Each complete executable has its own stdout/stderr log boundary, eliminating
the interleaved adjacent-target attribution error found in the timing draft.
Recorded test names and libtest totals must equal the discovered case set;
failed, ignored, measured, filtered, incomplete or duplicate results refuse a
successful receipt. Compilation-only outputs remain in the full graph proof.
Source head/tree, clean tracked content, toolchain, target, lockfile, metadata,
graph, listings, original command, executable hashes and logs bind each result.
No credentials or complete environment values are serialized into artifacts.

Receipts and logs are uploaded even on failure, but success is written only
after final source/graph/coverage checks. A separate required aggregate downloads
the exact four same-run artifacts and verifies their complete disjoint executed
union and identical inventory. Missing/failed/cancelled/skipped prerequisites
fail it. Linux keeps the final `rust` check name. Windows retains `Windows 1.88.0`
and final `Windows stable`: stable frontend/check/lint, partition receipts,
runtime aggregation, and unchanged fresh packaging/upload must all succeed.
Packaging remains a separate post-aggregate job and preserves the original
source-head checkout and package artifact name. Canonical scripts/verify and
scripts/verify-fast stay byte-identical.

Official semantics additionally read before this decision:
[Cargo runner configuration](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner),
[Cargo environment](https://doc.rust-lang.org/cargo/reference/environment-variables.html),
and [Cargo JSON artifacts](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages).
This is the implementation choice, not a claim that wrappers or equivalence have
run successfully. Actual two-platform comparison and independent review remain
mandatory; only lightweight Python runner tests are currently allocated locally.

Cargo source was also checked at
[`1fd17bcd77ee9d4883d0a766a5995184a4064ff9`](https://github.com/rust-lang/cargo/tree/1fd17bcd77ee9d4883d0a766a5995184a4064ff9):
`src/compiler/build_context/target_info.rs` initializes the normal host
configuration from the host target tuple even without `--target`;
`src/compiler/compilation.rs` selects that runner, then applies the original
package cwd, package/build-script variables, dynamic library paths and jobserver
inheritance to the runner process. Build scripts retain the ordinary host path.
The Python delegate must preserve inheritable descriptors as well as cwd/env
(`close_fds=False`), rather than quietly breaking Cargo's jobserver inheritance.
This supports retaining the Linux command without adding `--target`; actual CI
must still prove every expected executable reached the wrapper on both platforms.
No unstable host configuration is accepted.

Executable bytes are compared with the canonical no-run inventory **within each
job** before/after its actual execution. Across independent builds, require equal
source/toolchain/platform/graph/case identities, not assumed reproducible binary
hashes or Windows PDB paths. Each job retains its own actual artifact hash. No
source-only artifact manifest substitutes for a runtime receipt.

The same pinned Cargo `src/ops/cargo_test.rs` also injects the configured target
runner into rustdoc as `--test-runtool` / `--test-runtool-arg`. The rustdoc
delegate must verify this is **exactly our own** execution instrumentation and
remove only those fields before listing or executing the real tool. The result
is the original canonical rustdoc argument vector, including Cargo's actual
source paths, extern libraries, package `--test-run-directory`, flags and target.
It must not route dynamically generated doctest programs through the workspace
executable inventory. Unexpected runtools/arguments fail closed. The injected
and restored vectors are both recorded, and negative tests cover this restoration.

Each Windows allocation runs the existing `desktop-prepare.ps1` unchanged in
its own fresh checkout before Cargo discovery. This supplies the original
frontend checks/build, icons and licenses required by the desktop build without
transporting generated build assets or assuming equivalent machine paths.
Repeated frontend runs are recorded as repeated validation, not extra distinct
test coverage. The final packaging job also uses the unchanged packaging script
and its own preparation. There is no new packaging shortcut or stale binary reuse.

## Implementation handback — 2026-10-07

The new `scripts/ci_runtime.py` and its focused Python controls implement the
discovery, Cargo-owned delegates, complete original logs, within-job executable
hash checks and cross-job disjoint-union validation described above. Aggregation
reparses original metadata, both Cargo compilation graphs, all/ignored listings
and executed results; it does not trust a compact success label alone. Discovery
and actual original cwd/arguments must agree across jobs. A listed-only target
cannot carry an execution result. No executable bytes are transported between jobs.

Both workflows now separate complete runtime allocations and aggregation while
retaining the original required final names and existing check commands. Failure,
cancellation or skipping of a prerequisite makes the final required check fail;
Windows packaging runs only after its complete runtime aggregate. GitHub's
existing job limit, default test profile/thread count and workflow concurrency
remain unchanged. The new artifact download action `actions/download-artifact@v8`
was checked against its official tag, commit
`9000827ccba6bdab643e8b6fd33ac0654aef8333`.

Local validation completed: 15 lightweight Python tests pass via
`python -B -m unittest discover -s scripts/tests -p test_ci_runtime.py -v`.
They include a real harmless Python subprocess for cwd/env forwarding, complete
aggregate receipt/log reconstruction, altered/missing original logs, missing or
duplicate allocations, incorrect source/platform/run identity, false listed-only
outcomes, unsupported harness/listing semantics, complete doctest groups, zero
cases, incomplete named results and exact rustdoc instrumentation removal.
`git diff --check` passes. No local Cargo, npm, Rust/gameplay test, native app or
database work was performed for this slice. `scripts/verify-fast` and
`scripts/verify` are UNRUN on this head because that runtime was not allocated.

This is a review candidate, not accepted replacement CI. Independent full source
review, actual workflow syntax/execution, both real Cargo delegate paths, complete
two-platform receipts, packaging, and comparison with canonical main4cf are still
required. In particular, Python fixture tests cannot certify Cargo/rustdoc output
semantics, Windows command forwarding or hosted completion times. Unsupported
actual semantics must cause a visible failure and a separately reviewed fix.
No elapsed-time improvement or full workspace pass is claimed yet. Next action:
root/fresh reviewer inspect the frozen complete diff and preservation inventory,
then root may publish for genuine two-platform validation. Keep canonical main4cf
jobs running and preserve every terminal outcome.

## Hosted Windows fixture correction — before implementation

PR73 head `5413da89ade6045c197b5633988dcb02257bb9be` was independently
reviewed and published. Actual Windows stable-checks job112819324651 failed its
new `test_custom_harness_is_refused_before_invocation` Python control: the
expected `harness=false` refusal was preceded by `workspace package outside
checkout`. The fixture passes an unresolved `TemporaryDirectory` path, while
workspace discovery resolves its manifest before checking containment. This is
a fixture path-normalization mismatch on the hosted Windows environment; the
production allocation entry point already resolves its checkout root.

Apply only `Path(directory).resolve()` in that fixture. Preserve its custom
harness manifest, exact refusal assertion, all other controls and production
runner/workflow bytes. Rerun the lightweight Python controls, freeze and request
review before root publishes a successor. Actual Windows execution is still
required. Do not relabel the failed original run. The original Linux empty
Grapple allocation and remainder have uploaded artifacts, but their real complete
receipts still require inspection; no full-workspace aggregate passed.

The aggregate also requires one workflow run **and attempt** for all four
receipts. A partial retry that mixes earlier successful allocations with a newer
attempt refuses. Use a full workflow rerun when retrying this strict design.

The one-line fixture correction is implemented. All15 lightweight Python controls
pass locally again (0.739s), and `git diff --check` passes. Runner and workflow
blobs remain exact5413. No Cargo/npm/native/database execution was performed;
fresh hosted Windows proof and independent correction review remain pending.

## Windows executable-change diagnostics — before implementation

Published successor `8d23f55f36a355de00447a0651325bd7aedfbd84` passes the
15 Python controls on hosted Windows. Runtime jobs112903985987 and112903985905
in run37653439392 nevertheless fail the existing before-execution byte guard:
the `dmd-desktop` executable changes after the canonical no-run compilation.
Both preserved artifacts contain593 Cargo messages in each phase; removing only
the `fresh` field leaves identical graphs. Only the desktop executable is rebuilt
in the second phase. Its actual replacement hash and Cargo's dirty reason were
not recorded. Do not infer a cause from equal graph fields or a stable filename.

Root allocates a diagnostic-only successor. Keep the exact canonical argv,
phase order, discovery, graph checks, byte guards, harness execution, aggregation,
profiles, threading and timeouts. Set only Cargo fingerprint INFO logging for
both compilation phases, retaining complete stderr. Before each phase write a
separate diagnostic JSON file with the exact argv, cwd, source/run identity,
runner/Python identity and an explicit allowlist of relevant configuration
values; never serialize the full environment or credentials. On an executable
hash mismatch write expected and actual SHA256, target identity, path and guard
phase before refusing with the existing error. Diagnostic files must not match
the harness `*.receipt.json` glob and cannot confer a passing outcome.

Add lightweight controls proving mismatches still refuse and never execute a
harness or create its receipt, and that matching bytes retain the existing
execution path. Verify the diagnostic environment allowlist excludes unrelated
values. No local Cargo/npm/native/gameplay/database run is allocated. Freeze the
bounded patch for independent review before publication. First obtain an actual
dirty reason; no stabilization rebuild, binary normalization, accepted old hash,
guard relaxation or other build-semantic correction is authorized here.

Original failed artifacts and prior partial runs remain preserved. Original5413
Linux has since completed its genuine56-target/817-case union, matching completed
literal main4cf; this does not pass the current head or either Windows failure.

The diagnostic-only implementation retains all three executable byte guards
(before execution, after execution and final allocation check), their original
messages and failure behavior. Its separate JSON files are excluded from harness
receipt discovery. Both compilation phases retain their original argv and add
only the fingerprint logging environment value. All18 lightweight Python controls
pass locally, including the original15 unchanged bodies and three new controls
for pre-execution refusal, matching/post-execution/final guards and restricted
phase provenance. `git diff --check` passes. No project runtime was executed.
Next: independent full bounded source review, then root-controlled publication
and inspection of actual Cargo dirty reasons. The underlying cause remains open.

Independent review of unpublished4927674 found a diagnostic blocker: the chosen
`cargo::core::compiler::fingerprint` filter is an obsolete module namespace.
Actual8d Linux receipts identify Cargo1.99.0 commit
`5f94df4789f005f9a352888e8355ffc645b7ed0e`; the hosted Windows Rust commit
`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4` pins the same Cargo submodule.
That exact Cargo source's `src/compiler/fingerprint/mod.rs` documents
`cargo::compiler::fingerprint` and emits its INFO dirty-reason events through
the default module target. The earlier legacy-only filter would not capture
those events. Before publication, correct only the logging constant to
`cargo::compiler::fingerprint=info`, retain all prior controls and guards, rerun
the lightweight suite and freeze again for review. No build-semantic change is
introduced; the actual desktop dirty reason remains unknown.

Pinned official sources:
[hosted Rust Cargo submodule](https://github.com/rust-lang/rust/tree/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/src/tools/cargo)
and [Cargo fingerprint implementation](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/compiler/fingerprint/mod.rs).

The logging namespace correction is implemented; all18 lightweight controls pass
again (0.839s), and diff whitespace checks pass. The single constant is the only
code change after4927674. Hosted diagnostic output remains unrun and requires
fresh review/publication. Preserve4927674 as the reviewed-but-corrected candidate;
do not imply that its obsolete filter could diagnose the actual hosted rebuild.

Before final publication, root approves one additional bounded evidence fix:
persist the already-read `rustc -vV` and `cargo -vV` strings in both pre-phase
diagnostics. Currently those strings reach only `complete.json`, so a failure
loses direct toolchain provenance. Pass the existing strings to the diagnostic
helper without invoking any extra command or changing compilation. Extend only
the new provenance control, rerun the18 lightweight controls and freeze the full
cumulative patch for final review. No further optional diagnostic expansion.
