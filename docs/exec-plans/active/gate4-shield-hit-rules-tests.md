# Gate 4 — Explicit Shield hit rules regression support

Status: active; authoring only, no compiler or test execution claimed.
Writer: shield_rules_recovery (taking over preserved rules_architecture work),
branch codex/gate4-shield-hit-rules-tests.
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
- Fourteen source Shield regressions are authored, including both intent/order
  arrivals; real source use, components and controller refusal; natural20, fixed
  damage and Graze; three-use exhaustion and expiry; OA movement retention; and
  multiple spell rays with distinct causal issuers under the original cast key.
- Serialized hostile-state cases cover collecting and completed hit windows,
  original roll/cause/work/cover, selected authority, source cast, cost, defense
  effect and pending damage issuer. These are authored checks, not passing evidence.
- The generic run/run_meta/roll helpers still submit one command each. Only named
  roll_then_decline_hit_responses helpers add the explicit current-turn ordering
  and actual target controller's Continue for the exact just-produced hit.
- Static review fixed the shadowed window helper and removed inherited borrowed
  ownership from source creation's initial image. Source fixtures are explicitly
  reducer-level initial states, not application creation-path acceptance.
- rustfmt on the three changed test entrypoints and their modules passed, and
  git diff --check passed after memory/disk recovery. No compiler or test binary
  ran: the shared heavy slot remains reserved by root for aftermath verification.

## Exact next action

Root integrates these tests with the production Shield branch and the historical
ResumeHit request-ID binding fix. Then run the focused hit_shield, tactical_attacks,
tactical_movement and tactical_turns targets under the default stack with one build
job and CARGO_INCREMENTAL=0 before claiming any test passes. A source spell ray's
roll key belongs to its original cast, while attack.origin and issued_by belong to
the command that started that ray; an OA key belongs to its attack, not movement.
Do not loosen historical validation to an arbitrary same-occurrence roll search.
