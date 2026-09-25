# Gate 4 physical encounters and source casting consequences

Writer: root. Planning reference `456272e` on the preserved encounter integration
branch. Create a fresh `codex/` PR branch from fetched main after the turn-core PR32
merge; this document precedes that extraction. Gate4 remains active.

## Objective and scope

Connect real source weapons, paid shield changes, movement, opportunity attacks,
damage, knockout and falls through the existing table, physical dice and durable
turn scheduler. Advance the product's tactical rules, player agency, perception and
exact suspension requirements, Gate04 and ADR024/026. Preserve the current rules
coverage ledger; this slice is neither complete combat nor Gate4 acceptance.

Attacks and movement share interrupt state: crossing reach must suspend the original
segment, the selected owned reaction must resolve before movement resumes, damage
can break concentration/drop held gear, and lost flight/jump support can cause a fall.
These consequences must travel through the same source-authenticated queue. Treat
them as one coherent physical encounter objective rather than splitting their state
transitions into temporary implementations.

Include the already reviewed Immediate-only source casting/program execution.
The intended real damaging-opportunity-attack/concentration case needs a genuinely
installed ongoing spell; main plus PR32 has no legitimate casting entry and must
not gain an arbitrary effect-install escape hatch. Source Hold Person provides the
real prerequisite, while spell/physical attacks already share retained proof,
queue and vitality validation. Independent architecture review supports this
coherent expansion over stripping those interdependent paths into temporary stubs.

Area breath activation, new Ready/reaction casting and the separate privacy
protocol remain followups. Keep the exact supported source/program subset explicit
and reject unopened actions before costs. Do not claim general spell completeness.

## Planned extraction

1. Port current source attack, physical creature weapon, intrinsic melee/unarmed
   damage and opportunity adapters; source inventory/loadout/ammunition remain real
   identities. Include currently implemented Light/Nick/Graze choices, owned knockout
   choices, paid Don/Doff Shield and source/training AC. Do not claim unimplemented
   mastery/Savage Attacker/grapple/shove/Ready mechanics by association.
2. Port reviewed path/segment and falling reducers, perception fixes, jump clearance,
   supported movement modes and default-stack real landing scenarios. Re-enable
   airborne/liquid admission only with its actual automatic consequence resolver.
   Hidden truth is never a client-side target or route authority.
3. Activate shared queue dispatch/validation/raw history for these mechanisms and
   the reviewed source casting subset. Keep fixed raw-role numbers and original
   provenance; preserve actual cast grants, components, resources, target binding,
   player saves and separate concentration children. Ready and reaction casts stay
   explicitly unavailable before costs until their full source path is integrated.
4. Port production table projections/forms and actual SQLite tests. Preserve PR32's
   visible-purpose roll routing and cold-round tests, adapting only assertions that
   intentionally change when physical capabilities become available. Retain actual
   unsupported casting-mode and forged original-anchor rejection regressions.

## Acceptance and verification

- Demonstrate a real physical attack with raw hit/damage, item/hand/ammunition costs,
  source NPC weapon choices and damage/knockout durability through TableAction.
- Demonstrate actual damaging opportunity interruption, concentration child and
  resumed/rejected movement under source consequences. A miss/decline alone is not
  this evidence. Controller choice and Reaction use remain explicit.
- Demonstrate lost-support falling and each liquid landing option, retained request
  identity, default Windows stack, actual disk reopen, mirror restore continuation,
  same-command retry and no double spending. Invalid or forged input writes nothing.
- Player view excludes private map truth, target statistics, hidden NPC identities
  and raw source authority. The separately active per-audience transport privacy
  work remains required before Gate4 acceptance; do not hide known counter/ordering
  leaks or silently defer them to Gate5.
- Obtain independent full extraction review, run focused source/app/UI checks then
  canonical verification, all six exact-head CI checks, protected merge, fetched
  full-tree parity and post-merge checks. Prior integration source results do not
  count as verification of this new extraction.

The expanded diff is roughly25k lines, substantially test fixtures, from individually
reviewed source slices. Fresh review of the actual combined tree must separately
cover source/casting admission, shared attack/movement/fall order, application/
restore/origin/privacy boundaries and real SQLite/UI recovery. Earlier leaf green
heads never substitute for this combined verification.

## Current evidence and next action

No extraction or acceptance exists for this proposed PR yet. Reviewed source heads
and individual verification are recorded in the central Gate4 and weapon/movement/
falling/shield plans. PR32 source70b8333 has425 passing Linux Rust tests and424
passing local Windows GNU Rust tests under canonical verification, strict lint,
MSRV and guards. Its final Windows workflow now tests all workspace crates. That
expanded check caught only a non-persisted export timestamp comparison; reviewed
correction25be7f6 retains equality of all durable data. All four corrected Linux
jobs pass; the expanded native Windows test/installer run remains pending.
First finish PR32 exact-head verification and merge, then fetch main and create the
fresh bounded branch. The concrete source-program dependency is now recorded above;
port the reviewed Immediate-only implementation and preserve the one scheduler.
Full Ready/reactions, remaining spells, NPC morale/knowledge,
improvisation, encounter finish and packaged integrated acceptance remain Gate4 work.

Reviewed source training correction1b39980 is integrated in456272e. It follows SRD177:
untrained Shield use removes its AC bonus but does not itself forbid casting;
untrained worn armor and occupied component hands still prohibit the affected cast.
All43 source spell tests and strict domain/rules all-target Clippy pass, with
independent exact-head review. The initial genuine Cultist fixture lacked Hold
Person's material; the corrected fixture supplies that real component rather than
weakening admission. The correction changes no app/UI or reaction execution. Include
it in this combined source slice and rerun canonical and real app verification.

The new OA recovery scenario has a separate owned test branch planned from456272e:
PC0 equips a real Dagger through an ordinary attack, a source Cultist genuinely casts
Hold Person, then moves away and provokes PC0. Raw hit/damage must lead to the NPC's
concentration save before the original movement can resume. Test every accepted pause
on reopened disk and independently restored runtime, exact retry and forged restore.
It must preserve the reactor's own Reaction payment and the active mover's budget.
