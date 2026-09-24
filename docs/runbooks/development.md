# Development runbook

## Purpose

Keep long-running AI-assisted DMd development resumable, reviewable, and resistant to chat/context loss.

## Work types

Use distinct contexts for distinct roles when practical:

- **control/architecture** — roadmap, gates, ADR decisions, acceptance;
- **implementation** — one branch/PR-sized objective;
- **review** — fresh-context read-only review of a completed diff;
- **research** — external investigation that ends in a durable ADR/plan update when it affects the project.

Do not use one endless implementation chat as project memory.

## Starting work

1. Fetch the target PR/branch and current head.
2. Read `AGENTS.md`.
3. Read the active execution plan.
4. Retrieve only relevant ADRs/checkpoints/code.
5. Confirm the branch has one writer.
6. If the task is nontrivial and has no plan, create one.

## Branch model

- One writable execution context per branch.
- One PR-sized objective per branch.
- Prefer branches such as `gate1/journal`, `gate1/replay`, `fix/...`, `lab/...`.
- Never use `main` for direct agent development.
- Stacked PRs are acceptable when later work truly depends on an unmerged foundation; make the base explicit.

## Change loop

1. Establish the smallest coherent slice.
2. Read before writing; do not modify remembered file contents.
3. Prefer typed/mechanical constraints over prose reminders when a rule can be enforced.
4. Batch related multi-file changes into one Git tree/commit when the connector supports it.
5. Run/verify the fastest relevant checks.
6. Inspect failures directly; do not guess from job summaries.
7. Update the execution plan when decisions/scope change.
8. Before completion, inspect the complete diff and run full verification.

## Commit policy

A commit should represent one coherent reason for change. Avoid one commit per file merely because the connector writes files individually.

Good examples:

- `lab: add resumable agent development workflow`
- `persistence: commit state and journal atomically`
- `rules: add reaction timing windows`

Avoid noisy sequences such as `add file`, `fix format`, `add another file`, `fix same file` when they can be one prepared commit.

## Review loop

For significant changes, a fresh reviewer should first report findings without editing. Review the whole diff against:

- task acceptance criteria;
- relevant ADRs/checkpoints;
- tests and failure behavior;
- state/provenance/recovery implications;
- genericity/campaign isolation;
- documentation claims;
- unnecessary scope.

Then fix findings on the implementation branch and rerun verification.

## Completion

### Rules content integrity

Content under `content/` has pinned LF line endings so Windows and Linux checkouts share
the same manifest bytes. After a reviewed edit to the pinned SRD definitions, notice or
source metadata, run `python scripts/update-rules-manifest.py`, inspect the content and
manifest diff, then run normal verification. Python is developer tooling only; gameplay
loads the checked-in manifest offline. The distribution test verifies all declared bytes
through the production content catalog. A changed source/version needs its own provenance
and compatibility decision; regenerating checksums is not approval of changed rules.

### Final checks

Do not call work complete until:

- acceptance criteria are checked against actual behavior;
- the complete diff has been inspected;
- `./scripts/verify` passes locally or equivalent commands are directly verified;
- CI is green on the exact final head;
- the PR summary is current;
- the execution plan contains final validation and no hidden blockers.

The owner delegates in-scope active-gate PR merges, including architecture/save-format/content decisions, after exact-head review, required green checks, documented rationale, and expected-head protection. Follow `../checkpoints/gate-execution-protocol.md`: pause for the owner at gate end; surface scope reductions, waived acceptance, or destructive changes outside gate intent immediately.
