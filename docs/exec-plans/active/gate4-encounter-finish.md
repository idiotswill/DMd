# Gate 4 durable encounter conclusion and recovery

Writer: root. Branch `codex/gate4-encounter-finish`, from reviewed PR33 candidate
`8cb3084f8119f1b4d159728d2ea1b60ff5ec4a79`. Reconcile merged main before this PR.
This plan precedes code. Gate4 remains active; no gate acceptance is claimed.

## Objective and authority

Allow the actual table to finish a settled encounter, preserve its consequences,
end/reopen the play session and begin another encounter without recreating source
creatures, healing actors or refreshing limited resources. Advance product-definition
requirements for ordinary play, persistent consequences and exact save/quit/resume;
apply root AGENTS, Gate04, ADR024/026 and the coordinated reaction contract ADR028.

The current app refuses EndSession whenever an encounter flow exists and has no
public action that reaches TacticalPhase::Finished. Pure effect/source LeaveCombat
reducers exist but are not a playable finish path. Legacy EndCombat cannot safely
replace the tactical transition: it bypasses current source, equipment and lifecycle
authority. Implement one authenticated tactical action with a retained host ruling.

## Scope and invariants

- Finish is privileged, explicit and journaled. A conclusion does not invent victory,
  death, surrender, retreat, XP, loot or consent. Retain actual settled consequences
  and source-linked accepted dispositions; NPC/player decisions remain their owners'.
- Reject pending dice, consequences, saves, concentration checks, falling, source
  recharge/routines and reaction windows before mutation. Never delete unresolved
  work to make finish possible. Coordinate held Ready declarations with their writer:
  paid magic/resources are never refunded; a host finish cannot dismiss another
  controller's held spell or take their release/ignore choice.
- Preserve HP, death/knockout/stable recovery, concentration and lasting effects,
  equipment identities/loadouts, spent ammunition, ground item custody/location,
  source counters and witnessed knowledge. Scene/encounter conclusion is historical
  evidence, not a second mutable HP or resource authority.
- Use the existing internal lifecycle/source LeaveCombat operations to release
  combat cursors only after quiescence. Do not treat finish as an owner boundary,
  expiry, rest, recharge, six-second advance or concentration loss. Time/turn-bound
  effects need an explicit retained continuation policy; do not silently erase them
  or strand their source saves by discarding timing.
- EndSession must allow a genuinely finished flow and retain portable replay. New
  setup must preserve the prior conclusion and grounded physical items, while new
  initiative must not revive dead actors or reset daily/rest resources. Ordinary
  later commands must use current tactical authority, not weaken the blanket legacy
  guard. Finished history remains authenticated through the recovery anchor.
- Public Medicine stabilization and Help to end knockout must use real action,
  reach/target/controller authority and physical checks through the existing vitality
  reducer; no arbitrary HP/stable-state setter. These may be a separate coherent
  followup after the finish integration, and remain Gate4 until actually playable.

## Planned slices and evidence

1. Document source/architecture decisions and coordinate Ready quiescence. Define
   retained conclusion/history, outcome and post-combat timing contracts before
   changing shared dispatch. Source noncombat lifecycle is a dependency, not grounds
   to weaken expiry, agency or existing restrictions.
2. Implement the finish transition, exact validation/origin collection, actual table
   action/projection/control and new-encounter admission. Preserve existing source
   counters and all unresolved-work refusal paths.
3. Add real SQLite encounter finish/session end/cold reopen/second encounter cases,
   accepted retry, foreign/stale/changed-body rejection and forged-history restore
   rejection without durable writes. Include lethal/knockout and lasting-effect
   consequences, spent source resources and dropped gear.
4. Obtain independent full review, run focused rules/app/UI checks and canonical
   `./scripts/verify`, then all exact-head CI checks, protected merge, fetched tree
   parity and post-merge checks. Serialize local heavy builds with other gate work.

## Status and next action

Research only. The physical integration is still PR33; area, opaque audience
protocol and Ready/reaction work are separate owned branches. Existing source and
effect LeaveCombat helpers preserve resources/effects, but their public composition,
time-bound recovery and real session transition are not implemented or verified.
Resolve the Ready/post-combat timing contract, then author the bounded production
path. No source edits, tests, merge or complete ledger family are claimed here.
