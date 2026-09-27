# Gate 4 — Shared immutable tactical catalog

Status: authorized bounded implementation; no compilation, tests, benchmark or
performance result yet. Writer: source_registry_review. Branch:
`codex/gate4-immutable-catalog-cache`, based on reviewed Shield candidate
`bfe4aac67ed80a5aec5165cfdd198acdbbfec9c5`. Fetched main before branching was
`f441adedcf490504b6f1e3db1a964c023c511e47`. The preserved prior branch
`codex/gate4-shield-missile-app` remains at8537775. No PR is open for this work.

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

The sole local heavy slot belongs to the Magic Missile verification agent. This
writer may edit, format and commit; no build, tests or benchmarks may start until
the coordinator releases that slot. Do not push this candidate before review.
Reconcile the eventual verified PR45 main normally before final acceptance.

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

No build/test/benchmark has run. No acceptance or speedup is claimed. Next action:
implement the reviewed seam, format only changed Rust files, inspect and commit a
coherent candidate for independent review. Keep the heavy slot free and do not push.
