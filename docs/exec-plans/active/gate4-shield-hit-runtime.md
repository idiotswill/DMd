# Gate 4 — Source Shield at an accepted attack hit

Status: active implementation; no runtime or verification completion claimed.
Writer: root. Branch: codex/gate4-shield-hit-runtime, based on verified main
d82d7b2c28be3b74e6e2b0b8c85ffe8c042b1278. PR42 compatibility is frozen under
verification in a separate worktree; integrate only its verified merge. PR43 owns
source-creature player control and must be integrated before player-path acceptance.

## Objective and authority

Implement the first real source reaction through the existing table, one tactical
queue, physical dice and durable recovery: an accepted hit offers its target Shield,
the actual controller chooses it, and the defense applies before damage. Follow
AGENTS, product-definition's production/agency/recovery invariants, Gate04,
ADR026/027/028 and pinned SRD5.2.1 p161 (Shield) and p187 (simultaneous ordering).
The full reaction umbrella and twelve-family Gate4 ledger remain active.

## Scope and non-goals

- Explicit ShieldHitV1 execution boundary. Preserve all genuine Legacy/ReactionsV1
  histories and old unit UpgradeExecution1-to2. New targeted forward upgrade requires
  settled Active state, no raw work/resolution and no paid Ready declaration.
- Uniform current-turn ordering stage for every attack hit, independent of private
  source eligibility. Source-safe private intent, explicit ordering or exact-trigger
  delegation, then a fresh command from the selected controller to cast or decline.
- Genuine source Mage Protective Magic, current components/shared uses/Reaction,
  retained original roll and source damage, current defense and owner-Start expiry.
- Physical, intrinsic and spell attacks, including an opportunity attack inside a
  retained movement path. Child Shield uses the existing cast/frame stack; it cannot
  replace the attack, movement, area or parent casting records.
- Actual private desktop controls and file-SQLite cold retry/independent replay.

This bounded version does not introduce Counterspell, Magic Missile target windows,
Ready release/held spells, or child physical attacks. Those mandatory Gate4 mechanisms
need subsequent explicit semantic boundaries if they change previously accepted
execution. This PR must not be described as complete Shield or reaction coverage.
Do not add a general historical-fact bypass to make later mechanisms fit prematurely.

## Implementation decisions before code

Freeze the existing attack facts at the actual accepted AttackRoll. Keep the original
modifier, mode, AC, critical rule, damage and accepted raw history. A typed hit record
binds the exact work, cause and parent attack. Reconstruct the original source against
current unchanged geometry/equipment/conditions, excluding only defense effects
derived from committed Shield child casts authenticated to this exact window.
Re-evaluate same-source suppression on that read-only defense set: subtracting five
from live AC is incorrect when an earlier Shield was already active. Final hit uses
the live effective defense plus retained/source-checked cover; natural20 still hits.
No client-supplied effect IDs or numeric permissions are admitted.

An independent response starts its own ordering authority scope. Each action's work
remains in the existing frames and ancestry. The bounded Shield child does not need
to overwrite the current physical cursor; future Ready attacks require an explicit
suspended-cursor policy. Damage request issued_by remains the original accepted
attack roll's accepted_by, even when the defender's response resumes it.

Private acceptance is nonpaying intent. The selected caster must issue a fresh
current-head command. Revalidate source grant/components/budgets before costs; stale
or invalid requests write nothing. Never retime a prior command, fabricate System
authority or attribute the response cost to the player who ordered the trigger.

Existing tests of fresh live actions must follow the new current executor and make
explicit controller choices. Do not convert them all to historical replay to evade
the new pause. Untouched genuine old fixtures continue under their own versions.

## Acceptance and verification

1. Real Mage hit/Shield/miss and natural20/damage, later attack and own-Start expiry;
   shared-use exhaustion, spent Reaction and blocked V/S refusal without partial cost.
2. Genuine player-controlled Mage after PR43: owner cast/decline, wrong player and
   host substitution refused, actual attendance and opaque capabilities enforced.
3. Uniform zero/one/two private offer ordering surface, unrelated complete DTO,
   revision and transcript invariance; selected intent and exact-trigger delegation
   cannot transfer to a child occurrence or another attack.
4. Retained attack roll/cost/ammunition and OA movement cursor, original damage
   issuance cause, source spell/physical parent preservation, no second queue.
5. Cold file reopen at each decision; exact accepted retries; independently continued
   and restored histories; changed fact/cause/source/effect/ordering anchors rejected.
6. All four genuine PR42 fixtures unchanged and passing. Strict canonical verification,
   desktop checks/tests/build and all six final-head CI checks; full independent
   review; protected merge, fetched tree parity and final main evidence.

## Validation and next action

Pre-code source review only. Root's compatibility canonical run owns the local heavy
slot; no build runs from this branch yet. Add typed window/version state and narrow
rules integration, then table projection/transport/UI and genuine source regressions.
Integrate verified PR42 and corrected PR43 before final acceptance. No gate pause and
no movement into Gate5 at this PR boundary.
