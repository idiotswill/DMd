# Lab 0 — AI development lab

Status: in progress
Branch: `lab0/dev-lab`
PR: #2
Base: `gate0/foundation` @ `74a909ba463d748fa90f56e51e647fc1881525e8`
Verified head: pending

## Objective

Make DMd safe and efficient to develop through disposable AI/chat execution contexts by moving navigation, handoff, verification, recovery, and workflow rules into the repository.

## Scope

- root `AGENTS.md` navigation/invariant map;
- checked-in execution-plan protocol;
- development, CI, recovery, and GitHub-settings runbooks;
- explicit technical-debt ledger;
- canonical fast/full verification scripts;
- PR template for scoped, verifiable changes;
- GitHub Actions concurrency cancellation for superseded runs.

## Non-goals

- gameplay implementation;
- Gate 1 persistence/replay work;
- changing Gate 0 domain semantics;
- adding AI/STT/TTS/UI implementation;
- merging Gate 0.

## Relevant durable context

- `docs/checkpoints/gate-0.md`
- `docs/checkpoints/gate-0-review.md`
- `docs/architecture/002-state-and-events.md`
- `docs/architecture/009-typed-intent-command-boundary.md`

## Acceptance criteria

- [x] A fresh agent can discover current work from repository files without chat history.
- [x] Development rules prohibit concurrent writers on one branch and require head refresh after unexpected movement.
- [x] Nontrivial tasks have a standard resumable execution-plan format.
- [x] Local fast/full verification commands are canonical and documented.
- [ ] CI cancels superseded runs for the same PR/ref.
- [x] PRs have a standard objective/scope/validation/risk checklist.
- [x] Recovery from interruption, stale SHA, tool failure, or failed CI has a documented procedure.
- [ ] The full branch passes equivalent `./scripts/verify` checks and CI on the final head.

## Planned slices

1. Add repository navigation and execution-plan protocol. — complete
2. Add runbooks/debt ledger/PR template. — complete
3. Add verification scripts and CI concurrency. — complete
4. Open a stacked draft PR against `gate0/foundation`. — complete (#2)
5. Verify superseded-run cancellation and the complete final head. — in progress

## Decision log

- 2026-09-23 — Dev Lab is a separate stacked PR/branch so process tooling does not obscure the Gate 0 architecture review.
- 2026-09-23 — Repository files are authoritative handoff memory; chat/project instructions are behavioral bootstrap only.
- 2026-09-23 — Coherent multi-file changes should use one Git tree/commit to reduce CI churn and stale-head conflicts.
- 2026-09-23 — Repository rulesets are currently empty. The GitHub connector can read but not administer rulesets, so `main` protection is a documented manual repository-setting follow-up rather than an in-chat mutation.
- 2026-09-23 — CI run 121 completed successfully before the first superseding commit could cancel it, so that cancellation test was inconclusive.
- 2026-09-23 — CI run 122 is queued on head `47911198bf8182fd939a4cc1540b044a3b0cef76`; this plan-only commit intentionally supersedes it to test `cancel-in-progress` deterministically.

## Validation

- First implementation head `75f1a82d29ff1f887a2ef56e55f74047e9b27806` — CI run 121 passed.
- Intermediate metadata head `47911198bf8182fd939a4cc1540b044a3b0cef76` — CI run 122 queued and intentionally superseded.
- `./scripts/verify-fast` equivalent — pending on final head.
- `./scripts/verify` equivalent — pending on final head.
- CI — pending on final head.

## Risks / blockers

- GitHub `main` ruleset/branch protection cannot be created through the available connector permissions. Apply the settings in `docs/runbooks/github-settings.md` manually when convenient.
- Shell verification scripts assume a bash-compatible developer shell; CI provides one. Native Windows developer wrappers can be added when desktop development begins if useful.

## Next action

Confirm run 122 is cancelled by this superseding commit, then verify all CI jobs on the new head and update this plan to ready-for-review status.
