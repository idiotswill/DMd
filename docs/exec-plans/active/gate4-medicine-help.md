# Gate 4 — Tactical first aid

Writer: root. Branch: `codex/gate4-medicine-help`.
Base: verified Second Wind head c04e3b6; its main squash is b86e222.
Status: draft implementation and production-path tests authored; executable verification pending.

## Objective and authority

Expose the existing vitality Medicine consequences through an owned tactical Action,
physical Wisdom (Medicine) check, desktop choice and durable continuation. Advance
Gate04's actions/death/knockout and the product's physical dice, agency, source-faithful
rules and exact suspension contracts. Root AGENTS, product definition, Gate04, ADR026
and the source passages were read before implementation.

Pinned SRD5.2.1 p18 permits Help to stabilize a creature at zero HP with a DC10 Wisdom
(Medicine) check. Stabilization leaves it unconscious and requests 1d4 hours until
one HP recovery. The glossary's Knocking Out a Creature entry (p184) allows an Action and
the same DC10 check to end that source-owned unconsciousness without healing.

## Scope, choices and non-goals

- One typed FirstAid action specifies a target and purpose, never a DC, total, grant
  or state patch. The actor is the active, authorized encounter participant.
- Require physical access and a precisely located target. The source gives no numeric
  first-aid range; record the tactical adjudication of adjacent/touch distance with
  an unobstructed physical path. Do not advertise it as an explicit SRD distance.
- Reuse Action payment, conditions/skill modifiers, optional Heroic Inspiration,
  raw roll history, vitality consequences and the one existing continuation queue.
  Reserve deterministic raw role17; Counterspell15 and SecondWind16 stay unchanged.
- Keep actual stabilization distinct from ending source-owned knockout. No HP grant
  on first aid, no removal of unrelated unconsciousness, no automatic high roll.
- Player target choices must use existing actor knowledge; failure responses must
  not reveal unknown targets or hidden condition state through a new query.
- This does not complete all Help actions. Proficiency assistance, attack assistance,
  Healer's Kit/Utilize, improvisation and all remaining combat actions stay Gate4 work.
  No new class/catalog grants, rest subsystem, second resolver or Gate5 work.

## Acceptance and slices

1. Source-derived admission and paid Medicine work, canonical pending reconstruction,
   correct check modifiers/conditions, DC10 outcome and vitality follow-ups.
2. Controller-owned desktop form through current opaque revision/roll transport;
   no parser-only or developer bypass counts as production integration.
3. Reducer coverage for success/failure, boundaries, actual Action expenditure,
   ownership, contact, source eligibility, raw/Inspiration and unrelated conditions.
4. Real file-SQLite/cold reopening at Medicine and recovery dice, accepted retry,
   independent restored continuation and coherent forged state/history rejection.
5. Fresh full-diff review; serial focused/canonical verification and desktop checks,
   tests/build; exact-head Linux/native Windows CI, protected merge, fetched tree
   equality and post-merge checks. No check from another head is final-head evidence.

## Risks and next action

Supporting agents are quota-blocked; root owns implementation and a separate review
pass. Only one heavy local Rust/frontend process may run. Savage Attacker desktop
verification currently holds that slot. New work must later integrate with the
explicit reaction execution version and work ancestry without changing old replay.

Next: implement source admission and retained work using the existing Second Wind,
landing-check and vitality patterns; then actual UI and recovery tests. Gate4 remains
active and its full twelve-family ledger/eighteen-spell-mechanism matrix is binding.

The draft now uses FirstAid target/purpose, Action payment, source skill/conditions,
role17 and the existing queue. Pending validation reconstructs physical access,
target eligibility and the spent Action. The check uses the table's explicit natural-
extremes policy while keeping the raw die and numeric total unchanged: an internal
MedicineOutcome carries only the already resolved success to vitality consequences.
The earlier Medicine total primitive keeps its original semantics. Neither primitive
is a public table action or a way to submit an authoritative success flag.

The desktop selects only this actor's current contacts (host uses the host map),
excludes remembered contacts and exposes both care purposes. It explains the contact
adjudication and no-healing behavior. It changes no historical projection DTO/digest.
The default contact interpretation checks occupied-space distance within5ft and a
center-to-center physical path against movement-blocking obstacles; this is a
conservative table geometry ruling, not source text. Broader authored contact paths
belong to the remaining Gate4 environment/adjudication work.

Four reducer cases and one desktop case are authored. The genuine file-SQLite case
creates two PCs and a source Goblin Warrior, resolves its critical scimitar attack,
then tests normal zero-HP and explicit knockout consequences in separate campaigns.
The helper's Medicine and patient's recovery dice use opaque requests, cold reopen,
accepted retries and independent restored continuation. A structurally valid invented
Bonus Action expenditure must fail semantic restore without writing rows. These
tests have not run; formatting and whitespace checks pass. Next: draft PR CI, fix
actual failures, separate review and serial canonical/desktop verification.
