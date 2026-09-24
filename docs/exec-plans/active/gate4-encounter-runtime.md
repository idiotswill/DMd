# Gate 4 encounter resolver and durable continuation

Status: **Active — initiative verified; turn execution and integration pending.**
Branch: `codex/gate4-encounter-execution`, based on merged foundation main `580f487`.
Writer: root. The earlier `codex/gate4-encounter-runtime` branch retains development
history; the execution branch carries its exact final tree onto the merged foundation.

## Objective, scope and non-goals

Implement the pure, source-faithful encounter scheduler on the existing rules authority:
initiative, action/movement/feature budgets, interrupts, physical dice, reactions, Ready,
simultaneous effects, conditions, damage/death, weapon/unarmed/mount/spell/monster execution
and actor-knowledge-constrained proposals. This is a production dependency of all twelve
Gate 4 ledger families, not a substitute for subsequent desktop acceptance.

Use product-definition tactical philosophy, complete timing, player agency, hidden
information, physical dice and exact suspension clauses; Gate 4 checkpoint; ADRs 009–012,
018–025; pinned SRD 5.2.1. Full catalog population, progression/noncombat, autonomous broad
language and off-encounter living world retain Gates 6/5/9/7 respectively.

## Required design and acceptance

- One authority for initiative/turn/action/reaction state; no UI-held continuation or
  independently drifting HP, effects, concentration or budget copies.
- Every material reaction/order/target/save decision belongs to its legitimate controller.
  Simultaneous effects use the current turn's controller; tied PCs choose their own order.
- Persist typed remaining work before waiting. Damage, concentration saves, effects and
  movement can involve different actors and must survive quit/restart and exact replay.
- Derive attack legality from spatial truth and source capabilities. A player supplies intent
  and raw dice, never DC, cover, modifier, success or damage authority.
- OA resolves before the mover leaves reach; forced displacement and teleport do not provoke.
  Ready follows its completed perceptible trigger; readied spells spend resources on readying
  and require concentration until released or lost. No involuntary choice on a PC's behalf.
- Apply shared area damage once, resolve target saves in required/controller-chosen order,
  retain one concentration group with dependent effects, and support ongoing/simultaneous
  start/end/entry effects without duplicate triggering.
- Preserve historical rules/table event semantics. Tactical events and interpretations have
  explicit versioned dispatch. Record chosen NPC proposals/raw digital faces; replay no RNG.
- Compare state unchanged on invalid/stale/foreign input and serialize/reopen every pending
  stage. Test real application SQLite/replay/export paths after pure mechanics stabilize.

## Planned work

1. Receive integrated foundation types and source definitions; write typed continuation,
   action and outcome contracts with strict structural/state linkage validation.
2. Implement encounter setup/initiative/ties/turn transitions and budget accounting.
3. Implement legal paths and reaction/Ready scheduling, combat actions and weapon/unarmed/
   grapple/mount interactions, then grouped spells/features/effects and death workflows.
4. Add deterministic knowledge-limited NPC policy proposals and generic environmental
   rulings without unsupported world facts or hard-coded scene assumptions.
5. Review full diff; focused mechanics/replay tests, full verify, exact-head CI/merge. Root
   integration then links this through the real table/desktop and completes Gate 4 acceptance.

## Current evidence, risks and next action

Source read confirms revised fifth-edition surprise, underwater weapon behavior and
knockout differ from older rules; use the pinned source. Existing single pending roll,
concentration replacement and global slot boolean need deliberate orchestration. No
complete runtime or gate acceptance is claimed yet.

Implemented so far: condition interactions and turn-budget helpers (all 14 focused tests
passed, including selected-speed Dash regressions). The first versioned
`tactical.action_resolved` path establishes an
encounter and persists initiative groups, surprise, raw dice, Heroic Inspiration replacement
faces, player-only tie proposals/approvals and explicit source creature initiative values.
It shares RulesState rolls/timing and the existing atomic application commit/replay/restore
path. Raw legacy commands cannot bypass a live tactical flow. Encounter HP, attacks, movement
interruptions and the real table/desktop path are not yet implemented by this slice.

Independent early review identified rest interruption, visible Frightened disadvantage,
Inspiration rerolls and outstanding Human transfer choices. Fixes preserve nonparticipant
rests and old replay semantics. New application tests exercise restart after each player's
physical roll, exact semantic export/restore, hostile/duplicate submissions, tie agreement,
rerolls and fear/rest interaction. All five application tests pass, including identical
monster groups using explicit source initiative and host-decided mixed ties. These use
actual SQLite and semantic export/restore. This is focused evidence, not final verification.

Foundation PR #23 merged as `580f487` after exact-head review, 249 local Rust tests and all
six CI jobs, including fresh Windows packaging. Its spatial fixes, catalog pinning and
SQLx Send-future correction are integrated. Parallel effects, weapon and inventory slices
remain in their isolated worktrees until reviewed integration.

Exact next action: integrate the reviewed effect lifecycle and persistent equipment
contracts, with strict save compatibility guards. Complete turn/death/interrupt actions,
weapons and table/desktop without claiming the gate accepted early.

### Effect integration checkpoint

The reviewed lifecycle source `5942d326` is integrated with optional RulesState attachment,
group concentration pointers, unified ephemeral conditions (including charmer/grappler/fear
sources), immediate incapacitation cleanup, and retained effect-origin audit checks.
Schema 1/2/3 reject nested authority before migration relabeling as well as in the codec;
the original SQL migrations remain untouched. A typed narrow preflight preserves existing
late-migration atomicity coverage. New tactical authority must replay from a pre-tactical
anchor instead of trusting source-effect payloads in an initial snapshot.

Focused integration run: 19 effect tests, 13 schema compatibility tests and six tactical
application tests passed. This includes restart/replay and forged-anchor rejection before
writes. Domain-only weapon contract `87b60fd` and inventory contract `4b30722` are present;
their reducers and current-equipment validation are not yet integrated. Independent review
of the new root adapter and canonical full verification remain pending.

Next: integrate reviewed weapon/inventory/damage reducers, then turn-boundary scheduling
with source-derived durable rolls/choices. Retain the original command and typed occurrence
when deriving deterministic continuation identities; never generate dice or IDs on replay.

Integration checkpoint: inventory reducer `c089e89` (11 tests/Clippy, independently reviewed)
and weapon reducer `699cb0f` (17 weapon + 14 creation tests/Clippy, reviewed fixes for
Exhaustion/current armor) are copied into this branch. Inventory now has an optional
RulesState attachment, structural/source validation and codec/database/audit guards.
The combined tree has not yet been compiled; its application provisioning/current-AC
adapter and turn resolution remain active work. This checkpoint is not merge evidence.
