# Gate 4 — Authenticated body and equipment load facts

Status: implementation allocated, 2026-10-07. Sole branch writer is
source_review_oct7; root owns independent review and heavy verification. Branch
`codex/gate4-authenticated-load-facts`; checkout `gate4-authenticated-load-facts`.
Baseline is `b77b1597fb99d18643ed76f43202156f3bcf0fe0`, tree
`f56acf15ef3552436ccfbd8bdc2a199457dae7b3`. Fetched main is
`1a9de14c8a4418893b6664b89f99f0a0c0225ce1`. The baseline's full verification
is pending; inheriting it is not an acceptance claim.

## Objective, scope and product traceability

Supply the real table and desktop with authenticated body and item mass facts
and a custody-derived load view, as a prerequisite for faithful tactical carrying
and dragging. A normal user must be able to supply a missing physical fact
through an authorized form and resume it after restart. A type or source table
without that application path does not satisfy this slice.

Binding sources are root AGENTS, the [product definition](../../product-definition.md),
[Gate 4](../../checkpoints/gate-04-tactical-encounters.md), the
[gate execution protocol](../../checkpoints/gate-execution-protocol.md), the
[active tactical plan](gate-4-tactical-encounters.md), the public Grapple plan,
and the held-movement and combined-receipt plans. This advances source-faithful
mechanics, supported character interaction, conservative world facts, actual
physical inventory, player authority, private information, local play and exact
recovery. ADRs 009, 011, 012, 020, 024, 025, 027, 028 and 029 remain binding.

This is a coherent prerequisite within Gate 4. It does not implement carrying
support, change existing movement, complete ordinary drag/carry, or waive any
tactical acceptance. Broad shopping, travel, downtime and economy remain Gate 5;
complete catalog enumeration remains Gate 6. Any physical item already present
in a supported tactical route still needs an honest load treatment here.

## Source decision and preserved history

Use the pinned English SRD 5.2.1. The PDF SHA256 is
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Page 178 supplies Strength/size carrying limits and separate drag/lift/push
limits; page 182 supplies the Grappled per-foot cost and size exceptions. Root
selects the source-conservative interpretation that both constraints apply to
new mass-aware live operations. The text does not expressly waive pounds for a
grappled target. This records engineering interpretation, not an assertion that
an external ruling resolves all ambiguity.

Do not invent body weights from size, height, species or narrative. Source
equipment weights must identify their printed units and quantities. Missing,
variable or unrepresented facts remain unresolved rather than becoming zero.
A Host authorization boolean cannot replace an actual capacity calculation.
Where the source supplies no physical mass, a separately recorded accepted
world fact must be clearly distinguishable from a printed source value.

GroundDragV1's prior behavior, original accepted commands, paid movement prefixes,
audiences, digests, snapshots and byte-identical retries remain exact. Preserving
that history does not make its live mechanics capacity-complete. A subsequent
explicitly versioned operation must integrate capacity into new dragging and
carrying. That integration remains required by Gate 4 and is not deferred away.

External source-only design and independent review identify the prerequisite:

- `tooling/carrying-58fe854-design-proposal-2026-10-07.md`, SHA256
  `4777fc7bdea2ae6eaa14756475c6796744ab2a1842183a00217b8b68e98150c0`;
- `tooling/carrying-58fe854-design-independent-review-2026-10-07.md`, SHA256
  `94f48e26949d6198d37d7f27d037343d20ddd9e3ac4805b1dc2f9937c1fe1898`.

Those reports are navigation/evidence, not substitutes for this checked-in plan.
The baseline's production bytes equal their inspected 58 source; b77 changes
only two test setup files and documentation. Re-pin all inputs for implementation.

## Planned slices and decisions still to resolve

1. Inventory every currently materializable equipment identity against the
   immutable source: ordinary and creature weapons, armor, ammunition, supplies,
   containers, foci and spell materials. Distinguish bundle weight from each
   unit, empty container from contents, and actual coins from an abstract wallet.
   Unknowns must be explicit. Review printed tables visually where necessary.
2. Choose bounded exact arithmetic and catalog representation after that
   inventory. Retain all existing content bytes and fingerprints. A new immutable
   mass catalog/sidecar needs its own source identity and reviewed version; it
   cannot revise previously pinned character or creature definitions in place.
3. Specify the legitimate body/item fact producer, with original command,
   source/entity/item identity and amendment history. Determine who proposes and
   accepts PC facts, who authors NPC/custom-item facts, and the settled boundaries
   for changes. A client-supplied total is never authoritative load. Do not allow
   a new mass form to bypass source ownership or reinterpret old backstory.
4. Integrate optional omitted-when-absent state into the single owned execution
   and typed table path, behind explicit settled activation and the next reviewed
   presentation/transport/audit versions. Confirm all decoder allowlists, old
   producers and immutable retry behavior before selecting the version. This
   activation grants fact functionality, not a carrying pose or support record.
5. Compute load from actual custody, quantities and containment, counting each
   physical object once. Trace worn/held items, destroyed/missing items, containers,
   coins, fluids and unknown material facts. Bound graph work, reject cycles and
   report unresolved inputs honestly. Preserve the distinction between ownership
   and possession. Do not install a cached admitted total as current authority.
6. Expose authorized forms and useful load/source information through the real
   desktop and opaque owned controls. Preserve unrelated audience DTOs, transcript,
   revision and handle behavior; do not expose hidden facts through option ordering
   or detailed errors. Normal users need pounds and physical descriptions, not
   internal proof records, version numbers or implementation terminology.
7. Extend original accepted-history enumeration/replay, close/reopen, portable
   restore and rejection atomicity. Add genuine producer-driven application and
   frontend cases plus focused numeric/custody adversarial cases. Freeze the full
   delta for independent review, then schedule exact-head verification and native
   evidence with root's serial heavy allocation.

No production field, command shape, permission rule or version number is approved
merely by this planning list. Resolve the source inventory and concrete producer
design in a plan update before implementation. Avoid a disconnected catalog-only
completion claim.

## Acceptance criteria

- Source weight rows have exact printed references and unit semantics. Fractional
  ammunition, stack quantities, empty/full container distinctions and no-weight
  entries cannot be silently rounded, double-counted or treated as zero.
- A genuine current PC and an actual source creature can receive an authorized
  authored body fact, with source identity and original command preserved. No
  direct state, SQLite, JSON edit or synthetic positive proof establishes it.
- A real currently owned item uses an immutable printed mass or an explicitly
  permitted accepted item fact. Source facts cannot be overridden by unrestricted
  client values. Equipment transferred in custody changes the real computed load
  without rewriting ownership or the original mass origin.
- A complete supported load and an unresolved load are distinguishable. Missing
  body/item/container/coin facts prevent a claim that capacity is numerically
  satisfied. The app does not quietly remove real objects to obtain a total.
- Mutation timing and controller/session/head restrictions are explicit and
  tested through real controls. Pending attacks, rolls, held work or another
  player's actor do not grant an unintended fact-editing route.
- Historical v1-v4 producers, ordering, bytes and retries remain exact. New
  fact attachments require accepted activation and original fact commands;
  matching forged current/latest snapshots is insufficient provenance.
- Cold file reopen and independent portable restore at activation, fact admission,
  amendment and custody changes accept the same next input with identical complete
  outcomes. Tampered origin/source/value/quantity/custody/activation fails without
  mutating destination typed cells. Preserve all original test bodies and fixtures.
- Independent full source review, canonical `./scripts/verify-fast` and
  `./scripts/verify`, exact-head Linux/Windows checks and applicable native UI
  evidence are required before this slice is accepted. Focused tests supplement
  that evidence and cannot replace it.

## Remaining carrying behavior, unchanged Gate 4 obligations

Future support must persist across idle and turns, with authenticated admission,
pose updates and retirement under actual causal work. Before that implementation,
resolve independent displacement, carrier falling, changing capacity and target
death. Do not automatically drop a target merely to simplify code or reject a
genuine Shove/damage effect to preserve an invariant. Retaining a supported body's
identity/load through target death is the conservative design direction; it still
needs a real death transition and must not fabricate an inventory item.

Release/escape and holder incapacity need real support-loss/fall work even while
idle. Already issued opportunity/damage/concentration children keep their original
authority. Multiple targets/holders, nested carrying/cycles, new corpse pickup,
vertical modes, alternative poses and post-encounter lifecycle remain required
Gate 4 work. The current no-live-grips Finish/session guards remain until a
reviewed persistent-world transition exists. No gate or human playtest criterion
is weakened by this prerequisite.

## Validation status, risks and next action

No implementation, compiler, test, native or database execution on this branch.
Only the inherited clean source and this plan exist. Source inventory and producer
design are in progress outside Git. The original b77 integration worktree is
immutable under its separately scheduled focused run; Held0c60 has completed its
11-case focused selection and linting but its full CI/native scope remains open.

Main risks are invented mass, conflating catalog bundles with physical quantity,
unaccounted container contents, hidden-information leakage, unauthorized fact
amendment, and changing old accepted execution. Resolve those through actual
source and current producer paths rather than weakening load checks.

Next action: receive/review the two exact-b77 prerequisite inventories, record the
concrete arithmetic/catalog/authority/version design here, commit that plan update,
then allocate a single implementation writer. Root continues current runtime and
CI verification independently. No next-gate work is authorized.

## Concrete foundation decisions, 2026-10-07

Root read the complete equipment inventory and producer proposal, checked their
current production joins, and received a separate source review. This section
selects the following design for implementation; it does not claim runtime or
catalog acceptance. Receive the complete reviewed fixture correction
`58696ac1d0c55ef7f71cfb546fb92e97747ee437` by normal merge before implementation.
Its production/content bytes equal b77; all1109 existing case bodies are unchanged.
The dedicated integration checkout remains frozen under its separate runtime.

Source-only design evidence remains outside Git, with these exact hashes:

- equipment inventory JSON: `548da51e88aff50599c91cc5a600411c00ddf9636383f999132f9454cdb89b8a`;
- equipment design: `7834db3f7de51b7a539221c4b5ed98bda7ab5222e6cf21fe1ac20f3cc9bacd0b`;
- producer design: `9d72dd177c6a7167954af78e28968c4a8ca972d835c78d708e241bd13fca96af`;
- producer input manifest: `78795a1c0777d7a00a1eec137850b946d076a873219946240bc26f14fe5f1570`;
- owned-context clarification: `dfcdd7672df3dfb06305412a3ef02be04a3cb9d7161a66e95c5bf8cd6c8fab20`.

### Exact source arithmetic and applicability

Use checked integer micro-pounds: one pound is1,000,000 units. Ordinary body/item
facts must be positive; absence of knowledge is a tagged unresolved state. Use
checked u128 multiplication/addition and reject results outside u64. Transport
pounds as bounded decimal strings with at most six fractional places, parsed on
the server; no floats, signs, exponents, rounding, saturation or JavaScript Number
conversion. These are exact nominal game values, not measured physical precision.

Add an immutable equipment-mass-v1 sidecar, schema1, with exact ruleset/source
identity, source pages, quantity basis and four typed kinds: intrinsic per unit,
conditional inclusive gross, subtype required, and unquantified. Cover all62
registered IDs, while preserving the distinction between26 currently materialized
IDs and36 registry-only IDs. Retain every old catalog byte and source fingerprint.
Use the existing deterministic Fnv1a64 pattern over the fully typed catalog's
canonical serde serialization; pin schema/catalog/ruleset IDs with that fingerprint.
Declare the new distributed asset's byte length/checksum in the installed content
manifest and require exact installed bytes as with current creation/source files.
The fingerprint is consistency evidence, not save-file cryptographic authenticity.
Do not change the existing checksum algorithm or source grants as a side effect.

All quantities refer to physical units, not purchase bundles. Arrows are1/20lb,
bolts and sling bullets3/40lb, firearm bullets1/5lb and needles1/50lb per unit.
Count a container's intrinsic mass separately from tracked contents. Full waterskin
5lb is conditional gross; unknown fullness and empty/partial tare remain unresolved.
Holy Symbol needs actual subtype; gaming-set dashes and unpriced spell materials
are unquantified, never zero. Currency uses the source50 coins/lb relationship.

Select ordinary nominal masses for authenticated current Ogre Greatclub/Javelin
grants:10lb and2lb each, yielding16lb of listed gear for three Javelins. This is
catalog applicability to those ordinary registered identities, not a printed Ogre
body weight or a size multiplier. Explicit custom/oversized variants need accepted
physical facts and retain their identity across custody changes. They grant no new
combat property. All unlisted apparel remains a separate unresolved physical scope.

A source-known mass requires actual authenticated materializer provenance or a
typed Host acceptance of ordinary catalog applicability for a genuine original-
anchor/custom object. Matching a definition-ID string alone supplies neither.
Applicability acceptance binds actual ItemId, definition, physical unit semantics,
source catalog and expected current item context. A custom definition cannot
masquerade as a source grant or gain attack/focus controls through mass resolution.

### Fact authority and usable wallet representation

Use one optional omitted-when-absent physical-facts attachment with explicit
activation/catalog origin, current typed body/item/residual/currency records and
prior-origin amendment links. Bind each subject's real identity, source/profile
where present, precise scope, value/condition and original CommandMeta. An explicit
amendment may return a fact to unresolved without deleting original history.
Do not make generic text claims or independently editable totals authoritative.

The first producer is Host-only: Admin, no actor, exact campaign/head/session
(including matching absence), owned state and current opaque Host control. The
normal desktop exposes body pounds, missing item pounds/condition, untracked
apparel and actual coin denominations. Printed source facts are read-only.
Players read permitted facts for their own character; this slice creates no
parallel player-proposal queue or self-certification route. Controller changes
affect current viewing/decision authority, not accepted factual origin.

Activation/amendment requires setup or Finished encounter and no pending
declaration, roll, effect, creature routine/recharge, Inspiration transfer,
resolution or Ready work. An active settled turn is insufficient. Preserve
existing no-live-grips Finish/session rules and all exact source prerequisites.
Ordinary mass facts do not grant Grapple, source control, Ground or carrying.

Select denomination lots, not a competing gross-purse authority. Host explicitly
records CP/SP/EP/GP/PP counts representing the entire existing money_cp balance;
checked values1/10/50/100/1000 must sum exactly to that unchanged balance. One
unique physical currency lot, with actual custody and positive total coin count,
represents that existing wallet. It is not a second spendable value. No automatic
fewest-coin, all-copper, banked-money or zero-wallet assumption is allowed.

Bind realization to source-profile identity, wallet value and original command.
Amendment additionally binds exact prior realization and current lot quantity,
state and custody. Preserve lot identity, represented value and custody; a changed
denomination statement is an explicit correction of a physical fact, not an
unperformed currency exchange. Do not recreate missing/destroyed coins on read or
mint another lot on retry. A later actual payment/exchange needs its own atomic
producer. Custody may differ from wallet ownership; count the physical lot at its
actual carrier while retaining the single represented balance. Unexpected balance,
quantity or lot-state changes make applicability unresolved without blocking the
real causal mutation. Empty wallets create no zero-mass ItemInstance.

### Current load and continuing physical causes

Validate the entire item graph first, then follow actual custody to terminals,
count each ItemId once and apply current quantity. Include worn items, loans and
nested contents; two occupied hands are not two items. Reject cycles/dangling
references and overflow rather than treating failed traversal as zero. Spent zero
units are zero quantity; damaged material persists. Destroyed state still in
carried custody does not prove debris disappeared. Location/missing/destroyed
terminal custody is a location conclusion rather than annihilation of matter.

Return Complete or Unresolved with a known subtotal and authorized reasons.
Equipment load excludes the actor's own body; transported body mass includes its
unladen body plus complete physical load. This prerequisite shows these truthful
load views without admitting carrying or claiming capacity satisfied.

Residual apparel/payload has a specific accepted untracked scope, separate from
tracked equipment/currency and unladen body. Explicit absence is permitted; an
undefined blanket zero is not. Preserve the scope across unrelated tracked-item
transfers/quantity changes. Only a causal change affecting that payload, its form
or an actual materialization/overlap invalidates applicability. Binding residual
validity to any generic inventory change would make ordinary combat item handling
unusable with settled-only amendment and is not the selected behavior. Final
bounded scope representation must be reviewed before its implementation.

Retained fact authenticity and current applicability are different checks.
Real death, damage, source-form, quantity or custody events must still execute.
When a fact no longer applies, derive unresolved current load without rewriting
its origin or blocking the cause. Target death preserves body identity; it does
not create a corpse item, zero mass or automatically drop possessions. Gross facts
must bind their actual quantity/content scope and reject double-counted contents.

### One execution owner and deliberate version joins

Extend existing ExecutionContext/ReadContext with a separate private mass proof,
constructed only by CampaignExecution from its ClosedImage, fixed candidate and
CommandMeta. Preserve the existing Grapple guarded field, activation predicate,
is_owned/require_guarded and attack-reader branches exactly. Mass-only activation
must work without broadening those predicates. Carry the separate proof through
initial, nested and final candidate reads; generic public/raw contexts lack it.

Fixed typed mass producers observe their complete accepted delta. Mass validation
allows exact retained predecessor records or those current observed productions;
collection membership or caller-supplied receipts are insufficient. Compare the
whole attachment and any actually created currency lot, including disappearance,
before constructing the next ClosedImage. Add no second reducer, database, public
replace-state hook or caller callback. Original-anchor admission rejects existing
mass authority, and original audit/event replay reconstructs every fact/amendment.
Matching forged current state plus latest snapshot remains insufficient.

Select presentation/transport5 and command/observation envelopes6 for explicit
physical-facts activation; attachment schema1, CampaignState schema4 with absent
JSON omitted. Do not reinterpret old TableEvent1 or old accepted GroundDragV1.
Preserve all historical versions1..4, omission behavior, ordering and exact retry.
Mass absent retains the exact old version-selection branches. Mass present selects
v5 but every source/Grapple/Ground/Inspiration capability still requires its actual
activation; v5 is not capability evidence. Later activation of another feature
under mass requires its original predicate and a genuine command.

Update all real consumers together: TableApp and InspirationAward version/capability
selection; table-api request/channel and persisted outbox parsing; source-control
presentation selection; typed transport/meta/derive_intent; source/Ground/Inspiration
guards; rules_restore command envelope joins; historical audience/digest/control
validation; projection/binding version allowlists and strict state codecs. A real
presentation version carrier must support mass-only state. Accepted old retries
return their saved bytes before fresh revision/channel/attendance checks; new
old-version submissions after activation refuse atomically. Keep the SQL staged
export authentication before commit and all refusal atomicity.

Host sees full mass details. Other views follow established sheet/knowledge
authority; source-creature control alone does not disclose hidden physical facts.
If a total would reveal hidden inputs, withhold the total/detail rather than
subtracting hidden items. Unrelated private amendments retain other audiences'
DTOs/transcripts/revisions/handles. The explicit v5 activation is a version event,
not permission for later private operations to refresh everybody.

### Compatibility capture, implementation allocation and verification

An independent writer prepares a baseline586 capture-only branch. Production and
content there are exact pre-mass bytes. Capture genuine v3-to-v4 activation, Ground
movement/paid opportunity/raw prefixes, first/duplicate Inspiration award, transfer
recipient/decline and pending/consumed physical reroll through real app producers.
Retain full exports, original request/response/binding bytes, all audience DTOs,
controls/revisions, outcome and next legitimate input with source/harness hashes.
Root schedules it separately after review; no current capture pass is claimed.

The implementation must restore/retry those actual earlier-version artifacts
unchanged, then separately finish pending work, activate mass legitimately and
exercise Ground/Inspiration under v5. Same-build roundtrips cannot substitute for
frozen earlier-version evidence. Keep old fixtures and case bodies unchanged;
only unavoidable optional-None constructor additions may be separately inventoried
and reviewed, without changing an assertion or gameplay input.

Implementation is still unallocated pending the final source review and residual
scope closure. The next writer must implement the complete ordinary Host forms,
source/fact authority, nonzero-wallet route, honest custody load and recovery in
this one coherent branch. Meaningful numeric, authority, privacy, forged-history,
frontend and genuine application cases supplement the preserved regression suite.
No catalog-only or type-only completion claim is authorized. Root independently
reviews the frozen implementation and schedules canonical, exact-head CI and native
verification. All carrying/support/lifecycle Gate4 obligations above remain open.

## Independent review closure and source allocation

The producer review is source-only and clear after the selected closures:
`load-facts-b77b159-producer-independent-review-2026-10-07.md`, SHA256
`c7a9faed2cdf9a49739402ce7fb5f48dc85703c5250534f74e169fbc6fc280d7`;
audit `d9602e14985a80f6b45e94f422c4194b2d8b0736436de8742dceff6cc90f3898`.
Root selects its ordinary ItemId route for specifically described untracked
apparel/payload. This supersedes the undecided residual representation above.

The settled Host acceptance materializes a bounded named custom physical object
with a new deterministic ItemId, actual actor custody and authored positive mass.
It records that this is a separate previously untracked object, excluding body,
all actual source allocations, tracked contents and any represented currency lot.
Do not infer physical sameness from text. The producer cannot reuse an existing
identity, change someone else's source allocation, copy a registered equipment ID
to grant properties, or silently replace an earlier residual object. Use a fixed
custom-load identity class outside source equipment grants; it supplies no attack,
armor, focus or new spending authority. Subsequent facts refer to that same ItemId
and actual custody. Its materialization is observed by the same private mass
component and reconstructed from the original command, like currency issuance.

The normal form describes this as recording additional apparel/payload, with an
explicit acknowledgement that it excludes equipment already listed. Allow an
explicit coverage/absence fact when there is no additional untracked payload;
do not manufacture zero-mass objects. Bind coverage to the actor/body profile and
completed source allocation identities, not the current independent inventory set.
Known PC/source starting allocations must already be materialized or be explicitly
accounted for before claiming complete load, so later PrepareEquipment cannot
duplicate purportedly untracked starter clothing. A later source/form allocation
change requires unresolved coverage or an explicit supported causal update.
Ordinary sword drop/pickup, ammunition consumption and coin movement preserve
independent custom apparel and coverage. Real operations on that object use its
identity; no prose-residual-to-item conversion or second object database is added.

Also close the reviewed version joins explicitly: update the desktop's fixed
activation version overrides as well as context selection; old direct command/
observation APIs must reject fresh mass-only bypasses while keeping authentic
historical replay. Origin enumeration must handle physical facts independently
of optional rules state. Strict old-state/export migration and duplicate-field
authority detection must reject smuggled mass authority. These are required parts
of the selected single producer path, not optional follow-up cleanup.

Before source allocation, normally receive the complete reviewed PR71 correction
`86ed8ae17b47f58dd32f629b32a7d53faf577242`. It fixes only the shared cold-test
negative by making its payload actually differ, with an explicit inequality
assertion. All966 current PR71 test bodies and production are unchanged. Its fresh
CI remains pending; no donor acceptance is transferred. Preserve this complete
receipt and its original failure evidence instead of duplicating the fixture fix.

After that normal merge, source_review_oct7 becomes sole writer of this branch
for the complete foundation described here. Root stops source edits and owns the
independent full-diff review, normal publication and all heavy verification. The
author may make coherent code/plan commits and direct changed-file formatting,
but runs no compiler, Cargo, npm, project test, native package or database while
root's serial heavy allocation is occupied. No push or merge to main is allocated.
Report concrete implementation conflicts promptly and retain the current product
contract; no disconnected catalog, synthetic positive fact or version shortcut is
accepted. Freeze complete source and preservation evidence before handback.

### Implementation start and source provenance

Writer received clean `cd60648db894703c954e6f01f58d2d71dd42d388`, tree
`24eb509d978d94a215d5d2aca24e0cb4532233a5`, fetched origin and read this complete
plan. Compiler/runtime remain explicitly unrun. The inherited 86 retry fixture
has a newly reported Dash constructor compile error; root owns the corrected
donor and a deliberate subsequent receipt. Do not duplicate that correction here.

Catalog applicability for creature item identities will use a private immutable
grant index on the existing ClosedImage, accumulated only after successful
PrepareEquipment/CreateCreature operations during original owned replay. This
retains exact accepted physical identities across custody changes without adding
or editing old serialized grant bytes. Original-anchor generic items still need
the selected typed Host classification. The mass candidate proof remains separate
from Grapple and receives the exact closed predecessor's grant index; it cannot
be supplied by deserialization, definition text or a raw clone.

### Physical applicability refinement during implementation

Early independent review identified that a registered definition alone cannot
forbid an honest fact about a nonstandard physical object. Add an explicit typed
NonstandardUnit classification: the Host describes the physical difference and
records positive pounds per actual unit for this exact current ItemId. This is
an authored world fact that rejects ordinary catalog applicability, not an edit
to the printed catalog amount. Preserve the item's identity, definition, custody
and any prior source grant; do not grant or alter combat properties. Ordinary
Unit still cannot silently override a printed mass. The form must distinguish
nonstandard classification from ordinary catalog acceptance, and replay retains
that exact classification and original context across genuine custody movement.

PC body identity binds entity kind/identity, actual species, size and creation
source, rather than the entire mutable profile. Wallet, equipment, experience,
narrative and other unrelated profile changes must not invalidate body/coverage.
Coverage independently binds completed source allocations. Currency separately
binds the current wallet value, as already selected above.

### Fixed candidate continuity at an existing equipment join

Early independent review found that the ordinary after-attack equipment wrapper
clones its candidate unless the Grapple-only `is_owned` predicate is true. That
clone correctly fails the independent mass proof's identity check in M-only play.
Add a private `has_fixed_candidate` query solely for this clone/transaction choice;
it is true for either existing owner proof and grants no feature or permission.
Keep `is_owned`, `require_guarded` and attack-read admission unchanged. The existing
CampaignExecution still owns rollback. Add genuine M-only paid attack, completion,
and both Apply/Decline equipment choices through the current app and cold helper.


### Source candidate, 2026-10-07 — verification still pending

The complete foundation is implemented for independent review: an immutable
62-entry equipment sidecar; exact micro-pound arithmetic; typed settled Host
activation, body/item/coverage facts, explicit nonstandard applicability, real
custom ItemId materialization and exact physical currency-lot realization; a
private source-allocation index rebuilt by owned current/original materializers;
an independent candidate-bound mass proof; normal Host forms, private load and
item details; and v5 presentation/transport with command/observation schema6.
Existing S/G/T permissions remain distinct. `is_owned` and `require_guarded`
semantics are unchanged. `has_fixed_candidate` is used only at the existing
transactional after-equipment wrapper to retain either proof's exact candidate.

The final new catalog has 17,077 LF bytes, FNV1a64 `1ffe74159f6889cf`, and
SHA256 `5550faae8307d7d53bc19955c2d119e2e585e69649770135752a1172268f06ef`.
The additive manifest declaration pins those exact bytes; prior source assets
and source fingerprints are unchanged. Printed values stay read-only. Generic
original objects require Host classification; the nested-test sack's half-pound
mass is an authored world fact because it is outside the bounded catalog.

Load follows actual custody through bounded nested containers and counts each
object once. Inclusive/gross container facts refuse separately tracked children.
Unexpected currency identity, definition, quantity or condition stays unresolved,
including a forged Spent/zero lot; ordinary spent ammunition can still weigh
zero. Currency corrections preserve the actual lot identity and current custody,
and never add spendable value. Body identity excludes unrelated wallet, narrative
and XP changes. Coverage binds completed source allocations and physical body
identity, while actual additional apparel is an ordinary separately named item.

New Rust cases are source-authored and UNRUN: five complete application cases
(current PC/load/private forms, current Ogre/source control, same-ID nonstandard
throw/pickup, M-only paid after-equipment Apply/Decline, and current-build v4
finished encounter activation followed by v5 Inspiration); five private owned
proof/applicability cases (generic classification, candidate authority, producer
delta, nested contents/gross refusal, and actual-wallet altered-lot refusal); two
exact-arithmetic cases; and one installed sidecar missing/undeclared/rehash atomic
refusal case. New frontend cases cover the actual forms, exact decimal text,
explicit coins/nonstandard classification, opaque commands, lost acknowledgement
retry and malformed/old/unauthorized saved envelopes. Existing test bodies and
assertions remain exact except eleven required empty `mass: None` fields in
private execution-context constructors; a shared physical-creation driver module
is registered at its common parent so the new app cases use the unchanged actual
cold/replay/portable helper. Preservation is independently inventoried at freeze.

Early read-only reviews corrected concrete implementation/setup issues before
freeze: the M-only candidate clone join, currency's spent-ammunition shortcut,
actual paid equip-before/after sequencing, generic sack source applicability and
the offered Inspiration decline label. None relaxed a production guard or old
assertion. Raw current snapshots still cannot mint mass proof, and direct legacy
command/observation routes cannot submit fresh M-only work. Restores require the
original activation and exact accepted producer history, including M-only worlds.

Current-build v4/v5 continuation is **not** historical compatibility evidence.
The separate genuine pre-change v4 capture harness is frozen at `969627f`; its
three routes/22 cuts have not yet produced an archive under the serial heavy
allocation. No prior-version fixture was manufactured and no skipped consumer
was added. Required next work includes receiving the unchanged actual archive,
adding its consumer, and executing that consumer. Full slice acceptance remains
pending this prerequisite, exact-head independent source review, canonical
verify-fast/verify, focused and existing runtime cases, strict Clippy, frontend
checks, fresh CI and normal desktop evidence. Source formatting and Git/static
inventory are the only author checks so far; no compiler, Cargo, npm, project
test, native process or database was executed by the writer.

Root has authorized a normal whole-history receipt of the reviewed PR71 donor
`3a68bd793e47aad6fbadad9a3b889d47be906189`, tree
`10321b6e4df060b53445eedb9a5df9427e9999db`, after the coherent source commit.
It supplies the missing `DashSpeed::Speed` field in the inherited retry fixture;
do not duplicate it or claim donor CI as this candidate's validation. Origin was
fetched again before freeze; main remains `1a9de14c8a4418893b6664b89f99f0a0c0225ce1`.
After receipt and a clean freeze, root owns independent review and all heavy
verification/publication. The author must hand back exact tree/parents, complete
diff, old-body inverse and protected-source inventory. Carrying support, capacity
integration and every remaining Gate4 obligation remain open and unchanged.

### Independent capability-version correction, 2026-10-07

Root allocates `mass_capability_fix_oct7` as the sole writer for this bounded
correction, starting from clean `475d94e40a5e1652d176ffddae55d9b16e8a24df`, tree
`f79e6ced344f5a8f29f4b8d3f05d71bf53fae98b`. Origin main and the two published
donor branches were fetched; this mass branch remains unpublished. No donor
receipt is allocated here. The separate inherited Glaive fixture correction must
be received later as reviewed whole history by root.

The complete independent source review found one P1 desktop regression:
`table_grapple::view` uses the global presentation version for its independent
Grapple capability field. With physical facts enabled, it emits 5 instead of 3
for Grapple alone or 4 for Ground/Inspiration. The actual desktop then hides
the later Ground activation and Inspiration award controls, and its physical
Inspiration submission handler returns without sending. Review evidence is
`tooling/load-facts-475d94e-full-independent-review-2026-10-07.md`, SHA256
`0a33f531cab6488347fdd11354270160644938ad5781f6aa7687ca5563936d52`;
source-pin manifest SHA256
`1c9bae24860bc126ec9936d0493d137498439f4e78e01c79508056a75fa654bc`.

Keep the outer presentation, binding and fresh request version at 5 when mass is
active, and keep `physical.version = 5` and `source_control.version = 2`. Select
the Grapple field independently from the actual Ground activation: absent when
Grapple is absent, 3 for Grapple without Ground, and 4 with Ground. Preserve the
existing exact UI capability checks and all production admission predicates;
neither mass nor an inequality on the presentation version grants Ground.

Extend genuine current mass-case helpers at the M-only, M+G and M+G+T boundaries
to check all audience DTOs and actual saved projection/binding/request versions.
Retain all original Rust case bodies and all 23 prior mass frontend cases. Add
real rendered TableApp cases for later Ground activation over request 5,
first/excess awards and owner choices only with Ground, and physical Inspiration
submission exactly once over request 5. Include its retained original request
across uncertain acknowledgement and retry; keep original and replacement faces.

This diagnosis is committed before code. Root's serial heavy slot is occupied
by the genuine v4 capture. This writer may run Git, static source inventories
and direct changed-file rustfmt only: compilation, Cargo, npm, project tests,
native execution and database work are UNRUN. Freeze the coherent correction for
root's independent review; do not push, open a PR, merge or claim acceptance.
Historical corpus consumption, carrying/support/capacity and every remaining
Gate 4 acceptance requirement above remain unchanged.

The authored correction now selects only the Grapple DTO's 3/4 field from the
actual Ground activation. The global version function and all desktop production
code are unchanged. Genuine current helpers check M-only, M+G, M+G+T and prior
v4-to-M boundaries: three audience DTOs, real activation request/binding and
projection versions, plus atomic first/excess award refusals before Ground.
All 13 existing new Rust case bodies remain unchanged; only two of their setup
helpers gain additional assertions. Nine new rendered TableApp cases exercise
both staged activations, first/excess awards, absence of owner choices before
Ground, gift/decline, one physical reroll submission and exact retained retry.
The 23 prior mass frontend cases remain unchanged. Direct formatting and source
inspection are complete; all project execution and independent successor review
remain UNRUN/pending. This source commit does not claim that F1 passed runtime.

### Deliberate reviewed fixture-history receipt

The capability correction is coherently committed at
`1beb270ed8d838e83eff26e8cae33a8f80b9e43e`, after diagnosis-only parent
`0818782`. Root now authorizes a normal whole-history merge of exact reviewed
`1c86dd6c1bb61cd871f7d7db92493bf4c7d71e06`, tree
`52218c35130b067da3415cb0afcd506af5c1d3da`, without copying or cherry-picking.
Its independent report SHA256 is
`8645dc236e4abc0318bace71574231daa08d50e02130eab19a0c5bf102f3b5a8`.
The full donor source change and complete added plan evidence were inspected.

Both reviewed histories 586 and 3a are already common ancestors. Git's source-only
merge-tree preview is conflict-free and changes only the combined Glaive helper
module and combined-receipt plan. Expect both files to equal the donor exactly;
all other recipient entries must remain exact, including the capability fix and
physical-facts plan. The donor keeps the unrelated PC rejection and adds genuine
Host admission in a restored copy, with full unchanged-original and cold PC-path
assertions. Inventory the single approved old Glaive call substitution; removing
its new helper and reversing that call must restore the entire prior file.

After receipt, freeze exact parents/tree and source preservation for root's
independent review. All compilation, runtime, frontend and native checks remain
UNRUN. Root separately reports main has advanced to the reviewed physical slice
at `4cf815bd0f0d9b128612867ba829c9ac1c2549f7`; no main merge is allocated here.
The genuine 969 capture is still running, so its historical consumer remains an
explicit unmet dependency, with no replacement or skipped historical evidence.
