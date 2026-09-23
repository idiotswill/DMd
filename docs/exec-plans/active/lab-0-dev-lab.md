# Lab 0 — AI development lab

Status: in progress
Branch: `lab0/dev-lab`
PR: pending
Base: `gate0/foundation` @ `74a909ba463d748fa90f56e51e647fc1881525e8`
Verified head: pending

## Objective

Make DMd safe and efficient to develop through disposable AI/chat execution contexts by moving navigation, handoff, verification, recovery, and workflow rules into the repository.

## Scope

- root `AGENTS.md` navigation/invariant map;
- checked-in execution-plan protocol;
- development, CI, and recovery runbooks;
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

- [ ] A fresh agent can discover current work from repository files without chat history.
- [ ] Development rules prohibit concurrent writers on one branch and require head refresh after unexpected movement.
- [ ] Nontrivial tasks have a standard resumable execution-plan format.
- [ ] Local fast/full verification commands are canonical and documented.
- [ ] CI cancels superseded runs for the same PR/ref.
- [ ] PRs have a standard objective/scope/validation/risk checklist.
- [ ] Recovery from interruption, stale SHA, tool failure, or failed CI has a documented procedure.
- [ ] The full branch passes `./scripts/verify` and CI.

## Planned slices

1. Add repository navigation and execution-plan protocol.
2. Add runbooks/debt ledger/PR template.
3. Add verification scripts and CI concurrency.
4. Open a stacked draft PR against `gate0/foundation`.
5. Verify the complete head and close this plan.

## Decision log

- 2026-09-23 — Dev Lab is a separate stacked PR/branch so process tooling does not obscure the Gate 0 architecture review.
- 2026-09-23 — Repository files are authoritative handoff memory; chat/project instructions are behavioral bootstrap only.
- 2026-09-23 — Coherent multi-file changes should use one Git tree/commit to reduce CI churn and stale-head conflicts.

## Validation

- `./scripts/verify-fast` — pending
- `./scripts/verify` — pending
- CI — pending

## Risks / blockers

- GitHub repository rules/branch protection may require account-level settings unavailable to the connector; document any manual follow-up if so.
- Shell verification scripts assume a bash-compatible developer shell; CI provides one. Native Windows developer wrappers can be added when desktop development begins if useful.

## Next action

Create the batched Lab 0 commit, open the stacked draft PR, and verify its CI head.
