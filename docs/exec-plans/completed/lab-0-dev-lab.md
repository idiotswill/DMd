# Lab 0 — AI development lab

Status: complete
Branch: `lab0/dev-lab`
PR: #2
Base: `gate0/foundation` @ `74a909ba463d748fa90f56e51e647fc1881525e8`
Verified implementation head: `fd8e9df863e391f968d080fae007219423a1f3f5` (CI run 123: success)

## Objective

Make DMd safe and efficient to develop through disposable AI/chat execution contexts by moving navigation, handoff, verification, recovery, and workflow rules into the repository.

## Scope delivered

- root `AGENTS.md` navigation/invariant map;
- checked-in execution-plan protocol;
- development, CI, recovery, and GitHub-settings runbooks;
- explicit technical-debt ledger;
- canonical fast/full verification scripts;
- PR template for scoped, verifiable changes;
- GitHub Actions concurrency cancellation for superseded runs.

## Non-goals preserved

- no gameplay implementation;
- no Gate 1 persistence/replay work;
- no Gate 0 domain semantic changes;
- no AI/STT/TTS/UI implementation;
- no Gate 0 merge.

## Acceptance result

- [x] A fresh agent can discover current work from repository files without chat history.
- [x] Development rules prohibit concurrent writers on one branch and require head refresh after unexpected movement.
- [x] Nontrivial tasks have a standard resumable execution-plan format.
- [x] Local fast/full verification commands are canonical and documented.
- [x] CI cancels superseded runs for the same PR/ref (run 122 was cancelled by run 123's newer head).
- [x] PRs have a standard objective/scope/validation/risk checklist.
- [x] Recovery from interruption, stale SHA, tool failure, or failed CI has a documented procedure.
- [x] The implementation head passed the equivalent `./scripts/verify` checks and CI.

## Decision log

- 2026-09-23 — Dev Lab is a separate stacked PR/branch so process tooling does not obscure the Gate 0 architecture review.
- 2026-09-23 — Repository files are authoritative handoff memory; chat/project instructions are behavioral bootstrap only.
- 2026-09-23 — Coherent multi-file changes should use one Git tree/commit to reduce CI churn and stale-head conflicts.
- 2026-09-23 — Repository rulesets are currently empty. The GitHub connector can read but not administer rulesets, so `main` protection is a documented manual repository-setting follow-up rather than an in-chat mutation.
- 2026-09-23 — CI concurrency was tested in practice: run 122 on intermediate head `47911198bf8182fd939a4cc1540b044a3b0cef76` was cancelled when head `fd8e9df863e391f968d080fae007219423a1f3f5` triggered run 123.

## Validation

- Initial implementation head `75f1a82d29ff1f887a2ef56e55f74047e9b27806` — CI run 121 passed.
- Superseded intermediate head `47911198bf8182fd939a4cc1540b044a3b0cef76` — CI run 122 cancelled as intended.
- Verified implementation head `fd8e9df863e391f968d080fae007219423a1f3f5` — CI run 123 passed:
  - fast verification passed;
  - Clippy passed;
  - tests passed;
  - Rust 1.88 MSRV check passed;
  - genericity guard passed.
- Complete PR diff inspected against Lab 0 scope; no gameplay/domain semantic changes found.

## Remaining manual repository setting

The repository has no GitHub ruleset, and the available connector cannot administer one. Apply `docs/runbooks/github-settings.md` manually when convenient to protect `main` with PR/status-check/conversation-resolution/force-push/deletion rules.

## Closeout note

The commit that moves this plan into `completed/` is documentation-only. CI must still be green on the final PR head before merge, per `AGENTS.md` and the PR template.
