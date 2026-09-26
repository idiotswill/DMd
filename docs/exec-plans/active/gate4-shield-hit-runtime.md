# Gate 4 — Source Shield at an accepted attack hit

Status: active integration draft; all six checks pass for executable candidate
8b5cf52, including the actual Shield recovery case. Final aftermath integration,
canonical and exact-head acceptance remain pending.
Writer: root. Branch: codex/gate4-shield-hit-runtime, based on verified main
d82d7b2c28be3b74e6e2b0b8c85ffe8c042b1278. Verified PR42 main
2798b6b1d6263b5e321a1903d9fb4f2331b73895 and verified source-control PR43 main
e813e3a13911497902a3d4a55aec3c70653afb2a are integrated.

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

## Current verification checkpoint

The executable candidate with all six CI checks passing is
8b5cf52a250c5f381d22977391906cfedc53a630. Documentation successor7bc01af and
verified-main reconciliation98399da change no executable source. The latter's
whole tree equals7bc01af at926629f4aec461374b38e5ff57a93260e190d408.
No Gate4 criterion is waived; final integrated/canonical acceptance is pending.

- The fourteen Shield rules cases pass on the local default Windows GNU stack
  (57.29 seconds, jobs1/incremental0). The prior full tactical-attacks run passed
  all 107 existing cases and exposed a shared missing Mage-language fixture input
  in the fourteen new cases. Supplying the real required three languages corrected
  the fixture; production rules and assertions were unchanged. Logs are outside
  the repository: tooling/shield-hit-tactical-attacks.log and
  tooling/shield-hit-focused-r2.log.
- Linux run36193351652 passes all four jobs, including fast verification, strict
  Clippy, Rust1.88 and both guards. Actual runtime job108263431336 records715 Rust
  tests across54 result suites, zero failed/ignored, all121 attack cases and47
  table cases in1885.29 seconds. The owned Shield SQLite case passes. Checkout is
  synthetic mergee07b7db932aa1ab57280566e6c90a43406c2231a of8b5cf52 and
  main2798b6b. Fetched complete trees both equal
  cc35d233924c88344598dd8206688e34455fd6bb; their full diff is empty.
- Windows run36193351721 passes stable108263469783 and MSRV108263468702 at literal
  8b5cf52. Actual stable logs record717 Rust/54suites/47table in1950.51 seconds,
  zero failed/ignored,84 UI tests in15files, zero Svelte errors/warnings and138
  built modules. Fresh EXE/NSIS artifact10890038954 is231884106bytes; upload log
  and run-artifacts API agree on SHA256
  d2b4193963af65a9e814c55e2e73879be1c6ffcbb3c4e2497144fbeec393fc77.
- The genuine player-owned Mage SQLite case passes on Linux and native Windows.
  Updated source-control cases use fresh flow3; four genuine old exports retain
  their original bytes. Final integrated canonical and exact-head acceptance
  remain required after aftermath integration.
- Independent full production review and the actual source-control merge delta
  32d937b plus equivalent fixture correction8b5cf52 are clear. Review covered both
  parents, v1/v2 replay/projections, source authority, activation/attendance,
  role-specific opaque response routing, both UI test sets and actual scenario
  registration. Static review is not a runtime result.

Source-control's canonical verifier completed and released the heavy slot. Root
ran the otherwise identical source in an isolated diagnostic worktree,
06045b2 over7bc01af, with independently reviewed read-only capture hooks in the
Night Hag and owned Shield tests only. The hooks preserve genuine pre-change
flow3 exports for the next semantic boundary; they never merge into production.
The default-stack focused batch passes: Night Hag1/1 in500.88 seconds, source
units3/3 in0.13 seconds, actual source cases2/2 in177.07 seconds and the full owned
Shield case1/1 in1427.43 seconds. All four real flow3 captures are preserved.
Diagnostic successorf9698e1761ee0485c4cf0ff593f89a03f016302b adds only the exact
reviewed compatibility module and capture bytes for separate baseline restoration
and original-receipt retries. Its exact selected test passes1/1 in299.50 seconds
(compile25.99 seconds, exit0), covering all four byte/typed roundtrips, complete
restores, cold resumes, every original envelope retry and changed-body zero-write
refusal. Log: tooling/shield-baseline-f9698e1.log. This source-equivalent proof
does not replace final integrated canonical verification. The earlier disk and
committed-memory shortage is resolved; use jobs1, incremental0 and default stack.

### Integration and recovery decisions

The v2 source channel derives privileged-response authority from the actual
retained target. A fresh Begin3/UpgradeTo3 refuses unactivated player-owned
sources before they can become stranded; older Begin versions and unit upgrade
are unchanged. This additional guard runs behind privileged issuer checks so an
unauthorized Begin cannot disclose source-access state. The typed-state unit case
separately covers activation and historical admission.

The real SQLite fixture establishes both controllers as Present with accepted
EndSession/StartSession commands, creates the Mage through the content catalog,
assigns its owner, and supplies physical attack dice. Its table contract records
an agreed practice encounter; that prose does not prove typed PvP enforcement.
Cold independent continuation, exact retry, wrong-controller/Host theft, raw
canonical bypass, whole unrelated DTO invariance, original damage cause and
forged current/retired snapshots are all asserted. The prior hit snapshot is an
actual captured earlier image backfilled as a recovery anchor, not an old fixture.

Desktop tests retain both integration parents and exercise a selected source's
Shield/outbox retry across ownership changes. Host cannot substitute an owned
source response; an explicitly issued delegated ordering capability still works.
Every target acknowledges collection even without an available Shield. Continue
is free controller coordination, available when incapacitated; it is not a
fictional action or spent Reaction. This prevents eligibility-dependent public
ordering/timing from exposing private source information.

ResumeHit retains the exact accepted roll identity and authenticates its parent
AttackRoll occurrence, tactical role/purpose and issuance origin. It does not infer
that origin from the outer movement or casting resolution. Passing rules cases
cover an opportunity attack and a later Scorching Ray with distinct ancestry.

Early compiler diagnostics required the existing workspace UUID dependency, an
explicit Serde bound for optional opaque hit keys, a correct vector-removal test,
and strict-lint idioms. Final integrated32d937b passed compilation/MSRV; Clippy
identified a manual hand-membership scan, corrected equivalently in8b5cf52.
No lint exemption, assertion weakening or invented default capability was used.

### Verified prerequisite and next action

PR42 merged as2798b6b1d6263b5e321a1903d9fb4f2331b73895 with full-tree parity to
reviewed cbe9575 atddfe5b58c0c55ad2aab6e367de295bf7c1760e5e. All six post-main
checks pass: Linux36188457488 has695 Rust/44table cases; Windows36188457455 has697
Rust,67UI,zero static diagnostics,136modules and fresh EXE/NSIS. Artifact10888336544
is231464157bytes with SHA256
7cb2a446ffb122ca6aa1d78430c3c50b79eb8da78af0f77b1e0016747b79c19a.

PR43 is protected-squash merged as e813e3a13911497902a3d4a55aec3c70653afb2a.
Fetched whole-tree95ccff0f48d47f8a92e2a2eafab6f8385e9f7ce3 equals the independently
reviewed finala119d7f. Its exact canonical verification passes698Rust/54suites/
46table; all six final-head checks pass699Linux/701native Rust,77UI,0/0,137modules
and fresh EXE/NSIS. Artifact10890060494 is231627996bytes, SHA256
ed11f6b7fba78c8733a59cf524514c32e7de118d882011c51f0bd8dea9ffbc12.
Post-main runs36194898895/36194898980 separately pass all six checks on literal
e813e3a:699Linux/701native Rust,54suites/46table,77UI,0/0,137modules,7manifest
tests and fresh EXE/NSIS. Artifact10889754911 is231636930bytes, SHA256
5b1f74d35e3204bd16d8364c32e47d9967109bcc5979e7c2c6a1b23e735c52e6.
The bounded source-control plan is archived after that proof.

The planned remaining merge order is aftermath PR44, then this hit slice. PR44's
bounded conclusion currently validates ReactionsV1/flow2 only. Before final Shield
canonical acceptance, normally integrate its verified main, deliberately admit
the same retained aftermath cadence under flow3, and update fresh real session/
source scenarios to Begin3. Preserve already accepted flow2 images and source
timing; do not leave the new current executor unable to conclude hostilities.
If verification changes the merge order, perform that integration explicitly in
the later branch instead. Full encounter release remains separate Gate4 work.

Read any current CI failure before changing source. The registered player Shield
SQLite case and modified source-control unit/application cases now pass on the
default stack. Complete aftermath integration, canonical verification and final all-six-head CI with fresh
packaging, full delta review, expected-head merge and fetched tree parity. Record
post-main evidence in the PR. Continue the live-reaction umbrella inside Gate4;
Magic Missile, Counterspell, Ready release/held spells and all other open tactical
families remain required. Do not pause at this slice boundary or begin Gate5.
