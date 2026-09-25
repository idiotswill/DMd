# Gate 4 — Supported Fighter combat features

Writer: root, taking over after bootstrap_audit reached its usage limit.
Branch: `codex/gate4-fighter-combat-features`.
Base: reviewed physical checkpoint `629177634572740ed637da5b5640fc49bbc08083`.
Status: Second Wind source and production-path verification pass; final main-based head checks and protected merge pending. Savage Attacker remains separate PR37.

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

Corrected6403b65 passed fast verification, strict Clippy, both guards and MSRV.
Linux36141715529 then passed67 application tests and failed the new cold-declaration
case: the restored request origin still allowed only direct tactical actions or
legacy nested rules, despite the new strict nested-adjudication validator. Add only
the matching, previously validated Adjudicate→SecondWind envelope to that origin
join. Keep exact metadata, accepted audit, pre-tactical anchor and full semantic
replay checks. The unchanged cold case must pass on the new head before acceptance.

Linux36142366967 on8549b56 passed all application cases, including the unchanged
cold-declaration restore regression. The reducer suite then exposed a fixture setup
error: its Inspiration reroll had no Inspiration grant. Grant that resource in the
initial fixture before initiative, as the existing attack reroll regression does;
retain the actual expenditure, original raw die and replay assertions. No production
rule or rejection is relaxed. Full verification still awaits the corrected head.

Root imported area evidence718 and reconciled main172a15a only after verified
full-tree equality to718 and ancestry inclusion. Protocol PR35 sourceee0a9c9 then
merged without conflicts in58dd03c. Inspection confirms the Second Wind changes
remain additive to the extracted projection, modern transport and strict restore
paths. Its exact envelope remains independently authenticated. This combined branch
will be reviewed and verified locally after protocol's serialized canonical batch;
fresh CI must cover the combined source. Stack PR36 on PR35 until protocol merges,
then reconcile its verified squash and retarget main. Savage Attacker stays separate.

Combined841e660 passed all four Linux jobs36143802483 (644 Rust tests). Windows
36143802526 passed MSRV and desktop checks/tests, then overflowed the default test
stack in the existing source-area cold-retry fixture. No gameplay assertion failed;
the process exited STATUS_STACK_OVERFLOW during that scenario. Its outer phases
already use heap-pinned futures, but submit→submit_both→execute_both→execute_table
still nests large recovery poll frames. Pin those helper boundaries as well, keeping
both actual runtimes, all assertions and default stack size. Native rerun is required;
this is a proposed bounded fixture correction, not a claimed passing result.

The corrected source f669389204dccd20cd9847ce0eeed2318add01a8 passed full local
`./scripts/verify`:643 GNU Rust tests, strict workspace all-target Clippy, formatting,
check and both guards. Local desktop verification passed56 tests, zero static errors/
warnings and the134-module production build. Logs are tooling/pr36-canonical-f669.log
and tooling/pr36-ui-{check,test,build}-f669.log. All six source CI checks passed:
Linux36145973970 (644 tests) and Windows36145973959 (646 native tests, MSRV,
desktop and fresh offline packaging). The existing area cold-retry case now passes
on the default native stack with every assertion retained.

Root's fresh review inspected source grants/costs, pending reconstruction, raw and
Inspiration handling, exact nested declaration authority, real cold file-SQLite
retry/restore, UI ownership and the final area-future fix. Supporting reviewers
remain quota-blocked. No source defect remains from this pass; full Gate4 completion
is not claimed. Savage Attacker has its own source, cold recovery and UI PR37.

PR35 finaldc0baad passed all six checks and merged as mainc9b82072b6bfd93455bdab2a7712dd0352088d93.
Fetched full-tree equality to dc0baad was verified before importing its three evidence
documents and reconciling squash ancestry. The resulting e1a10de source is byte-
identical to verifiedf669389 across crates/apps/content/manifests/scripts/tests/CI.
Next: final evidence-head CI on PR36 retargeted to main, expected-head protected
merge, fetched tree equality and post-merge checks. Continue Gate4 after this slice.
