# Gate 4 — Reaction and Ready source audit

Status: **Active; narrow Shield-training correction verified; reaction execution remains unimplemented.**
Branch: `codex/gate4-spell-shield-training`; base `ba028fe`.
Writer: rules_architecture. Root owns integration and gate acceptance.

## Objective and boundaries

Correct the source distinction between armor training and Shield training in the
existing spell binding path. Record an implementation-ready audit for Ready,
Shield, and Counterspell using the shared tactical resolution, without implementing
new reaction commands in this correction. This advances the product's authoritative
rules, physical dice, player agency, reaction timing and exact recovery requirements
(`docs/product-definition.md`, especially combat timing and save/resume), Gate 4,
ADR 026 and the active spell/casting plans. It does not change the product or gate
finish line. Full catalogs remain Gate 6 and rituals/long casting remain Gate 5.

Pinned SRD 5.2.1 sources: armor training p.177; physical spell components pp.105–106;
Reaction pp.10/186; Ready pp.186–187; Counterspell p.120; Shield pp.161–162;
concentration p.179; Attack action p.177 and simultaneous effects p.187.

## Acceptance criteria and work

1. Check in this plan before production edits. Inspect the integration at `e64c53f`
   for the audit and use parent-approved `ba028fe` for the isolated correction.
2. Untrained worn armor continues to prohibit casting; an untrained Shield alone
   does not. Current physical hands, speech and material access still decide whether
   the actual spell's components can be supplied. Do not alter armor class here.
3. Add paired spell-binding regressions for real Shield plus free/occupied hand and
   trained/untrained worn armor. Both accepted queries and rejections are immutable;
   serialization retains the same outcome. Use existing executable spells so tests
   do not claim an unavailable reaction spell executor.
4. Record the required retained state, source clauses, ordering, privacy, recovery
   boundaries and real test scenarios for the next Ready/reaction implementation.
5. Independent exact-diff review by environment_audit. Run focused spell tests and
   strict domain/rules Clippy only after the globally serialized compiler handoff.
   Root performs canonical/integrated verification before merge.

## Non-goals and risks

No public Ready/Shield/Counterspell execution, content-grant invention, new queue,
save format, app/UI change or Gate 4 completion claim. A private unsupported spell
program must remain rejected before costs. The main correction risk is accidentally
relaxing worn-armor or physical-hand validation while removing Shield training as
a casting prerequisite.

## Validation and next action

Plan commit `e476b6c` preceded code. The correction retains supported physical worn
item checks and removes only Shield training as a casting prerequisite. Two binder
regressions cover imported prepared casting and a genuine Cultist Fanatic source
activation with trained leather/untrained Shield, each with free/occupied hands;
untrained leather remains rejected. Accepted bindings survive serialization, and
queries/rejections leave state unchanged. These use existing executable spells,
not an invented Shield reaction executor.

Formatting and diff checks pass. Independent environment_audit source/fixture review
is clear against SRD177. The first focused run compiled and passed 42/43 tests; the
new Cultist fixture incorrectly omitted Hold Person's actual material component.
Binding correctly rejected it. The test-only correction supplies the real carried
`spell-material:hold-person` ItemId and checks the occupied-hand material-access
failure. It has separate independent review; no production guard was weakened.

All **43 spell tests** now pass, and strict domain/rules all-target Clippy passes.
Commands: `cargo test --locked --offline -p dmd-rules --lib tactical_spells::tests`
and `cargo clippy --locked --offline -p dmd-domain -p dmd-rules --all-targets -- -D warnings`.
Logs outside the repository: `tooling/logs/gate4-shield-training-tests-20260925.log`
(initial failure), `gate4-shield-training-tests-20260925-r2.log` (43 pass), and
`gate4-shield-training-clippy-20260925.log` (success). The serialized compiler slot
was explicitly released to the area author, then protocol. Root still owns the
combined/canonical and actual application verification before merge; this correction
does not claim new reaction playability. Next: hand the exact reviewed correction to
root and coordinate the verified area base and exclusive scheduler ownership.

Root has authorized the subsequent complete
reaction/Ready execution slice on a fresh branch from reviewed integration, after the
area checkpoint and explicit shared-file handoff. This correction does not change
those shared files; the next slice must continue through actual source/app recovery
tests rather than stop at new schema or helper contracts.

## Ready and reaction audit of integration `e64c53f`

### Reusable code and necessary dependency changes

`tactical_spells.rs` already provides source plans, `begin_cast`, `advance_cast`,
slot expenditure and the correct Countered/Interrupted/Held/Released distinctions.
`reservations.rs` accounts for unresolved slotted casts; `TacticalEffects` supplies
owner-relative expiry and concentration; the shared continuation supplies physical
dice, voluntarily failed saves, Inspiration, Legendary Resistance and nested vitality.
Existing perception, component binding, weapon plans, movement/OA and falling are
the execution authorities. Reuse them instead of accepting trigger/DC/AC flags.

The current `tactical/casting.rs::begin` rejects Ready, then immediately calls commit.
Its validator admits only Committed/Released casts. `binding.rs` supports neither
Caster protection programs nor reaction binding; `retained.rs` requires immediate
entity selections. Shield is pinned content with two protection nodes but has no
executor. Counterspell is absent from the catalog and needs its exact source descriptor,
trigger and save-to-interrupt node plus content fingerprint/resource-pin updates.
No currently supported creature feature grants these two spells. Do not invent a PC
grant or call a prepared-caster fixture a supported character-creation path; the real
source grant/production entry path must be explicit in the subsequent slice.

Root requires closing that gap through the actual admitted table path: a pinned SRD
NPC with genuine Shield/Counterspell features, available host creation and cast controls,
is a valid bounded production entry. Imported prepared-slot fixtures alone are not.
Do not invent class/level/slot grants or expand PC catalog enumeration from Gate 6.
Normal UI/controller assignment for player-controlled source creatures is still absent;
that summons/control interface remains required Gate 4 work, not a fixture-based claim.
Coordinate respondent actor/audience contracts with the player-protocol author before
app integration; the reacting actor need not be the current turn's actor or trigger source.

Three representation constraints must be resolved before composing all Ready actions:

- `resolution.attack` and `.movement` each hold one current cursor. A readied attack
  during another attack/cast, or readied movement during another move, must suspend
  the parent with its work ownership intact. Use bounded typed records referenced by
  the existing frame stack (or a validated stack of suspended cursors), never a second
  resolution or overwritten parent. Cleanup must remove only that occurrence's work.
- Cast work currently looks up a local `u16` occurrence. A held spell created by an
  earlier command can collide with the new resolution's ordinal. Use a composite
  original-command/occurrence reference or a separate checked attachment key, retaining
  the original identity for raw dice/effects and the actual release command as cause.
- Ready survives when the current resolution drains and across other actors' turns.
  Store bounded `TacticalReady` records at flow scope, not in the per-turn budget or an
  eternally pending resolution. Its actual execution enters the one frame stack.

### Proposed typed contracts and ownership

Add domain `tactical_reactions.rs`: a Ready record containing actor, original
`CommandMeta`, declared turn, source action choice, typed trigger, and optional held
cast; a reaction-window reference containing triggering command/occurrence, semantic
milestone, eligible controller, and accepted/declined response. Window references
must bind to live work, not be client-authored evidence. Proposed public choices are
Ready, React and IgnoreTrigger with opaque live decision IDs; no public operation may
declare itself Countered, grant a reaction, or apply an arbitrary interruption.

New rules `tactical/reactions.rs` derives windows and admissible responses; separate
`tactical/ready.rs` handles declaration/expiry/release. Casting and attacks emit typed
milestones and call these modules. Keep original cause, reacting actor, current turn
controller and raw roller separate. All optional responses remain controller choices.
Ready's bounded automatic trigger vocabulary can initially cover witnessed completed
movement steps, attacks and casting. Other natural circumstances require explicit
host adjudication with retained witness/evidence; prose or an omniscient target lookup
cannot authorize them. Bound reaction depth, work and occurrence allocation before
spending, and reject forged/orphan/duplicate window or suspended-parent references.

App work follows the new durable player-projection protocol: actor-safe offer labels,
original input nonce/revision on uncertain delivery, and acceptance-time visibility.
Do not expose hidden reactant counts, spell names, internal frames or global sequences.
Rules restore must collect all declaration/trigger/response/held-cast origins and
replay the exact milestones. Earliest pregame anchors cannot introduce Ready/window
authority. Format compatibility guards must reject silently discarded new authority.

### Source clauses and ordering

- **Common:** reactions normally occur after the trigger (p.10); OA explicitly occurs
  before leaving reach (p.15). One reaction lasts until the actor's next start (p.186).
  Nested work finishes before the parent resumes. Simultaneous effects belong to the
  current turn's controller (p.187); ordering never chooses another PC's response.
  Hidden alternatives need an explicit host ordering/delegation boundary rather than
  leaking their existence through ordering cards.
- **Shield:** offer after a genuine hit is resolved, before damage or hit riders;
  preserve the exact raw attack faces and reevaluate the hit with +5 AC. Natural 20
  still hits. It is Self, V/S, with no sight requirement to the attacker (pp.161–162).
  Also offer for actual Magic Missile targeting before any dart damage. Add a
  source-bound protection effect queried by effective AC and spell damage prevention,
  not a mutation of base/equipment AC. It lasts until the caster's next start even
  when cast on their own turn, needs no concentration, and cannot stack with itself
  (p.106). Existing pre-roll AC reconstruction must account for the authenticated
  later Shield effect without rewriting the original roll or granting free misses.
- **Counterspell:** open a casting window after components/action and new concentration
  begin, before spell effect/ordinary slot commitment. Reactor must see the casting
  creature within 60 feet and the actual cast must have V, S or M after source waivers.
  Being able to hear an unseen caster is insufficient. Derive CON save versus the
  reactor's real spell DC, using the shared save/LR/Inspiration path. Failure cancels
  the exact parent spell, wastes its action/bonus/reaction, but does not expend its
  slot (p.120); source feature/material costs have no general refund clause. This is
  the 2024 saving-throw spell, not automatic level comparison or an ability check.
  Counterspell can itself be countered if another genuine eligible reaction exists.
  Retained reservations must prevent a caster promising two slots on the same turn
  while the parent slot is awaiting that source-specific decision (p.105).
- **Ready declaration:** spend the Ready action on the actor's turn, select a
  perceivable trigger and response, and expire before the next own turn can act
  (pp.186–187). Ignoring an occurrence spends nothing and retains Ready for another
  eligible occurrence; reacting consumes the reaction and that Ready response.
  Readied ordinary actions are not concentration effects.
- **Ready attack:** execute the chosen Attack action through the real planner at
  release, paying the reaction but no second Action. This differs from an OA: an
  Attack action has the p.177 equip/unequip allowance. Separate action identity from
  its payment kind instead of using a Reaction window to erase that permission.
  Own-turn Extra Attack/Light clauses do not become off-turn entitlements; source
  feature restrictions and the actual turn still apply. A ranged readied attack is
  legal when its current weapon/range/components permit it.
- **Ready move:** use a bounded response movement allowance up to current Speed,
  separate from the active creature's normal movement/Dash budget. Derive current
  terrain, source movement modes, cost and actual prefix through the existing spatial
  evaluator; never write the reactor's movement into the active actor's spent total.
  It uses a reaction and therefore can provoke OA (p.15). Recheck after nested damage,
  concentration, displacement or falling before resuming either move. Document the
  selected-speed/special-mode convention consistently with ADR026.
- **Ready spell:** only action-time spells; cast now with components, action, source
  resources and slot reservation/commit. Counterspell is possible at this casting,
  not again merely when held energy is released. Holding even an instantaneous spell
  requires concentration and replaces previous concentration immediately (pp.179/187).
  Release spends only the reaction, binds chosen targets against current geometry,
  and does not recheck components or repay slots. Target-at-release is the existing
  explicit binding convention. Add a retained-plan target-binding API: current
  `bind_spell` insists on the original command's current head and fresh resources.
  Loss, replacement, death/incapacitation or expiry dissipates held magic with no
  refund. On release, a concentration spell keeps the same group and starts its
  actual effect duration; a nonconcentration spell ends only its own held group.

The spell-targeting milestone relative to simultaneous casting interrupts and the
granularity of natural Ready circumstances need a documented typed convention.
Do not imply that SRD prose uniquely specifies every sub-step or substitute an
arbitrary client timestamp for observed trigger order.

### Implementation sequence and acceptance evidence

1. Shared bounded reaction windows, suspension ownership and flow-lived Ready
   identity/expiry, with serialization and malformed-state tests; no disconnected
   prototype dispatcher. Coordinate exclusive shared-file ownership with area work.
2. Add source Counterspell descriptor and Shield protection executor, source grants,
   exact hit/casting windows and nested cancellation/save completion. Keep unsupported
   programs rejected before costs until each complete path is available.
3. Ready attack/move/spell adapters, generic witnessed/adjudicated triggers and actual
   app controls under the durable audience protocol; preserve one queue and budget
   authority. All three responses remain Gate 4 acceptance work.
4. Real SQLite close/reopen at every offer, nested raw save, failed-save choice and
   release; replay mirror, same-nonce retry, stale/foreign controller rejection,
   tampered current and historical records rejected before any restore rows.

Required meaningful regressions include:

- Hit total exactly base AC and base AC+5, natural 20, miss/no offer, unseen attacker,
  Shield already active, own-turn expiry, one reaction, and actual Magic Missile
  multiple-dart prevention without fabricated attack/damage rolls.
- Counterspell 60-foot boundary/cover/sight and component-waived invisible cases;
  raw CON success/failure, voluntary failure, Inspiration/LR, actual slot exception,
  replaced old concentration remaining gone, nested counter-counterspell and exhausted
  same-turn slot reservation. Rejected input cannot consume another actor's response.
- Ready ignore then later trigger, expiry at next start, no hidden-trigger oracle,
  same-turn/off-turn attack distinctions and real physical equip/ammo; a readied move
  that provokes OA, loses flight or is displaced, preserving both completed prefixes
  and unrelated parent work. Unequal actors' speeds and prior Dash catch budget mixing.
- Ready spell initially replaces concentration, survives round/restart without holding
  the turn open, loses a damage save before trigger, releases at changed target range,
  releases despite hands occupied after casting, pays once, and expires with no refund.
  Nested old-held/new-cast ordinal collisions and altered cause/trigger/controller
  receipts must fail deterministically. Paired hidden histories give equal player views.
