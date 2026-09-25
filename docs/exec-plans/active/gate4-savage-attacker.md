# Gate 4 — Tactical Savage Attacker

Writer: root. Branch: `codex/gate4-savage-attacker`.
Base: Second Wind/protocol integration841e660 (draft PR36/PR35), with area main172a15a.
Status: planned; no implementation or executable evidence yet.

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
