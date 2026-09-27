# Gate 4 — Shared immutable tactical catalog

Status: all six candidate CI checks pass at `faca063`; the controlled five-history
ABBA comparison passes. Owned-Shield measurement, canonical verification, final
successor checks and post-main proof remain pending. Root is
the current sole writer after the implementation handoff. Original writer:
`source_registry_review`. Branch:
`codex/gate4-immutable-catalog-cache`, based on reviewed Shield candidate
`bfe4aac67ed80a5aec5165cfdd198acdbbfec9c5`. Fetched main before branching was
`f441adedcf490504b6f1e3db1a964c023c511e47`. The preserved prior branch
`codex/gate4-shield-missile-app` remains at `8537775`. Draft PR:
<https://github.com/idiotswill/DMd/pull/47>.

## Objective and boundaries

Share one validated immutable instance of the tactical catalog compiled into this
binary. Runtime callers currently parse the same embedded bytes repeatedly. This
is a static observation, not evidence that parsing is the dominant application
cost. The cache must preserve source definitions, ordering, fingerprints, exact
wrapper errors and all historical execution behavior.

This supports the product definition's local production gameplay and accurate
restart/replay requirements within active Gate4. It does not accept a tactical
family or complete Gate4, and does not replace Gate13 performance acceptance.
AGENTS, the gate execution protocol, the active Shield plan and reaction umbrella
remain binding. ADR028's execution/history boundary is unchanged because no command,
state, event, source definition, serialization or executor semantics change.

Root owns the sole local heavy slot. The superseded uncached Magic Missile run
was intentionally stopped with all four cases unfinished; its logs and candidate
backups remain preserved, and it supplies no complete-case result. All current
Magic Missile acceptance remains required in its separate slice. This cache's
measurements and canonical verifier run serially with no other heavy local job.
Reviewed development/CI publication is permitted while local verification runs;
it does not accept the slice or waive any remaining check.

## Design before implementation

- Add a neutral `tactical_definitions::bundled_tactical_definitions` accessor,
  returning `Result<&'static TacticalDefinitions, DefinitionError>` through
  `OnceLock<Result<TacticalDefinitions, DefinitionError>>`.
- Retain lazy validation and error results for immutable compiled bytes. Map the
  unchanged `DefinitionError` at each existing wrapper. In particular, routing
  tactical callers through `CreatureError` would change their error text and is
  excluded. Existing creature lookup delegates to the neutral cache.
- Route runtime reads of `TACTICAL_DEFINITIONS_JSON` through this accessor, using
  borrowed definitions and cloning only fields already owned by returned values.
  This includes tactical/spell lookups, app attack/opportunity presentation,
  fighter mastery choices and the separately cached equipment registry.
- Keep `TacticalDefinitions::from_json` fully validating arbitrary input. No
  initializer recursion, mutable global state, eager panic or default fallback.
- Do not change live filesystem/manifest/byte-integrity checks in
  `dmd-app::rules_runtime::load_rules_pack`, state/source-pin/fingerprint validation,
  replay coverage, command acceptance, persistence, projection or authorization.
- Do not remove repeated state validation as part of this optimization. Do not
  cache campaign-dependent results. Future immutable historical source revisions
  must retain their own exact material; this accessor supplies this binary's fixed
  catalog and is not a latest-definition resolver or unknown-pin fallback.

## Static observations and planned slices

`tactical::definitions` reparses the entire catalog during group validation. An
established tactical command validates input and output through both kernel and
explicit tactical validation, giving at least four such calls before further
initiative/save/weapon lookups. Spell validation and app options independently
parse the same bytes. `tactical_creatures::creature_definitions` already uses a
OnceLock, and the equipment registry has another one-time catalog parse. These
call counts are not measured elapsed-time attribution.

1. Implement the neutral cache and migrate only immutable bundled runtime reads.
2. Add focused equivalence coverage for complete definitions/serialization,
   shared source access and unchanged arbitrary-input validation. Inspect the
   complete diff independently before compilation.
3. Once the heavy slot is available, measure baseline and candidate under the same
   toolchain, profile, one build job, incremental0 and default Windows stack. Exclude
   compilation and use serialized runs. Record exact SHAs, commands, times and
   limitations; do not compare unrelated concurrent canonical timings as a speedup.
4. Reconcile verified Shield main, complete correctness/measurement/canonical/CI
   acceptance, then protected merge and fetched main tree verification.

## Acceptance and measurement

Source JSON, manifest, genuine fixture bytes, fingerprints, catalog order and
wrapper errors must remain unchanged. Existing malformed-definition tests retain
their input-specific failures after the bundled cache has initialized.

Use the unchanged genuine `legacy_reactions_v1_replay` corpus as the first
application comparison, then the actual
`table_hit_cases::owned_source_shield_reopens_each_decision_preserves_attack_cause_and_rejects_changed_hits`
file-SQLite scenario. Preserve every cold reopen, independent continuation,
accepted receipt, hostile-history and no-write assertion. Repeat serial baseline
and candidate runs sufficiently to distinguish variation; report raw measurements
and avoid a performance claim if the difference is inconclusive. A separate narrow
parse-versus-cached-lookup measurement may establish mechanism cost, but cannot
substitute for production-path timings or correctness.

Required focused checks include tactical definitions, creature profiles, inventory,
spell/attack source tests and the five genuine historical continuations. Run the
existing `rehashed_or_undeclared_tactical_catalog_cannot_authorize_campaign_mutation`
and `pending_rules_export_restores_and_content_changes_fail_before_mutation` controls
after cache warmup; installed content remains revalidated and tampering must write
nothing. Preserve the preflight content/database-wait tests as well.

Before completion require `./scripts/verify-fast`, `./scripts/verify`, desktop checks
as applicable, full independent exact-head review, all six required final-head CI
jobs including packaging, expected-head merge, fetched tree parity and post-main
proof. No earlier Shield or Missile result belongs to this new candidate.

## Risks, verification status and next action

Risks are changed error prefixes, accidental ownership/order changes when replacing
`into_iter`, duplicate caches, initializer recursion and accidentally bypassing
installed-content checks. Static audit must identify every runtime embedded parse;
test parsers and arbitrary-input loaders remain direct.

The implementation routes all seven production embedded parse entry points through
the neutral cache. The creature wrapper keeps its public return/error contract;
fighter and equipment results clone only the strings their previous owned parser
moved. The two new definition tests compare complete values/serialized identities,
shared creature access, distinct valid arbitrary input and its unchanged schema
error after cache initialization. Bundled catalog files, manifests and genuine
historical fixtures are unchanged.

Independent static caller audit confirmed exact wrapper error mappings, all seven
entry points and no initializer cycle. Root reviewed the complete code/test patch
and found no defect. Changed Rust files passed standalone rustfmt with child-module
traversal disabled; `git diff --check` passes. At that initial checkpoint no compiler,
test or benchmark had run; static checks are not acceptance or evidence of a speedup.

### Candidate CI and explicit warm-cache controls

Exact `392e036f88f258aec391abe1beb1584373e0bc82` passes all six CI checks.
Linux run 36308646440 records 726 Rust tests across 54 suites, zero failures or
ignores, all 50 table scenarios in 518.21 seconds and five genuine ReactionsV1
cases in 30.79 seconds. Actual checkout is synthetic merge
`f358c6d66850a2e2f3e320f8236fe76e73035f0e`; its complete tree and the literal
candidate equal `48c784c2320b85e66847f6fa65ff10356e19bc03`.

Windows run 36308646447 passes 728 Rust tests across 54 suites, zero failures or
ignores, all 50 table scenarios in 423.80 seconds, 89 UI tests in 16 files, zero
static errors/warnings and a 139-module build. Both Windows jobs checked out the
literal candidate. Fresh release EXE/NSIS artifact 10928309000 is 231895305 bytes,
SHA256 `de151e5fd9c0753aadd4ffea58b6797c645ace4e7918c8ad57a472b0da4107a9`;
the upload log and artifact API agree. These are CI observations, not a controlled
local speed comparison, canonical local result or final successor-head acceptance.

Root's test review found that the two named installed-content controls initialize
plain kernel campaigns, so a focused run need not warm the tactical cache first.
The successor explicitly initializes the bundled cache before either control.
All original file tampering, restore, failure and no-write assertions remain.
This makes the planned after-warmup regression proof independent of test order.
At that checkpoint, the new statements still required execution on the successor head.

### Current candidate, parent and controlled measurement

Exact `faca063e7d84b2852371c05f93f040d53985bfbc` passes all six CI jobs.
Linux run 36310097267 records 726 Rust tests across 54 suites, zero failures or
ignores, all 50 table cases in 348.86 seconds, and five genuine ReactionsV1 cases
in 21.08 seconds. Actual synthetic checkout
`5c157b87841fe759c68d527a2a5b6090acc60b81` and the literal candidate share tree
`fe71f3ce1c17deab9897d4ba6d0b0707657d3397`.

Windows run 36310097275 records 728 native Rust tests across 54 suites, zero
failures or ignores, all 50 table cases in 769.49 seconds, and five genuine
histories in 42.56 seconds. Both jobs use literal faca. The 89 UI tests in 16 files,
zero static errors/warnings, 139-module build and fresh EXE/NSIS pass. Artifact
10928792549 is 231896387 bytes, SHA256
`e63607c5d86634d66d8b620fef88210c35603379853f315b67085da62de6bdf4`;
actual upload logs and artifact API agree. Both explicit warmed-content controls
pass on Linux and native Windows. Root and independent exact-delta review are clear.

Shield PR45 protected-squash merged as
`a0b12d2d0144a744e3419c1ba69e2d7aac64fd79` after all six bfe checks passed.
Fetched main and bfe share full tree `1954f41fa0f2fd7c2e61540f328ef8546e0bce0b`.
This branch normally reconciles it in `8da8e3c6f250a6f166b652530b36af2f47a164ff`;
the merge's entire tree equals faca, with no file change. Independent reconciliation
review is clear. Literal Shield post-main runs 36311244326/36311244361 are still
active at this checkpoint; do not infer their final result from source checks.

The controlled local comparison uses separately rebuilt and preserved test
binaries from baseline `bfe4aac67ed80a5aec5165cfdd198acdbbfec9c5` and candidate
`faca063e7d84b2852371c05f93f040d53985bfbc`. Both use GNU Rust 1.98.1, the same
unoptimized test profile with debug assertions and overflow checks, one Cargo job,
incremental compilation disabled, default Windows stack, and identical
`--test-threads=1 --nocapture` arguments. Compilation is excluded. Content,
dependencies, measured tests/support and genuine fixture bytes are unchanged.
The compiler's source paths, fresh build records, copied binary hashes and runtime
content paths were independently audited; no working-directory confound was found.

Five-history `legacy_reactions_v1_replay` ABBA wall times, in execution order:

| Run | Source | Seconds | Result |
| --- | --- | ---: | --- |
| 1 | Baseline bfe | 304.34 | All five passed |
| 2 | Cached faca | 88.62 | All five passed |
| 3 | Cached faca | 89.75 | All five passed |
| 4 | Baseline bfe | 304.52 | All five passed |

Each process starts its own immutable cache. Mean elapsed time falls from 304.43
to 89.18 seconds, a 70.70% reduction (3.41 times the throughput for this fixed test
selection). This measures these local debug/SQLite/recovery workloads, including
their identical cleanup and logging overhead. It is not packaged-runtime speed,
general gameplay latency, or Gate13 hardware-performance acceptance.

External evidence is under `tooling/cache-measurement-binaries-bfe-faca/` and
`tooling/cache-measurement-reactions-bfe-faca/`: exact source metadata, build logs,
copied binary SHA256 hashes, raw test logs, arguments, UTC starts and wall times.
The test runner and build harness are `tooling/measure-catalog-cache.ps1` and
`tooling/build-cache-measurement-binaries.ps1`. The owned-source Shield ABBA run
uses the same binaries/constraints with its exact test name and is in progress;
no result is claimed for it here.

Next action: finish the serial owned-Shield comparison, then run canonical
`./scripts/verify` on the reconciled source. Its full suite must include the warmed
installed-content controls, unchanged catalog/profile/inventory/spell/attack checks
and genuine histories. Publish this reviewed evidence checkpoint for final-head CI
while those local checks run. Complete all six exact-head checks, independent final
delta review, protected merge, fetched full-tree parity and literal post-main proof.
Record the final source-qualified results in PR47 and archive this bounded plan in
the next receiving branch after proof is complete. No complete acceptance is claimed.
