# Gate 4 - Primary attack execution

Status: Active; implementation and production integration pending.

## Objective and ownership

Branch `codex/gate4-primary-attacks`, worktree `gate4-attacks`, based on root
`5e410c6` plus reviewed turn bridge `7f6db28` (local cherry-pick `0fa40d4`).
Connect canonical physical-weapon and source-creature attacks to the one authoritative
TacticalResolution, raw attack/damage dice, vitality, and concentration continuations.
This slice owns new domain/rules attack modules, tactical dispatch/work integration and
focused tests. Root owns application/table/UI, NPC equipment construction and live AC.
No source NPC equipment WIP from the root checkout is included in this branch.

## Contracts and scope

Product clauses: source-faithful rules authority, physical player dice, player/actor
identity separation, durable invalid-input atomicity, and complete production encounters.
Gate 4 tactical encounters and ADR025/026 govern spatial/state and continuation boundaries.
Pinned SRD5.2.1 pp15-18,89-90,176-191,255-257 and represented stat-block source pages.

- Accept only typed actor choices, physical ItemIds, source feature IDs and raw faces.
  Source definitions, geometry, effects, equipment and grants derive all numeric rules.
- Reuse existing weapon preparation and accepted weapon history for chosen ability/grip,
  Loading, Light/Nick/Cleave, ammunition and equip-before/after behavior.
- Retain one typed attack intent/resource receipt with exact original command provenance;
  restored pending requests must be reconstructed from canonical source and retained
  inputs, never arbitrary after-state images or player-authored modifiers.
- All attack, damage, knockout/mastery decisions and concentration follow-ups use the
  existing TacticalResolution. Creating action work must not re-observe a turn boundary.
  After-End source actions nest in the existing continuation and preserve its remaining
  source windows; they must not advance the turn before the attack finishes.
- Source feature activation pays once. Ordinary loot weapons use ordinary equipment
  proficiency; stat-block attacks use their explicit bonuses/formulas and source riders.
- No network, random generation, direct persistence, or player-control automation here.

## Acceptance and verification

1. Complete accepted ordinary primary attacks through raw hit/miss/critical and damage,
   fixed damage, temporary HP/death, controller-owned knockout and concentration work.
2. Preserve physical identities, quantity/custody, equip timing and accepted history;
   no duplicate ammo spending or action/refund on continuation, replay or restart.
3. Derived range/cover/perception/conditions, natural1/20, underwater automatic misses,
   and explicit source attack bonuses remain authoritative.
4. Unsupported source riders or mastery consequences reject before action/attack/ammo
   costs. An accepted attack never silently discards a mandatory or chosen consequence.
5. Focused tests serialize/replay each accepted boundary, corrupt retained requests,
   change issuer/actor/head, and verify unchanged originals after invalid operations.
6. Run focused Rust tests and strict Clippy in the globally serialized build slot.
   Root owns canonical full verification and real application/native acceptance.

## Planned slices and remaining Gate 4 work

- Define retained attack work and request reconstruction; connect ordinary weapon path.
- Connect source feature receipt path with canonical damage and supported hit clauses.
- Add explicit post-hit decisions and reuse existing damage/concentration source work.
- Validate authority, source/resource provenance and each restored stage; regression tests.
- Independent review, focused verification and coherent checkpoint for root integration.

Sap/Vex/Slow require typed non-condition effect modifiers, not magic labels or a second
TacticalFlow effect authority. Unsupported modifiers, Charge movement predicates,
reaction triggers/interrupts, improvised/environmental attacks and broader source feature
programs remain active Gate4 obligations until connected. They are not moved to Gate6.
The first closed pipeline will explicitly reject cases it cannot finish before spending
resources. This source slice alone does not claim an end-user feature or completed gate.

## Evidence and next action

Existing weapon/equipment/history/mastery, spatial cover/perception, tactical conditions,
turn bridge, vitality and grouped concentration APIs inspected. Turn bridge exact-head
read-only signoff complete; author reports20turn+14creature+31damage tests and strictClippy.
The first ordinary-weapon draft now retains one source-reconstructed attack in the shared
resolution. Action/attack/bonus budgets and ammo are reserved once; after-attack equipment
and history settle atomically with damage before concentration/effect children execute.
That ordering prevents later unconsciousness from being followed by an accidental re-equip.
Thrown weapons retain their physical ItemId and move to the target's recorded position
in the encounter scene. This is a coarse landing convention under the encounter's
explicit geometry adjudication, not an SRD rule prescribing an exact landing square.
Precise trajectories, interception and hazardous landing surfaces require the later
geometry/adjudication continuation; no item destruction or recovery is inferred here.
Knockout and Graze pause before consequences and retain controller-owned decisions.
All persisted stages rederive range/cover/perception/hit facts; a review-found malformed
Graze anchor could otherwise reinterpret a successful attack as a miss, now covered by
a dedicated corruption regression. Every test helper serializes and replays each accepted
command, then runs domain, rules and tactical validation.

The first checkpoint supports ordinary weapons, Light/Nick and Graze. Other masteries,
source creature attack features, guessed-location attacks, partial-submersion adjudication,
mounted weapons, Savage Attacker choice and attack-trigger reactions remain explicit next
Gate4 work. They are not represented as complete by this checkpoint. Unsupported derived
masteries reject before any expenditure; no second effect authority is introduced.

The first integration run exposed a real knockout/rest conflict: a source-mandated Short
Rest can begin during combat, whereas the legacy kernel prohibited all such rests. The
new optional `TacticalRecovery.knockout_rest` authorization retains the original knockout
and actual rest-start command plus its start instant. It must match the existing sole
`RestProgress`; it is not another resource or rest clock. Healing/first aid can remove
Unconscious without cancelling that rest. Damage, initiative, or accepted strenuous actions
clear the authorization and rest; waking, rejected proposals and EndTurn do not. Finishing
early rejects even after waking. Legacy rests without this source evidence keep their prior
restrictions. Application restore audit must authenticate both nested command origins.

Focused verification: 19 attack scenarios and 20 existing turn scenarios pass, including
raw/replay/corruption boundaries and a woken rest interrupted only by accepted activity.
All 33 pure vitality tests pass, including new recovery provenance, restart, waking and
early-completion cases. The two incorrect initial expectations were corrected against source:
fixed Blowgun damage remains 1 even on a critical hit, and an adjacent thrown attack under
Disadvantage supplies two d20 faces. The rest-validation defect was fixed in production code.
Strict domain/rules all-target Clippy, formatting and diff whitespace checks pass.
Independent movement/turn review closed the forged Graze anchor, automatic-miss pause,
rest activity and awake early-completion findings; final committed-head review follows.
No full-workspace,
SQLite, desktop or gate-completion claim is made for this isolated checkpoint.

Next: obtain exact source review, then hand the coherent ordinary attack
checkpoint to root. Root's independent Box<TacticalResolution> change needs Box::new at
the new attack allocation. Movement then owns shared dispatcher/pump/work additions while
this slice owns new source/OA adapters under attacks and tactical_attacks. Consume verified
NPC equipment/live-AC and movement contracts before compiling those next adapters; no
competing queues, fake item identities or discarded mandatory source riders are permitted.

## Target-knowledge follow-up

Root's integration review found a privacy ordering defect after the ordinary checkpoint:
weapon range planning ran before the unlocated-target rejection. A retained hidden entity
ID could therefore distinguish in-range and out-of-range positions by rejection text.
The isolated fix checks current actor knowledge before source/geometry planning and gives
one non-disclosing error for missing and unlocated targets. A regression submits the same
hidden ID near and far, then an absent ID, and requires identical errors without action or
ammo changes. Formatting/diff checks pass; this follow-up's test and Clippy are pending in
root's queued integration batch, not represented by the prior 19-test evidence. New source,
opportunity and spell admission must preserve the same knowledge-before-geometry boundary.
