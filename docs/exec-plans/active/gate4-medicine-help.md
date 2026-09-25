# Gate 4 — Tactical first aid

Writer: root. Branch: `codex/gate4-medicine-help`.
Base: verified Second Wind head c04e3b6; its main squash is b86e222.
Status: PR39 merged with verified full-tree parity; post-merge checks running.

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

## Implementation and review

FirstAid uses target/purpose, Action payment, source skill/conditions, raw role17 and
the existing continuation queue. Pending validation reconstructs physical access,
target eligibility and spent Action. An internal MedicineOutcome carries the resolved
success under the explicit table natural-extremes policy; the raw die/total and old
Medicine-total primitive retain their semantics. Clients cannot submit an outcome.

Contact uses occupied-space distance within five feet and an unobstructed center path
against movement-blocking obstacles. This conservative table ruling is explicitly
explained in the UI and is not attributed to a source numeric range. Ground must be
settled; payment interrupts rest. Current actor contacts exclude remembered contacts.
No historical projection DTO/digest changes. Ending knockout removes only its own
source-owned unconsciousness, leaves unrelated conditions and never heals.

Four reducer tests cover actual Action payment, authority, contact/opaque barriers,
Poisoned disadvantage, Inspiration, outcomes and independent unconsciousness. The
file-SQLite test creates two PCs and a source Goblin Warrior through ordinary table
flows, resolves its real critical scimitar attack and exercises both zero-HP and
knockout care. Medicine and recovery dice cross cold reopening, opaque ownership,
accepted retry and independent restored continuation. A coherent invented Bonus
Action cost passes structural validation but semantic restore rejects with no rows.
The desktop regression exercises actual controller/target/pending behavior.

Root performed a separate complete 20-file diff review; supporting agents remain
quota-blocked. Reviewed integration b65d163 imports fetched Savage main cd8d4a4,
retaining both adjacent Rust/TypeScript action variants. No confirmed open defect.

## Verification evidence

Reviewed source: cea43b3bd9b83ddae45a24bd2ffb261252096574 (2026-09-25).

- Local canonical `./scripts/verify`: 654 GNU Rust tests, strict all-target Clippy,
  workspace checking/formatting, genericity and architecture guards all pass.
- Local desktop: zero static errors/warnings, 60 tests and 135-module build pass.
- All six exact-source checks pass: Linux run36155900135 and Windows36155900122.
  Actual logs confirm 655 Linux and657 native Windows Rust tests; native desktop
  has60 tests, zero errors/warnings and135-module build. Both declared MSRV jobs pass.
- Native stable produced a fresh offline installer, artifact10875241131
  (231256117bytes). This proves packaging, not an interactive human playtest.
- Savage PR37 post-merge runs36155615130/36155615628 pass all six checks. The current
  fetched main remains cd8d4a4432c9e83d1c0c8ce65b79591bec425c8a.

## Risks and exact next action

PR39 merged as487239c3f14685ba0c2a998574080aedb8b57a9e on2026-09-25 using expected
heada0b4b570f60fc8fdf6d6f136dbde2ff6870ebd17. All six final-head checks pass:
Linux36162218754 has655 Rust tests; Windows36162218749 has657 native Rust tests,
60 desktop tests, zero static errors/warnings,135-module build and installer
artifact10877631252 (231249974bytes). Both MSRV jobs pass. Fetched full-tree equality
was verified, as was production-code parity to locally verified cea43. Post-merge
runs36165482851/36165482845 are running. Complete their verification and retain the
remaining Help/Utilize/finish obligations below. The earlier merge instructions below
record the prior evidence stage; no second merge is required.

Changes after cea43 are evidence documents and strengthened test setup; production
code parity to cea43 must be verified. Await all six checks on that final head, refresh PR/main, inspect its full
diff and merge PR39 with expected-head protection. Then fetch main, check full-tree
parity and post-merge checks; record the resulting hashes in the next Gate4 checkpoint.
Only one heavy local Rust/frontend job may run at once.

PR38 must import this after the verified merge and initialize current execution work
ancestry without reinterpreting legacy accepted history. General Help, Utilize,
contact adjudication, post-combat finish and every other unresolved Gate4 family
remain required. This slice does not close Gate4, narrow the twelve-family ledger
or eighteen-spell-mechanism matrix, or authorize Gate5.

The subsequent cross-slice review found the barrier fixture overlapped occupied space
and the incapacitated-helper fixture lacked required Prone state. Both now begin from
valid aggregates, asserted before rejecting first aid. The barrier occupies an empty
gap within the five-foot contact bound and actually exercises contact rejection;
no production validator was weakened. All four strengthened Medicine reducer tests
pass locally. Final-head CI must include these test corrections before merge.
