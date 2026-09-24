# Gate 4 — Typed effect lifecycle

Status: **Focused implementation verified; exact-main review/full verification pending.**

## Objective and branch

Implement reusable, deterministic effect identity, concentration grouping, overlap,
expiry and trigger scheduling on `codex/gate4-effects-verified`, based on merged
foundation main `580f487`. The original `codex/gate4-effect-lifecycle` source branch is
retained. Root is the sole writer of this verification branch.
The root encounter resolver will authorize source-defined actions, call this reducer,
persist its state in `RulesState` and pending trigger consequences atomically, and supply application
and desktop acceptance. This slice alone does not complete a spell or Gate 4.

## Scope and constraints

- Own new domain/rules `tactical_effects` modules, their exports and tests.
- Preserve all legacy rules event v1 interpretation, mechanics and persistence.
- Typed records retain source command provenance and stable caller-supplied identifiers.
- Support grouped concentration, target-local ending, overlapping same-source effects,
  owner-relative turn/time expiry, damage and turn triggers, and per-target/per-turn
  zone limits. Suppression retains underlying effects and their duration.
- The internal reducer accepts only facts and definitions already validated by the
  authoritative encounter resolver. It is not a player/provider command endpoint.
- No spell catalog enumeration, arbitrary state patches, new language authority or
  unsupported complete-spell claim. No edits to existing mechanics/engine/spatial files.

Relevant contracts: product definition sections on authoritative timing, explainability,
player agency and exact suspension; Gate 4 checkpoint; ADRs 002, 009–012, 018–020 and
025. Source: pinned SRD 5.2.1 pp105–106, 109, 117–118, 139–141, 162–165, 174 and 179.

## Interface and decisions

`apply_effect_lifecycle(campaign, current, meta, action)` returns a sequence-neutral
`EffectLifecycleTransition { next_effects, ended, triggers }`. Root integrates typed
trigger consequences into its durable continuation. Unknown/malformed state fails
closed before mutation. IDs are supplied by the caller, never generated in resolution.

`TacticalEffects::validate(campaign)` checks retained provenance, group membership,
unique identities, expiry and trigger history. Concentration starts before a spell
resolves and replacement immediately ends the old group (SRD179). Target-local
cleanup does not remove another target; suppressed overlapping castings remain alive.
The reducer stamps `established_at` when installing an effect. Equal-potency overlap
uses that installation order, not the potentially older casting command of a held
spell. Source provenance remains intact; an unstamped persisted effect is invalid.

Existing `ActiveEffect` validation requires a concentration owner's single pointer to
equal each effect ID. Therefore adapter output is an **ephemeral condition query view**
with no legacy concentration owner; separate owner/group bindings remain authoritative.
Do not persist these views in legacy effects or invoke legacy concentration mutations.
Root's new resolver must bridge condition queries and concentration saves explicitly.

Observation queues durable tickets without choosing their simultaneous order (SRD187).
The current turn's controller supplies that choice through root's typed continuation.
`trigger_is_applicable` must be checked before constructing a save/damage roll; selecting
another consequence can change overlap precedence. `ResolveTrigger` consumes the
selected ticket; its save result is already validated by root's normal raw-dice path.
An inactive ticket requires explicit `SkipInactive`, so a suppressed payload cannot be
silently applied. Ordinary distinct damage occurrences retain distinct tickets even
when amounts match; shared once-per-target-per-turn limits are checked at resolution.

`LeaveCombat` is allowed after authoritative initiative and pending consequences end.
It clears the turn cursor/limits while preserving lasting effects and concentration.
Root must account for owner-relative expiry during any noncombat elapsed-time transition;
this reducer does not invent turns or discard every lasting effect on combat exit.

## Acceptance and planned slices

1. Serializable domain types, bounded structural validation and provenance checks.
2. Pure install/end/observe transitions, suppression and condition adapter.
3. Source-derived interaction tests: concentration replacement, individual escape,
   weaker-effect reappearance, relative expiry, positive-damage endings, zone entry and
   turn sharing, exact serialization/resumption, stale/foreign/malformed rejection.
4. Independent review; root integration; serialized focused tests and strict lint.
   Root runs canonical verify-fast/verify and production-path acceptance after integration.

## Remaining Gate 4 generic execution obligations

This finite checklist records the source audit; none moves to Gate 6 merely because
the first catalog sample has ten spells. Individual source entries/catalog grants are
Gate 6; reusable combat execution stays Gate 4. The lifecycle slice addresses only
identity, groups, expiry, suppression and trigger scheduling in the relevant rows.

| Family | Required capability; source examples/pages |
| --- | --- |
| Casting transaction | Components/hands, effective slot level, spending phase, one slot per turn, interruptible casting; 105–106 |
| Ordered effects | Hit/miss/save branches, independent rays, simultaneous damage, actual-damage references, bounded extra dice; Acid Arrow107, Scorching Ray159, Sorcerous Burst163, Vampiric Touch171 |
| Effect identity | Shared concentration, target-local endings, strongest/latest overlap, hidden invalid-target behavior; 106,179 |
| Trigger lifecycle | Start/end/damage/movement/attack/casting/interaction triggers, staged saves, owner-relative expiry; Hypnotic Pattern141, Sleep163 |
| Interruptions | Shield, Constitution-save Counterspell with unspent slot, fall response, retargeting/redirecting defenses, accepted order; 120,130,150,159,161–162 |
| Ready | Pay for and concentrate on held spell, perceivable trigger, reaction release after trigger, expiry; 186–187 |
| Persistent zones | Fixed/attached origin, exclusions, moving-zone contact, once per target per turn; 117–118,164–165 |
| Terrain/barriers | Wall panels, damaging side, opacity/collision, anchoring, section HP and burning; 172–174 |
| Spell proxies | Intangible proxy versus creature, caster-attributed attack, repeated action activation; 118,139–140,165 |
| Transportation | Push versus compelled movement, teleport, temporary removal/reappearance and belongings; 112,124–125,150,190 |
| Perception | Observer-specific illusions, discovery, sensory/light changes, filtered results; 106,151–152,178–191 |
| Forced behavior | Source-linked charm/fear commands and safe routes, controller/target reaction costs; 116,126,130,166 |
| Summons | Source-derived actor, control policy, command costs, initiative relation, defaults and lifetime; 108–109,130–131,166 |
| Forms | Retained/replaced statistics, gear/anatomy, identity and source-owned temporary HP; 153–154,161,170 |
| Budget modifiers | Restricted extra action, action/bonus-action alternatives, attack/reaction limits and ending effects; Haste139, Slow163 |
| Protection links | Damage transfer, derived healing, individual dispel checks, suppression without deletion; 109,124,173 |
| Monster features | Explicit source numbers, mixed multiattack/replacements, reaction triggers, per-feature use/recharge; 255–257 |
| Legendary/lair | After-other-turn opportunity, one use, incapacitation, reset and feature cooldown; lair-dependent budgets; 257,318–319 |

Important edition boundaries: Polymorph uses Temporary HP while retaining real HP;
Counterspell uses a Constitution save. No generic initiative-count-20 lair action is
assumed without an SRD source. Long casting/ritual scheduling and enduring noncombat
world consequences remain Gate 5; active combat effects are not deferred with them.

## Validation, risks and next action

All sixteen focused interaction tests pass. Every successful scenario re-resolves the
same operation from a serialized prior lifecycle state and compares the whole result.
Independent read-only review identified occurrence identity, early queue capacity,
new-combat cursor reset and expiry-ticket target integrity; each now has a regression.
Additional tests cover delayed-effect installation order, per-turn cursor availability
and time expiry discovered during unrelated zone observations.

Verified on this slice's final source tree with workspace-local Rust GNU, one build job:

- `cargo test -p dmd-rules --test tactical_effects --offline`: **16 passed, zero failures**.
- `cargo clippy -p dmd-rules -p dmd-domain --all-targets --offline -- -D warnings`: **passed**.
- Owned-file Rust 2024 formatting and `git diff --check`: **passed**.

Shared-target metadata initially reused another worktree's domain export artifact;
refreshing this branch's domain `lib.rs` timestamp forced the correct rebuild. No source
workaround or shared-cache deletion was used. Canonical full verification and actual
save/restore/desktop acceptance remain root integration work, not claimed by these tests.

Parent owns the global build slot. Risk: a structurally
valid initial snapshot cannot authenticate its own history; application journal/audit
replay remains required. Trigger geometry and source authorization belong to the
encounter resolver, not inferred from an untrusted effect record. Root must validate
definition-derived effect/trigger payloads against pinned content before accepting a
restored initial anchor; this module's structural validation cannot prove that a
well-formed custom damage/DC record was authorized by its named source.

The source reducer is isolated into a six-file PR so it can be reviewed and verified
without bundling every developing tactical subsystem. Its four new source/test/plan
files match the original reviewed source commit `5942d326`; only export placement is
reconciled with merged main. Canonical `./scripts/verify`, current-head independent
review and CI are required before merge; none are claimed from the earlier focused run.

Next: finish exact-head review and full verification, merge this bounded reducer slice,
then refresh the integration branch onto current main. That integration must attach the
optional RulesState field/legacy-input guards, group concentration pointers, condition
queries, authenticated replay and due-ticket continuation. All eighteen effect-family
obligations and packaged encounter acceptance above remain active Gate 4 work.
