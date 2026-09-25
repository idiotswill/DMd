# Gate 4 source melee action and Charge continuation

Writer: environment_audit. Branch: `codex/gate4-primary-attacks`.
Starting integrated checkpoint: `935a207` (reviewed casting `81c1a4e` plus the
separately reviewed departure helper `5435743`). The helper's execution evidence
belongs to the movement author's final combined batch at `4a6f879`, now merged.

## Objective and governing scope

Complete the source Warhorse Hooves/Charge action through the real tactical reducer:
source admission, one action cost, raw attack and damage, conditional Prone, nested
vitality/concentration and deterministic replay of every pending stage. Root AGENTS,
Gate4, the product's source fidelity, authoritative movement, provenance and exact
combat resume requirements govern this slice. Existing source catalog bytes remain
unchanged.

SRD5.2.1 page364 gives Hooves +6, melee reach5ft and 2d4+4 Bludgeoning. If a Large or
smaller creature was approached 20+ feet straight immediately before the hit, add 2d4
Bludgeoning and Prone. There is no saving throw. The implementation must not import
an older-edition Strength save or a bonus attack. The reusable admission may accept
other source Action melee attacks only when every applicable effect already has a
complete supported path; unsupported feature/effect choices reject before cost.

## Ownership and design

- Own `tactical_attacks.rs`, `tactical/attacks/**`, focused attack tests and this plan.
- Movement author approved only the `CreatureAttack` action/dispatch addition in
  `tactical.rs` and removal of the unused annotation from `straight_approach` when
  this source becomes its real consumer. No other shared turn/work/pump changes.
- The public choice names target, canonical feature ID and optional actual weapon
  ItemId. Actor, printed bonuses, dice, reach and conditional effects derive from
  the active actor's immutable creature profile. No client DC/modifier/proof fields.
- Invoke the existing creature feature scheduler for source timing and use limits.
  Spend the central Attack action exactly once. Keep the ordinary source attack in
  the existing TacticalResolution attack slot, raw work and vitality children.
- Snapshot the validated straight-movement origin and endpoints before admission
  clears continuous movement. Retained proof validates actor/campaign/head, endpoint,
  traveled distance and actual approach geometry. The app must also authenticate
  that original movement command through accepted-event replay.
- No placeholder equipment for intrinsic attacks. Required physical implementations
  must be held, intact and in actual actor custody; reject unsupported source use.

## Acceptance and verification

Add public reducer tests with real source construction and actual movement: Charge
after20ft; below20ft; turn/direction/action interruption; too-large target; normal miss;
critical doubles base and extra dice; Prone immunity; source/foreign/stale/unavailable
input unchanged; retained proof/raw request tampering; serialized replay before raw
attack, damage, knockout choice and target concentration as applicable. Verify that
ordinary opportunity Hooves does not acquire own-turn Charge from old movement.

Run focused attack/movement/turn tests and strict domain/rules Clippy only after the
shared compiler handoff. Root owns full canonical checks, application origin audit,
desktop control/projection and production acceptance. New source code is not yet
implemented or tested at this planning checkpoint.

## Remaining Gate4 work

Source mixed Multiattack and legendary actions, ranged source attacks/physical weapon
extras, remaining mastery saves/movement/ongoing modifiers, grapple/shove and broader
spell/zone/summon/transformation paths remain explicit active obligations. This slice
does not accept Gate4 or redefine those obligations. Next action: implement retained
own-turn source admission and Charge proof in the owned attack modules, then obtain
fresh movement/source review before the serialized test batch.

## Implementation and focused verification

The source Action adapter and ten public-reducer scenarios are authored. The
adapter snapshots validated continuous movement, ties its origin to the latest
completed movement receipt, then spends the source/central action once. It derives
all Hooves numbers and Charge requirements from the pinned definition, with no
new raw-roll purpose or competing continuation queue. Critical dice, same-type
resistance rounding, Prone immunity, miss, knockout and concentration use the
existing attack and vitality paths. Fake ItemIds and unsupported source activations
reject before any accepted cost.

The new application audit path is
`attack.admission.CreatureAction.approach.origin`; `attack.origin` retains the actual
source attack command. The original movement event remains necessary to prove
historical geometry; shape checks do not authenticate an invented import anchor.

Movement checkpoint `4a6f879` was merged after its author's 42 attack, 18 movement,
24 turn tests and strict Clippy passed. The first actual Charge batch now passes all
52 attack tests (including ten new source cases), 18 movement tests and 24 turn tests:
94 passed, zero failures or ignored. Every accepted fixture command serializes its
event/state and compares deterministic replay, including the raw-attack, damage,
knockout and concentration pauses. Sixteen retained-source/geometry/cost mutations
and raw-request tampering reject. The critical regression distinguishes correct
sum-before-resistance from separately rounded same-type damage pools.

Executed: `cargo test --locked -p dmd-rules --test tactical_attacks --test
tactical_movement --test tactical_turns`, jobs=1 on the workspace GNU toolchain.
The actual output is retained outside the repository at
`research/gate4-charge-first-tests.txt`. Strict all-target domain/rules Clippy also
passes (`cargo clippy --locked -p dmd-domain -p dmd-rules --all-targets -- -D warnings`),
with output at `research/gate4-charge-clippy.txt`; formatting and diff checks pass.
The movement author independently reviewed exact clean `2a16e37`, including the
pinned SRD page, source admission/proof lifecycle and all ten new test cases, and
reported no remaining bounded source, authority or replay finding. That review did
not rerun the builds; the executed evidence above is the author's verification.
Root owns the application origin audit, production UI/SQLite integration, canonical
verification and gate acceptance. This bounded source slice is ready for integration;
the remaining Gate4 obligations above are unchanged.
