# Gate 4 — Source Shield at an accepted attack hit

Status: active draft implementation; no runtime or verification completion claimed.
Writer: root. Branch: codex/gate4-shield-hit-runtime, based on verified main
d82d7b2c28be3b74e6e2b0b8c85ffe8c042b1278. Verified PR42 main
2798b6b1d6263b5e321a1903d9fb4f2331b73895 is integrated. PR43 owns
source-creature player control and must be integrated before player-path acceptance.

## Objective and authority

Implement the first real source reaction through the existing table, one tactical
queue, physical dice and durable recovery: an accepted hit offers its target Shield,
the actual controller chooses it, and the defense applies before damage. Follow
AGENTS, product-definition's production/agency/recovery invariants, Gate04,
ADR026/027/028 and pinned SRD5.2.1 pp161–162 (Shield) and p187 (simultaneous ordering).
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
3. Uniform zero/one private Shield offer ordering surface, unrelated complete DTO,
   revision and transcript invariance; selected intent and exact-trigger delegation
   cannot transfer to a child occurrence or another attack. Two eligible controllers
   cannot both cast self-only Shield against a single hit target; retain the general
   two-respondent ordering tests, and require actual multiple-respondent privacy
   acceptance with the later Counterspell/Magic Missile trigger family.
4. Retained attack roll/cost/ammunition and OA movement cursor, original damage
   issuance cause, source spell/physical parent preservation, no second queue.
5. Cold file reopen at each decision; exact accepted retries; independently continued
   and restored histories; changed fact/cause/source/effect/ordering anchors rejected.
6. All four genuine PR42 fixtures unchanged and passing. Strict canonical verification,
   desktop checks/tests/build and all six final-head CI checks; full independent
   review; protected merge, fetched tree parity and final main evidence.

## Validation and next action

Typed version/window, source Shield admission, child commit/resume, original-damage
cause, source reconstruction and strict retained invariants are authored but have
not compiled or run. Independent review caught completed-occurrence reconstruction
and an eligibility timing leak; both corrections are being implemented. Every hit
target must acknowledge collection even when no Shield is available. Continue is
cost-free coordination, legal for an incapacitated/ineligible target and owned by
its actual controller; it is not an SRD Reaction or an invented fictional action.
Affirmative intent and final casting require the genuine current source. This makes
the public ordering/wait stage independent of private source eligibility. Test both
arrival orders and complete unrelated DTO/revision/transcript invariance through
private selection. A later general multi-respondent family retains its own semantic
boundary and must not infer priority from arrival or default choices.

The compatibility canonical run passed on facd988; final cbe9575 passed all six CI
jobs with 695 Linux/697 native Rust and 67 UI tests, plus the fresh native installer.
Protected PR42 main has the identical full tree ddfe5b58c0c55ad2aab6e367de295bf7c1760e5e;
post-main runs 36188457488/36188457455 remain pending. The
local disk/commit-memory failure is recovered with 28 GB free. All further local
compilation is serialized with one job and CARGO_INCREMENTAL=0; aftermath focused
verification currently owns the heavy slot. No build has run from this branch yet.
Initial owner-safe hit DTOs, role-specific opaque capabilities and transport admission
are authored. Damage effect observations retain an optional earlier cause while
lifecycle stamps remain the actual execution command; original None images stay
unchanged. Finish exhaustive integration and desktop controls, then genuine source
regressions. Rules fixture/test authoring may use an isolated child branch with a
separate writer; no concurrent writes to this branch. This is incomplete WIP, not a
verified production milestone.
Desktop response controls and their direct/real outbox retry cases are authored.
The genuine player-owned source SQLite case is drafted and deliberately awaits
PR43 before registration/compilation. Isolated app/rules fixture drivers retain
single-command raw helpers and explicit current-window decisions. Static review
found a nested-attack ancestry mistake: ResumeHit now retains the exact accepted
roll ID, validated against its parent AttackRoll occurrence, tactical purpose and
actual issuance origin. It cannot guess that origin from the outer movement or
casting resolution. Targeted opportunity-attack and later-ray tests must prove it.
Fourteen source Shield rules scenarios and explicit fresh-flow application drivers
are integrated from isolated test writers. Only formatting and static diff checks
have run; compiler, rules, desktop and SQLite evidence are still outstanding. An
early draft PR may run remote diagnostics while the serialized local slot is busy.
Its merge requires all acceptance above, including the player-owned source case.
The first draft CI at d3e3ed3 caught an undeclared UUID dependency used by pure
Shield option previews (Linux stable/MSRV jobs 108248022555/108248022760). The
workspace already pins UUID; declare it directly in dmd-rules and regenerate the
lock edge, then rerun the actual compiler checks. No failed check is waived.
The same draft's Windows frontend step passed 72 tests, zero static errors/warnings
and a 137-module build (job 108248022369); its Rust failure was the same missing
dependency. Follow-up 53d1197 compiler jobs 108249026632/108249026705 exposed an
unnecessary inferred Serde Default bound on optional hit keys and one test using
set removal on a vector. Explicit deserialization bounds preserve absent fields
without inventing default capabilities; the mutation test now removes by value.
Runtime and final-head acceptance remain pending.
Integrate corrected, verified PR43 before final acceptance. No gate pause and
no movement into Gate5 at this PR boundary.
