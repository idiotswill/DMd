# CI runbook

## Canonical verification

During iteration:

```bash
./scripts/verify-fast
```

Before declaring implementation complete:

```bash
./scripts/verify
```

CI remains authoritative for the repository/runner environment; local success does not replace a green run on the final PR head.

The architecture guard requires Python 3.11+ for standard-library TOML parsing. CI
installs that minimum explicitly. Local verification probes `python3`, then `python`;
set `PYTHON` to an interpreter executable path when neither name selects the intended
installation (for example, with Windows Store aliases). No Python packages or network
access are needed by the guard, and Python is not a gameplay runtime dependency.
`./scripts/check-boundaries` runs its regression fixtures before checking the repository.
It resolves normal, development, build and target-specific Cargo dependencies, including
renamed packages and workspace inheritance, and retains production-source import checks.

## Superseded runs

The CI workflow uses a concurrency group per workflow + PR/ref with `cancel-in-progress: true`. A newer push should cancel an older run for the same line of work.

A cancelled obsolete run is not a failure. Only the newest head matters.

## Failure procedure

1. Confirm the workflow belongs to the current head SHA.
2. Identify the exact failed job and step.
3. Read compiler/test/lint output from that job.
4. Classify the failure: formatting, compilation/type error, lint, test, invariant guard, dependency/tooling, or infrastructure.
5. Make the smallest justified change.
6. Never disable a check simply because it caught a real defect.
7. Rerun/observe CI on the new head.

## Stale status rule

Never say “CI is green” based on an older commit. Record the verified head in the active execution plan.

## Current Rust checks

The workflow currently covers:

- formatting;
- locked workspace compilation;
- Clippy with warnings denied;
- tests;
- declared Rust 1.88 MSRV compilation;
- campaign-genericity guard;
- architecture guard and its alias/inheritance/failure regression tests.

Add checks when a recurring failure class can be prevented mechanically, but keep the fast loop reasonably fast.
