# Gate 4 — Creature source identity and feature execution

Status: Focused implementation verified; exact-head source review and production integration pending.

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

Chimera (p273) provides a bounded additional source fixture: mixed Ram/Bite/Claw,
optional available Fire Breath replacing Claw, extra Bite damage with Advantage and
the source recharge/save area. Existing Multiattack definitions keep their semantics;
new source-defined slots/limits express mixtures and substitutions without accepting
player-authored mechanical definitions. Selected Adult Red Dragon features (318–319)
exercise a maximum-one Scorching Ray substitution and individual Legendary Action
limits. Command116, ScorchingRay159 and Fireball131 have typed source clauses; Detect Magic and
in-lair XP remain explicit adult-dragon omissions. Source component waivers are
recorded exactly (dragon Material only; no blanket NPC exemption). Full creature catalog remains Gate 6; reusable
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

Phase 2 implements source selection/routine validation, source budget/recharge
transitions, strict runtime validation and knowledge-only NPC proposals. Final focused
verification passed **29 tests**, zero failures/ignored: 14 scheduler/policy, 5 source
profiles, 9 definitions/manifest and 1 Legendary Resistance unit test. Commands:

- `cargo test -p dmd-rules --test tactical_creature_schedule --test tactical_creature_profiles --test tactical_definitions --locked --offline`
- `cargo test -p dmd-rules --lib tactical_creatures --locked --offline`
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets --locked --offline -- -D warnings`

Strict Clippy and owned formatting/diff checks passed. The first attempt detected a
stale shared-target domain artifact from a different worktree; only own source-file
timestamps were refreshed to force the correct rebuild. The actual source tests then
identified an undeclared Fireball dependency in the new dragon spellcasting, which was
fixed by adding the verified SRD131 definition. Final evidence uses the rebuilt current
source. Root owns canonical full verification on the integrated reviewed head.

The source reviewer checked Chimera, Command, Scorching Ray and the dragon clauses
against SRD273/116/159/318–319 and reviewed source scheduling/proof/policy boundaries
without finding a blocker. Exact final-head review, including the added Fireball
dependency, is requested after this commit; it is not claimed from the earlier pass.

The source reducer is internal. Each `BeginFeature` returns exactly one enclosing
activation cost; root applies it atomically with the source state and opens the primitive
continuation. `TakeStep` spends a selected replacement only when that step starts and
returns the original enclosing activation receipt and the current primitive invocation
command separately. SRD257 Multiattack belongs to the Attack action; only a new
activation grants that action, never each nested step. A nested spell must not charge a
second action or innate use, including a source Legendary Action that permits casting.
Counterspell's explicit spell-slot exception (120) does not refund an already-used
innate feature; source per-rest use is paid on accepted initiation. Spell execution and
slot spending remain the spell transaction's responsibility.

Recharge hooks are observed for each participating source creature at every central
start/end boundary, before ordinary actions; exact order among simultaneous work is
chosen by the controlling player. The crate-private accepted-turn hook preserves the
original enclosing command issuer/actor (including Player A observing creature B);
public user-authored observations remain privileged. Declining a Legendary Action
closes only that other-turn window and spends no use. Caller-generated request IDs must also be unique in
the application's complete accepted history. The reducer checks all retained local
requests, but does not reconstruct discarded journal history. `FinishRest` requires
root's already-validated completed-rest transaction; it does not establish rest duration
or interruption facts itself. Context/controller changes are privileged and cannot
replace a pending routine or recharge request.

Legendary Resistance consumes a sealed rules-only proof of the exact still-paused
failed save, including an authenticated voluntary failure. The turn adapter must
validate that live occurrence and construct the proof; external callers cannot
construct or deserialize it. Source caps/lair context and duplicate occurrence use
are checked again here. It is not an action and is not prohibited solely by
Incapacitated. Applying the changed save and clearing its slot belong to the same
root transaction.

NPC capabilities are projected for autonomous source creatures only; host/player
control and PC records are rejected. Policy has no campaign/encounter input, accepts
bounded actor knowledge, and produces suggestions only. It never uses remembered
contacts as current attack targets, hidden cells as routes, opponent sheets, dice or
ambient randomness. Source reach is a conservative proposal filter, not geometry
authority. Movement, reactions, targets, spell decisions and final action validity
are rechecked by the root resolver.

Remaining: obtain exact final-head source/reducer review and supply the committed
checkpoint to root. Root must complete source initialization, single-cost
application transactions, real source gear/custody and required material-component
provisioning (SRD257), failed-save/recharge continuation, projected NPC decisions,
authenticated replay and native encounter acceptance. Counterspell/Acid Arrow source
extensions are coordinated separately with the spell writer; none of these source
definitions claims complete spell execution, catalog completion or Gate 4 acceptance.

Later changes to an already accepted creature definition require explicit content/profile
migration or retention of its prior pinned definition for replay. Adding capabilities
must not silently replace an accepted source fingerprint or reset its use counters.
