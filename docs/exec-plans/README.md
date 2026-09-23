# Execution plans

Execution plans are the durable handoff format for nontrivial DMd work. They exist so a fresh chat can resume safely without reconstructing prior conversation history.

## Lifecycle

1. Create a plan in `active/` before substantive implementation.
2. Keep it current as scope, decisions, validation, or blockers change.
3. At completion, record the verified head/PR and final validation.
4. Move the plan to `completed/` in the same closing change when practical.

Small typo/docs-only edits do not require a plan.

## Required template

```markdown
# <task name>

Status: planned | in progress | blocked | complete
Branch: <branch>
PR: <number or pending>
Base: <branch/SHA>
Verified head: <SHA or pending>

## Objective
<one concrete outcome>

## Scope
- ...

## Non-goals
- ...

## Relevant durable context
- ADR ...
- checkpoint ...

## Acceptance criteria
- [ ] ...

## Planned slices
1. ...

## Decision log
- YYYY-MM-DD — decision and why.

## Validation
- `./scripts/verify-fast` — pending/pass/fail
- `./scripts/verify` — pending/pass/fail
- CI — pending/pass/fail

## Risks / blockers
- ...

## Next action
<exact first action a fresh chat should take>
```

## Rules

- Plans describe execution state, not permanent architecture; permanent decisions belong in ADRs.
- Do not duplicate entire ADRs or checkpoint documents into a plan; link them.
- A plan must be honest about unverified work.
- If implementation diverges materially from the plan, update the plan before continuing.
- Do not delete completed plans merely to reduce clutter; they are historical implementation context.
