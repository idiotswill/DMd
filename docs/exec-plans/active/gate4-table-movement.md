# Gate 4 movement and opportunity choices at the table

Writer: root on `codex/gate4-encounter-execution`. Dependency: independently verified
shared movement/source attack continuation, followed by movement privacy and falling
integration. This advances Gate4 movement/reactions and exact save/resume through the
existing table boundary. It does not establish completed Gate4 acceptance.

## Objective and boundaries

Connect actual movement and owned opportunity choices to the table command and durable
retry path. Show only the acting creature's own position/capabilities and currently
perceived target labels. Host map truth is not an NPC planner's knowledge. No new queue,
synthetic weapon, caller-supplied reach/cost/attack bonus or hidden crossing coordinates
may become a permission. Source movement/reaction validation remains in the resolver.

Root AGENTS, product-definition spatial/player-authority/privacy requirements, Gate4 and
proposed ADR026 govern this work. Ordinary typed movement cannot manufacture teleportation
or forced displacement. Natural direction/distance input should become an explicit path
proposal, with unsupported intent kept unresolved rather than guessed.

## Work and acceptance

1. Integrate the coherent movement/OA checkpoint. Collect all retained movement, accepted
   reaction and original crossing CommandMeta records in restore audit validation.
2. Project the owned mover's choices and a selected reactor's actual source melee options.
   Keep the original target fixed to the witnessed crossing; expose no hidden source facts
   to other viewers. Distinguish ordinary attack from spending the offered reaction.
3. Add physical controls with retained typed actions. Raw attack/damage/concentration
   dice continue through the existing roll view; generic continuation labels preserve
   ordering authority without exposing another creature's hidden state.
4. Exercise a real SQLite movement -> opportunity -> physical dice -> concentration ->
   remaining movement sequence, ownership rejection and restart at pauses. Add UI tests
   for viewing-channel changes, spent reaction and unavailable/stale choices.
5. Verify movement privacy correction, falling/stopped-movement receipts and actual
   packaged encounter behavior before final acceptance.

## Current status and known risk

Plan created before table implementation. Root Light/Nick table changes are independently
reviewed and await backend compilation; source movement/OA author is finishing exact-head
verification. A known full-route truth oracle is recorded: current movement admission
preflights all unseen obstacles/occupants. The movement author will separate atomic
invalid-proposal rejection from an accepted travel attempt that commits its legal prefix
and stops at an actually encountered obstacle/exhaustion. No hidden ID is disclosed by
that outcome. Near/far/absent hidden geometry and displaced-mover regressions are required.
This checkpoint is not ready for a movement-complete or gate-complete claim.

Next: import the verified shared checkpoint, adapt audit/continuation projections and
implement the table controls while the casting author owns the next shared-queue slice.

## Integration checkpoint

The independently reviewed source/OA head `f69dee626dcb9fd1407fab99c4212fd21c04cb51`
is integrated by `de48b38`. Its author passed 28 attack, 10 movement, 32 spatial and
20 turn tests plus strict domain/rules all-target Clippy. Root preserved all existing
table changes. The app now collects original movement/progress/segment/decision/crossing
origins, including the crossing retained inside an active source attack; only physical
weapon sources carry an equipment origin. Generic attack roll labels also cover intrinsic
creature and Unarmed attacks without calling them weapons.

Movement methods use current source speeds, Dash budget and the actor's own position
projection. A bounded direction/distance parser expands only explicit route legs into
adjacent typed steps; unsupported combined actions and unavailable modes fail locally
without inventing permissions. Opportunity projection offers the witnessed target, held
physical weapons, source creature features or Strength Unarmed damage to its controller.
Reaction controls cannot borrow ordinary Attack-action equipment changes.

All 30 frontend tests, Svelte checks (zero errors/warnings) and the production build pass.
Logs outside the repository: `tooling/gate4-movement-ui-{check,tests,build}.log`.
Independent review of the movement projection/parser found no source boundary defect.
It identified a component viewer-reset test gap; forms now include host/player/actor in
their identity, tested with the same actor and position across a host-to-player switch.
The actual TableApp already unmounted its panel during that switch. No production leak
was demonstrated. Opportunity projection/control review remains next.

The new SQLite scenario exercises accepted movement, a retained source-gear opportunity,
attack or decline, physical raw dice and continued movement after independent restore.
It verifies unspent Action, conditional Reaction spend and actual reached position/cost.
All application tests and strict workspace Clippy are compiling now; no Rust success is
claimed for these new table changes yet. Falling, privacy-safe accepted-prefix movement
and concentration after a damaging table reaction remain required integration work.

## Application verification and review fixes

The first new movement test failed because its source-created Goblin still had its
Scimitar stowed. The fixture now takes a real ordinary Attack on the Goblin's own turn,
equips that physical Scimitar through the permitted BeforeAttack change, submits a
natural-one miss and returns to the next player turn. Both attack and decline then
pass through the independently restored opportunity without bypassing gameplay setup.

All 59 application tests pass, including 17 table-loop and three recovery cases.
Durable log: `tooling/gate4-movement-app-tests.log`. Independent opportunity review
found a separate projection bug: a held weapon with a free second hand could make a
legal two-handed reaction in the resolver but was hidden by the UI filter. The filter
now permits that grip while rejecting another held item; focused projection regressions
and strict workspace Clippy remain next. No damaging-reaction/concentration or
falling/privacy completion is inferred from the passing miss/decline scenario.

The independent reviewer supplied a projection regression in `table_opportunity_tests`.
It starts from runtime-created characters/starting gear, then uses explicitly isolated
read-only acquired-equipment snapshots. Greatsword and Quarterstaff cover required and
Versatile two-handed grips across six held/free/blocked arrangements. Every option is
cross-checked against the actual reaction weapon planner with no equipment change,
including source damage and unchanged state. This is projection evidence, not journaled
acquisition or a new starting shop. The regression is attached and awaiting compilation.
