# Gate 4 physical attacks, movement and interrupted consequences

Writer: root. Planning reference `ba028fe` on the preserved encounter integration
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

Source casting/program execution and area breath activation remain separately
reviewed followups. Preserve their final wire contracts but reject unsupported
nonempty cursors, work, raw roles and action variants before mutation. Retain source
spells and area branches on the full integration worktree. Do not accept a spell
attack admission merely because the surrounding physical attack is implemented.

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
3. Activate shared queue dispatch/validation/raw history only for these mechanisms.
   Source spell-only branches must be closed consistently across action admission,
   persisted cursor/work validation, attack admission and replay. Keep fixed raw-role
   numbers and original provenance. Use function-bounded edits and inspect the full
   diff; previous extraction showed that repeated branch text is unsafe to replace
   without enclosing-function context.
4. Port production table projections/forms and actual SQLite tests. Preserve PR32's
   visible-purpose roll routing and cold-round tests, adapting only assertions that
   intentionally change when physical capabilities become available. Retain actual
   unsupported spell authority and forged original-anchor rejection regressions.

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

## Current evidence and next action

No extraction or acceptance exists for this proposed PR yet. Reviewed source heads
and individual verification are recorded in the central Gate4 and weapon/movement/
falling/shield plans. PR32 source70b8333 has425 passing Linux Rust tests, strict lint,
MSRV and guards; its Windows packaging/local canonical verification remain pending.
First finish PR32 exact-head verification and merge, then fetch main and create the
fresh bounded branch. If separation requires a source program dependency, record
the concrete dependency and retain complete validation rather than introducing an
unchecked placeholder. Full Ready/reactions, remaining spells, NPC morale/knowledge,
improvisation, encounter finish and packaged integrated acceptance remain Gate4 work.
