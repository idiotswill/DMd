# Gate 4 — Guarded Grapple condition lifecycle

Status: **plan only; awaiting root and independent exact-head review before any
source authoring.** No implementation, executable verification or public Grapple
acceptance is claimed by this plan.

Writer: `gate4_ground_oct4`, transferred by root for this plan only on 2026-10-04.
Branch: `codex/gate4-grapple-condition-lifecycle`; sole writable checkout
`gate4-grapple-condition-lifecycle`. This is a separate follow-on to the guarded
core in development [PR56](https://github.com/idiotswill/DMd/pull/56). That core
branch stays frozen and is not this writer's checkout. PR56 and its development
dependencies remain unaccepted; do not merge this work into a development base or
treat a stacked branch as accepted main.

## Exact starting point and authority

Root supplied clean head `5296d06f2619027a9ce554131cb0ecd8c80dc607`, tree
`7fd8d82ffb7f2412271e821e99d5b93f8f02b0fc`. Branch, clean status and identities
were checked before writing. A fresh `git fetch origin main` resolved main to
`dbf1d633460473183324b4ec519e8d1980884b5c`; it did not move this assigned branch.
The supplied head's changes from `afc40ef5d4fb194ea30149e3a6592f4c6259c30f`
are core-plan documentation only. Actual production source remains afc40ef,
whose tree is `2243331dec35e6f41c79e06ea554e1284e0c469c`.

Read root `AGENTS.md`, the product definition, roadmap, gate execution protocol,
Gate04, ADR025/026/028, the full parent lifecycle contract and relevant effective
hands/core plans and consumers. These durable sources remain binding:

- [Product definition](../../product-definition.md): authoritative local combat,
  physical dice, provenance, no partial mutation, durable recovery and the
  feature-completion rule. Internal tests cannot substitute for actual product
  interaction, persistence or native acceptance.
- [Gate04](../../checkpoints/gate-04-tactical-encounters.md),
  [gate execution protocol](../../checkpoints/gate-execution-protocol.md), and
  ledger row `grapple-unarmed`: advance actions/conditions and exact suspension;
  no gate completion or receiving-gate change is implied.
- ADR[025](../../architecture/025-tactical-state-geometry-and-compatibility.md),
  [026](../../architecture/026-interruptible-encounter-resolution.md), and
  [028](../../architecture/028-reaction-execution-and-work-ownership.md): one
  spatial/condition authority, typed owned work, source provenance, old semantic
  dispatch, original-anchor replay and audience-safe application behavior.
- [Grapple lifecycle](gate4-grapple-lifecycle.md),
  [effective hands](gate4-grapple-effective-hands.md), and
  [guarded core](gate4-grapple-resolver-core.md): complete source behavior and
  temporary boundaries. Their source clauses include SRD5.2.1 pp182/190
  Grappled/Grappling/Unarmed Strike and p180 Dodge. Pinned source and full gate
  obligations are unchanged.

External design input read and reconciled against these actual consumers:
`tooling/grapple-next-temporal-checkpoint-ground-readonly-2026-10-04.md`, SHA256
`db26a403319ef3e859b7cf88c8bc0f040e7aabc484dd497bddc68713a30df730`.
The full authenticated-resolver map remains design input, SHA256
`e9ecc2a7df6785fbdb3dd8db7e7ec5cbe63652d5a681193b366b75ab3889ab67`, with
independent review `3b40b3efee4a26f6b8a6a9df44cfa6e8f3afd4660a65a6bab79a33a18c1c76df`.
This checkpoint is a guarded subdivision of map steps4–5, not their completion.

## Objective, scope and safe boundary

Make the existing private core's actually established ordinary grips contribute
Grappled to shared mechanics, including Speed zero and source-sensitive attack
Disadvantage. Permanently end an actual Dodge at establishment without corrupting
the original completed DEX save. Preserve the same private Attempt/save/owned
equipment/withdrawal/Escape/release producers; no alternate reducer is introduced.

Supported private contexts remain those of the current core: its own Attempt,
save/LR/equipment work, its selected Escape, or an idle live relation with no
unsupported suspended consumer. Actual geometry stays settled and within range;
the holder stays capable of sustaining the grip. The only lifted private
restriction is the current refusal based solely on the target's active Dodge,
after this checkpoint implements its real consequence. The existing refusal for
establishment requiring non-Hover flight-loss work remains. Grounded sources with
Fly capabilities can have current Fly Speed zero without creating a fall; no
airborne loss is silently admitted through the generic pump.

All public authority remains closed. Preserve:

- all seven new-family `grapple::guard_action` denials under Live and Historical,
  and permanent flow5 requirements; no new command or flow version;
- kernel and tactical early retained-authority refusals, raw19/20 guards,
  original-anchor/old-schema rejection, and the retired-raw hand refusal;
- `admission::supported_context` refusals for mixed attack/Shove/hit review,
  movement/OA, casts/missiles/areas, falls, other work/ordering and unsupported
  cuts/refreshes; current holder-incapacity/death and range-break refusals;
- `attacks/planning::reconstruct` and `falling/validation` temporary Grapple
  history/causal-flight refusals; no public state, source or historical admission.

Pure queries may derive the intended condition from a typed private-core state;
that is not permission for an ordinary public action to consume that state.
The shared adapter must neither validate an arbitrary source into authority nor
bypass these entry/restore boundaries. Public and table guard controls must remain
passing when root eventually runs them.

Non-goals here: attack/request cuts, release through unsupported waits, automatic
holder/range endings, self-only movement, OA refresh, causal falling, general last
consumer retention, completed raw-history admission, original replay, app/DTO/UI
activation, source changes, Ogre/Ground integration and actual LR positives.
No new schema/serialized fields are proposed. Surface a concrete missing receipt
for separate review before expanding that scope.

## Save validation: pending source versus completed evidence

The immediate counterexample is concrete. `grapple/saves::expected_save` derives
DEX Advantage from current `turns::dodge_context`; `grapple/validation::validate_evidence`
currently repeats that derivation even for a final save/live grip. Installing
Grappled makes Speed zero and `turns::refresh_dodges` removes the Dodge. Rederiving
that already completed save would now produce Normal instead of its original
Advantage request. Suppressing Dodge only while the condition exists would also
wrongly restore Dodge after release.

Use the existing precedent in `tactical/shove/validation.rs`: source request
rederivation is required while the save is unfinished; final evidence retains the
issued request and proves its raw/decision result. This does not make a saved
request authenticate itself. Implement the following distinction together:

1. New and unfinished saves (`proof == None`, including the actual LR pause and
   an unfinished save retired by withdrawal) still require exact current source
   derivation, actual STR/DEX modifier, armor/cover/conditions/Dodge, source DC,
   owner and entered/selected work. Preserve the entered producer's existing
   transient before pending/Automatic decision installation. Do not require a
   future receipt or permit a bare no-die durable save.
2. A final route requires the actual final proof and its matching stage/outcome
   or live/retained grip. Retain source/anatomy/DC/declaration checks; canonical
   role19 key/request ID, target roller and one-d20 die specification; original
   choosing/issuing metadata; actual accepted raw purpose/request/faces/resolved
   arithmetic or the exact real automatic/voluntary decision; cancellation
   consistency; target-owned finalizer and chronology; and the existing actual
   LR decision/expenditure binding. `TacticalGrappleSave::validate_shape`,
   `saves::evidence`, and the final arithmetic checks remain concrete consumers.
   A fabricated `proof: Some` cannot skip those checks. Secret/digital issuer
   restrictions and finalizer/accepted-by distinctions stay unchanged.
3. Do not compare a final request to current Dodge/condition state, replace its
   mode, alter original accepted faces, or reinterpret its historical modifier.
   This bounded validator preserves the authenticated producer's retained
   evidence; it does not prove an arbitrary forged initial image is historical.
   Coherently changed request/raw/proof copies still require original command
   replay to disprove. All public retained/raw/restore guards therefore remain
   mandatory. No test may claim matching copies alone establish source history.
4. No Dodge-specific receipt is currently proven necessary. No empty generic cut,
   future-grip RequestIssue cut, saved mode override, caller-selected relation
   list or historical admission permission is added. Pending readers of other
   relations remain refused until the full read-context checkpoint implements
   their actual producer/cut/consumer contract.

The existing restricted LR assumption stays explicit and unchanged: the core's
final validator checks required/declined LR against current availability because
it admits no intervening source/resource changes in that context. This is not
general LR history. Once a genuine legal source exists, a later last-use spend
must not invalidate an earlier declined final save; the later temporal/source
checkpoint must authenticate original availability and expenditure chronology.
Do not remove the current checks here just to admit synthetic LR history.

## Live-only projection and establishment consequence

Extend `tactical_effect_adapter::condition_effects` with a pure deterministic
iterator over only `RulesState.tactical_grapples.active`. Each view is Grappled,
targeted at the declaration's target and sourced from its actual grappler. Keep
legacy and tactical-effect iteration unchanged before this additive suffix;
no live attachment means the original sequence and values are identical.

Use an internal deterministic EffectId derived from the existing GrappleId in a
separate explicit condition-view UUID domain, e.g.
`dmd.tactical.grip.condition.v1\0`, with a constant neutral label, `Expiry::Never`
and no concentration owner, following the existing ephemeral lifecycle adapter.
This query identity allocates no raw/work occurrence and is never serialized or
used as an independently removable effect. The live relation owns duration.
Pending reservations, resolution proofs/cuts/ends and historical raw records
must never project a condition. Distinct incoming holders must retain distinct
source views even when a condition set deduplicates the Grappled enum.

Do not write these views to `RulesState.effects` or `tactical_effects`, invent a
concentration group, query source/anatomy inside the iterator, or call whole-state
validation from it. Existing valid producer/core validation owns actual source
admission; the iterator is allocation proportional to the real live relations,
not a second authority or a new grip limit. Confirm a distinct domain cannot
confuse the adapter's existing legacy/tactical IDs or effect removal paths.

In the real final-failed/nonimmune branch of `grapple/saves::finish`, install the
live grip and final save/outcome, run the actual `turns::refresh_dodges`, then let
the existing owned after-equipment/pump progression expose its next pause. Keep
the whole command's existing clone-and-validate atomicity. The exact ordering
must validate at every real durable pause; no whole-state validator may demand an
outcome before it is written. Source success, immune no-effect and withdrawal
must not end an otherwise valid Dodge. Release and Escape remove only the
selected grip; neither restores a Dodge that establishment already ended.

Separate or rename `admission::deferred_projection` as necessary so its non-Hover
flight guard remains both at initial admission and immediately before successful
establishment. Do not merely delete that helper or broaden mixed-context support.
The blanket Shove coupling refusal remains; projected new grips must not quietly
enable forced movement. No property of a source-built test should erase these
remaining unsupported cases to make it pass.

## Shared-reader audit and containment

| Actual reader | Intended effect and checkpoint constraint |
| --- | --- |
| `kernel/validation::active_conditions` and `tactical_conditions::attack_conditions` | Live target gains Grappled. Source-preserving `any(source != attack_target)` keeps Disadvantage against a third body and against one holder when another still holds. Query controls are allowed; public attack execution/history remains denied. |
| `tactical_conditions::effective_speed`, `turns::{speeds,dodge_context,refresh_dodges}`, `tactical_movement::capability`, `spatial/movement::speed` | All available locomotion speeds become zero, even with Dash grants/base-speed increases. No copied movement profile or second budget. Permanent Dodge removal is the only new committed consequence. Voluntary Move/self-only/forced motion remain unimplemented contexts here. |
| `spatial/falling::flight_loss_fall`, `falling::{require_settled_before_action,queue_losses}`, `turns::pump` | Current zero Fly Speed would reach generic loss detection. Preserve pre-establishment non-Hover loss refusal and prove that rejected establishment creates no generic fall/child, grip, raw change or paid change. Source Air immunity remains a paid no-effect, not a manufactured nonimmune Hover case. |
| `shove/geometry::path`, physical/intrinsic/spell/source attack readers, OA options | The projection makes their current condition queries aware of live grips. Their execution/retained-read restrictions remain; do not invoke them as a new admitted public path, weaken option equality, or change source damage. Pure condition assertions are not claimed attack/OA integration. |
| `may_harm`, area Charmed restriction, initiative, fear loops in attack/save/check/Medicine/liquid landing | These use specific Charmed/Frightened/Invisible/etc. predicates. Adding Grappled must not synthesize those states, alter visibility/knowledge or create new owned choices. No new condition means the exact old path remains. |
| vitality context, effect concentration/drop logic, `can_act`, spatial awareness/perception | Grappled alone is not Incapacitated, Unconscious, Prone, a concentration break, an Item drop or a visibility change. Keep those semantics and empty-hand boundaries unchanged. Actual incapacitated/dead holder transitions remain refused pending their causal integration. |

The projection necessarily enters common read-only mechanics. Containment is the
unchanged public/restore guard plus the private core's supported-context checks,
not a second conditions implementation or a feature flag. Audit all call sites
again if the assigned base changes. Preserve existing body/profile/inventory
validation without recursion through the new iterator.

## Meaningful authored controls after source authorization

These are planned, **UNAUTHORED and UNRUN**. Extend the existing private producer
fixture and real immutable definitions; retain its explicit synthetic executor/
fresh-turn scaffolding label. Private handler calls are not accepted application
journal, cold SQLite, portable replay or native play evidence.

| Control | Discriminating assertion |
| --- | --- |
| Actual Dodge producer and failed DEX save | Advance the actual private turn path to the target, invoke real Dodge, then advance to the legal grappler. Do not push a TacticalDodge record as the positive setup. Choose DEX through the target-owned core and submit the actual two physical d20 faces. Establishment retains the original Advantage request/faces/arithmetic and final proof while removing the actual current Dodge. Validate at AfterEquipment and after creating-resolution retirement with a live grip. |
| Release/after-equipment and later read | Release that established grip through its real owner during the existing owned AfterEquipment wait, then legally apply/decline equipment. Historical Established outcome/proof/end and paid attack remain; the actual hand frees and Dodge stays absent. A later newly derived save uses current Normal conditions when no other source changes them. Cover before-used allowance as well as after decline so clearing the creating resolution does not hide the evidence seam. |
| Success, immunity, withdrawal | A resisted DEX save and withdrawals before request, during physical request and after applicable accepted evidence keep their ordinary paid/equipment rules and do not remove a valid Dodge. A real installed Air target supplies immune no-effect where reachable. Do not mutate immunity/Hover or copy a Dodge record to claim a genuine source positive. |
| Live-only/source-sensitive projection | No condition from a provisional Attempt, completed resisted/immune/withdrawn attempt, ended retained proof or raw-only record. One live holder exempts only attacks directed at that holder; two distinct incoming sources keep Disadvantage against either while the other remains. Remove only one relation and prove the other still projects/occupies its own hand. Use actual private producers when supported; otherwise mark the isolated relation topology synthetic. |
| Zero Speed without incidental effects | Actual grounded source-built target becomes Speed zero; all available special speeds also derive zero and Dash cannot increase zero. A grounded real Large Chimera is a valid candidate for actual walk/Fly capability coverage after validating its full source setup; unsupported extra modes use labeled pure-query controls. Do not rewrite its profile to claim source proof. Grappled alone leaves HP, inventory, concentration, perception, Prone and Incapacitated unchanged. |
| Final evidence cannot self-authorize | Missing/false final proof, wrong stage/outcome, paid origin, actor, chosen/accepted/finalizer identity, request/key/work, one-d20 shape, mismatched raw mode/faces/resolved arithmetic, canceled accepted request and automatic/voluntary substitution reject. Existing hidden/digital ownership controls remain. Unfinished request source tampering still fails current derivation. Coherently forged final images remain refused by public retained/raw/anchor guards; original semantic replay forgery coverage remains an activation prerequisite. |
| Unsupported temporal contexts remain closed | Actual source size refusal for Huge Adult Red Dragon remains. Preserve the original oversized control when replacing only the old synthetic Dodge-refusal half. Keep grounded/airborne source geometry distinct; real flight-loss-required admission/establishment refuses atomically. Mixed attack/movement/OA/fall/other work, range-broken or incapacitated/dead holder and stale/foreign release remain unchanged full-state refusals; no pump-created generic FlightLost substitutes for future GrappleFlightLost. |
| Compatibility and authority | All seven new commands reject under both policies/older executors, including no-effect/retirement paths. Existing exact app nonprojection/authorization controls remain. Compare absence path/legacy Grappled effects and complete original fixture/raw/suite bytes. No new default field, source pin, accepted fixture or history is introduced. |

Avoid tests that merely assert a copied implementation formula. Exercise the
actual producer-to-final-evidence transition and its release/retirement boundary.
Preserve the original core controls unless the reviewed Dodge behavior directly
changes their assertion; retain the dragon half and all unrelated assertions.

## Planned slices, verification and next action

1. **This plan commit only.** Return its exact clean head/tree and full diff to
   root and an independent reviewer. No source authoring before their review and
   an explicit sole-writer source assignment.
2. **After authorization, commit writer/status before code.** Implement the
   unfinished/final evidence split, pure live projection, real permanent Dodge
   consequence and direct guard reconciliation together with meaningful controls.
   Do not publish a separately half-wired projection or expand the schema.
3. **Static handback.** Review the whole diff, guards, all shared readers,
   fixture/raw/receiving-suite byte preservation and whitespace/formatting as
   authorized. Return a clean exact head with accurate authored-versus-run status.
4. **Root-scheduled verification.** Root owns the sole heavy slot, presently Air
   canonical verification. Future allocation must run formatting/strict Clippy,
   new focused condition/core controls and relevant existing domain/core/hands/
   source/app guard controls, then canonical `scripts/verify-fast` and
   `scripts/verify` as required for the final candidate. Record actual commands,
   environment, exact heads, logs and failures; no old successful SHA substitutes.
   Root coordinates independent review, any draft publication and eventual CI.

The inherited afc core plan records root's 46 focused controls, formatting and
strict three-package Clippy pass. That is qualified dependency evidence, not a
run of this plan/branch or complete core/feature acceptance. Earlier failures
remain in the parent plan. No compiler, test, npm, build, database, native, CI,
push or PR mutation is authorized by this plan-only assignment.

Acceptance for this guarded checkpoint will mean source is coherent, reviewed
and verified at its exact head, the real private Dodge/save/projection controls
pass, and all authority/unsupported-context guards remain intact. It never means
runnable Grapple or Gate4 completion. General original-issue authority, all
attack/source-damage cuts, causal itemless-holder breaks, explicit self-only
movement, range/OA refresh, flight/landing, last-consumer retention, raw history,
source/Finish/session boundaries, recovery, privacy, transport/UI and native
acceptance remain prerequisites to activation. Drag/carry, special body parts,
PvP consent, class/feat exceptions, real spellcaster/reach acquisition, broader
reaction/Ready and every other Gate4 family remain Gate4 obligations.

Genuine LR is still unmet: present ordinary Human/Goblin sources cannot grapple
the Huge Adult Red Dragon. The separate additive Ogre and Ground branches remain
closed, incomplete dependencies until their full reviewed source/finite gear/
physical/ground/UI work and admission are accepted. No source payload, size, HP,
condition or LR attachment is invented here. Physical LR and real knockout-led
automatic LR require separately proven causal production paths, including the
earlier-decline/later-last-use chronology noted above.

Risks to resolve within this bounded source checkpoint are lost completed-save
evidence validation, accidental generic fall production, overbroad deletion of
deferred guards, duplicate persisted conditions, hand/source recursion, and
mislabeling synthetic control topology as accepted play. A concrete unsupported
gap is surfaced for plan review before its dependent code changes; scope is not
silently widened or acceptance reduced.

Exact next action: root and an independent agent review this clean plan-only
commit against `5296d06` and the cited consumers. Await explicit source ownership;
leave PR56 and every other branch untouched.
