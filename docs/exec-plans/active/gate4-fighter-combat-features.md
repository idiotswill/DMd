# Gate 4 — Supported Fighter combat features

Writer: root, taking over after bootstrap_audit reached its usage limit.
Branch: `codex/gate4-fighter-combat-features`.
Base: reviewed physical checkpoint `629177634572740ed637da5b5640fc49bbc08083`.
Status: active; Second Wind draft authored; formatting checked, executable verification pending.

## Objective and source authority

Make the existing supported Human Fighter/Soldier's Second Wind and Savage Attacker
usable in real tactical play. Advance source-faithful combat, physical dice, player
agency and exact save/resume (product definition lines24–30,249–251,489), Gate04 and
ADRs009–012/024/026. Use one tactical continuation and existing table/SQLite authority.

Pinned SRD5.2.1 p48: Second Wind spends a Bonus Action and heals raw1d10 plus Fighter
level; supported level1 has two uses, Short Rest restores one, Long Rest all. Page87:
Savage Attacker once per turn on a weapon hit rolls the weapon's damage dice twice and
uses either set. It does not reroll added damage, the hit, or grant extra actions.
Existing source provenance/attribution and character creation grants remain authoritative.

## Ownership and scope

Ready author owns shared scheduler/domain execution. First write separate source-bound
leaves/tests; agree actions, work, pending role, history and validation before shared
edits. Reserved SecondWind raw role16 follows Counterspell15. Second Wind uses the
existing resolution; Savage Attacker stays the real AttackDamage role. Protocol author
owns opaque transport. Root owns integrations, GitHub and gate acceptance. No Rust or
frontend execution without the root's explicitly assigned serialized compiler slot.

## Acceptance

- Real immutable character profile plus current feature/resource state derives grant,
  Fighter level and availability; no caller permission, level or total.
- Second Wind atomically spends one use and the active actor's Bonus Action, requests
  raw1d10, then uses vitality healing/max-HP bounds and owned condition/rest semantics.
  Reject absent grant, incapacity, spent/exhausted resources, stale/foreign actor unchanged.
- Optional Savage Attacker retains both raw weapon-dice sets, explicit selection and
  source attack/global turn use. Include critical weapon dice and off-turn reactions;
  never repeat added feature/spell dice, unarmed/spell/Graze or fixed damage. Existing
  Heroic Inspiration may replace only one valid die with one actual expenditure.
- Actual table choices remain controller-owned, retain original retry envelopes and
  expose only eligible feature/raw requests; no legacy bypass.
- Actual file-SQLite scenarios use created PCs/real equipment/combat, cold reopen at
  raw/choice phases, exact retry and independently restored mirror continuation.
  Invalid input/forged history writes nothing; box phases on the default Windows stack.
- Independent exact-head review and focused source/app/UI checks plus strict lint
  precede root canonical/CI/native integration. Record actual failures and exact evidence.

## Slices and non-goals

1. Pure source plans and two-set validation with source/critical/negative/turn tests.
2. Coordinated action/work/history integration and source-origin restore validation.
3. Real table form/choices and SQLite recovery/controller/audience evidence.
4. Review, serialized verification and root integration.

No extra classes/levels/catalog grants, extra attacks, general reroll permission, new
rest subsystem, other masteries/grapples/mounts or completed Gate4 claim. Preserve
existing rest restoration; broad rest/progression remains its existing owned scope.

## Evidence, risks and exact next action

Root AGENTS, physical plan, product clauses and exact SRD passages read. Worktree is
isolated. Existing legacy feature code is only a source reference: it rejects active
encounters. Existing SavageAttackerRoll repeats full request sets; tactical reuse must
prove all nonweapon indices identical, or retain a typed subset, never duplicate riders.

The first plan-write tool failed during host handshake and made no file changes;
memory recovered and no compiler has been started. Next: agree pure APIs with Ready,
author leaves/tests, checkpoint for review, then coordinate shared wiring and actual UI.

Root reconciled current main21cf176 and area source f65f2a4 in3c0aaf5 before
implementation. No code conflicts or changes beyond the imported area source.
The Ready writer has stopped at its separately committed foundation, so root owns
both later shared integrations. Keep the Second Wind raw identity tag16 reserved.
Implement Second Wind first as a dedicated source-derived work item in the existing
resolution. Its retained actor and pre-spend use count must reconstruct the owned
source feature and one paid Bonus Action while awaiting raw dice; the journal
authenticates admission. Reuse existing vitality healing and physical roll/Inspiration
handling. Do not create a fake spell, reset rests, or route active combat through
the legacy kernel. Savage Attacker follows as a separate coherent change.
Area canonical verification currently owns the serialized compiler slot.

Second Wind draft now has a public tactical action, retained source/payment work
and tag16 request. It derives the immutable created Fighter profile, spends the
Bonus Action and one use atomically, requests1d10 plus actual Fighter level, and
uses existing vitality healing and owned Inspiration/raw history. Its decoder and
pending validator reject changed source/payment/request images. No rest resource
reset, additional Action, fake spell or legacy tactical bypass was added.

The current-turn panel offers the owned sheet's remaining uses. Ordinary "I use
Second Wind" declarations also route host adjudication through the same tactical
path while initiative is active, preserving the original host envelope and owned
declaration. Restore admits only that specific nested action and verifies the real
declaration by semantic replay; noncombat legacy behavior remains unchanged.

Three authored reducer cases cover payment/turn/maximum HP/Inspiration/raw/replay,
source and authority refusals, and forged pending state. One authored real file-SQLite
case covers direct and declared uses, cold reopen at payment and dice, exact retry,
foreign actor/wrong dice channel refusal, and an arithmetic-consistent invented
extra expenditure rejected by semantic restore. One UI case checks ownership,
Bonus Action, depleted uses and pending-work controls. These tests have not run.
Rustfmt and whitespace checks pass. Next: compile/fix this checkpoint after the
already queued area UI, Ready correction and protocol canonical batches; then
complete source Savage Attacker, current protocol/Ready integration and review.

Draft PR36 isolates the Second Wind change against area PR34; Savage Attacker will
follow separately within this same gate. Initial Linux36141460203 failed compilation
at the exhaustive recorded-roll role validator: the new SecondWind role was missing.
Add its actual single d10 constraint, preserving every prior role and validation.
This is the first executable failure, not a passing source result; rerun CI on the
corrected head while local heavy verification remains serialized.
