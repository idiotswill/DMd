# DMd agent map

DMd is a local-first tabletop RPG runtime and autonomous-DM project. The repository is the authoritative engineering memory; chat history is not.

## Start here

Before repository work:

1. Fetch the current branch/PR head. Do not trust a remembered SHA.
2. Read this file.
3. Find the active execution plan under `docs/exec-plans/active/`.
4. Read only the ADRs, checkpoint docs, runbooks, and code relevant to the task.
5. Confirm the task scope, non-goals, acceptance criteria, and verification commands before writing.

If no execution plan exists for a nontrivial task, create one before implementation.

## Source-of-truth order

When sources disagree, prefer:

1. current code + executable tests/invariants;
2. accepted ADRs in `docs/architecture/`;
3. current checkpoint/acceptance documents in `docs/checkpoints/`;
4. the active execution plan;
5. PR/issue descriptions;
6. chat summaries or prior conversation context.

Do not silently reconcile contradictions. Surface them and update the appropriate durable source.

## Repository map

- `crates/` — production Rust crates and authoritative domain/application boundaries.
- `docs/architecture/` — architectural decisions and invariants.
- `docs/checkpoints/` — gate definitions, acceptance criteria, and gate reviews.
- `docs/exec-plans/active/` — resumable plans for current bounded work.
- `docs/exec-plans/completed/` — completed plans retained for historical decisions.
- `docs/runbooks/` — development, CI, and recovery procedures.
- `tests/` — cross-cutting/regression fixtures.
- `scripts/` — canonical local/CI verification helpers.
- `.github/` — CI and pull-request workflow configuration.

## Hard engineering invariants

- The engine must remain independent of any current campaign, party, setting, or human-readable entity name.
- Language/AI output is never authoritative game state.
- Provider-specific JSON/grammar output must become application-defined typed proposals before core validation.
- Trusted command issuer identity is separate from any in-world actor inferred from speech.
- GitHub, Drive, cloud AI, and network availability must not be required to play a local campaign.
- Durable state mutations require validation, provenance, and eventually atomic state + event-journal persistence.
- Do not hide known debt or failed assumptions in placeholders; record them explicitly.

## Branch and chat discipline

- One writable execution chat/agent per branch.
- Never allow two agents to write concurrently to the same branch.
- A branch should represent one PR-sized objective.
- If the branch head changes unexpectedly, stop writing, fetch the new head, inspect the intervening changes, and reconcile.
- Prefer one coherent multi-file commit built from a Git tree over one commit per file.
- Do not force-push over unknown work.

## Execution plans

A nontrivial task plan must record:

- objective and branch/PR;
- scope and non-goals;
- relevant ADRs/checkpoints;
- acceptance criteria;
- planned slices;
- decisions made during implementation;
- validation status;
- blockers/risks;
- exact next action for a fresh chat.

Update the plan when reality changes, not only at the end.

## Verification

Use `./scripts/verify-fast` during iteration and `./scripts/verify` before declaring implementation complete.

Never claim code, CI, repository state, or behavior is correct unless directly verified. A green check on an older SHA is not evidence for the current head.

When CI fails:

1. identify the exact failing job/step;
2. read the actual compiler/test/lint output;
3. make the smallest justified fix;
4. rerun on the new head;
5. do not weaken tests/lints merely to turn CI green.

## Review protocol

For substantial changes, separate implementation and review roles when practical. A fresh review pass should inspect the full diff before fixes and look specifically for:

- correctness defects and missing tests;
- architectural boundary violations;
- state/provenance/replay risks;
- campaign-specific coupling;
- stale documentation or unsupported claims;
- scope creep and accidental placeholders.

## Human approval boundaries

Do not merge or mark accepted without explicit human approval when work changes architecture checkpoints, save-format compatibility, irreversible migrations, licensing/content boundaries, or other high-impact foundations.

## Context economy

Do not load the whole repository or every ADR by default. Use this file as the map and progressively retrieve only the context required by the active task.

The repository must be resumable by a fresh chat without relying on private scratchpad state or long conversation history.
