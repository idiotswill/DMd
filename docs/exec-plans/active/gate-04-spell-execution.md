# Gate 4 — Source-derived spell casting and effect programs

Status: **Source primitives reviewed and focused verification passed; integration remains active.**
Branch: `codex/gate4-spell-execution`, initial base `05afd1239d165813837fbdcdf76cfc2980092c1a`;
root integrations `1c7dac593565001fbd80f1c5e0a63135ad4b83d9` and
`5114159bdb38eb1f04f720b69628d9f8a61bd11a` merged and creature source
`0f388c0fbabcd771775ef1229c18a7d5887433f6` cherry-picked to obtain compiling dependencies.
Writer: bootstrap_audit; root owns scheduler/application attachment and final verification.

## Objective and scope

Add durable typed spell transactions and bounded source-derived effect plans to the
production rules path. Reuse the one TacticalResolution scheduler, effect lifecycle,
physical equipment, HP reducer and canonical tactical definitions. Public input selects
source grants, targets and legitimate choices; it cannot provide DCs, damage, modifiers,
resource receipts or arbitrary effect patches.

This slice advances product-definition authoritative rules, physical dice, player agency,
hidden-information and exact suspension requirements; Gate 4 tactical encounters and
ADR 026. Source is pinned SRD 5.2.1 pp.16, 105–106, 179, 186–187 and represented spell
pages. Historical kernel/event semantics remain unchanged.

## Acceptance and planned slices

1. Source-derived cast/grant/component/payment contracts and durable cast phases;
   concentration replacement at casting start, source interruption and Ready semantics.
2. Compile supported canonical effect descriptions into bounded typed programs with
   source branches, shared versus independent roll groups and derived result references.
3. Meaningful invalid-input, source-resource, serialization and resume regressions;
   independent review; serialized focused tests and strict lint. Root performs canonical
   verification and application/persistence/desktop integration.

No second scheduler, catalog rewriting, invented NPC levels, implicit player decisions,
ambient random numbers, public state-patch escape hatch or complete-spell/Gate 4 claim.
The eighteen-family audit in `gate-4-effect-lifecycle.md` remains the completion checklist:
casting, ordered effects, identity, triggers, interruptions, Ready, zones, barriers,
proxies, transport, perception, behavior, summons, forms, budgets, protection links,
monster features and legendary/lair execution. None is deferred to Gate 6 merely because
a sample does not cover it. Full catalog population remains Gate 6; long casting/ritual
scheduling and enduring noncombat consequences retain Gate 5 ownership.

## Decisions and dependencies

- Existing TacticalEffects owns groups, overlap, expiries and tickets. New spell work
  attaches to TacticalResolution frames and deterministic source operation identities.
- New concentration replaces old when casting begins even if subsequently countered
  (p.179); Counterspell wastes casting action but does not expend its slot (p.120).
- Ready pays at casting, holds concentration even for a normally nonconcentration spell,
  releases only after an accepted perceptible trigger and ends at the next own start
  (pp.186–187). Releasing must never charge again.
- Invalid target types retain expenditure and player-safe apparent outcome (p.106).
  Unsupported program capabilities are a distinct pre-cast product boundary.
- A shared damage roll applies to simultaneous saving-throw targets (p.16), not every
  occurrence in every spell. Nested consequences retain distinct occurrence identities.
- Creature source grants/limited-use counters belong to the creature reducer; coordinate
  a single payment boundary rather than duplicate its resource state.
- Source-definition/catalog additions belong to environment_audit. New files, exports,
  tests and this plan are this writer's only scope.
- Cast identity uses the original command, caster and a checked `u16` occurrence
  allocated by the shared resolution. The direct-entry convenience function uses zero;
  nested work must call `plan_spell_cast_at` with a distinct central allocation below
  32,768. The same identity derives its concentration group and queued program.
- This first program vocabulary covers the currently pinned sample descriptors. It
  does not execute targets, geometry, reaction offers or program nodes independently.
  Root must bind those to the shared scheduler and retain source creature invocation
  separately from its enclosing prepaid activation. Source component evidence is
  internal and checked against actual item identity, custody and hand access.
- Counterspell defers slot expenditure until the interruption window closes. Reaction
  admission must also account for outstanding same-turn slot commitments, so nested
  casting cannot promise a second slot and fail only after the reaction has resolved.
  Shared frame admission owns this reservation; no duplicate resource pool is added.
- `plan_spell_from_feature` consumes the exact non-Serde plan returned by the source
  creature reducer. It preserves the invocation even when a trusted NPC consequence
  originates in another actor's command, checks the pinned profile and feature, and
  requires root to retain/reproduce the enclosing source activation. A bare retained
  spell plan cannot authenticate that historical payment. Component waivers are
  derived per source feature; the Dragon's material waiver does not waive V or S.
- Independent source-adapter review found that a Multiattack spell substitution must
  not become a Ready action. The adapter rejects Ready under an Attack, bonus-action
  or legendary activation. A genuine later-command Dragon routine-step regression
  preserves both command origins, allows Immediate casting and rejects Ready without
  mutation. Direct planning now accepts Prepared grants only; source creatures must
  enter through the feature adapter so this boundary cannot be bypassed.
- Independent scheduler review found that a just-started concentration spell could
  lose its group and then have no terminal continuation. Typed `Interrupt` now
  requires actual group loss or source inability to act and ends Casting as Interrupted
  with the retained reason and ordinary prepared
  expenditure, no queued program and no deletion of another group's concentration.
  This applies the general slot rule (p.105) and concentration loss (p.179); Counterspell
  retains its explicit exception (p.120). The distinct no-slot failure exception for
  longer casting times is outside this immediate-casting reducer and stays Gate 5.
  Nonconcentration casts can likewise finish after death/incapacitation; Begin, Commit
  and Ready release check the scheduler's current `can_act` fact. Structural restore
  rejects a concentration-loss reason on a plan which never owned a group.

## Validation, risks and next action

Typed plans, source compilation, phase transitions and 24 focused regressions are implemented.
The first focused run could not compile the initial base's unfinished integration fields
and exhaustive matches. Merging the root checkpoint fixed those dependencies without
duplicating their implementation. All 18 tests then passed on the source tree preceding
the six source-feature/interruption follow-up tests. Strict Clippy found one Ready
conditional in this slice (fixed) and four integration lints in kernel validation/turns
(fixed by their owners and merged here). The root's serialized build-slot rule applies.

The exact 24-test code head `100cb336ee4017c223ea722b05ad6801d24761b7` was then
attempted after merging the integration/lint fixes. The compiler failed in `dmd-domain`
before reaching these tests: allocation of 2,359,312 bytes failed, followed by allocation
of 1,572,864 bytes and process status `0xc0000409`. Read-only host measurement after exit
showed only 230,480 KiB of virtual/commit capacity free; no compiler remained. No user
process was terminated and no system pagefile setting changed.

After the owner freed memory, the exact code compiled and 22 of 24 tests passed. The
two new source-creature fixtures were correctly rejected because they omitted the
central encounter participant. The fixture correction adds a participant using the
source-built movement/senses and makes round 3 agree with the single-actor third turn;
no production rule or expected outcome was weakened. On the corrected source tree:

- `cargo test -p dmd-rules tactical_spells --lib`: all **24 passed**.
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets -- -D warnings`: **passed**.
- Formatting and whitespace checks: **passed**.

Full independent source review covered domain contracts, program compilation, creature
composition, phase/resource semantics and all regressions. Its Ready/interruption
findings were fixed; final review of the fixture correction found no remaining blocker.
Canonical workspace verification, CI and real application/save/restore/desktop evidence
remain root integration obligations, not claims made by these focused checks.
Primary risks are premature resource commits, hidden-target leaks, source context drift,
duplicate spending on resume, and a plan-only implementation misrepresented as completed
gameplay. Next: integrate this reviewed source slice into the shared encounter resolver,
attach durable authority with legacy future-field rejection and semantic replay, and
verify the complete application path before any production acceptance. All eighteen
families above remain the Gate 4 completion checklist; this source slice does not close it.
