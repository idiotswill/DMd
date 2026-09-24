# DMd agent map

DMd is a local-first tabletop RPG runtime and autonomous-DM project. The repository is the authoritative engineering memory; chat history is not.

## Start here

Before repository work:

1. Fetch the current branch/PR head. Do not trust a remembered SHA.
2. Read this file.
3. Find the active execution plan under `docs/exec-plans/active/`.
   Use `docs/checkpoints/roadmap.md` and `docs/checkpoints/gate-execution-protocol.md` to locate the active gate and its operating boundaries.
4. Read `docs/product-definition.md` when the task changes product behavior, gate acceptance, user-facing completeness, world simulation, or release/endurance expectations.
5. Read only the ADRs, checkpoint docs, runbooks, and code relevant to the task.
6. Confirm the task scope, non-goals, acceptance criteria, and verification commands before writing.

If no execution plan exists for a nontrivial task, create one before implementation.

## Source-of-truth boundaries

Different durable sources answer different questions:

- `docs/product-definition.md` is authoritative for what the finished product must become.
- Current code + executable tests/invariants are authoritative for what the current build actually does.
- Accepted ADRs in `docs/architecture/` define architectural decisions/invariants.
- Current checkpoint documents in `docs/checkpoints/` define scoped gate acceptance.
- The active execution plan defines the current bounded task and its verified progress.
- PR/issue descriptions summarize work but do not override the sources above.
- Chat summaries and remembered conversation context are navigation aids only.

An incomplete implementation does not redefine the product finish line. Do not silently reconcile contradictions; surface them and update the appropriate durable source.

## Repository map

- `docs/product-definition.md` — end-state product contract and endurance acceptance.
- `crates/` — production Rust crates and authoritative domain/application boundaries.
- `docs/architecture/` — architectural decisions and invariants.
- `docs/checkpoints/` — gate definitions, acceptance criteria, and gate reviews.
- `docs/exec-plans/active/` — resumable plans for current bounded work.
- `docs/exec-plans/completed/` — completed plans retained for historical decisions.
- `docs/runbooks/` — development, CI, and recovery procedures.
- `tests/` — cross-cutting/regression fixtures.
- `scripts/` — canonical local/CI verification helpers.
- `.github/` — CI and pull-request workflow configuration.

## Product-completeness invariants

- DMd must become a fully functioning, production-intended tabletop RPG application, not a prototype, proof of concept, technology demo, SDK/framework, scripted campaign demo, or collection of disconnected subsystems.
- Intermediate gates may intentionally expose incomplete functionality, but accepted work must advance the production path rather than substitute a disposable parallel implementation.
- Architecture, schemas, interfaces, mocks, unit tests, or scripted demos alone do not constitute end-user feature completion.
- A feature must ultimately participate correctly in the real application path under realistic persistence, recovery, campaign, and player interaction conditions appropriate to its gate.
- Research prototypes/benchmarks are permitted when isolated and clearly non-production. They must not be used to claim a production gate is complete.
- The finished game must remain capable of meaningful new play after authored starting material is exhausted.

## Living-world invariants

- The campaign world must continue to develop through simulated time and autonomous actors/processes where circumstances justify it; it must not freeze awaiting player triggers.
- New rumors, leads, requests, conflicts, discoveries, opportunities, threats, and quest-like situations must be able to emerge causally from changing world state rather than only from scripted content.
- Players may ignore, redirect, exploit, or miss situations; relevant world processes continue appropriately.
- World change must be grounded in state, time, knowledge, capability, resources, geography, goals, and prior events rather than arbitrary drama generation.
- The Director manages presentation/pacing/spotlight. It may surface supported developments but may not invent unsupported authoritative world truth, rewrite dice, or manufacture convenient causes merely to make a session exciting.
- Important world changes need enough provenance to answer why they happened; “the AI decided it was interesting” is not an acceptable authoritative cause.

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
- relevant product-definition clauses, ADRs, and checkpoints;
- acceptance criteria;
- planned slices;
- decisions made during implementation;
- validation status;
- blockers/risks;
- exact next action for a fresh chat.

Update the plan when reality changes, not only at the end.

Every future gate/checkpoint must identify which product-definition requirements it advances and which remain deferred. Passing a gate does not imply the overall game is finished.

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
- product-definition/gate-traceability violations;
- prototype/demo shortcuts presented as production completion;
- stale documentation or unsupported claims;
- scope creep and accidental placeholders.

## Autonomous gate work and owner boundaries

The owner authorizes Codex to create branches, implement, open PRs, review/fix its own work, and merge in-scope PRs within the active approved gate. Review the complete exact head, verify required checks on that head, satisfy the slice acceptance criteria, record debt, and merge with expected-head protection. Routine PR or merge approval is not required.

Architecture/checkpoint/save-format/content decisions within the approved gate may be merged after rigorous verification and documented rationale. A gate may span several coherent PRs/branches; retain exactly one writer per branch.

At the end of each gate, perform the integrated production-path review, verify final merged `main`, update checkpoint/plan evidence, and pause with the summary required by `docs/checkpoints/gate-execution-protocol.md`. Do not start the next gate until the owner says to continue. The owner-approved roadmap bootstrap is the explicit exception: merge it and proceed directly to Gate 2 on a fresh branch.

Surface true blockers immediately: unresolved legal risk without a safe implementation path, proposed product reduction or waived acceptance, destructive/data-loss decisions outside gate intent, unsafe security/privacy issues, irreconcilable repository movement, or required human/hardware evidence unavailable in the execution environment.

No plan or gate may silently weaken `docs/product-definition.md` to make acceptance easier. Deliberate product-scope changes require explicit human approval.

## Context economy

Do not load the whole repository or every ADR by default. Use this file as the map and progressively retrieve only the context required by the active task.

The repository must be resumable by a fresh chat without relying on private scratchpad state or long conversation history.
