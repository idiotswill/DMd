# GitHub repository settings runbook

The repository currently has no rulesets. The connected GitHub tool can read rulesets but cannot administer them, so these settings require a manual GitHub UI/API action by a repository administrator.

## Recommended `main` ruleset

Target branch: `main`

Recommended rules:

- require changes through a pull request before merge;
- require status checks to pass before merge;
- require conversation resolution before merge;
- block force pushes;
- block branch deletion;
- allow repository administrators to bypass only for genuine recovery/emergency use.

For a solo repository, do **not** require a separate approving reviewer merely for ceremony. Human approval remains a project process requirement for architecture/save-format/high-impact checkpoints even when GitHub does not require another account's review.

## Required status checks

Once the Lab 0 workflow is merged and check names are stable, require the current CI checks corresponding to:

- Rust formatting/check/Clippy/tests (`rust` job);
- declared Rust minimum-version check (`msrv` job);
- campaign-genericity guard (`genericity-guard` job).

If job names change, update the ruleset rather than keeping obsolete required checks.

## Merge policy

Recommended defaults:

- use PRs for all agent-authored changes to `main`;
- prefer squash merge for small focused implementation PRs when intermediate connector commits are not valuable history;
- retain normal merge commits when preserving a meaningful stacked/architectural history is useful;
- do not enable automatic merging for architecture checkpoints or irreversible/high-impact changes without explicit human acceptance.

## Verification after configuration

After creating the ruleset:

1. confirm a direct push to `main` is rejected or restricted as intended;
2. confirm a PR with failing required checks cannot merge;
3. confirm unresolved review conversations block merge if enabled;
4. confirm force-push/deletion protection is active;
5. record any deliberate deviation from this runbook in the relevant execution plan or ADR.
