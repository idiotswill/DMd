# Gate 4 durable turn scheduler

Status: compiling reducer checkpoint; focused behavioral tests and review pending. Branch
`codex/gate4-turn-scheduler`, base `d5ea75e`.

## Objective and ownership

Advance Gate 4 initiative/turns, action economy, owner-relative ongoing effects,
death saves, physical dice and exact suspension with an authoritative start/end-turn
scheduler. First-turn work must finish before actions. Ordering belongs to the current
turn's controller, while raw rolls and optional failure belong to each affected roller.
No arbitrary lifecycle operation or vitality state patch is a public command.

Own domain tactical flow/new resolution contracts, rules tactical dispatcher/initiative/
validation/new turn and continuation modules, budget helpers, focused tests and this plan.
Root owns table/application presentation, inventory/AC, combined restore glue and production
integration. Coordinate narrow PendingPurpose/recovery/core matches explicitly; do not
overwrite other RulesState additions. Source creature schedules are a separate dependency.

Relevant contracts: AGENTS; Gate 4 checkpoint; product-definition player agency, complete
tactical timing (lines 477–489), physical dice and exact save/resume (674–676); ADRs
009–012, 018–025 and 026. Pinned SRD 5.2.1 supplies initiative/round time pp.13–14,
death/stability pp.17–18, Attack p.177, Dash/Disengage/Dodge p.180, Prone p.186,
saves/simultaneous ordering p.187. Confirm exact source anchors during implementation.

## Boundaries and acceptance

- Retain RulesState as single timing, action/reaction, HP and raw roll authority.
- On initial active actor and every later Start/End boundary, queue source effects/death
  saves before advancing. Preserve original command, typed occurrence and exact work.
- Current turn controller selects simultaneous order; a target's controller supplies its
  save/inspiration/voluntary-failure decision. Never infer one PC's choice from another.
- Derive continuation IDs from original command, typed role, subject and occurrence;
  replay never calls RNG. Store raw dice in existing pending/history records.
- EndTurn validates active actor authority, finishes End work, resets one turn's budgets,
  advances six world seconds only on a wrapped round, then processes next Start work.
- Implement source-legal Dash, Disengage, Dodge, standing and Attack-action grant. Full
  attacks, movement interrupts, spells, NPC policy and desktop acceptance remain separate.
- Serialize/replay every pause; reject stale/foreign/repeated input unchanged. Preserve
  existing initiative application fixtures and legacy event semantics.

## Proposed durable model

TacticalFlow owns optional TacticalResolution and actor-relative Dodge records; budget
retains current-turn Disengage and a typed Attack opportunity. Resolution retains boundary
origin/actor/number, remaining simultaneous work, selected work and followups. A typed roll
key includes original CommandId, role, subject and occurrence, deriving a UUIDv5 request.
New PendingPurpose references this key; the kernel delegates reconstruction to tactical
validation and refuses legacy submission. Optional per-actor TacticalRecovery lives beside
existing rules data, contains causes only, and never copies HP or death counters.

The source vitality reducer is independently reviewed before integration. Conditions,
concentration, dropped items, rest interruption, stable-recovery rolls and damage-triggered
work must commit with the corresponding HP change. A partial followup cannot be discarded.
No direct lifecycle/vitality API is exposed through TacticalAction.

## Work and verification

1. Agree domain/core interfaces; independently review committed damage/recovery dependency.
2. Implement first/start/end boundary work, deterministic raw requests and explicit order.
3. Implement core turn actions and authoritative source vitality/effect followups.
4. Validate restored pending/receipt structure, replay, source cases and hostile input.
5. Independent review, focused tests/Clippy, then root combined application/full CI checks.

All compilation is serialized globally. Current queue is damage completion, root combined
integration check, then this slice when root releases. Formatting alone does not require
the build slot. No implementation or acceptance is yet claimed. Next: dependency review
and concrete field agreement, followed by the first coherent domain checkpoint.

## Compiling checkpoint (2026-09-24)

Integrated and independently reviewed damage commits `0b09536` and `dbe8cf7`;
provenance rejects future same-command occurrences, and voluntary death-save failure
adds one failure without a fictional die. Equipment dependency `b6d887a` preserves
actual cause metadata for involuntary drops. Domain checkpoint `615676a` checked with
`cargo check --locked -p dmd-domain`; UUIDv5 adds only pinned sha1_smol 1.0.1.

The first reducer/core checkpoint implements boundary frames, raw continuation rolls,
automatic/voluntary saves, effect acknowledgement, damage/concentration/stability
followups, six-second rounds, core turn actions, source request reconstruction and
recovery attachment. `cargo check --locked --offline -p dmd-rules --all-targets` passed.
This is compilation evidence only; focused scheduler tests, Legendary Resistance pause
integration, independent review and combined application verification remain pending.
The current underwater defense query requires the creature volume to be enclosed in an
authored water volume. Unconscious drops held ItemIds including a held shield (SRD191);
the voluntary Utilize don/doff cost (p92) does not charge that involuntary consequence.
Custody becomes the encounter scene location while Flow retains exact ground position.

Next: focused turn/death/effect/restart/hostile-input regressions and the source creature
failed-save decision hook, then strict Clippy and independent review. This checkpoint
must not be treated as gate completion or shipped before those checks.

## Focused source/replay verification (2026-09-24)

All 13 `cargo test --locked --offline -p dmd-rules --test tactical_turns` tests pass.
They serialize each accepted event and state, replay each exact before/event pair, and
cover initial death-save suspension, foreign/stale/forged input, natural 1/20 and
voluntary failure, stable raw d4 recovery, independent ordering/rolling controllers,
automatic condition saves, damage/Inspiration/concentration, six-second rounds,
selected-speed Dash, Dodge, Attack opportunities, Incapacitated/Stunned standing,
clock overflow, physical weapon/shield drops, and legacy Unconscious activation.
The first run exposed and fixed the CharacterStatus death synchronization gap.

End and next Start can execute under one accepted command; carry occurrence allocation
across that boundary to avoid deterministic request identity reuse. Damage observations
retain the actual effect source. Becoming Unconscious through effect consequences drops
held equipment too, including when removing an overlap reveals that condition.

All 31 `cargo test --locked --offline -p dmd-rules --lib tactical_damage` tests pass.
The additional source-success operation increments a death save success without
inventing a face or natural-20 healing; the source reviewer confirmed this bounded
change. Legendary Resistance must invoke it before applying the original failed save.
Domain failed-save checkpoint `ad72861` is present but source LR dispatch remains the
next dependency; the ordinary turn path does not claim that unfinished feature.
Strict Clippy, independent complete scheduler review, combined application tests and
final head verification remain outstanding. Root aggregate `90c3c23` was merged only
for integration dependencies; root must cherry-pick this slice's followups, not its
verification-only merge commit `12b601f`.
