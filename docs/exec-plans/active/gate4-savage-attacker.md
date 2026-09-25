# Gate 4 — Tactical Savage Attacker

Writer: root. Branch: `codex/gate4-savage-attacker`.
Base: Second Wind/protocol integration841e660 (draft PR36/PR35), with area main172a15a.
Status: source, recovery and desktop verification pass; final main-based head CI and protected merge pending.

## Objective and boundaries

Make the supported created Soldier's source Savage Attacker usable on a real
weapon hit in tactical play, including an off-turn Opportunity Attack. Use the
existing attack continuation, physical dice, source grant, global combat turn,
atomic application transport and cold SQLite replay. Advance Gate4's combat
actions/weapon families and the product's physical-dice, agency and recovery
contracts. AGENTS, Fighter feature plan, ADR026/027 and pinned SRD5.2.1 p87 read.

The source permits once per turn rolling the weapon's damage dice twice and using
either set. The confirmed hit and damage modifier are not rerolled. Critical weapon
dice belong to the repeated pool; added nonweapon dice are rolled once. Unarmed,
spell, Graze and fixed damage do not gain permission from the presence of dice.
The implementation must rederive the immutable character grant and retained source
weapon plan; a payload or UI flag grants nothing. Current catalog expansion remains
Gate6; other still-required Gate4 mechanics remain in their existing plans.

## Planned slices and acceptance

1. Retain a source-derived weapon-dice prefix count alongside both raw damage sets
   and explicit selection. Preserve legacy Savage records with their original full-
   pool semantics. Reject changed shared extra dice and invalid prefixes. Inspiration
   changes one actual die: a weapon die in its selected set, or the one shared extra
   die in both representations, with exactly one resource expenditure.
2. Add optional typed tactical submission to the existing AttackDamage request.
   Recheck the real hit/weapon/grant/current turn and spend once only when acceptance
   succeeds. Retain normal single-roll submission as declining the optional feature.
   No new queue, reset, fabricated source, hidden automatic choice or duplicate damage.
3. Project availability only with the audience's owned real pending roll. Expose two
   physical weapon sets and the choice in the actual desktop roll form. Preserve
   opaque request handles, original retry body and public/private dice authority.
4. Reducer/source tests: raw/critical/selection/decline, same-turn refusal, another
   actor's turn OA, foreign actor, missing grant, nonweapon/fixed damage, bad dice,
   Inspiration and hostile retained record. File-SQLite tests: cold reopen at the
   damage pause, exact accepted retry, restored mirror and forged state/history
   rejected without writes. UI checks cover the actual controls and dispatched body.
5. Review full diff; run focused tests and strict lint, canonical verify and desktop
   check/tests/build at the serialized compiler slot. Final exact-head CI, protected
   merge, fetched full-tree parity and post-merge checks precede slice acceptance.

## Risks and next action

Legacy `SavageAttackerRoll` currently repeats a whole request and must not silently
reinterpret historical accepted payloads. The new optional source prefix will be
explicit and checked against tactical purpose; original legacy records omit it.
Only independently verified code can be integrated as accepted. Supporting agents
remain unavailable due account quota; root owns writing/review and records that
limitation. Protocol canonical currently owns the local heavy compiler slot.

Next: implement the pure raw-pool validation and source-derived tactical admission,
then actual transport/UI and cold-recovery coverage. Do not claim declaration/schema
or leaf tests alone as end-user feature or Gate4 completion.

Compatibility decision: new live roll affordances use a read-only, snapshot-consistent
query authorized by the audience revision, opaque roll handle and selected roller.
They do not enter or alter immutable v1 presentation digests or accepted retry bodies.
Adding availability fields to the old replayed DTO would reinterpret historical
digests, so that approach is rejected. Old campaign/transport history remains under
its original projection. This query grants no action authority: the submitted typed
action rederives source, payment, dice and turn under the normal writer transaction.
Tests must prove the query writes nothing, foreign/stale handles fail, an older
initialized presentation remains valid, and accepted modern retries stay exact.

Draft922b22f is PR37, stacked on PR36. Linux36145262618 passed MSRV and both guards,
then strict Clippy rejected a nested player-channel `if`; collapse it without changing
authorization. Compilation reached the application. The tests were not reached by
that job. New authored coverage now includes a genuine file-SQLite critical dagger
hit, preexisting presentation history, read-only/foreign option queries, normalized
opaque dice, independent restored continuation, cold exact retry and current-state
forgeries that remain structurally valid but contradict the journal. Two roll-form
cases and one full table/outbox retry case exercise the actual source controls.
These tests remain unrun pending the serialized local slot and corrected CI.

Native36146302726 on af4c4cf stopped at frontend static checking: the new table test
had one extra closing brace and the new component test used a matcher not installed
in this project. Split the expected raw payload into a named object and use the
existing null assertion for absent controls. Preserve the assertions and actual UI
behavior. Add explicit source-spell and fixed unarmed availability/refusal checks
to their existing genuine attack regressions. Fresh CI and all local checks remain
required; neither this correction nor the new recovery scenario is yet verified.

Source47fc39a2f1d7fe046ae8bf5d532c99b47d9637cd now passes full local
`./scripts/verify`:649 GNU Rust tests, strict workspace all-target Clippy, formatting,
check and both guards. Local desktop verification passes59 tests, zero static errors/
warnings and the134-module production build. Logs: tooling/pr37-canonical-47fc.log
and tooling/pr37-ui-{check,test,build}-47fc.log. All six source CI checks pass:
Linux36146796194 (650 Rust tests), Windows36146796251 (652 native Rust tests,
59 desktop tests, MSRV and fresh offline packaging). Actual logs were inspected.

Root's separate full-diff review covered source/critical weapon pools, optional normal
submission, once-per-global-turn use, off-turn OA, both raw sets, shared extra dice,
single Inspiration expenditure, read-only owned availability, opaque submission and
unchanged retry envelopes. Real SQLite cold/mirror/hostile replay and UI regressions
passed. Supporting independent reviewers remain quota-blocked; no source defect
remains from this review. This is one combat feature, not completed Gate4.

PR36 finalc04e3b6 passed all six exact-head checks36149421185/36149421183 and merged
as b86e2220709a7462da31547dfe921925ce0c0877. Fetched full-tree parity was verified
before importing its evidence and reconciling squash ancestry. This branch's source
remains byte-identical to verified47fc across crates/apps/content/manifests/scripts/
tests/CI. Next: final main-based evidence-head CI, protected merge, fetched parity and
post-merge checks; continue the remaining Gate4 work.
