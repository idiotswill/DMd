# Gate 4 — Explicit application hit-response test drivers

Status: authored integration support; compilation and execution pending.
Writer: source_control_recovery, taking over the preserved environment_audit draft.
Branch: codex/gate4-shield-hit-app-drivers, based on e5f8fce.

## Objective and boundaries

Adapt fresh application scenarios to ShieldHitV1 while retaining their physical
dice, current source rules, actual controller authority and persistence assertions.
This supports the active Shield hit plan, Gate 4 and the product definition's
player-agency and durable-recovery requirements. Genuine historical fixture bytes
and old executor compatibility tests remain unchanged.

The explicit helper accepts exactly one current hit window: the current turn's
controller orders it, then the actual target declines. Each is its own command
using that controller's current opaque presentation. It does not drain arbitrary
work or provide an automatic production response policy. Raw-roll, mirrored
command and cold-retry helpers retain their single-command behavior.

## Acceptance and integration

- Fresh fixture Begin operations use ShieldHitV1.
- Successful attacks explicitly acknowledge their new hit windows; misses and
  unrelated physical rolls gain no hidden decisions.
- The helper asserts two accepted decisions, unchanged raw history and Reaction
  expenditure, and original-hit causation for any subsequent damage request.
- Independently continued OA histories use the same explicit canonical decisions
  and cold retries, without sharing newly generated opaque handles.
- Assigned source controllers cannot fall back to Host. The parent branch adds
  SourceCreature support after verified PR43 integration and registers this helper
  in table_loop.rs. The parent owns the new Shield SQLite case and module wiring.
- Parent integration must compile, run focused/default-stack application cases,
  then pass the canonical suite and exact-head CI. No tests have run on this draft.

## Verification and next action

Recovered the preserved edits after resource exhaustion and statically inspected
all changed fixtures. Targeted rustfmt and git diff --check passed; no compile or
tests ran. Heavy checks remain serialized under the parent agent's control.
Commit the coherent helper/fixture changes for parent cherry-pick, then continue
the separate source-control branch. Gate 4 remains active; no gate-end claim.
