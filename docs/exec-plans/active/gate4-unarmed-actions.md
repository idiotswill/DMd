# Gate 4 — Own-turn unarmed damage

Writer: root. Branch: `codex/gate4-unarmed-actions`.
Base: fetched main cd8d4a4432c9e83d1c0c8ce65b79591bec425c8a.
Status: PR40 merged with verified full-tree parity; post-merge checks running.

## Objective and authority

Expose ordinary unarmed damage through the actual owned Attack action, physical d20,
existing attack/vitality continuation, desktop controls and durable table history.
This advances Gate04 actions/attacks and the product's physical dice, source fidelity,
player authority and exact suspension requirements. Root AGENTS, product definition,
Gate04, gate-execution protocol and ADR026 have been read. PR39 first aid is merged;
PR38 reaction work remains an independent in-progress slice. No next gate begins.

Pinned SRD5.2.1 p190 was read from the verified source extraction outside the repo.
An Unarmed Strike uses a punch, kick, headbutt or similar blow within5ft. Its Damage
option rolls Strength plus proficiency and deals1+Strength bludgeoning damage.
It needs no free hand, unlike Grapple, and fixed damage is not doubled by a critical.
The source has no minimum-one damage clause; the existing damage floor remains zero.

## Scope and boundaries

- One target-only typed action. No arbitrary ability, damage, DC, ItemId or grant.
- Use current actor authority, precise target knowledge, source proficiency and
  conditions, five-foot reach and the existing cover/geometry rules before payment.
- Spend one existing Attack action attack. Start the normal one-attack action only
  when needed; honor an already admitted remaining attack, with no new extra attacks.
- Retain a typed own-turn unarmed admission tied to the paid attack window. Preserve
  legacy Opportunity admission/wire and source attacks. No fictitious weapon receipt,
  equipment change, ammunition cost, mastery or Savage Attacker weapon-dice benefit.
- Enter the same attack roll/automatic fixed damage/knockout/concentration work.
  Unsupported class-specific substitutions require source grants, not UI guesses.
- The desktop uses existing actor-specific current contacts, like first aid, without
  altering legacy projection shapes/digests. Explain the fixed damage and no damage die.
- Grapple, Shove, Escape, dragging, remaining Help/actions/masteries, PvP-consent
  enforcement and all reaction/spell mechanisms remain explicit Gate4 work. This
  bounded damage path does not complete the grapple/unarmed family or the gate.

## Acceptance and verification

1. Source/cost/ownership/knowledge/range/full-hands admission and critical/fixed damage;
   raw request, invalid input, Inspiration and forged paid-admission regressions.
2. Actual table-created characters/source opponent, file SQLite cold reopen at raw
   d20/knockout, exact accepted retry and independent semantic restore; coherent
   invented cost/source history must reject without writing restored rows.
3. Controller-owned desktop action, no optimistic budget/HP changes, pending-work
   blocking and current-contact target selection; component checks/test/build.
4. Separate full-diff review, serial focused/fast/canonical verification and all six
   exact-head Linux/native checks. Protected merge, fetched tree parity and post-merge
   verification. Root is sole writer; independent final-head review also completed.

## Current evidence and exact next action

Reviewed combined source: f9602a8454fe09958ef04ac3d6fc26ba8410d485 (2026-09-25).
Local canonical verification passes661 GNU Rust tests, strict all-target Clippy,
formatting/checking, genericity and architecture guards. The actual41 table-loop
tests pass, including both first aid and unarmed cold/retry/semantic recovery cases.
Combined desktop checking has zero errors/warnings; all61 tests and136-module build
pass. All six source checks pass: Linux36163502155 has662 Rust tests; native Windows
36163502287 has664 Rust tests,61 desktop tests and fresh offline installer
artifact10876968168 (231299398bytes). Both declared MSRV checks pass.

PR39 merged as487239c3f14685ba0c2a998574080aedb8b57a9e after all six final-head checks.
Fetched full-tree equality to sourcea0b4b57 was verified. That source was already an
ancestor here; reconciliationf2e6f84 preserves the entire f9602a8 tree unchanged.
Root separately reviewed the complete15-file Unarmed diff after the first-aid merge,
including source/cost reconstruction, valid negative fixtures and actual SQLite/UI.

Final f400d34740f919eb67efa796ab7d44686f924dbf has independent full17-file review,
verified production parity and all six passing checks36166645612/36166645636:
662 Linux/664 native Rust,61 UI, zero static diagnostics and fresh installer
10880017473 (231291314bytes). Protected squash merged PR40 as
a84c5a11359e8e91a3cfa0d2e6a5946082df3da2. Fetched source/main trees both equal
c3d15edacc6b2892749e61a8548651743e006e68. Post-merge runs36169863765/36169863809
are running; finish recording their results in the continuing Gate4 evidence.

Only one heavy local Rust/frontend job may run. No Grapple/Shove/Escape, equipment
allowance or PvP-consent completion is claimed; the full twelve-family ledger and
eighteen spell mechanisms remain. Continue the approved active Gate4 work.

## Earlier implementation history

The following records earlier authored/focused stages; current evidence above supersedes
their pending-check notes.

Only one heavy local Rust/frontend job may run; the Unarmed reducer batch owns that
slot now. Source admission and the paid window adapter, actual UI and recovery tests
are authored. Formatting/whitespace checks pass; no executable pass is claimed yet. Later integrate verified first aid and reaction execution versions,
initializing current work ancestry without reinterpreting old accepted history.
The complete Gate4 twelve-family ledger and eighteen-mechanism matrix stay binding.

## Draft implementation and review

UnarmedStrike carries only the target. Its typed UnarmedAction admission retains the
actual AttackAction window, pays one attack, interrupts rest, clears continuous travel
and enters the existing physical attack queue. Validation rederives Strength/source
proficiency, exhaustion, conditions, reach, cover and armor training. Legacy own-turn
weapon and opportunity wire images remain unchanged. Body attacks do not receive the
source's underwater weapon-only penalty (SRD p16). Fixed damage floors at zero, has
no damage die and never spends Savage Attacker or creates a weapon receipt.

Six reducer tests cover full hands, ownership, failed/no-cost admission, actual open
Attack windows, forged source/cost, natural extremes, zero damage, knockout, Poisoned,
exhaustion, Inspiration and PC/creature armor/proficiency. A new file-SQLite test uses
normal campaign/PC/source Goblin creation and three rounds of fixed damage into an
actual knockout choice. Each attack/raw die/knockout crosses cold reopen, accepted
retry and independent restore. A structurally coherent invented Bonus Action must
fail semantic restoration without rows. Desktop coverage checks own current contacts,
foreign/remembered exclusion, paid attack availability and pending work.

Root separately inspected all changed source and test files. Further executable
failures must be corrected without weakening assertions. Next: finish focused tests,
open draft CI early, run actual file-SQLite/UI checks, then canonical/full-head review.
Equipment changes accompanying an unarmed attack, Grapple/Shove/Escape and PvP consent
remain explicit Gate4 work, not an implicit relaxation of those source requirements.

The focused batch found two invalid test setups before source admission: an obstacle
embedded in occupied space and a modified fighter profile inconsistent with its
creation receipt. The barrier now sits in an unoccupied five-foot gap on the actual
grid; assertions validate the starting aggregate before the requested rejection.
Untrained armor now uses a real source wolf with borrowed leather and its correctly
recomputed armor class; fighter/goblin source profiles remain unchanged. All six
focused unarmed reducer tests now pass. Full app/UI/canonical and CI remain pending.

The actual file-SQLite regression passes locally (237.64s), including all three rounds,
knockout, cold retry, independent restored continuation and semantic rejection.
Desktop static checking reports zero errors/warnings; all60 tests and the135-module
production build pass. The form now explicitly says Strength modifier, Proficiency
Bonus and zero damage floor. Integrate reviewed first aid before canonical verification
so the merge candidate exercises both independent source actions together.
Legacy Opportunity reconstruction remains unchanged; current-source OA armor-training
parity belongs at the explicit live-response version boundary in Gate4, not an
unannounced change to old accepted attacks. All other recorded obligations remain.

Normal integration imports first-aid sourcea0b4b57 with both adjacent typed actions,
forms and tests retained. Medicine production is identical to verified cea43 and its
strengthened tests pass, but its final CI/protected merge remains pending. The combined
source/UI needs fresh canonical and desktop evidence; do not inherit isolated passes.
