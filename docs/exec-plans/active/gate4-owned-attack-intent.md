# Gate 4 owned attack declarations

Status: plan before implementation, 2026-10-08. Source-only development allocated;
runtime and native acceptance unrun. Branch `codex/gate4-attack-intent`, no PR yet.
Root creates this plan, then explicitly transfers the sole branch writer to
`v5_capture_plan_review_oct8`. Root independently reviews the resulting complete
diff. The existing receiver remains frozen and its verification retains priority.

## Objective and authority

Let an attending player describe a single ordinary melee weapon attack for their
selected PC, review the resolved target and physical weapon, supply any material
missing choice, then send the existing tactical command through the durable outbox.
This advances Gate4's ordinary tactical declarations and the product definition's
natural input, player agency, questions-versus-actions and real application path.
It is bounded support, not complete language understanding or Gate4 acceptance.

Binding sources: root AGENTS, product-definition, Gate04 tactical checkpoint and
gate-execution protocol, ADR024 table authority, ADR025 geometry, ADR026 owned
resolution and ADR027 audience-safe transport. The selected player/actor, visible
options and current opaque revision supply context; prose never supplies issuer,
hidden knowledge, legality, outcomes, modifiers, source features or canonical IDs.

## Baseline and scheduling

The branch starts from reviewed receiver b62d8e719b4998a401745c41cc7cd9ee3fa681b5,
tree994183da202488546990114e3ff0019deb52c2c5. That candidate is published as draft
PR76 against actual main c101c202a9d7302e5ac60d791d3d6c5a4ed30e84 but is not yet
accepted. Its canonical local run and eight hosted runtime allocations are active.
No parent pass or source-only review accepts this new branch. Receipt into main
depends on accepted family integration and this branch's own required verification.

Only source/plan/test authoring, read-only inspection and static diff checks are
allocated while the receiver owns the local heavy slot. No Cargo/npm/build/test,
database/native work or global configuration changes in this branch. Preserve the
frozen receiver, executors, previous targets and original run evidence. Record any
parent correction explicitly and reconcile its whole history before final freeze.

## Scope and non-goals

- Client-only interpretation using the selected PC's existing `attack_options`.
  The actual current actor's scoped targets are the only name-resolution source.
  Exact full visible labels may be normalized for case/whitespace; duplicate or
  missing labels require material clarification, never arbitrary first-match use.
- One ordinary held melee Attack. Resolve only current ordinary purpose, physical
  weapon ItemId, located target, allowed ability and actually held lawful grip.
  Ambiguous weapon, ability or hand choices stay explicit. No automatic draw,
  stow, pickup, ammunition, thrown/ranged, source feature, reaction, Light/Nick,
  movement prefix or multi-action execution. Existing controls remain available.
- Questions, negation, hypotheticals, third-party declarations, unsupported clauses,
  claimed success/DC/authority and compound actions cannot execute a recognized
  prefix. Preserve the original draft for correction and explain missing support.
- Interpretation itself writes nothing authoritative. Show a clear preview and an
  ordinary Attack button for this intermediate text surface; no artificial spoken
  commit keyword. Confirmation creates exactly the existing typed action and uses
  unchanged `act({Tactical:{action}})` and persisted outbox/retry behavior.
- Bind transient draft/preview to campaign, player, selected PC, session, encounter
  and audience revision. Invalidate on context/options/pending changes and on lock;
  stale asynchronous view replies must not reactivate old confirmation. Recheck
  the actual option IDs and material choices immediately before submission.
- Restrict the entry point to the current attending PC's owned active turn with
  an available ordinary attack. Host and source-controlled creature channels do
  not silently substitute for that PC. Pending rolls/decisions/uncertain outbox
  block confirmation through the existing game lock.

Do not change server Text interpretation, TableIntent, existing transport DTOs,
serialization, source catalogs, execution/capability versions, rules/authority,
historical digests or corpus. Do not add an AI/provider dependency or a parallel
resolver. Other actions, movement composition, stealth/guessed targets, source
attacks and broad autonomous interpretation remain their existing Gate4/Gate9
obligations; none is declared complete by this bounded surface.

## Acceptance and verification

1. Commit this plan before implementation. Inspect current scoped option/context,
   actor/attendance and outbox code and preserve their authoritative boundaries.
2. Add bounded pure interpretation/choice code and a reachable desktop component,
   then wire the confirmed existing action into the same normal command path.
3. Add meaningful paired frontend tests: actual ordinary phrase through component
   to exact callback; no execution at interpretation; material ambiguity; duplicate
   labels; current held item/grip/ability; unsupported/negated/question/compound
   input; no hidden or another observer's target; stale context/options, foreign
   channel/actor, absent owner, pending work and uncertain outbox. Preserve all
   existing test bodies and fixtures. A parser-only test is not integration proof.
4. Independent full-diff review, fix actual findings, then freeze. When the heavy
   slot is transferred, run all ordinary frontend install/check/test/build and
   required canonical repository verification. Retain original commands/outcomes
   and failures. Do not claim a parent result as current-head execution.
5. Publish the coherent reviewed PR with current evidence; require exact-head
   Linux/MSVC/MSRV/guards/full runtime and normal packaging under repository policy.
   In the verified package, use ordinary PC creation/equipment/initiative; enter
   an attack declaration, inspect/correct an ambiguous draft, confirm once, enter
   physical-roll QA faces, and normally close/reopen pending work. Verify the
   existing real action pays once and preserves its target/item/request. Questions
   and unsupported compound declarations must leave the campaign unchanged.
6. Record exact source/package/runtime/native evidence and limits in this plan,
   merge only after parent acceptance and protected exact-head checks, verify
   literal main, and keep the separate human Gate4 encounter obligation open.

## Risks and next action

Main risks are guessing a material choice, prefix execution, hidden-name discovery,
cross-channel/stale preview submission, and a separate command path bypassing the
outbox. Keep the grammar bounded and readable, preserve draft text on clarification,
and exercise the actual containing TableApp context in tests. Presentation labels
are untrusted text and must render as text. No arbitrary internal ID from prose
may enter a command.

Next: sole writer inspects the current context/options path and implements the
bounded client slice. Source-only work is not acceptance; runtime remains unrun.
