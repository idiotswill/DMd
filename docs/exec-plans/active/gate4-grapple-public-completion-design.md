# Gate 4 — Approved Grapple ownership implementation design

Status: APPROVED FOR IMPLEMENTATION by root on 2026-10-05. Companion to
[public completion plan](gate4-grapple-public-completion.md).
The following is the exact reviewed external design; its historical read-only
status describes the design review, while the companion plan records the current
sole-writer source allocation and all execution restrictions. Source SHA256:
`06de4ac493fd290e08e59725a7e995d1596b59ced7b9aa0b1ac1e90d8a2a5c32`.

# Grapple replay ownership: resolved design addendum

Status: READ-ONLY PROPOSAL, 2026-10-05. No source allocation, branch, dependency
receipt, compiler, database, native run or publication is implied. This resolves
the ownership choice left open in the 387-line public-completion proposal. It
does not activate Grapple or replace that proposal's mechanical/UI acceptance.

Input: clean `gate4-grapple-attack-read-context` at
`ade8e93b450d6e02027afc7615a555db16b2915d`, tree
`a6097d485244a2ed5443765fb4ee40c34625892b`. The original completion map SHA256 is
`e539c6c4ecd79f57f830cecb7fa6dbd73ae2f452cee0dff1cd6d0d199b260ef2`; the prior
public-completion proposal SHA256 is
`46cf95a27407aac065859eadb59ac9f1eff393d1bc42274fc702bc10ea0e64cb`.
Root owns the queued exact-ade focused82 execution. This addendum reports no run.

## Decision and dependency boundary

Use a **rules-owned whole CampaignState continuity owner**, with only closed
typed operations crossing from app. Keep the pure composed TableEvent reducer
in `dmd-app/src/table_engine.rs`: it interprets original text, translates the
original TableAction, invokes the rules-owned operation, constructs the original
outer event, checks historical equality, and accepts the prepared step. Move
the actual state-mutating match arms and their admission checks into
`dmd-rules/src/table/reducer.rs`. There must be one implementation of each state
mutation, shared by ordinary legacy and authenticated execution.

This is a split between envelope composition and state mutation, not two
independent state reducers. The app must not mutate a candidate afterward.
Neither layer accepts a state replacement, arbitrary field patch, closure that
mutates state, serializer-created proof, ambient `grapple_enabled: bool`, or
caller-supplied mechanically derived result.

Important correction to the initial avenue investigated: although Cargo alone
would permit rules -> conversation without a cycle, `scripts/check_boundaries.py`
line 18 explicitly forbids it. Keep that guard. Keep `LocalText`,
`interpret_local_text`, declaration recursion and conversation routing in app /
conversation. Do not move a parser into domain or introduce a neutral crate to
evade that restriction. No new crate or Cargo dependency edge is required.

Current graph remains: app -> rules / conversation / persistence / domain;
rules -> domain; conversation -> domain; persistence -> domain. Existing
serde/uuid dependencies remain sufficient. Rules never imports app, persistence,
SQLx or a transport/presentation DTO.

## Existing constructors and exact migration boundary

1. `tactical/grapple/execution.rs:7,15`: crate-private
   `GuardedGrappleExecution<'pack>::new(CampaignState, &'pack RulesPack)` accepts
   an ordinary active flow5 baseline, refuses existing Grapple records and has
   no Clone/Deserialize/mutable accessor. Its only constructor callers are the
   two private execution test modules (`execution/tests.rs`, `attack_tests.rs`),
   including cloned/decoded/exported-state refusal controls. It is not an app
   history constructor. Preserve those controls and their old baseline meaning;
   no public alias or widened constructor may admit serialized modern state.
2. `ReadContext::ordinary` is currently called by kernel validation, tactical
   validation/turn validation, failed-save validation, casting/Shield and Savage
   helpers. `ExecutionContext::ordinary` is used by public tactical dispatch,
   continuations, turns, Grapple/equipment/save/lifecycle adapters. Keep ordinary
   as the no-authority path; it must continue to reject imported Grapple records.
3. The only production guarded-command construction currently lives in private
   `GuardedGrappleExecution::apply`, plus deliberate private test construction.
   Its candidate identity pointer is compared, never dereferenced. Extend this
   same producer/read mechanism internally, rather than exporting its fields.
4. App `resolve_table` / `replay_table` currently accept only raw state. Their
   production call sites are `table_transport_runtime.rs:874,1133`,
   `rules_restore.rs:298`, and `RulesReplayApplier::apply` at
   `rules_runtime.rs:335`. These are all accounted for below.
5. `RunnableCampaign` is a Clone-able read DTO. It must not become an execution
   capability. `RulesReplayApplier::new(pack)` is a stateless adapter with
   `apply(&self, &mut CampaignState, event)`; it cannot reconstruct continuity
   from a selected recent snapshot. It remains a legacy ordinary adapter.

## Concrete public/private Rust surface

Names below are proposed source names, not claims that these types exist.
Store the small, immutable `RulesPack` by value in the owner (it already derives
Clone). This avoids an app wrapper borrowing its own pack. ContentCatalog still
resolves the exact pack and verifies installed bytes before construction.

```rust
// dmd-rules::table; fields private, no Clone or serde implementations.
pub struct CampaignExecution {
    image: ClosedImage,
    pack: RulesPack,
}
struct ClosedImage { state: CampaignState }
pub struct TableRead<'a> {
    image: &'a ClosedImage,
    pack: &'a RulesPack,
}
pub struct PreparedTableStep<'a> {
    owner: &'a mut CampaignExecution,
    candidate: ClosedImage,
    produced: TableProduced,
}
pub struct AppliedTableStep<'a> {
    before: ClosedImage,
    after: &'a CampaignExecution,
    produced: TableProduced,
}

impl CampaignExecution {
    pub fn from_original_anchor(
        anchor: CampaignState, pack: RulesPack,
    ) -> Result<Self, TableError>;
    pub fn read(&self) -> TableRead<'_>;
    pub fn prepare_table<'a>(
        &'a mut self, meta: &CommandMeta, operation: TableOperation<'_>,
    ) -> Result<PreparedTableStep<'a>, TableError>;
    pub fn prepare_table_replay<'a>(
        &'a mut self, meta: &CommandMeta, operation: TableOperation<'_>,
        expected: ExpectedNested<'_>,
    ) -> Result<PreparedTableStep<'a>, TableError>;
    pub fn prepare_legacy_replay<'a>(
        &'a mut self, event: LegacyRulesEventRef<'_>,
    ) -> Result<PreparedTableStep<'a>, TableError>;
    pub fn into_state(self) -> CampaignState; // consumes authority
}
impl PreparedTableStep<'_> {
    pub fn before(&self) -> TableRead<'_>;
    pub fn after(&self) -> TableRead<'_>;
    pub fn produced(&self) -> &TableProduced;
}
impl<'a> PreparedTableStep<'a> {
    pub fn commit(self) -> AppliedTableStep<'a>;
}
impl AppliedTableStep<'_> {
    pub fn before(&self) -> TableRead<'_>;
    pub fn after(&self) -> TableRead<'_>;
    pub fn produced(&self) -> &TableProduced;
}
impl TableRead<'_> {
    pub fn state(&self) -> &CampaignState;
    pub fn validate(&self) -> Result<(), TableError>;
    pub fn query(
        &self, issuer: CommandIssuer, query: &RulesQuery,
    ) -> Result<RulesAnswer, RulesError>;
}
```

`TableProduced` contains the existing message/mechanics, optional actual
RulesEvent and TacticalEvent, and optional pure `TableSessionChange` with the
existing `Start { session }` / `Replace { expected, next }` values. App converts
that last enum exhaustively to persistence's unchanged `SessionChange`.
`ExpectedNested` contains optional borrowed RulesEvent/TacticalEvent, not a
callback or expected state. `LegacyRulesEventRef` has only Rules/Tactical event
variants; both are refused if the owner state has a table. It cannot enable
Grapple. It exists only to retain the current non-table original replay route.

`from_original_anchor` runs complete ordinary semantic validation and the
existing pre-tactical anchor predicate: no Grapple records, encounter history,
source-access activation, flow, tactical effects/inventory/recovery/creatures.
Add the proposed Grapple activation and received physical-creation provenance
to this predicate explicitly when those sources are received. No schema/current
marker, exact head number or claimed source pin exempts the original anchor.
The rules constructor validates a candidate baseline; only app's full export
walk can establish that it is the campaign's actual original retained anchor.

`prepare_*` borrows the owner mutably but leaves its image unchanged. It creates
one boxed candidate at a fixed address, derives the exact next sequence, admits
the typed operation, runs the shared producer, checks observed evidence and
complete post-state semantics, then seals the candidate in `ClosedImage`.
The transient GuardedCommand is dropped before returning; no raw pointer lives
in the owner, a read handle, or an across-await result. No input reference/meta
borrow is stored; produced provenance owns its normal cloned CommandMeta.

App then forms TableEvent from the *original* action and `TableProduced`.
Historical outer/child mismatch drops the prepared step without advancing the
owner. `commit` is infallible, consumes the prepared step and moves the previous
ClosedImage into AppliedTableStep while installing the candidate. This provides
both exact historical images without cloning or re-authenticating old state.
The step's borrow prevents another command until its before/after validation is
finished and it is dropped. SQL commit remains a separate app responsibility.

`TableRead` can only be issued from an owner/prepared/applied closed image. It
cannot outlive that image or an intervening mutable operation. A state clone,
serialized value, matching ID/sequence or matching hash does not construct one.
The contained full state has already passed a producer-preserving transition.
Internal `ReadContext` gains a closed-image read variant in addition to ordinary
and current-candidate reads; fields/constructors remain private. Recorded-roll
membership is checked against that exact certified image, and all other kernel,
tactical, source/equipment and table semantic checks still run.

## Closed typed table operations, including nonmechanical ones

`TableOperation<'input>` is an exhaustive borrowed enum, not serialized. Each
variant carries only the original action's typed input, plus a TableIntent for
Declare/Correct. IDs and revision numbers retain their existing types. The
following are the complete current seventeen arms and the one proposed arm:

| Operation | Input and mutation authority |
| --- | --- |
| UpdateContract | `&TableContract`; same host/no-session/no-pending checks and immutable mechanical house rules |
| AddPlayer | PlayerId, `&str`; same host/setup/unique-ID checks; inserts exactly one player |
| CreateCharacter | three original IDs, `&CharacterCreationInput`; actual builder, world/mechanical/profile insertion, original nested RulesEvent |
| PrepareEquipment | CharacterId, `&[ItemId]`; actual existing receipt/materialization producer |
| CreateCreature | `&TableCreatureCreation`; actual original/full-pin source producer with Live/Historical admission distinction |
| EnableSourceActorAccess | `&[TableSourceAdoption]`; complete exact adoption set and existing settled/current-host checks |
| SetSourceCreatureController | EntityId, CreatureController; actual schedule producer, ownership and attendance checks |
| Tactical | `&TacticalAction`; existing active/closed-session checks, actor authority and shared tactical dispatcher |
| PrepareBattlefield | `&TableBattlefieldSetup`; actual source-derived placement/Establish producer, never a supplied encounter state |
| StartSession | ID, `&str`, `&[SessionParticipant]`; actual profile, retained-death/aftermath attendance checks and Start session delta |
| EndSession | no proposed state; original finished/aftermath boundary and exact Replace session delta |
| SetSituation | `&TableSituation`; host/current-session checks and refusal of invented challenge resolutions |
| Declare | `&str`, `&TableIntent`; trusted player binding, idle checks and new origin-bound pending proposal only |
| Correct | CommandId, revision, `&str`, `&TableIntent`; exact owner/revision, checked increment, new origin only |
| CancelDecision | CommandId, revision; clears only the exact owned pending declaration |
| Adjudicate | CommandId, revision, RollRequestId; actual pending intent supplies existing derived RequestTest/SecondWind, never caller totals |
| SubmitPhysical | RollRequestId, `&[u16]`; actual pending roller/dice source and existing accepted challenge-result derivation |
| EnableGrappleAccess | no supplied marker/proof; host-only explicit settled flow5 activation derives its origin from CommandMeta |

The truly nonmechanical arms are UpdateContract, AddPlayer, Start/EndSession,
SetSituation and Declare/Correct/CancelDecision. Do not call creator/equipment,
source-control, battlefield, Adjudicate or SubmitPhysical a nonmechanical rebase.
They invoke their actual producers and must preserve all earlier mechanical
evidence. Current exact conditions/messages remain the initial behavior; any
new active-grip restriction must be named in the plan and tested, not introduced
as a side effect of this relocation. Source ownership changes retain the exact
existing settled-work boundary and actual SetController producer. They change
current control and its origin, not historical grip/source proof identities.
If proof validation currently conflates those identities, fix historical/current
comparison rather than clearing a grip or silently banning that outer action.
Accepted old-request replay still precedes current-controller admission.

App translation has one small match for Declare/Correct to call the existing
deterministic interpreter against `owner.read().state().table.situation`.
It owns that proposal locally, drops the read borrow, then lends the proposal
to prepare_table. The remaining arms are mechanical field-for-field mapping.
Rules validates proposal structure and original domain constraints; text or an
unresolved proposal cannot directly become mechanical results. Crucially the
original outer event still stores text, not a new authoritative interpreted
payload. Full staged replay independently reinterprets that saved text and
compares the entire generated event AND resulting state. A caller-supplied
different intent therefore cannot become an accepted app history. This is the
existing typed-proposal/trusted-app boundary, not proof supplied by the parser.

## Source moves and producer preservation

Move only pure input structs from `table_protocol.rs:106-156` into rules::table:
TableCreatureCreation, TableBattlefieldSetup, TableCreaturePlacement and
TableCharacterPlacement. Re-export the identical types from app; keep their
serde tags/defaults/omissions and every original TableAction/TableEvent byte.
TableAction, TableEvent, TableOutcome, TableViewer and all view/transport types
remain app-owned. Add no dependency on an app presentation type in rules.

Move the current table_engine mutation arms, host/setup/idle/session/player/
pending checks, plus pure `table_creatures::create`, `table_equipment::prepare`
and `validate_character_equipment`, `table_tactical::prepare`, and source-control
activate/assign/settled/adoption/ownership/attendance authorization into
rules::table internal modules. Keep their view/catalog formatting functions in
app. App wrappers may preserve old helper names for existing tests, but must
delegate to the one shared reducer or read helper. No copied second match.

The ordinary stateless resolve/replay wrappers remain available to old callers
and the stateless replay adapter. They use the same reducer with ordinary reads
and retain explicit refusals for Grapple activation/records. All modern table
operations use CampaignExecution. Preserve the private GuardedGrappleExecution
control constructor; do not make its permissive active baseline the public
owner constructor. Shared tactical dispatch/producer observation stays one body.

Separate certified retained history from *current* Grapple action eligibility.
The present private `require_execution(flow5 active)` and `validate_delta` calls
to `flow(before/after)` cannot be used indiscriminately for every table action:
anchors have no flow; finished/replaced encounters retain accepted Grapple rolls.
Validate immutable raw/proof/cut/history inheritance for all operations, and
require current flow5 and current body/hand/turn constraints only for operations
which use them. Finish/replacement must use the real archival producer and
compare inherited trace/proof bytes, not waive the delta check. Outer-only arms
must preserve all mechanical evidence exactly. Nested ordinary Rules producers
must also use context-aware validation and actual roll observation where legal;
do not append an arbitrary suffix from a returned mutable state as 'evidence'.
Existing kernel restrictions on active tactical effects/flow remain enforced.

## One full application authentication walk

Proposed app-private result and sole constructor:

```rust
pub(crate) struct AuthenticatedTableHistory {
    execution: dmd_rules::table::CampaignExecution,
    presentation: PresentationHistory,
}
pub(crate) fn authenticate_history(
    export: &CampaignExport, pack: RulesPack,
) -> Result<AuthenticatedTableHistory, String>;
```

No Clone/serde or state/proof setter. The constructor owns the existing
rules_restore/presentation_history full walk; it returns only after all checks.
Remove the current recursive ownership arrangement (presentation validation
calling the semantic visitor and callers separately validating raw current).
Use a concrete private frame consumer, with no caller callback:

```rust
// dmd-app::table_presentation_history; owns all presentation cursors/ledgers.
struct HistoryVerifier<'export> {
    export: &'export CampaignExport,
    history: PresentationHistory,
    // Existing bootstrap/next-record counters and consumed binding sets.
    legacy_selection: Option<LegacySelectionCheck>,
}
impl HistoryVerifier<'_> {
    fn anchor(&mut self, read: TableRead<'_>) -> Result<(), String>;
    fn step(&mut self, before: TableRead<'_>, after: TableRead<'_>,
            row: &EventJournalRow) -> Result<(), String>;
    fn finish(self) -> Result<PresentationHistory, String>;
}
```

The rules_restore loop owns CampaignExecution, session verification and the
snapshot map; it calls this consumer with each AppliedTableStep's immutable
reads. `LegacySelectionCheck` is a private, closed app record containing the
original audited CommandMeta, requested channel and a matched flag. Its one
operation compares player/character/actor at that original head exactly as
the existing recovery closure does. It cannot return state or authorize a
transition. A private `authenticate_legacy_recovery(export, pack, request)`
constructs this check from the original matching audit/observation, calls the
same complete walk, and returns the existing LegacyTableAcknowledgement only
when both history and selection pass. Ordinary authenticate_history selects no
legacy check. Retire the generic visit_rules_history closure after these two
actual uses (presentation and legacy selection) are migrated. This is a fixed
read-only consumer, not a closure-controlled state or authority callback.

Keep these checks in their original strength and order where dependencies allow:

1. Upgrade and structural export validation, strict codec/schema boundaries,
   campaign/sequence/time/identity checks for current and every snapshot.
   Ordinary snapshots still receive ordinary full semantic preflight. A modern
   snapshot is not individually promoted: its full semantic check is deferred
   until it equals the replay-owned image at that exact head.
2. Original earliest anchor refusal, supported event/command kind+version,
   unique command/event identity, exact audit metadata/payload joins, one event
   per accepted command, journal contiguity, raw-table-bypass refusals, all
   retained ruling and origin checks against the same original anchor.
3. Replay each typed event through the owner and app envelope composer. Exact
   nested event and complete outer event equality must precede step acceptance.
   Validate every generated state with its owner-issued read, check row time,
   then compare every snapshot at that sequence for whole-state equality.
   The equality plus full owned semantic validation supplies the snapshot's
   full semantic validation; decode-only validation never substitutes for it.
4. Keep session identity reuse/expected previous image/status rules and the
   exact final session-ledger comparison. Preserve current-state whole equality.
5. At every original before/after head, run immutable audience/observation,
   bootstrap/revision/capability/DTO-digest and exact request/response binding
   checks. Source/controller changes must use original historical images.
   Every stored projection/observation/binding must be consumed/accounted for.

Snapshots without their prefix, matching snapshot/current tampering, or state
copied from a valid different history still fail. A raw prefix that generates a
Grapple event outside a table fails. A successfully validated pure event prefix
alone is not AuthenticatedTableHistory: audits, sessions and presentation must
also succeed. No SQL write precedes restore preflight.

## Complete live/read/query/replay caller routing

| Existing caller | Required route |
| --- | --- |
| protocol_pack (transport:1020) | Split content resolution from authority. Decode only to resolve the exact campaign content; authenticate the complete transaction export and return the wrapper. Never raw-validate modern current first. |
| submit_presented_table (1087) | Begin current transaction, export, authenticate once, check exact saved binding first, then current revision/controller/opaque capability, compose+prepare+commit owned step, write state/event/projection/binding, re-export the **complete staged transaction** and authenticate again, then commit SQL. |
| execute_legacy_table (812) | Same owned transaction route; preserve old protocol/retry contract and legacy inability to smuggle a new activation. No cloned state promoted into an owner. |
| observe_legacy_table (942) | Query owner-issued read; no game-head step. Persist original observation/projection/binding and repeat complete staged-export authentication before SQL commit. |
| presented_table_view (1061) | Build/bootstrap presentation only from authenticated owner head plus its validated history; observation/revision changes do not mint gameplay continuity. |
| table_source_control_options (557) | Exact wrapper head; original source ownership/adoptions and authorization queries. |
| table_creature_options (590) | Authenticate history as currently required; read-only catalog output uses the owner's exact pack. |
| table_roll_options (618) | Wrapper read for pending request, source ownership and context-aware Savage dice. |
| recover_legacy_table_request (688) | Validate all history/bindings first. Recover from exact borrowed historical frame, not a separately decoded snapshot; return a request DTO only. |
| table_view (table_runtime:403) | One consistent export/read transaction for state, events and observations, then authenticated projection; remove the independent open + later journal/pagination race. Keep the old public read result shape. |
| submit_table_text / propose_observation | Interpret the owner-head situation; character/PendingRoll questions use TableRead::query. A raw RunnableCampaign clone supplies no capability. |
| read_host_situation / character_creation_options | Open/authenticate original history; return ordinary copied DTO/catalog data, never an owner or proof cached for a later command. |
| query_rules (rules_runtime:122) | For table campaigns load one authenticated export and call TableRead::query; ordinary non-table path keeps existing validation. |
| open_campaign / resume_campaign (lib:135,146) | Resolve exact content and authenticate a consistent export before returning runnable state. Keep RunnableCampaign a read DTO; keep non-rules lifecycle behavior. |
| create_campaign / create_table_campaign | Complete ordinary validation and strict modern-authority refusal at creation. Original initial state only; explicit accepted producers establish later authority. |
| restore_campaign (lib:153) | Upgrade/resolve exact content, authenticate complete export before raw restore transaction. Reuse the validated pack and expected full image afterward; no fallible new file read or bare raw semantic validator after commit. |
| make_runnable / validate_rules (lib:184,204) | Separate ordinary creation validation from wrapping an already authenticated exact returned image. No `is_modern` boolean skip; wrapper consumes verified internal result or compares exact returned raw state with it. |
| replay_rules (rules_runtime:44) | Table route uses the complete original-anchor export walk and returns the resulting state. Do not use latest-snapshot replay for modern authority. |
| RulesReplayApplier::apply (294) | Legacy ordinary behavior retained; explicitly refuse new activation/records instead of creating an owner from its mutable snapshot argument. |
| execute_rules / execute_tactical / raw digital adapters | Preserve current table bypass refusals. No public raw shortcut gains modern authority. |

`export_campaign_in_transaction` already exists. If open needs its lifecycle
metadata at the same head, add a small persistence read-in-transaction adapter
sharing current lifecycle load code; do not modify recovery semantics or cache
a proof across different exports. Raw persistence APIs remain content-agnostic.

`project_table`, frozen v1/v2 projectors, `table_tactical::view/view_v2`, their
attack/casting/Shield/area/movement/OA option helpers, source/equipment views,
`accepted_event_visibility`, `build_record`, `validate_record`, current_view,
and transport event-binding validation must receive the borrowed authenticated
read when they invoke semantic rules helpers. Add context-aware query/planner
entry points that reuse the existing implementations. Existing raw wrappers
stay ordinary and cannot upgrade state to a read capability. Historical v1/v2
output logic/digests remain frozen; v3 is a distinct planned presentation path.
Read-only indexing/formatting may borrow `read.state()` directly, but any call
which normally validates source/rolls/hands must pass the read context onward.

## Why not a detached continuity proof

A token bound only to campaign/head permits substitution of a different state
at the same head. Adding a serialized-state hash ties a token to bytes, but
does not provide an authenticated way to advance it across app-owned session,
situation, setup, controller or declaration mutations. An `advance(proof,
new_state)` or 'nonmechanical changed fields' callback would be the exact state
replacement bypass we are trying to avoid. A proof that re-executes a closed
typed reducer on every step can be safe, but then it needs this same reducer
and duplicates state ownership/comparison. The whole-state owner eliminates
the extra state/proof pairing and makes borrow lifetimes enforce exact reads.

The chosen owner therefore contains the nonserializable continuity internally;
the app wrapper adds complete persisted-history authentication. No digest or
marker becomes authority by itself. The transient before image is owned only
for validation and disappears when the step is dropped.

## Required design/implementation evidence and remaining limits

Minimum coherent graph changes are: rules table owner/typed reducer plus shared
kernel/tactical read plumbing; pure input re-exports/session-delta conversion;
app table envelope composition; one semantic+presentation history constructor;
all caller routing above; and later version3 transport/UI integration. No new
crate, dependency-guard exception, SQL migration for ownership or event-version
rewrite is justified by this bridge. Actual UI/version3 still needs the exhaustive
receiving-tree audit requested by root; Ground/Offstage only1/2 is root evidence,
not a substitute for the audit after normal receipt.

Add genuine app tests for: marker -> actual grip -> safe outer-only action ->
cold file reopen -> replay/portable restore -> next accepted grip action; full
before/after audience/binding validation across controller/session boundaries;
finished/replaced encounter retaining exact raw/proof history; rejected prepared
step leaving owner and every destination SQL row unchanged; raw/clone/serialized
state refusing re-entry; missing prefix and snapshot/event/audit/interpretation
tampering; source/pack/controller/head substitution; and exact old accepted retry
after later control changes. Preserve every old synthetic/legacy body and digest
control, with only specifically planned temporary activation closures migrated.

Rust snippets are a design, not type-checked source. The lifetime design avoids
self-references and allows before/after reads; implementation still needs exact
compiler verification. Current closed temporal profile, condition/flight/cut
coverage and archive-aware validation remain required engineering work from the
prior plan. This bridge cannot paper over them. It does resolve the authority
choice: there is no permitted state setter, unchecked nonmechanical rebase or
callback-controlled mutation. Any newly discovered producer which cannot keep
that invariant must be reported before public activation, not admitted by a
special-case validation bypass.

Next action: root reviews this resolved design and allocates a plan-first source
branch only after the frozen receiving evidence and intended scope are clear.
Then implement the coherent production path and its tests; do not stop at a new
private-owner checkpoint. Gate4 remains open.
