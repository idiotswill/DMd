# Gate 4 — Creature source identity and feature execution

Status: Active; bounded primitives, application integration pending.

## Objective and branch

On `codex/gate4-creature-execution`, base `fa538dc`, preserve source-faithful creature
statistics outside encounter lifetime, resolve reusable source feature budgets/recharge,
and propose bounded NPC behavior using only the acting creature's projected knowledge.
This advances Gate 4 combat, provenance, morale/agency and exact-resume requirements.
Relevant contracts: product definition, Gate 04 checkpoint, ADRs 018, 025 and 026.

## Scope and boundaries

Own new domain/rules tactical_creatures modules, their tests and exports. Parent also
authorized narrow tactical_definitions/tactical.json/source tests/manifest edits for a
real mixed/replacement Multiattack fixture. Do not change existing kernel, mechanical
types, spatial, app or persistence files. Root attaches the optional authority, adapts
legacy validators only when a source profile exists, and supplies versioned journal,
restore, application, dice and production UI integration. No GitHub actions by this agent.

Legacy replay retains its meaning. Creature profiles use level zero as a non-PC sentinel,
not CR or Hit Dice as a fabricated character level. A valid pinned creature profile is
mandatory for this exception; no-profile legacy validation remains unchanged.

## Source and design

Pinned SRD 5.2.1 pp255–257: explicit source attack/save/skill/initiative/PB values, true
Hit Dice, source senses/movement/traits, restricted Spellcasting, Multiattack, recharge,
limited use and Legendary Actions. Represented stat-block clauses remain explicitly
bounded by their source coverage. No imported initiative-20 lair subsystem.

Chimera (p273) provides one bounded additional source fixture: mixed Ram/Bite/Claw,
optional available Fire Breath replacing Claw, extra Bite damage with Advantage and
the source recharge/save area. Existing Multiattack definitions keep their semantics;
new source-defined slots/limits express mixtures and substitutions without accepting
player-authored mechanical definitions. Full creature catalog remains Gate 6; reusable
execution mechanisms remain Gate 4.

Profiles retain exact creation CommandMeta, content/definition identity and fingerprint,
source-selected size/languages, controller classification, and average or recorded raw
HP generation. Mechanics preserve current HP/death/conditions while immutable source
statistics are checked against the profile. Real HD pools permit d4 through d20 and
need not equal a character level. Source attacks/spells never become arbitrary legacy
kernel attack IDs. Source gear/loot materialization remains separate from a stat-block
attack's exceptional flourishes (SRD255).

Feature scheduling is pure and deterministic. Encounter-scoped turn identities prevent
cursor collisions after combat restarts; recharge/limited resources persist between
encounters. Recharge requests retain caller IDs and raw d6 results; only a spent ability
may recharge, at its own start or an accepted completed rest. Legendary actions use one
eligible other-creature end boundary, refresh at own start, and are unavailable while
Incapacitated. Multiattack steps preserve source slot identity and selected legal order;
root owns targets, geometry, actual roll continuations and central action costs.

NPC policy accepts only a projected actor view, source capabilities, its own health and
bounded goals/morale. It cannot inspect campaign/encounter truth, infer unseen current
positions from memory, choose dice, issue accepted actions or control player characters.
Typed proposals include reasons, retreat/surrender/negotiation and source feature choices;
root revalidates any accepted proposal against authoritative state.

## Acceptance and slices

1. Durable source profile/context and validated construction; exact stat/HD tests.
2. Source-backed mixed/replacement routine fixture and closed capability derivation.
3. Per-feature/recharge/legendary scheduler, persisted pending inputs, source timing,
   strict corrupt-state rejection and invalid-operation no-mutation regressions.
4. Knowledge-limited deterministic proposals and tests proving no hidden-state input or
   automatic PC control, including retreat/surrender behavior.
5. Independent review, focused tests/strict Clippy in the global serialized build slot.
   Root performs canonical full verification and application/replay/native acceptance.

## Verification, risks and next action

Phase 1 domain contract and source-profile construction/query adapter implemented.
`cargo test -p dmd-rules --test tactical_creature_profiles --locked --offline` passed
all **5 tests**, covering every represented source profile, explicit modifiers/real HD,
recorded HP faces, immutable source identity with mutable vitality and invalid inputs.
Strict domain/rules all-target Clippy passed; owned formatting/diff checks passed.

The scheduler data structures are a contract at this checkpoint, **not runnable resource
authority**: full source runtime validation, reducers and policy are still pending. Root
may implement the level-zero/source-query adapter against the profile API; attaching
mutable scheduler state must wait for the validated reducer. Build slot released to root.

Next: source routine fixture, scheduler validation/transitions and policy tests, then
independent review and root production integration. This slice is not a claim of complete
monsters, spells or Gate 4.
