# Gate 4 owned attack declarations

Status: coherent source-only draft for independent review, 2026-10-08;
frontend/runtime/native verification UNRUN. Branch `codex/gate4-attack-intent`, no PR yet.
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

## Source decisions — 2026-10-08

The source-only implementation uses a separate desktop declaration affordance;
the existing Talk at the table and server Text/replay interpretation stay exact.
Interpretation enumerates whole visible label combinations rather than splitting
at words such as "with" or "and" inside names. Duplicate complete parses retain
material choices. One supported target may omit a weapon; only an actually unique
held candidate can resolve that omission. Bare "I attack" requires explicit target
selection even with only one offered target; its unique held weapon may resolve
only after that selection. Review retains every ambiguous material choice. No prefix of an unsupported
longer declaration is submitted.

The original physical planner `tactical_weapons/equipment.rs:139–174` allows an
already-held weapon to use TwoHands when its other slot is free, and an explicit
OneHand choice to release the other slot. The UI filters existing offered grips:
OneHand must already hold that ItemId in its named slot; TwoHands needs the item
already held and both slots free-or-that-item. This creates no equipment change
or new permission; the existing command still revalidates. Multiple grips and
abilities require explicit choices. Source-feature offers are excluded here.

The transient context includes the actual options, budget and refresh generation
as well as campaign/session/revision/PC/source selection/encounter/round. Removing
the panel while ineligible or locked destroys drafts even if that same opaque
context returns later. Confirmation recomputes from current options both in the
component and in TableApp immediately before using the original outbox route.

Root approved a bounded duplicate-label discriminator: the selected PC's own
current, non-Remembered contact position may be displayed only for IDs already
in attack_options. It never introduces targets or resolves prose. Multiple
targets always require explicit choice; indistinguishable duplicate labels without
distinct own-visible positions block confirmation. Other observers/Host truth and
remembered locations cannot disambiguate. This display context participates in
preview invalidation. Duplicate held items display their actual held hand(s).

## Source checkpoint and validation — 2026-10-08

The draft adds the local proposal helper, owned-PC context helper, reachable
declaration component, a shared new test fixture and two additive test files.
TableApp adds the component, a current-context recheck and its existing Attack
submission call. The existing refresh generation becomes reactive so it can
invalidate this new keyed component; its increment and async identity checks are
unchanged. Existing send/save/retry/Text bodies, transport DTOs, backend/rules,
source content, old tests and historical corpus remain untouched.

Authored coverage is eleven pure proposal/context cases and twenty containing
TableApp cases after the two explicit parameter tables expand. These cover the
real callback/outbox boundary, no submission on interpretation, supported full
phrases, refusal of unsupported prefixes, ability/grip and duplicate target/item
choices, own-contact-only discriminators, stale selections/options/contact data,
absent/Host/source/foreign-turn/pending boundaries, uncertain delivery/reload,
another action's outbox and unchanged original Text retries. These are source
inventories, not executed results or claims about actual dice/native behavior.

Source self-review and `git diff --check` passed. No compiler, Cargo, npm, Vitest,
Svelte check/build, project parser invocation, native UI or database was run.
All runtime validation and native acceptance are explicitly UNRUN. Root's b62
family allocation still owns the heavy slot. No branch publication or acceptance
is claimed. Next: freeze this draft for root's complete independent diff review;
make only allocated corrections, then await exact-head verification allocation.

Root's initial source review identified that a bare declaration must not silently
resolve its sole target. The helper now retains an empty target until the player
selects it, with paired pure and full TableApp tests covering the disabled button,
no outbox/API before selection and confirmation, and the unchanged exact callback.
This correction is authored and statically reviewed; its tests remain UNRUN.
