# Gate 4 — Magic Missile target Shield and simultaneous impacts

Status: active implementation; both original flow3 capture scenarios and the
source-equivalent four-file baseline test passed. All four exact exports remain
unchanged. Flow4 domain/rules implementation and focused cases are authored but
uncompiled and unrun. Four independent flow3 continuation cases remain unrun.
No runtime correctness, app integration, or acceptance is claimed.
Branch: `codex/gate4-shield-missile-runtime`.
Writer: `missile_implementation`, exclusively for this branch; root coordinates
separate app integration and serializes all heavy builds.
Development base: `98399da19def1f5a4cbf6d7ef230bd98c6d1c421`, which integrates
verified PR43 main `e813e3a13911497902a3d4a55aec3c70653afb2a` into the Shield
candidate. Its complete tree equals `7bc01afc46f3591e38e5168072496e91c115c10d`;
production source is unchanged from Shield candidate
`8b5cf52a250c5f381d22977391906cfedc53a630`.

This is a separate development branch, not an expansion of PR45. PR45 still needs
its own complete runtime/final verification and protected merge. After both genuine
flow3 capture scenarios pass, all four exports are baseline-restored and their exact
bytes/hashes/provenance are frozen, flow4 development may proceed here while PR45's
remaining verification runs. No flow4 acceptance or merge is allowed until verified
PR45 main and any subsequent prerequisite correction are fetched, reconciled and
verified on this branch. Parent/test evidence never substitutes for exact final-head
evidence. Heavy commands remain serialized under root's explicit slot coordination.

Incoming integration dependency: PR44 aftermath may merge before final PR45. Root
will reconcile its currently flow2-specific retained-cadence validation with flow3
on the Shield branch. Flow4 must preserve that resulting supported aftermath/session
path after fetching verified PR45 main; it must not strand a new Begin4 behind a
marker that validates only2/3. Do not preemptively edit PR44 source on this branch.

## Objective, product traceability and boundaries

Complete the next real reaction slice: a source-authorized Magic Missile cast binds
every dart target, offers each distinct target its own pre-impact Shield decision,
records individual physical damage faces, and resolves a retained simultaneous
impact set in the current-turn controller's explicit occurrence order. All work
uses the existing tactical resolution/frame stack and production table transport,
desktop, SQLite transaction, presentation history and semantic recovery paths.

This advances `docs/product-definition.md` sections Player control, Rules hierarchy,
Passive/hidden/private information, Speaker/player/character/authority identity,
Complete tactical timing, Session start/end and exact suspension, and Feature-
completion rule. It advances Gate04 reactions, damage/death, concentration,
controlled entities, player-safe visibility and exact mid-combat recovery.
ADR026 and ADR028 govern the single authoritative state, source provenance,
same-time ordering, independent response ownership and historical interpretation.

Scope is immediate Magic Missile using the already pinned spell program, genuine
Night Hag source casting and genuine Mage Protective Magic, plus every currently
supported source grant that resolves to the same spell program. Preserve existing
attack-hit Shield behavior in the new executor. Do not invent a table-creatable PC
spell grant or silently expand character creation to demonstrate this slice.

Non-goals: Counterspell; physical or magical Ready release; new reaction families;
general suspended attack/movement stacks; new PC classes; scene/timing release;
global deferred vitality; general spell batching; broad replay-performance changes.
Ready/off-turn missile integration remains required in the Gate4 reaction umbrella,
with its own trigger/parent authority acceptance when that producer exists. No
scope item is moved to Gate5 and no Gate4 acceptance is waived.

## Required source and architecture evidence

Use the selected local source, not recollected edition rules or external rulings:

- `content/srd-5.2.1/source.json` pins SRD5.2.1, CC-BY-4.0, SHA256
  `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
  Local PDF: `C:/Users/jadra/Documents/ChatGPT/DMD/research/srd-5.2.1/SRD_CC_v5.2.1.pdf`;
  same-directory `.txt` is the inspected extraction. Verify the pin again at branch entry.
- SRD p146, Magic Missile: visible creatures within 120 feet; three darts; each
  deals 1d4+1 Force; all strike simultaneously; one additional dart per slot level
  above 1. Current typed definition is `content/srd-5.2.1/tactical.json:1513`.
- SRD pp161–162, Shield: Reaction when hit by an attack roll **or targeted by
  Magic Missile**; V/S; self; through the start of the caster's next turn; +5 AC
  and no Magic Missile damage. It does not require an attack roll from the missile.
- SRD p187, Simultaneous Effects: current-turn table controller chooses the order.
  SRD p10/186 reaction timing and pp105–106 components constrain the response.
- SRD p16 shared damage wording concerns simultaneous saving-throw targets. It
  does not by itself establish one shared d4 for Magic Missile. SRD pp17–18 govern
  damage at zero HP/death, and p179 governs concentration on damage/incapacitation.
- Preserve the exact Night Hag spellcasting adaptation/source pages in
  `content/srd-5.2.1/tactical.json` and its explicit omissions in `NOTICE.md`.
  Its at-will level-4 grant produces six darts and waives Material only; V/S and
  actual source ownership still apply. Mage Protective Magic retains its shared
  limited-use pool, actual components, Reaction and next-own-Start expiry.
- Record this timing interpretation in ADR028 (with an ADR026 cross-reference),
  the active bounded plan, the reaction umbrella and affected Rules Coverage Ledger
  rows. Preserve attribution/NOTICE and the pinned source payload unless a separately
  justified content correction is required.

## Explicit adjudication to record before implementation

The source explicitly requires simultaneous strikes and controller-selected order.
It does not fully specify network collection, repeated-target damage-instance
grouping or the implementation of nested consequences. The following is the bounded
application interpretation; do not present it as the only possible source reading.

1. **Target commitment.** Admit the complete ordered dart allocation at the real
   cast command, including repeated targets. Bind source grant/program, cast
   occurrence, source actor, spatial/perception admission and each original target.
   No allocation change is allowed after any target response or damage face.
2. **Pre-impact responses.** One common missile-target trigger has one respondent
   per distinct targeted creature, not one Shield offer per dart. Every such owner
   acknowledges the collection stage, even if ineligible; that acknowledgment is
   cost-free coordination, not an invented fictional action or Reaction. Private
   affirmative intent does not spend or guarantee permission. The current-turn
   controller gives an explicit total ordering instruction or exact-trigger Host
   delegation, independent of zero/one/two eligible Shield offers. Selected owners
   make fresh current-head cast/decline commands. Resolve every selected Shield and
   its children before the missile amounts/impacts proceed. Revalidate each selected
   grant, source use, components, consciousness and Reaction before spending.
3. **Amount collection.** Retain one actual d4 face for each admitted dart and its
   source +1, using one existing pending raw request at a time. Complete the whole
   collection before the first missile vitality mutation. Deterministic collection
   order is bookkeeping, never impact priority. The approved first implementation
   retains all dart faces, including those subsequently prevented by Shield; it
   neither fabricates zero faces nor replaces separate amounts with a shared roll.
   The coordinating writer explicitly accepted this all-faces collection policy
   together with per-dart damage instances and normal nested child draining.
4. **Impact commitment and order.** Create a sibling frame containing every dart's
   distinct impact occurrence. Selection is by occurrence, with a meaningful known
   dart/target label; repeated actors cannot collapse into an actor-only ranking.
   The actual current-turn controller selects the next impact. An ancestor's area
   consent or the earlier response-order delegation does not grant impact ordering.
   Prefer the existing work-choice transport. No UUID, arrival order or silent
   initiative fallback makes a material impact choice.
5. **Damage instances and children.** Each selected dart is one Force damage
   instance. Ordinary defenses/temp HP and current vitality apply separately.
   Each actual damaging instance may produce its own concentration save and
   damage-at-zero failure. Magic Missile is not an attack, critical hit or melee
   knockout source. A concentration failure can remove dependent effects before
   the next impact. Existing nested concentration/effect/falling consequences drain
   before another sibling impact is selected. There is no aggregate HP packet and
   no policy requiring all HP changes before all consequence work.
6. **Committed strikes survive consequences.** Once bound/committed, later damage
   cannot retarget or erase another dart, even if a child incapacitates/kills the
   caster, moves a target or removes concentration. Do not re-run cast permission,
   range or perception on each impact. A target already dead when its occurrence
   is selected gets an authenticated no-effect completion; retain its raw/target
   evidence, never resurrect it or create corpse HP. Distinguish this from removing
   the occurrence. Source-defined damage prevention is evaluated without reopening
   a new target-trigger window after amounts are known.

This needs a **response/amount collection barrier**, not a deferred-vitality barrier.
`tactical/areas.rs::start` already collects saves before its impact frame, while
`turns::pump` and `continuations::apply_vitality_from_cause` drain nested children.
Use that frame behavior deliberately; do not change area semantics as a side effect.

## Compatibility contract: absent/0/1/2/3 and planned 4

`crates/dmd-domain/src/tactical_reactions.rs:35` currently defines only Legacy=1,
ReactionsV1=2 and ShieldHitV1=3. Numeric flow0 is invalid; it is not an old executor.

| Existing state/request | Required behavior after this slice |
|---|---|
| Pre-tactical state / omitted optional authority | Remains absent; no invented flow, source access, response or impact records. |
| Numeric flow0 / other unsupported numeric version | Still rejected by `from_flow_version` and strict validation. |
| Flow1 Legacy / omitted old Begin execution | Exact original replay; old suspended work finishes; old unit `UpgradeExecution` still means 1→2. |
| Flow2 ReactionsV1 | Exact original programs, sequential missile behavior, raw IDs, bytes, receipts and existing upgrade semantics. |
| Flow3 ShieldHitV1 | Exact accepted hit-Shield windows and original sequential missile behavior; saved hit decisions and raw pauses still finish. |
| Proposed flow4 `ShieldMissileV1` | Adds the committed-target/response/amount/impact pipeline and retains hit-Shield support. |

The approved target is `ShieldMissileV1`/flow4; confirm the number remains unused on
fresh main before implementing it.
New live Begin requests require the new current version. Existing accepted requests
recover before fresh admission, source ownership or current-revision checks. Add
the new targeted settled upgrade, preserving historical `UpgradeExecutionTo {
execution: ShieldHitV1 }` and old unit upgrade exactly. No downgrade, paid-Ready
discard, raw-request loss, budget reset, heal, time advance or silent upgrade.

Audit `tactical.rs::validate_live_execution` carefully: once 3 becomes historical,
its existing Respond/Order/Delegate/Cast/Decline hit actions must remain allowable
to finish an old pause. Do not let ordinary fresh actions continue under a weaker
executor. Extend every exact-version-3 hit/work-trace/projection gate to explicit
supported-feature membership for 3 and 4, while keeping v3 interpretation unchanged.

Before changing production semantics, capture genuine pre-change flow3 histories
through actual commands: a partially resolved Night Hag missile cast and a selected
hit-Shield pause, with exact original requests/responses and pre-tactical anchors.
Keep all four existing PR42 exports unchanged. Do not manufacture an old capture by
editing version fields or inject new pauses into existing journals. Preserve
`SpellDamageShare::PerDart`, compiled program/source fingerprints and old request-role
tags; the tactical executor selects the new schedule. New retained authority is
omitted from old state and strictly rejected when attached to an older flow. Schema
1/2/3 preflight rejection of tactical authority remains; schema4 is not permission
to trust an unauthenticated first/current/intermediate snapshot.

Transport/presentation versions are separate from tactical execution. Preserve
table transport1 and activated source transport2, their original envelope versions,
audience hashes and exact historical decoding. New optional missile controls are
absent in old projections. Any actual normalization change needs a new explicit
envelope version; do not opportunistically reinterpret existing `HitResponse` bodies.

### Genuine flow3 partial-dart capture procedure

Do this before any flow4 runtime edit, with the heavy slot explicitly available.
The existing production-route test
`table_night_hag_cases::night_hag_current_catalog_casts_six_real_darts_through_cold_owned_transport`
calls `create_sources`, `prepare`, `begin_cast` and `finish_darts`; the last helper
asserts HP `19 - 2 * dart` before each raw submission and HP7 after all six. Those
are executable sequential-semantics evidence, not assertions to delete or relabel
as historical while continuing to run a fresh flow4 Begin.

1. Root has prepared the separate generator worktree
   `C:/Users/jadra/Documents/ChatGPT/DMD/tooling/missile-legacy-capture` at detached
   `06045b2b8f6cff6d895a07be46ea7cb1b4087a2f`, through
   `b11733bb3c5d163d6854c1ed55765890e650e9a2`, based on
   `7bc01afc46f3591e38e5168072496e91c115c10d`. Its only differences are a diagnostic
   plan and read/export hooks in the existing Night Hag and owned Shield tests;
   production source is unchanged. The generator has one writer and must never
   merge. Do not amend its runtime or reinterpret Begin3. Its capture/verification
   run passed as recorded at the checkpoint below. The complete 7bc01af..06045b2 hook delta was
   independently inspected read-only; this is not compilation or runtime evidence.
2. The prepared hook observes `finish_darts` at zero-based `dart == 0` and `2`,
   before each iteration's raw submission: before the first face, then after two
   genuinely accepted amounts with the third pending. Set `DMD_CAPTURE_MISSILE_DIR`
   to a fresh existing external directory. It uses `export_campaign`, the existing
   `to_json` serializer, flow3 assertion and `create_new` output semantics; it never
   replaces campaign state or calls historical replay for play. Both previous
   `cold_amount` reopen/mirror/retry checks have run before the second capture.
3. Verify the before-first export has HP19 and the first pending amount. Verify the
   after-two export has flow3, six original Night Hag targets, two completed source
   occurrences, two accepted d4 faces of1, HP15, the third distinct amount request
   pending, Action spent and no fabricated slot/use. Preserve command IDs, sessions,
   audit/events, raw issued/accepted metadata, transport bindings, presentation
   records, snapshots and the pre-tactical anchor exactly as exported. The complete
   accepted inputs/outputs already exist in `TableTransportBinding.request_json`
   and `response_json`; extract copies of the original cast/second-amount envelopes
   for readable provenance if useful, never regenerate their bodies or revisions.
4. Let the unchanged scenario complete and preserve its passing log. Optionally
   retain the final export as separate replay evidence. Do not stop at producing a
   file while leaving the original sequential assertions unexecuted. Save the hook
   diff, generator SHA, invocation, toolchain, result and SHA256 of every artifact.
5. Remove the diagnostic hook after successful capture (or discard only the isolated
   generator checkout after its evidence is safely retained). It is not production
   code. Commit the reviewed immutable fixture bytes and provenance manifest on the
   compatibility/new-slice branch; never hand-edit IDs, versions, HP or bindings.
6. A replay test restores the partial export, verifies the genuine HP15/third-roll
   pause, retries the exact already-accepted second envelope to its original receipt,
   and continues the last four darts under flow3 using actual projected commands.
   New requests after independent continuation use that database's own projected
   revisions; do not assume random new audience revisions equal the generator's.
   Verify HP13→11→9→7, no new target Shield window, original cast/resources and complete
   termination. Also preserve a whole-history replay check and rejection with no
   writes for a changed accepted second body. Exercise the before-first capture too.
   This test remains separate from the
   fresh flow4 Night Hag production scenario, which must assert its new barrier.

The same prepared generator also exports the otherwise unchanged genuine owned
Shield scenario at `shield-hit-v1-selected.json` (after the first offer/order,
before casting) and `shield-hit-v1-post-cast-damage.json` (after Shield against a
natural20, before reporting the pending damage). Both are real flow3, transport2
histories, including ownership, costs, opaque requests and completed Shield proof.
Run that complete scenario too; validate and freeze all four exports, not only the
two missile files. The later replay suite must finish both old hit stages using
their real controllers, retain original damage issuance cause, and recover exact
already-accepted requests before fresh admission. Do not synthesize either pause
from a typed-state fixture or call the exporter itself a replay success.

## Concrete production design and current code map

All paths below are repository-relative and were read at the stated baseline.
New type/module names are proposals, not claims that those symbols already exist.

| Existing location/symbol | Required bounded work |
|---|---|
| `crates/dmd-domain/src/tactical_reactions.rs:35`, `TacticalExecutionVersion`; `tactical_resolution.rs:71`, `TacticalWorkKind`; `:198`, `TacticalResolution` | Add execution4, omitted-when-empty retained missile records and typed amount/impact continuation kinds. Keep them in the existing resolution/frames. |
| `crates/dmd-rules/src/tactical_spells/program.rs::compile_spell_program` (line48; PerDart branch around164); `execution.rs::spell_amount_request` (27), `spell_amount_operation` (64) | Reuse exact source d4+1 compilation/amount validation. Preserve old compiled program identity. New impact path must not invoke old late-target suppression in a way that deletes committed work. |
| `crates/dmd-rules/src/tactical/casting.rs::begin` (61), `commit` (322), `start` (476), `finish` (538), `finish_cast` (628), `validate` (712) | Branch scheduling only for flow4 canonical PerDart Magic Missile. Replace immediate amount→vitality with retained amount and later impact. Extend the complete cast/work partition; FinishSpell cannot complete while any impact is outstanding. |
| `crates/dmd-rules/src/tactical/hit_reactions.rs::shield_choices` (36), `respond` (197), `order` (247), `cast` (301), `finish_shield` (350); `casting.rs::shield_admission` (221) | Reuse narrow admission/cost/effect helpers. Add explicit typed parent dispatch for missile-target responses; do not fabricate an attack or overwrite `hit_review`. Preserve hit-only paths and their historical shape. |
| `crates/dmd-rules/src/tactical/turns.rs::push_frame` (126), `pump` (291), `choose` (382); `continuations.rs::apply_vitality_from_cause` (958) | Queue one complete impact frame after collection; use normal child drain. Keep current-turn ordering distinct from responding/rolling actors and enforce the actual owner on the new choices. |
| `crates/dmd-rules/src/tactical/work_trace.rs::register` (15), `scopes` (102), `tactical_frame_host_ordering` (196), `validate` (260) | Register every new node and source parent exactly. Independent response and impact authority cannot inherit unrelated area delegation. Validate retired nodes against exact cast/dart/raw identities, not a broad matching-roll search. |
| `crates/dmd-app/src/table_hit_reactions.rs::view` (22); `table_tactical_choices.rs::continuation` (6); `table_tactical.rs::view_with_source_access` | Add a missile-specific private DTO and meaningful occurrence choices. During collection hide generic continuation choices. Keep unrelated complete DTOs/revisions/transcripts invariant through private intents. |
| `crates/dmd-app/src/table_source_control.rs::authorize_tactical` (231); `table_transport_runtime.rs::request_meta` (22), `derive_intent` (89) | Route explicit source-owner missile decisions; guard new Begin/upgrade against unactivated Player-owned sources. Resolve authority from retained respondent/current turn, not client choice.actor. Preserve privileged-first privacy order and accepted-retry lookup. |
| `crates/dmd-app/src/table_transport.rs::presented_view` (235); `table_presentation_history.rs::capabilities` (47), `digest` (90), `validate_history` (405); `crates/dmd-persistence/src/table_projection_store.rs::ProjectionCapability` (47) | Issue separate audience/window/respondent/role capabilities. Add a new missile response input rather than reinterpret accepted HitResponse payloads. SourceCreature allowlist must include it; direct raw canonical handle bypasses reject. |
| `crates/dmd-app/src/rules_restore.rs::command_origins` (805), `validate_origins` (1071), `visit_rules_history` (64) | Collect all admission/order/intent/selected cast/decline/amount/impact causes; semantically authenticate earlier and current images. Preserve pre-tactical anchor and atomic rollback on rejected restore. |
| `apps/desktop/src/components/HitResponsePanel.svelte`, `EncounterPanel.svelte`, `TableApp.svelte`, `table-api.ts`, `tactical-api.ts` | Reuse source actor selection and exact outbox recovery. Add distinct missile controls/labels; support several owned respondents without merging their decisions. Host sees source-safe ordering, never substitutes for a Player-owned response. |

Prefer a new focused `tactical/missiles.rs` module and corresponding domain record
file if this keeps hit reconstruction stable. A retained missile record should bind
the existing cast occurrence, original trigger cause/work, distinct respondents,
explicit response-order/delegation decisions, selected child and completed Shield
proofs, and every `(node, dart index, target, amount key, impact occurrence, completion)`.
The records contain evidence/status, not a second executable queue. Multiple records
must be distinguishable if a later producer introduces nesting; this slice need not
implement that producer. Existing cast records remain the one casting/resource truth.

The partition validator must prove each dart is in exactly one stage: awaiting amount,
amount retained/awaiting impact, currently applying with its children, or completed.
Raw collection is not source-program completion. Each cast has exactly one FinishSpell;
all expected darts exist once; duplicate/missing/out-of-range occurrences fail closed.
Shield child completion binds an explicit hit or missile parent. Fresh Shield casting
metadata, original missile command, accepted amount command and later impact execution
command remain distinct; never rewrite a prior CommandMeta to the new head. Use exact
raw keys and retained causes for source attribution, while lifecycle stamps use the
actual command performing that transition, as `apply_vitality_from_cause` requires.

## Privacy and actual desktop behavior

Show the same public/current-turn coordination stage for every admitted missile
targeting event, irrespective of eligible or accepting Shields. Each target owner
sees only their own source choices. Source-control v2 may have one human owning more
than one targeted actor; every respondent capability remains actor-specific. Selection
may change only that owner's view until a fictional result is actually public.

Use the existing ADR028 participant ranking plus explicit unlisted policy for
response order; order over potential respondents, never over a disclosed eligible
set. Distinct accepted Shields execute in that order after all acknowledgments.
One target's Shield protects all darts directed to it for the duration; another
target's intent cannot spend that Reaction or source pool. Revalidate selected
responses without leaking failure reasons to other audiences.

Impact order is a separate occurrence decision. In this slice the immediate caster
is the current actor and knows its admitted allocation. Label its controls with
stable dart identity and admitted known target; do not expose HP, concentration,
source uses, hidden eligibility or hidden damage totals. Preserve the original
known-target label if a later child changes visibility, without learning new facts.
Any later off-turn producer needs its own knowledge-safe occurrence-order design.

All cast/response/face/order submissions use production opaque handles and current
revision/session ownership. Save the complete original request before transmission.
Restart, changed actor selection or reassignment cannot rewrite an uncertain request;
recover its exact receipt first. Invalid/stale/wrong-owner/changed-body requests leave
the normalized full portable export unchanged. Closing the app works at every pause;
do not invent an EndSession exception for unresolved work.

## Acceptance matrix

| Area | Required concrete evidence |
|---|---|
| Source and amount | Genuine Night Hag level4 six-dart cast; compiler/binding cases for base3 and supported upcast counts; V/S and sight/range rejection before cost; actual source grant/use accounting, no forged PC slot grant. Individual accepted d4 faces retained. |
| Commitment barrier | All allocations fixed before any response; all response decisions/Shield children settled and all dart faces retained before first HP change. Mixed/repeated targets cannot be changed after any face. Opposite private-intent arrival gives the same selected response order. Amount-report order never chooses impact order. |
| Shield resources | Zero/one/two genuine Mage respondents; accept/decline and selected decline; shared-use exhaustion, spent Reaction, full hands/component failure; each accepted cast pays once; one Shield blocks all its target's darts; next own Start expires it; independent later attack-hit Shield still works. |
| Same-time order | Repeated actor with unequal dart amounts; distinct targets; explicitly choose reverse occurrence orders and observe retained order and any meaningful downstream difference. Do not validate an actor-only/default/arrival ordering shortcut. |
| Vitality and concentration | Separate Force instances, resistance/temp HP behavior, concentration saves at source DCs, first failure ends dependent effects without deleting remaining impacts; real HP→0, stable→unstable and death-save failures; remaining darts after death are retained no-effects, never critical failures or corpse HP. |
| Nested integrity | Existing real concentration/effect children drain before siblings. Supported child-induced state changes cannot cancel/retarget remaining strikes. Qualified pure invariant tests may cover future-producer states, but must be labeled and cannot replace actual app evidence. No claim of Ready/Counterspell completion. |
| Authority/privacy | Actual Player-owned Hag/current turn and Mage respondents, including source-only attendance and one human owning multiple actors; Host/foreign substitution rejected. Full unrelated DTO/revision/transcript equality for zero/one/two hidden offers and both arrival orders. Exact-trigger response delegation cannot order unrelated impacts/children. |
| Work/source integrity | Reject forged target/index/raw key/faces/role, omitted or duplicate dart, false completion, wrong parent, altered Shield proof, changed cause/issuer and forged delegation. Preserve strict typed decoding and source reconstruction. |
| Compatibility | Genuine old PR42 bytes unchanged; pre-change flow3 missile and selected hit captures continue independently; all old retries unchanged; numeric0 invalid; old upgrades retained; new live weak Begin refused; non-idle/paid-Ready/foreign/downgrade upgrade rejection writes nothing. |
| SQLite/replay | Real file close/reopen at target collection, private order/intent, selected response, intermediate/final amount, impact choice and concentration-child pauses. Recover exact accepted requests; continue independently restored history. Compare persisted state plus normalized full exports for no-write refusals. |
| Hostile history | Restore genuine current and backfilled earlier images successfully first; then corrupt only an earlier missile/response/impact image and reject atomically with the destination unchanged. Current correctness cannot hide forged retired work. |
| Desktop/production | Actual source selection, repeated-target allocation, private response and occurrence-order controls, physical faces and lost-ack outbox after ownership/session changes. Packaged app restart demonstration through the new path; mocked UI tests supplement real commands. |

Prefer two focused real app scenarios: a multiple-owner/multiple-target Shield/privacy
case and a repeated-target death/concentration case, using existing genuine builders.
Use `crates/dmd-app/tests/support/table_night_hag_cases.rs::finish_darts` (421) and
its test at491 as the old-behavior reference; do not erase its historical evidence
when adapting fresh-live drivers. `support/table_hit_cases.rs` and `table_hit_driver.rs`
provide reviewed source/retry helpers. Proposed new test module:
`support/table_missile_cases.rs`, registered in `tests/table_loop.rs`; rules cases
may live in proposed `tests/tactical_attacks/missiles.rs`.

Use cold checkpoints for every distinct persisted stage and each material owner
transition. Avoid redundant inspection-only opens: compare already exported current
images after real command/restore validation. Keep genuine cold resume, independent
semantic restore/continuation, exact retry and full no-write comparisons. Record
actual timing; do not claim a speedup, weaken integrity or parallelize heavy builds.

## Domain/rules implementation checkpoint - 2026-09-27

The resumed writer fetched live main, read AGENTS/this plan/ADR028 and rechecked
SRD SHA256 `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
The previous writer's uncommitted type scaffolding was preserved and completed
into a first engine checkpoint. It is not yet compiled, tested, pushed, or accepted.
Root currently owns the sole heavy slot for PR45 verification.

- Add `ShieldMissileV1`/flow4 and explicit hit support for3/4. New live Begin and
  targeted upgrades require4; historical typed upgrade-to3 remains replayable,
  unit upgrade remains1-to-2, and old flow3 hit decisions remain completable.
- Add omitted-when-empty `resolution.missiles`, typed response/amount/impact
  evidence and existing-frame work kinds. No source program/fingerprint, old
  export, old raw-role tag or legacy scheduler is rewritten.
- The real commit binds one respondent per distinct original target. Private
  intents and explicit response order/delegation precede freshly owned selected
  Shield cast/decline. Each child retains its source completion evidence.
- The amount phase retains all original per-dart physical requests without HP
  mutation, including Shield-protected darts. One complete occurrence frame then
  applies separately chosen impacts with ordinary nested consequences. The actual
  current-turn owner controls impacts; response delegation cannot transfer that
  authority. Original amount acceptance remains the vitality cause.
- Source/decision/retired-node and live partition validators check exact cast,
  target, raw key/request/face, response owner/order, Shield source effect, and
  amount/impact completion identities. Historical semantic replay remains required
  to authenticate prior geometry/knowledge and transitions.
- Six source-built reducer/replay cases are authored for all-faces/reverse-order,
  actual Mage Shield with both intent arrivals, selected decline/delegation,
  concentration child drain, caster death with committed no-effects, and hostile
  target/raw/barrier/retired-node mutations. These are initial-image rules cases,
  not actual application creation or SQLite evidence. They have not run.
- Shared `tactical_attacks` fresh fixture now begins4. Minimal app work-choice
  edges suppress generic private-stage work and label impact occurrences by their
  committed known dart/target. Full DTO/opaque transport/source control/UI/history/
  restore integration and remaining fresh-current fixtures are still pending.

Next: a separate writer branches app/UI integration from this coherent checkpoint;
this writer retains domain/rules and rules tests only. Complete static independent
review, adapt remaining fresh rules fixtures to4 without touching genuine old
captures, then compile/fix and run focused rules/old-flow continuations when root
releases the slot or after coherent app integration permits draft CI. The entire
acceptance matrix above remains mandatory. Integrate verified PR45 main (including
PR44 aftermath) normally before acceptance, explicitly admitting aftermath4 while
preserving2/3. No flow4 merge or Gate4 completion is claimed.

## Static follow-up after the engine checkpoint

Root's independent static review found a genuine ownership distinction: after the
penultimate dart's concentration child, the target's save command can automatically
resume the final singleton dart. That lifecycle command must not impersonate the
turn owner's material selection. Each dart now records optional `selected_by`
separately from actual `completed_by`; only explicit sibling choices carry a turn-
owner selection. An automatic impact must be the sole remaining occurrence, and
its retained completion must be the final source occurrence. A sustained-
concentration regression is authored through the fifth foreign-owned save and the
sixth automatic impact, then the sixth save. This correction is unrun.

The focused source reducer matrix now has nine authored cases, adding per-instance
Force resistance/temp-HP and one failure per zero-HP dart through death/no-effect
completion. A separate pure source compiler test checks all supported slot levels
keep individual d4+1 Force and source dart counts. This is not a claim that prepared
Magic Missile is available through current character creation. Remaining fresh rules
Begin helpers now request4; the old unit-upgrade test additionally replays typed3
unchanged and proves fresh typed3 is rejected before an explicit settled upgrade4.
Strict validation also rejects a current flow4 cast with a removed target barrier,
and extra/mismatched retired amount or impact nodes. None of these changes has been
compiled or run while root owns the shared build slot. App integration receives the
new `selected_by` origin through an explicit normal merge.

## Retired casting identity review follow-up

A second independent root review identified that casting ordinals are reserved
outside the traced work allocator. Per-respondent validation alone could not prove
that two separately retained completed Shield proofs had different ordinals.
Flow4 now validates one shared namespace across live casts, completed missile
source casts, every completed missile Shield, retained hit-Shield completion, and
all traced work occurrences. Earlier execution semantics are unchanged. A tenth
rules case constructs a real source Mage Shield completion first, then performs
explicitly labeled structural proof/work collision corruptions and requires the
namespace-specific refusal. It is authored and formatted only, not run.

## Multiple source respondents

An eleventh authored rules case extends the explicitly isolated initial image with
a third genuine source profile: one Player-owned Night Hag and two independently
owned Mages. It accepts both private intents in both arrival orders, requests the
reverse respondent order explicitly, casts both real Shields, verifies distinct
reserved source ordinals and chronological response execution, retains all six
faces and completes both impact orders without damaging either protected Mage.
The existing source-builder helper now accepts an explicit actor/controller/index
for this third initial profile. No actual app creation, privacy, recovery or runtime
pass is claimed by this additional unrun reducer/replay case.

## Development checkpoint and exact next steps

The branch is created from the stated development base. This plan and ADR028 record
the approved per-dart/all-faces/normal-child-drain interpretation. The initial commit
79b17bf changes documentation only. Root's isolated generator at06045b2 ran the actual
Night Hag scenario successfully:1/1 in500.88s, and the owned Shield scenario1/1
in1427.43s, default Windows stack. Their logs are
`tooling/shield-diagnostic-06045b2-capture-missile.log` and
`tooling/shield-diagnostic-06045b2-owned-shield.log`. All four genuine exports are
imported unchanged, with lengths/hashes and original authority recorded in
`crates/dmd-app/tests/fixtures/shield-hit-v1-06045b2.md`. No build has run from this
branch, and nothing is pushed at this checkpoint.

The authorized compatibility-only module is now authored and rustfmt-formatted:
`crates/dmd-app/tests/legacy_shield_hit_v1_replay.rs`. The exact85cff17 module and
all four captures were copied into isolated diagnostic successor
`f9698e1761ee0485c4cf0ff593f89a03f016302b`. A complete diff confirms identical
production source/dependencies/content; only read/export hooks in the two original
scenario drivers, docs and historical fixtures differ. Its separately selected
baseline test compiled in25.99s and passed1/1 in299.50s, exit0, default Windows GNU
stack, jobs1/incremental0, recorded in `tooling/shield-baseline-f9698e1.log`.
This source-equivalent capture check does not replace final-head verification.
Missing named captures fail
explicitly; no fabricated file or conditional skip is permitted. One separately
selectable baseline test covers all four captures' byte/typed roundtrip, actual
file restore/resume and every original request/response binding's exact retry,
with normalized full-export equality. Four separate continuation cases preserve
independent cold paths: first/after-two missiles toHP7, selected owned Shield to a
miss, and natural20 post-cast damage toHP75 with original issuance cause. Every new
accepted continuation also proves its own lost-ack retry and changed-body no-write
refusal. Preserve original transport1/2 and source/PC/Host channels.

The actual fourth image exposed one test-only expectation, corrected in a separate
commit before runtime development: `attack.damage_roll` remains None while its raw
request is pending and is assigned only upon acceptance (`attacks.rs::finish`).
The fourth continuation now asserts the absent accepted record, exact pending
work/key/request ID and original cause, then the accepted record after continuation.
The correction is formatted/static-inspected but uncompiled/unrun. No capture or
production behavior changed. The exact85cff17 baseline test above is unaffected.

The capture gate passed: both unchanged generator scenarios, frozen genuine bytes
and the separately selected baseline import/roundtrip/receipt test. Root authorized
the approved flow4 domain/rules work after committing the artifacts and the stated
test correction. Heavy commands still require root's shared-slot authorization.
The four full continuation cases remain mandatory focused/canonical acceptance;
they may run after versioned development begins so the one heavy slot is used
efficiently. Their authoring does not imply execution, and failing continuation
semantics cannot be waived. Production runtime remains unchanged in this test stage.

1. Finish both original flow3 capture scenarios in the isolated generator. Root must
   record their actual passing results, baseline restore of all four exports, exact
   hashes/lengths and original request-binding provenance. If either fails, diagnose
   and fix the real prerequisite on its owning branch; do not fabricate a fixture or
   start flow4 runtime work around the failure. Preserve the pre-tactical anchors.
2. Import the reviewed immutable artifacts and manifest into the new branch; add
   genuine flow3 replay/continuation assertions before modifying runtime semantics.
   Freeze the second-amount, selected Shield and post-cast damage exact envelopes.
   Fresh flow4 production drivers remain separate from those historical tests.
3. After the capture gate passes, implement the next bounded domain/engine change:
   explicit flow4 dispatch, retained missile/cast/target identities, typed Shield
   parent attachment and a complete amount/impact partition in the existing frames.
   Preserve all old wire bytes and raw-role identities; no table/UI acceptance is
   claimed by these types alone. Reconcile any simultaneous PR45 correction as an
   explicit normal integration, keeping one writer on this branch.
4. Implement the uniform target Shield collection path, selected-controller source
   admission/cost and typed child-parent dispatch. Keep hit behavior/captures passing.
   No branch with a new version alone is feature acceptance.
5. Implement amount collection and the full occurrence impact frame, causal damage
   attribution, source/retired-work validation and ordinary nested consequences.
   Add focused rules acceptance, especially death/concentration and old continuation.
6. Wire authority, private DTOs/capabilities, v1/v2 history, source transport, restore
   origins, desktop forms and outbox. Register the actual SQLite tests rather than
   leave uncompiled support files; adapt only fresh-live fixtures to the new current
   executor and keep raw helpers single-command.
7. Run focused and canonical verification only when the coordinating writer releases
   the shared heavy slot. Use one build job, `CARGO_INCREMENTAL=0`, normal Windows
   test stacks and measured logs. Do not increase stack or relax lint to hide defects.
   Current valid command targets, with proposed future filters clearly identified:
   - `cargo test --locked -p dmd-rules --test tactical_attacks missiles`
     (the `missiles` filter is to be introduced by step5).
   - `cargo test --locked -p dmd-app --test table_loop missile`
     (new scenarios plus inspect which existing names match).
   - `cargo test --locked -p dmd-app --test legacy_reactions_v1_replay`
     plus the newly registered genuine flow3 replay tests.
   - `./scripts/verify-fast`, then `./scripts/verify` from repository root.
   - In `apps/desktop`: `npm run check`, `npm test`, `npm run build`;
     run the repository's native packaged path and actual restart acceptance.
8. Before final acceptance, fetch verified PR45 main and any intervening PR44/main
   changes, reconcile them and rerun the required checks on the resulting exact head.
   Independent reviewer inspects the complete exact head, old/new queue dispatch,
   authority/privacy and all recovery tests. Fix concrete failures on new heads;
   checks on an older SHA are not final-head evidence. Require every current CI job
   from `.github/workflows/ci.yml` and `desktop.yml` (currently six checks), final
   source/tree parity, protected expected-head merge and fetched main verification.
9. Update the bounded plan, reaction umbrella and coverage ledger with actual counts,
   SHA/PR/checks/artifacts, measured limitations and receiving Gate4 work. Resume the
   next authorized Gate4 slice; pause only at the complete gate's evidence-based
   closeout as required by `docs/checkpoints/gate-execution-protocol.md`.

## Risks and honest stopping criteria

The largest implementation risks are old flow3 pauses being blocked by the new
current-version guard; duplicate cast/dart partitions; resurrecting/erasing work
after a child consequence; reusing attack-only parent/cause assumptions; leaking
private offers through ordering controls, counts or revisions; and borrowing area
or earlier-trigger delegation. Each has explicit acceptance above.

All-faces collection is an explicitly approved starting implementation policy,
not a source quote. Per-dart damage instances and child drain are likewise the
recorded adjudication. The genuine-capture prerequisite still blocks runtime code
until its actual evidence is frozen. Broad atomic
aggregation or deferred vitality would be a separate design change, not a small
optimization. A source/model incompatibility must be resolved in the plan and tests,
never hidden by a broad validator exception or synthetic historical fixture.

The imported generator evidence above belongs only to06045b2; the separate baseline
pass belongs to source-equivalent f9698e1 and exact85cff17 test module. The four
continuation tests remain unrun. Existing Shield review/test evidence remains
attributable only to its exact recorded head. Counterspell, Ready release,
off-turn missile integration and the full Gate4 production encounter remain open.
