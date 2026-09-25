# Gate 4 — Explicit Shield hit rules regression support

Status: active; authoring only, no compiler or test execution claimed.
Writer: rules_architecture, branch codex/gate4-shield-hit-rules-tests.
Base: e5f8fce0962c64f6c8fd38dc1ecdbfb4bc90e25c.

## Scope and source

Support the primary Shield hit implementation with rules integration tests only.
The root writer owns all production and application code. Follow AGENTS,
ADR028, the active gate4-shield-hit-runtime plan and pinned SRD5.2.1 Shield
(pp161–162), reaction timing and simultaneous effects (p187). This slice does not
claim Counterspell, Magic Missile response windows or Ready release completion.

## Implementation

1. Keep fixture run/run_meta/roll as single-command boundaries with their existing
   serialization, deterministic replay and full-state checks.
2. Update genuinely fresh Begin inputs to ShieldHitV1. Preserve historical
   fixtures and the old unit UpgradeExecution1-to2 contract.
3. Add explicitly named test-only roll_then_decline_hit_responses and exact-window
   response drivers. Each hit receives a real target-owner Continue and a separate
   current-turn-controller ordering command. Never automatically answer damage,
   knockout, Savage Attacker, opportunity, concentration or other pending work.
4. Add source Mage tests using build_creature and real materialized source gear:
   hit-to-miss, natural20 damage, expiry, shared source uses, components and ownership,
   opportunity parent preservation, original damage cause and malformed retained
   facts/decisions/effects. Retain hostile-state checks and test both response orders.
5. Report production defects to the root writer; do not change production to make
   a test pass. Commit coherent test changes with actual evidence qualification.

## Verification

Formatting and diff checks are permitted while the shared heavy-build slot belongs
to another writer. No compiler, test binary or frontend run until explicit handoff.
Then run focused rules test targets and strict applicable Clippy only as coordinated
with root. Root owns combined production/app verification and PR integration.

## Evidence

- Base source and existing attack fixtures read. No tests run in this worktree.
- Historical source branches, captured fixtures and application tests remain outside
  this writer's scope.
