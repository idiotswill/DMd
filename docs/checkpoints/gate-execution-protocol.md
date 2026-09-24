# Gate execution protocol

Status: **Owner-authorized operating model**

This protocol defines how Codex executes the DMd roadmap after the roadmap bootstrap is merged.

## Autonomy within a gate

Codex is authorized to create branches, implement, test, open/update PRs, review/fix, and merge PRs required to complete the active gate.

Routine merge approval is delegated to Codex provided:

- the PR is in scope for the active gate;
- the exact head was reviewed;
- required checks pass on that exact head;
- relevant acceptance criteria are satisfied;
- architecture/product invariants are not weakened;
- known debt is recorded rather than hidden.

## One writer per branch

Only one writable agent/chat may own a branch at a time.

Parallel work is allowed through separate branches/PRs with deliberate integration.

## Gate completion

Before declaring a gate complete, Codex must perform an integrated review of all work merged for the gate and verify the checkpoint through the production path.

The gate checkpoint and execution plan must contain exact evidence, not remembered claims.

## End-of-gate pause

The standard human interaction point is the end of a gate.

After the gate's final integration review, Codex must stop and report:

- final status;
- final `main` SHA;
- merged PRs;
- capabilities delivered;
- architecture decisions;
- verification evidence;
- unresolved debt/blockers;
- external human/hardware acceptance still required;
- proposed next-gate objective.

Codex must not begin the next gate until the owner says to continue.

## Scope changes are different from implementation autonomy

Autonomous merge authority does not authorize Codex to make acceptance easier by silently narrowing `docs/product-definition.md`.

If a gate exposes a product requirement that appears impossible, unsafe, legally unavailable, or fundamentally incompatible with the accepted architecture, surface that conflict rather than redefining success.

## Early-stop exceptions

Codex should not interrupt for ordinary technical choices.

Stop before the gate-end checkpoint only when a real owner decision is required, such as:

- proposed product-scope reduction;
- unresolved legal/commercial-distribution ambiguity with no safe implementation path;
- irreversible destructive change outside the accepted checkpoint intent;
- security/privacy issue requiring a policy choice;
- required physical playtest/hardware evidence unavailable to Codex;
- repository movement/conflict that cannot be safely reconciled.

