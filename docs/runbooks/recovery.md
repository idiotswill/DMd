# Recovery runbook

This runbook is for interrupted chats, context loss, stale GitHub writes, tool failures, unexpected branch movement, and failed CI.

## Fresh-chat recovery

A fresh execution chat should be able to resume without old conversation history:

1. read `AGENTS.md`;
2. identify the intended branch/PR;
3. fetch its current head;
4. read the active execution plan;
5. read only the linked ADRs/checkpoints and touched code;
6. inspect current CI/review state;
7. execute the plan's `Next action`.

If the plan is stale, update it before implementation.

## Unexpected branch movement / stale SHA

If a write fails because the blob/head SHA changed, or the branch head differs from the expected head:

1. stop all writes;
2. fetch the new branch/PR head;
3. inspect intervening commits/diff;
4. determine whether another writer exists;
5. reconcile changes explicitly;
6. refresh file/blob SHAs;
7. continue only when one-writer ownership is restored.

Never overwrite unknown changes or force-push merely to restore an expected SHA.

## Tool/API failure

- Distinguish “request failed” from “repository changed.”
- Verify whether the mutation actually landed before retrying.
- Retrying an uncertain write without checking can duplicate changes.
- Record persistent connector limitations in the execution plan rather than repeatedly rediscovering them.

## CI failure

Follow `docs/runbooks/ci.md`. Pull actual logs; do not infer failure causes from a red summary badge.

## Conversation/context exhaustion

Before abandoning a long execution context:

1. update the execution plan with completed work and decisions;
2. record the exact branch, PR, and latest known head;
3. record verification status honestly;
4. write one concrete `Next action`;
5. ensure no critical decision exists only in chat prose.

The next chat should trust the repository over any generated conversation summary.

## Rollback

Prefer additive fixes/reverts through Git history over destructive history rewriting. High-impact data/save migrations must have an explicit recovery/backup strategy before they are introduced.
