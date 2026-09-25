# Gate 4 durable encounter conclusion and recovery

Implementation lead and sole branch writer: aftermath_recovery, delegated by root
after bootstrap_audit's preserved implementation checkpoint.
Branch `codex/gate4-encounter-finish`. Gate4 remains active; no gate acceptance is
claimed. Root authorized the bounded first aftermath slice on 2026-09-25.

Current baseline is verified main `2798b6b1d6263b5e321a1903d9fb4f2331b73895`,
merged normally at `318b62e`. Draft PR: https://github.com/idiotswill/DMd/pull/44.
Status: all original focused tests pass after environment recovery: four rules,
two genuine file-SQLite scenarios and all 71 UI tests; Svelte reports no errors or
warnings and the frontend build passes. A subsequent comparison-only fixture
refinement still needs execution. Canonical, integrated source-controller evidence
and final exact-head CI remain outstanding. The first focused attempt hit OS112
before any tests (details below). No production acceptance or upstream merge is
claimed. Night Hag main merged normally at `4244f64`; the verified PR42 historical
corpus is now integrated. Reviewed PR43 source control remains a dependency.
Initial normal merge `a76c68c` reconciled inherited pre-squash PR33 conflicts to
authoritative `d5d1db7` main. At that initial merge, the branch's two genuine commits
since `8cb3084` changed only this plan and the full tree differed from main only by
this file. No independent production source was discarded. Source-control PR43 must
still be reconciled at its reviewed checkpoint before final integrated acceptance.

## Authorized first production slice

Implement durable **hostilities concluded**, retaining the same initiative cadence,
geometry and single consequence queue; allow a quiescent aftermath session to close
and resume with genuine controller bindings. This is not terminal timing release or
new battlefield setup. Those mandatory Gate4 obligations remain below as follow-up.

- A host issues `ConcludeHostilities` with an explicit, non-default
  `ContinueExistingOrder` cadence policy and bounded ruling text. The accepted boxed
  optional record retains exact original metadata, clock and current turn. The UI
  explains this is a GM timing choice: ongoing saves and durations continue in the
  existing order. It grants no victory, surrender, loot, healing, rest or elapsed time.
- Keep `TacticalPhase::Active` mechanically. The additive aftermath marker changes
  presentation and session admission only; existing paid actions, source hooks,
  expiry and controller choices keep their source implementation. Never call
  `LeaveCombat`, clear Ready, reset resources, or advance time at conclusion.
- Conclusion requires no selected/pending consequence, dice, source routine/recharge,
  due falling or uncommitted table decision. Paid Ready may survive at an idle
  boundary. The marker is immutable and a second declaration is rejected; ordinary
  actions can still occur on the retained cadence if hostile activity renews.
- EndSession permits only quiescent marked aftermath, never a pending continuation.
  A new session can bind a retained dead character for its existing encounter
  decisions, without admitting dead actors into fresh initiative. A resumed session
  requires every retained player controller explicitly Present with its established
  character/source binding. Missing/absent owners are rejected before session creation:
  no attendance-update action exists to recover a stranded future mandatory save.
  Absent controllers are never replaced by host decisions. PR43's reviewed source
  control channel must be integrated and tested for the actual player-owned NPC path;
  the current base has no public player-source controls.
- Preserve old JSON by omitting the boxed marker and projection when absent. Add
  strict validation, original-action provenance and pre-tactical-anchor protection;
  keep exact accepted retry and historical/live execution separation unchanged.
- Implement actual host UI, session controls and real source-backed SQLite evidence.
  Keep private ruling text host-only and reveal no hidden participant/work counts.

The first-slice acceptance scenario uses genuine source Mage Armor and Hold Person,
a real dying character and paid physical Ready. Conclude at a material-choice-free
boundary, prove unchanged time/HP/resources/effect deadlines/Ready and retained source
counter state, end the session, quit/reopen, establish actual attendance, then continue
the same death-save, target-end save and Ready expiry through physical dice. Execute
identical accepted inputs on an independently restored mirror and retry each original
request after cold reopen. Reject pending/foreign/repeated/missing-policy/forged-origin
inputs without durable writes. Test old captures and unknown future-state guards.

Root granted the exclusive heavy build slot after disk and committed-memory recovery.
Focused rules/SQLite/UI checks use one job, default thread stacks and disabled
incremental compilation. Root also authorized draft publication with honest evidence;
full canonical/exact-head verification and independent review remain required before
root performs a protected merge.

## Objective and authority

The full finish workstream will allow the actual table to finish a settled encounter,
preserve its consequences, end/reopen the play session and begin another encounter without recreating source
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
- Reuse the actual Medicine stabilization and Help-to-end-knockout paths present in
  the audited source, with real action, reach, target/controller authority and physical
  checks through the vitality reducer. Do not introduce an HP/stable-state setter for
  postcombat recovery. Their existing combat behavior does not prove an aftermath path.

## Mandatory follow-up design and source interpretation

Read-only audit baseline: `fb83db7821adfce39f24f4de0707e95a3b49df34` on
2026-09-25. This is a later source reference, not this plan branch's implementation
base or a statement that every referenced path is merged or verified here.

Pinned SRD5.2.1 anchors, read from the verified extraction outside the repository:

- p13: a combat round represents about six seconds; participants take ordered turns.
- pp17-18: dying creatures save at their turn's Start; a Stable creature has no death
  saves, loses stability on damage, and can recover 1 HP after a physical 1d4 hours.
- p106: time-span durations last for their stated span; source spell dismissal has
  its own creator/incapacity prerequisites, rather than a general host cleanup power.
- p141: Hold Person repeats its save at each affected target's End while it lasts.
- p179: concentration ends through its actual source causes or its creator's choice.
- pp186-187: Ready lifetime/response and current-turn simultaneous ordering remain
  material source obligations. Ending hostile activity does not choose those outcomes.

The source does not prescribe a digital postcombat turn clock or say that an encounter
conclusion cancels spells or converts every next-owner-boundary duration to six seconds.
The proposed continuation of the existing round cadence after hostile activity is a
**GM timing interpretation**, to be explained and explicitly retained at admission.
It is not a claimed verbatim SRD rule, an automatic player pass, or a new expiry clause.
The first slice now implements only explicit retained existing-order cadence. Terminal
release, free elapsed time and replacement still require the decisions below; none is
claimed complete by the first slice.

### Why the existing helpers cannot finish this task

At the audited head, `tactical_effects::LeaveCombat` preserves groups/effects but clears
the effect turn cursor and per-turn trigger uses. `Observe(Time)` handles absolute
deadlines; owner-relative expiry and turn-trigger saves still need `Observe(Turn)`,
which requires authoritative `rules.timing`. Clearing timing alone therefore strands
relative effects and Hold Person saves. Legacy `RulesAction::EndCombat` instead removes
legacy `AtTurn` effects; legacy `AdvanceTime` does not run the current effect/vitality
continuation. Neither is a safe replacement, and their public guards must remain.

Current Finished validation forbids timing/pending resolution, EndSession rejects any
flow, and battlefield setup rejects an existing encounter. New initiative starts at
turn 1. These are implementation dependencies to change deliberately, not evidence that
a new Finish button already works. Ground item positions currently belong to the flow,
so simply replacing the encounter would also lose physical location authority.

### Proposed state transitions

1. **Conclude hostile activity.** A host-issued, journaled conclusion retains its
   original command, current encounter/cursor/clock and bounded ruling. Require all
   currently selected raw dice, consequences, failed saves, falling, reaction windows,
   source routines and recharge requests to be settled. Record no invented victory,
   surrender, loot or new elapsed time. A conclusion is historical evidence, not a
   second copy of mutable HP, resources or items.
2. **Aftermath while turn obligations remain.** Preserve the same initiative order,
   index, round, turn number, budget, geometry and single TacticalResolution. Continue
   actual End/Start boundaries, advancing six seconds only at the existing round wrap.
   Preserve owner-relative and legacy AtTurn expiry, target-end saves, unstable death
   saves, Ready, Dodge/Disengage lifetimes, source hooks and per-turn usage. Retained
   source turns can generate recharge/other consequences; conclusion itself cannot.
   Existing controller-owned EndTurn, raw dice and simultaneous choices remain real
   commands. Any later batching of elapsed aftermath time must pause for material
   controller decisions, with no fabricated action, pass, raw roll or consent.
3. **Release timing.** A derived, validated scan must establish that no relative expiry,
   turn-trigger rule, unstable death-save actor, held Ready, turn-scoped stance or
   unresolved work would be stranded. Only then invoke the current LeaveCombat helpers
   and reach Finished. Keep absolute-duration effects/groups, concentration, recovery
   clocks, rest records, HP, source per-rest pools, inventory and knowledge. A quiescent
   Aftermath as well as Finished should permit session end/reopen without advancing
   game time; reopening still requires genuine attendance/controller bindings.
4. **Advance free elapsed time.** Add an authenticated current-authority transition
   retaining original command, start, requested target and progress. Give the existing
   consequence stack an explicit no-turn context; do not invent a turn actor or create
   a second queue. Stop at the earliest effect/group/recovery deadline, observe the
   actual time, resolve resulting expiry, damage, falling, choices and raw dice, then
   resume the original interval. Do not skip intervening saves or grant a completed
   rest, healing or replacement benefit merely because the clock moved. Ordering
   simultaneous deadlines without a current turn needs an explicit GM adjudication
   policy; it must not silently borrow another PC's source ordering authority.
5. **Start another settled encounter.** Retain a conclusion reference and create fresh
   encounter/scene/setup/initiative authority while reusing actual entity, source and
   item identities. Preserve surviving absolute effects and their original deadlines,
   concentration, HP/death/recovery, spent uses and ammunition. Reconcile only per-turn
   markers at a legitimate new Start: for example, an old `savage_attacker_turn == 1`
   must not accidentally deny the new encounter's turn 1. Do not reset per-rest pools
   or recharge availability at conclusion/setup. Archive or rehome dropped-item spatial
   records under their authenticated old scene/location before replacing the flow;
   do not delete them, teleport them onto the new map or recreate starting equipment.

### Unresolved immediate renewed combat

A fresh-initiative or changed-participant encounter while Aftermath still has relative
effects cannot simply reset the cursor, expire them, or omit targets no longer on the
new map. A bounded renewed-hostility path may retain the existing cadence and geometry;
it must be identified as continued timing, not a completed fresh-encounter handoff.
True replacement needs explicit owner-boundary rebasing and offstage target timing,
including source saves, player authority and the relationship to the new initiative.
That design is unresolved. It remains an active Gate4 requirement before full acceptance,
not a deferral to Gate5/6 or permission to claim arbitrary encounter replacement.

### Historical and audience compatibility

New records should be absent from old saves unless explicitly introduced by an accepted
current-version action. Follow ADR028's historical/live execution separation and root's
post-PR38 version decision; do not reinterpret old Legacy/ReactionsV1 events or saved
AtTurn numbers as seconds. Extend strict codec/database preflight and origin audits for
any new authority, including conclusion, elapsed interval/progress and ground locations.
Replay must derive these from the original accepted actions, not trust a snapshot's own
claim. Preserve the pre-tactical recovery anchor, original retry body and causal metadata.

Keep historical presentation bytes and acceptance-time audience classification intact.
New aftermath controls must preserve opaque revisions and controller-owned raw rolls;
private offstage effects must not introduce victim counts, private work cards or transcript
entries into unrelated player views. Finishing a session or encounter cannot reveal a
secret simply because the host can inspect its remaining timing obligations.

## Required production regressions for the proposal

These are the full finish workstream requirements, not existing passing evidence.
The first slice now validates portions of 1, 2, 4, 6 and 7 below. The complete
requirements, especially timing release and second-encounter handoff, remain open.

1. Create a genuine source Hold Person through the table, conclude while it is active,
   and retain the affected controller's repeated End save across session end/reopen,
   physical raw submission, exact retry and independently restored continuation.
2. Preserve a source owner-relative defense at conclusion and expire it only at the
   correct owner's next boundary. Cover a partial round and multiple simultaneous
   consequences with explicit ordering; a held Ready must not be removed by the host.
3. Cast actual source Mage Armor, carry its effect into a second encounter, and prove
   expiry uses the original eight-hour deadline rather than eight hours from setup.
   Cover its real armor-donning end trigger and concentration/overlap cleanup separately.
4. Exercise genuine dying, Stable and knockout actors through aftermath. Retain physical
   recovery dice and interrupted recovery history; prove no early healing or fabricated
   rest benefit, including damage and a restored pending recovery consequence.
5. Begin a second encounter with spent Second Wind/source uses, spent ammunition and
   persistent loadouts. Prove no grants duplicate, dead actors do not revive, grounded
   items remain at their old location and a genuine new turn has correct Savage Attacker
   eligibility. A first real source Start may recharge by its normal roll, not by reset.
6. Use actual file SQLite and boxed phases on the default Windows stack. Reopen/retry
   every new pending boundary and drive an independent portable mirror with the same
   accepted inputs. Forge structurally coherent conclusion, cadence, deadline, progress
   and ground-location histories and reject restore without durable rows. Normalize only
   the export request-time timestamp when comparing otherwise identical persisted saves.
7. Compare unrelated player DTOs/revisions/transcripts during hidden aftermath work;
   reject foreign, stale and changed-body commands without costs. Exercise actual desktop
   controls, then packaged finish/session/restart/second-encounter acceptance. Source-only
   tests or a synthetic effect fixture do not substitute for those application paths.

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

Main fetched; root AGENTS, product clauses, gate protocol/checkpoint, ADR024/026/028,
current production boundaries and pinned source passages read. The normal baseline
merge is complete and its plan-only difference proved. Plan commit `cb52625` preceded
production writes. Authored first-slice changes now include:

- Boxed optional conclusion record, privileged typed policy/ruling admission,
  quiescence checks and exact accepted-origin audit. No timing/source/effect mutation.
- Additive, omitted-when-absent projection with host-only private ruling. Actual host
  form requires an unchecked policy choice and ruling; session controls respect the
  same derived boundary. Session start permits only genuinely retained dead PCs to
  keep their established controller, without admitting them into fresh initiative.
- Four focused rules tests cover complete preexisting-state preservation, paid Ready
  lifetime, pending death-save refusal, malformed policy/origin and historical absent
  serialization. The isolated HP fixture is explicitly not production source proof.
- Two genuine file-SQLite cases use actual catalog Mage Armor, Cultist Hold Person,
  paid PC Ready, and a Goblin's physical critical hit causing a real dying PC. New
  modern-envelope steps are cold-retried and independently restored/continued. They
  exercise repeated target saves, death saves, later dead-PC session binding, unchanged
  resources/time/effects, private ruling projection and zero-write forged restore.
- Three UI cases cover explicit choice, uncertain-request lock, host/current-executor
  ownership and ruling privacy. The small shared medicine preparation helper is only
  exposed to the sibling test module; its existing scenario remains unchanged.
- A fourth UI case now drives the actual TableApp conclusion form, unconfirmed
  restart/exact envelope retry, EndSession affordance and new StartSession controller
  binding. This is a mocked IPC/component test, not native packaged acceptance. The
  SQLite corruption matrix also replaces the complete snapshot list with the current
  post-conclusion image to prove it cannot self-authorize as a recovery anchor.
- Root-approved fail-closed session attendance checks now reject an absent retained
  controller or omitted PC binding without durable writes; each real session rollover
  tests both refusals before starting with all genuine owners. Host text explains the
  requirement. Verified Night Hag main merged cleanly as `4244f64`; aftermath source
  was unchanged by that prerequisite merge.
- Independent review caught a fixture-only false comparison: two newly accepted
  commands in independent databases generate distinct random audience revisions.
  The test now compares accepted command/outcome and semantic state across mirrors,
  while requiring byte/exact full-response equality on retries within each database.
  No production protocol or assertion of durable retry identity was weakened.

`cargo fmt --all --check` and `git diff --check` pass after the normal PR42 merge
and the comparison-only helper refinement. Original focused rules, SQLite and UI
verification passed as detailed below. Fresh independent production review is clear
on exact `5e6536f`; production source is unchanged by later documentation, helper
and PR42 integration work. Next: integrate verified PR43, cover actual source-only
controller session rollover, obtain a new shared slot for focused/refined-helper
and canonical verification, then complete independent exact-head review and CI.
Full release/new encounter remains mandatory Gate4 work and cannot be moved to Gate5/6
or silently represented by the marker.

### First focused verification attempt (2026-09-25)

Exact source head `5e6536f` has fresh independent review through its final attendance,
mirror, anchor and actual TableApp test delta. After root's explicit slot grant, the
focused `cargo test -p dmd-rules --test tactical_turns aftermath -- --nocapture` used
jobs1, no `RUST_MIN_STACK`, and refreshed crate timestamps for the shared target.
It exited101 **before any test** because writing the rules archive failed with OS112
(insufficient disk space). Log: `DMD/tooling/aftermath-rules.log`, process91812.
No Rust source diagnostic was reported.

Only about75MB remained on C:. The generated `tooling/target/debug/incremental` cache
measured19.6GB. Its exact absolute path, containment and absence of reparse points were
verified; no Cargo/rustc process remained. Root authorized deleting only that replaceable
cache and rerunning with `CARGO_INCREMENTAL=0`. Automatic approval review blocked the
native PowerShell deletion before process creation with reason `blocked by policy`.
No deletion occurred or alternate deletion route was attempted. Root subsequently
confirmed environment recovery (28.3 GB disk space available) and reassigned the
preserved work to aftermath_recovery. The focused sequence subsequently passed with
`CARGO_INCREMENTAL=0`, as recorded below. No blocked cleanup was retried.

### Focused verification recovery (2026-09-25)

On production checkpoint `5e6536f` plus plan-only edits, the identical focused rules
command completed successfully: 4 tests passed, 32 unrelated cases filtered, default
Windows stack, jobs1 and `CARGO_INCREMENTAL=0` (session68456). Log:
`DMD/tooling/aftermath-rules-recovery.log`. The two genuine file-SQLite aftermath
cases subsequently passed on source-equivalent head `378884f`: 2 passed, 44 filtered,
1563.92 seconds (session28380, exit0, default Windows stack). Log:
`DMD/tooling/aftermath-sqlite-recovery.log`. The magic case exercised genuine source
Mage Armor and Hold Person, paid Ready, conclusion privacy/forged-history refusal,
session rollover, physical repeated save and Ready expiry. The dying case retained
the genuine Goblin attack's zero-HP result through physical death saves, repeated
session rollovers, actual death and a final controller-owned dead-character turn.
Both preserved every authored cold reopen, independent portable continuation,
original retry, changed-body rejection and complete normalized export comparison.

Frontend verification then passed on the same production sources (session45578,
exit0): Svelte 0 errors/0 warnings; all 71 tests across 15 files, including the three
aftermath controls and actual TableApp uncertain-request/session scenario; 137-module
production build. Log: `DMD/tooling/aftermath-ui-recovery.log`. These component and
SQLite results do not claim native packaged end-user acceptance. The heavy slot was
released to the PR43 source-control integration writer before any later compilation.

### Comparison-only fixture refinement and integration

After the first complete passing run, root authorized a narrowly scoped helper
refinement reviewed independently by shield_rules_recovery. The three current-state
comparison reads in `cold_step` now decode actual post-commit persisted export images
instead of reopening the primary/mirror campaigns solely to obtain the same state.
The explicit real file close/open/`resume_campaign`, independent portable restore,
both accepted command paths, both original retries, changed-body rejection and full
normalized export equality all remain. No recovery checkpoint or assertion was
removed. These removed opens validate current images/content rather than replaying
the complete journal; no speed improvement is claimed before measurement. This
refinement is formatted and statically reviewed but its execution remains pending.

Verified main `2798b6b` was merged normally as `318b62e`. The integration brings the
four genuine legacy reaction captures and their executable corpus without changing
any aftermath production source or UI. Final PR44 acceptance still requires PR43's
genuine player-owned source channel, its source-only aftermath session regression,
canonical verification, independent final review and all six checks at the final
exact PR head. Terminal release and new-encounter admission remain mandatory Gate4
follow-up, not completion claims for this PR.
