# Gate 4 — Reviewed ground-drag implementation design

Status: **Approved design; implementation/verification pending.**
Root approved the original concrete proposal and the continuous-support amendment
on2026-10-06 after independent public_grapple_oct6 review. The support amendment
below overrides the initial endpoint-support wording. The active
[execution plan](gate4-grapple-held-movement.md) owns current allocation and status;
historical statements below that request allocation are retained provenance, not
a contradiction of that plan. Root requires whole public-fix receipt before
production edits and separately owns every heavy verification/publication slot.

These complete external reviewed texts are retained here so source work is
resumable without private scratch files. Original proposal SHA256:
`6151d0ae5ee5233a88191ff769f6e3f4c32d9862e46b4d4c057fc6482b864607`.
Support amendment SHA256:
`20c373403d987269b3436962d39b9aeb79d169609858a0763ee3b5bacc4aa106`.

---

# Grounded single-target Grapple movement: proposed allocation

Read-only design, 2026-10-06. Reviewed code pin:
`afbaf9b15b5a86872993d9f1f2ae0da0a81e9ddc`, tree
`e2a11fa9a1b9494dda6ac8dc7a787666091dfe88`. This extends the source analysis in
`grapple-9fcb-drag-carry-proposal.md`; it is not implementation or runtime evidence.
Root's proposed receiver is `codex/gate4-grapple-held-movement` at
`b6ef6207d37f99bf3edb35349cd59b59cf4c81d3`. Verify its complete code/test identity
against this pin before allocating its sole writer. No repository files changed.

## Bounded result and source rule

Implement one explicit **ground drag** of one directly held target, through the
existing MoveSegment continuation and real table/player/desktop/persistence path.
Both actual bodies translate by the same nonzero horizontal adjacent-grid delta;
each remains supported at its own current elevation. Holder modes are Walk/Crawl.
Other outgoing/incoming relations are not recursively transported. Keep the
existing Move and MoveSelfOnly behavior. Unsupported carry, vertical movement,
jump/swim/climb/burrow/fly transport and multiple selected bodies remain Gate4 work.
The slice neither grants airborne support nor handles corpse cargo.

Use the specific Grappled rule in pinned SRD5.2.1 p182: surcharge one extra foot
per foot, except Tiny or at least two sizes smaller. Root has accepted that rule
for this single-target slice without invented mass. PDF SHA256 is
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`, matching
checked-in source.json. General mass, actual carry pose/support and multiple-target
surcharge/nesting decisions are required before those later slices, not silently
answered here. The legal movement/occupancy/OA clauses are pp14-15 and185.

## Proposed action, authority and durable shapes

Names below are concrete proposals for the writer, not existing types. All new
records deny unknown fields; new attachment fields default to None and omit None.

- Canonical `TacticalAction::MoveGrappled { grip: GrappleId, path:
  Vec<TacticalMoveStep> }`. This means GroundDragV1 in this implementation, not a
  caller-supplied carry boolean, target position, cost or permission set.
- Typed reducer/table owner derives the actor from the current turn, then derives
  target, physical hand, exact live source proof and poses from the named grip.
  Require authenticated CampaignExecution, transport activation, exact flow5,
  current issuer/controller/session/head, no prior unfinished movement, settled
  falls, supported ordinary body geometry and a living target. No new attack cost,
  target input or hand reservation is invented. A holder with incoming Grappled
  still has Speed0. Selecting one grip never substitutes another if it ends.
- `TableGrappleTransportOffer { actor, grip, kind: GroundDragV1 }` is a canonical
  server-only offer. Expose an opaque `ProjectionCapability::GrappleTransport
  { offer }`, displayed with an owner-safe held-creature label. A new transport
  input `MoveGrappled { option: CommandId, path }` resolves that audience handle
  to the typed action under the current revision. Do not put a canonical grip ID
  into the browser input or turn a direct raw action into an authority bypass.
- Add `ground_transport: Option<TableGrappleTransportAccess>` inside existing
  TableGrappleAccess. Its exact content is `{ version: GroundDragV1, origin:
  CommandMeta }`; the original OrdinaryGrappleV1 origin remains unchanged.
- Add **one** `transport: Option<GrappleTransportHistory>` to
  TacticalGrappleResolution. Reuse ordinary TacticalMovement and its ordinary
  holder receipts; do not duplicate an independently executable movement object.
  A transport history must match resolution.origin and, while present, the exact
  movement.origin/actor/path/start/initial_spent; grapple_self_only must be None.
- `GrappleTransportHistory { admission, steps, stop }` contains
  `admission: GrappleTransportAdmission { origin: CommandMeta, kind:
  GroundDragV1, grip, holder, target, holder_from, target_from,
  path: Vec<TacticalMoveStep>, spent_before: u32 }`;
  `steps: Vec<GrappleTransportStepReceipt>`; and an optional stop described below.
  The path copy deliberately permits complete prefix/source-work validation after
  finish_movement removes TacticalMovement. It is bounded and exact, not another
  route cursor. Retain the full original TacticalGrip once in context.proofs.
- Each step is `{ work: TacticalWorkKey, step: u16, cause: CommandMeta,
  holder: BodyDisplacement, target: BodyDisplacement, mode: MovementMode,
  ordinary_cost: u32, haul_cost: u32, holder_size: CreatureSize,
  target_size: CreatureSize }`; BodyDisplacement is `{ actor, from, to }`.
  Exactly two distinct body identities, same delta, nonzero displacement,
  admission identity and contiguous step index are mandatory. The producer uses
  the actual currently entered MoveSegment key, never the next queued occurrence.
  While movement exists, each step must match its ordinary holder receipt exactly
  in cause/from/to/mode and total cost. Sizes are historical source-derived inputs,
  authenticated by original replay, not caller permissions.
- `stop: Option<GrappleTransportStop { cause: CommandMeta, ended_grip:
  GrappleId }>` is a narrow latch for an ended selected relation. It must bind the
  unique retained GrappleEndReceipt, including its exact cause command. Other
  ordinary next-step failures use existing Stopped/Interrupted behavior; do not
  serialize a caller-selected generic bypass reason.
- Add omitted `transport: Option<GrappleTransportResult>` to
  TacticalMovementResult: `{ kind: GroundDragV1, grip, target, target_start,
  target_endpoint, ordinary_cost: u32, haul_cost: u32 }`. Produce this only from
  the accepted transport prefix and actual final target pose. This summary is
  necessary because last_movement survives resolution retirement and turn budget
  reset, whereas its detailed receipts do not. It grants no live relation or
  support. The owner authenticates it against original MoveGrappled replay;
  forged/missing summary and forged retired-only attachment must be detected.

Make context.is_empty account for transport. Existing attachment detection already
sees resolution.grapple; nevertheless explicitly reject transport on a missing
activation/proof/flow5, including when no live grips or movement remain. Extend
origin enumeration, original action matching and orphan-field checks. A raw
decoded/current/snapshot image supplies no new execution authority. Preserve all
private ordinary-context refusal controls and all original test bodies.

## Version decision: new v4, preserve v3 exactly

Use **presentation/transport v4**, activated by a new accepted Host
`EnableGrappleTransport` operation at an already Grapple-enabled, fully settled
Active flow5 boundary and current session. Existing live grips may remain; they
are authenticated existing state, not adoption input. Reject activation while
work/rolls/held Ready/source routines are pending, using existing settled checks.
Do not replace or replay EnableGrappleAccess as a different operation.

This is necessary for the requested historical guarantee, even though public v3
is not yet released. table_presentation_history.rs:284-355 recomputes both the
complete serialized presented-view digest and ordered capability vector, while
table/source_control.rs:12-19 currently chooses v3 for every Grapple marker. An
unconditional added drag menu changes already-authenticated v3 records. Omitted
new fields alone do not help when the producer now fills them in old states.

Before the new activation, retain the exact v3 producer, output, option ordering,
self_only_required behavior and old transport acceptance. After activation,
presentation_version returns4, the v4 movement view offers explicit self-only or
ground-drag selection, and fresh v3 inputs are refused. Original accepted v3 retry
envelopes still resolve from their immutable saved binding before fresh-version
admission. All historical v3 digest/handle/visibility comparisons remain exact.
The activation request is v4 against the prior revision; its accepted event
produces the first v4 presentation. Existing surviving capability handles retain
their identities as current validate_record requires.

Wire plumbing must cover TableProjectionRecord and TransportBinding allowed
versions (currently1|2|3 in table_projection_store.rs), the v4 request/response and
desktop DTOs, and action/conversation audit schema **5**, because the existing
envelope rule is request.version+1 (table_transport_runtime.rs:538-544). Extend
only those enumerated readers/validators; keep old encodings and nested gameplay
event versions unchanged unless an actual reader requires otherwise. Never let a
new action/field ride an old envelope. The protocol activation is an application
capability upgrade; avoid presenting implementation version numbers in gameplay.

## Joint geometry and exact expenditure

Add an internal coupled-ground-segment evaluator sharing existing physical
geometry helpers. It accepts a proved pair and returns both displacements plus
the holder segment/cost. It grants no reusable forced-movement authority.

1. Derive each before pose from the actual scene and accepted prefix. Validate
   both current bodies, support, grid alignment, range and remaining capability.
   Require supported ordinary bodies and horizontal Walk/Crawl; neither body's
   own elevation changes. No ordinary-body proxy for Air Form or other unsupported
   geometry. Target Prone/Speed0 does not require target Crawl or spend its budget.
2. Project **both** after poses into one temporary encounter. Run each body's
   existing bounds, actual occupied volume, solid/terrain swept-volume and diagonal
   solid-corner checks. Use actual destination support for both and reject any
   segment that would enter existing falling/flight-loss handling; this slice
   supplies no hovering, landing, water or carry exception.
3. Third-party occupancy uses the ordinary grid passage/end-space rules for each
   actual body against every stationary participant. Intermediate permitted
   passage remains permitted; neither chosen final endpoint may overlap another
   body. A selected body's vacated space is evaluated at the joint time, not as a
   stationary obstacle. Separately prove the selected pair do not overlap before
   or after: their identical linear translation preserves relative separation at
   every intermediate time. Thus follower movement into a vacated holder cell is
   legal without deleting either body from collision checks. Do not intersect
   their swept unions as if bodies visiting a point at different times collide.
4. Holder difficulty uses its own normal terrain/occupancy calculation. The
   target's different route does not add an invented second terrain surcharge.
   Let D be holder grid distance in half-feet (0<D<=10), C the existing holder
   ordinary cost, H = 0 if target Tiny or holder.rank-target.rank>=2, otherwise D.
   Total = C+H, checked once against current Speed/Dash and cumulative spent.
   Thus ordinary five-foot walk costs20units; difficult walk30; difficult crawl40.
   Keep actual Speed, grants, completed-prefix costs and run-up unchanged.
5. Once all offered holder OAs and descendants settle, re-evaluate the whole pair
   against the current scene. Write both positions, one holder expenditure and
   both linked receipts in the same staged transition. Interrupt rest for both
   actually displaced bodies. Publish observer changes only after the atomic pair.
   Any target-only wall/corner/occupancy/allowance failure moves neither body.

The narrow receipt migration is in tactical_movement.rs:458-480. With no proved
transport history, keep the old cost<=30 and ratio<=3 predicates exactly. With a
matching GroundDragV1 step, apply the old bounds to **ordinary_cost**, require H
exactly from authenticated sizes, and require receipt.cost=C+H<=40. Check
Walk/Crawl and their valid base multipliers, chronology, progress and budget as
before. Missing/mismatched proof is corruption, never a reason to use relaxed
bounds. Original replay rederives terrain/cost from each historical transition;
never recost an old prefix using today's terrain, live grip set or source menu.

Also migrate **validate_result:169-224**, independently flagged by public_grapple_oct6
and then source-checked here: its current completed_steps*30 bound otherwise
rejects a finished four-cost crawl after detailed movement is removed. With no
transport summary keep that old predicate exactly. With the authenticated new
summary require ordinary_cost<=completed_steps*30, haul_cost<=completed_steps*10,
and exact total equality to spent_after-spent_before, plus the existing unchanged
chronology/count/endpoint constraints. While detailed history remains, require
both sums and summary identities/poses to match it and the actual finish result;
after retirement original action/event replay establishes those exact sums.
Include result.transport in new-attachment detection/version refusal and the
rules_restore original-action matcher; a v4 table marker alone must never excuse
an ordinary Move result with relaxed expenditure.

## OA, release and causal endings

Use existing MoveSegment/MovementOpportunity nodes and selected attack readers.
The holder provokes before either position changes. The transported target does
not provoke merely from involuntary displacement. In movement::options:228-231,
the hypothetical after-distance must also move the reactor **when that reactor
is the selected target**. Equal translation preserves their reach; leaving the
target at its old position would fabricate its OA. Other reactors remain fixed.
Use the same pair-aware evaluator for offer, queued validation, selection and
pruning; maintain declined versus automatically Unavailable decisions.

When the selected grip ends, retain its exact proof/end and latch the transport
stop. Retire only unaccepted movement OA prompts and queued MovementOpportunity
siblings with normal Unavailable accounting. Preserve the one underlying
MoveSegment, TacticalMovement prefix and every already accepted OA/attack, hit,
damage, Shield, concentration or fall child. The raw selected attack is historical
work and must remain answerable after release. Once its children finish, the
underlying MoveSegment sees the latch and calls existing finish_movement without
crossing: Stopped for voluntary release, Interrupted for causal loss. If there is
no accepted child, the same pump can stop immediately. Never convert to self-only.

The latch is needed before final release validation: today's queued-OA validation
calls next_segment and would reject an absent transport relation. Merely waiting
for a later command to prune it can make release itself fail. Conversely, deleting
the whole movement immediately can orphan an already accepted attack's ancestry.
Extend only the transport stop/prune path; preserve old ordinary/self-only paths.
Permit historical prefix validation from the retained proof after the last live
grip becomes None. A retired grip must have its exact ending; it grants no current
Speed, condition, hand reservation or further crossing.

For independent target displacement while children run, compare its real current
pose with the accepted target prefix before offering/committing a fresh crossing;
stop as Interrupted. Preserve that child's consequences. Do not require a target
to stay at its historical prefix while validating a real child that displaced it.
Neither existing falls nor flight-loss work is canceled by the new intent; permit
those children to finish and stop the grounded route. No new fall support cause.

Replace the singular moved_actor check **only for proved transport MoveSegment
occurrences** with the receipt's two actually changed bodies. Retain old logic for
every other movement/Shove/fall path. For each out-of-range relation, choose its
displaced endpoint from that exact receipt (deterministic ID order if needed),
and keep existing GrappleEndCause::OutOfRange { work, moved_actor }. Validate the
end against the stored exact work/cause/body receipt after TacticalMovement has
been removed. This permits dragging B away from a third holder C and moving A
away from an unselected held D without pretending C or D moved. The selected A-B
range itself remains unchanged. End only actual invalid relations; do not walk
the graph to move more bodies. Retain transport history until the shared
resolution retires, along with ends/cuts/falls. At that point original replay and
last_movement remain the normal persistent evidence; no idle support is granted.

## Work bounds, completion evidence and extension seams

No new bytecode/work kind is needed. There is one route <=1024 steps, one holder
OA scan per actual existing MoveSegment attempt, and at most one two-body receipt
per committed index. No target OA scan or recursive graph traversal is added.
Keep exactly one next uncommitted MoveSegment and existing offered/decision count
bounds against participants (movement.rs:684-775). Preserve turns.rs limits of
128 nested frames,32768 occurrences/pump iterations and work_trace's32768 nodes;
preserve spent/run-up10000. Account all receipts with checked arithmetic and
steps<=path.len; each receipt names one allocated entered node, never every retry
node. Existing reaction children can already use multiple occurrences per step,
so do not assert path.len equals total MoveSegment nodes. Range-end scans remain
bounded by real live grips; a coupled receipt does not allocate one work per grip.

Required new positive intersections: genuine Human/Goblin both holder directions;
source-assigned player; multi-step floor/solid-top follower geometry; ordinary,
difficult and crawl+difficult cost; split movement/Dash; target Prone; real holder
OA with before poses and physical dice; no hauled-target/fabricated reciprocal
OA; last-grip release during raw OA/damage children; another holder's range ending
from target movement and unselected target ending from holder movement. Each
must assert both poses, exact expenditure, target budget/Speed0, original hand and
source identity. Large Ogre->Small Goblin exemption awaits normal whole source
receipt; no Tiny production source exists at the pin, so arithmetic-only Tiny
controls must not be called source acceptance. Negative geometry tests alone do
not replace these positives.

Add real file-close/reopen and independent portable cold restore at admission,
partial prefix, pending OA/raw damage and released-last-grip cuts; then execute
the same next input and compare full state/outcome. Assert immutable old retries
after upgrade/controller/head changes, refusal of former-controller new input,
and exact unrelated audience DTO/transcript/revision/error equality for hidden
changes. Tampering with grip/proof, activation, path, either pose, size exemption,
cost, occurrence, prefix, stop or last-movement binding must reject atomically
without altering any unrelated destination typed cell. Preserve private82,
public25/frontend9 bodies and historical version/digest captures. Canonical/CI
and native evidence remain separate exact-head obligations, not achieved here.

Future carry can reuse the action pipeline, opaque selection, coupled geometry,
additive-cost component and moved-body receipts, but must introduce a reviewed
durable support/pose relation that outlives a movement. Future multi-target can
generalize the bounded body set/selector after its surcharge, duplicate-target,
nested/cycle and partial-release policy is decided. Versioned transport kind and
one current reducer avoid a throwaway alternate implementation. Do not expose
placeholder carry/multiple-target choices or call this slice drag/carry complete.

**Proposed next allocation:** root/peer review these shapes and v4 choice, then
one writer on the docs-only receiver makes a plan-first implementation. Baseline
public verification remains independent and all failing baseline evidence stays
at its original head. No source/test execution was authorized by this memo.


---

# Required ground-support amendment

2026-10-06; source pin `afbaf9b15b5a86872993d9f1f2ae0da0a81e9ddc`.
This normative amendment supersedes the endpoint-support wording in
`grapple-afbaf9b-ground-drag-design.md` (SHA256
`6151d0ae5ee5233a88191ff769f6e3f4c32d9862e46b4d4c057fc6482b864607`).
All its other bounded scope, authority, v4, receipt and continuation decisions
remain proposed unchanged. Root requested this correction before source allocation.
public_grapple_oct6 independently identified the issue and source anchors; this
author then read the actual pinned helpers in full. No repository write or runtime.

## Positive support requirement

For **each of the two bodies**, prove both dry geometry and load-bearing support
at **every time t in [0,1]** along its actual horizontal linear segment. Neither
fall_destination(None), validate_physical_position success, two supported
endpoints, a swept bounding box, nor fixed-distance sampling proves this.

The existing map convention is positive horizontal footprint overlap, not full
footprint coverage: spatial/falling.rs:112-119 explicitly says that even a narrow
solid ledge catches a body and mere edge contact does not. Preserve that convention
for this ground-transport query. Do not substitute buried_support's full-foot-space
coverage rule, which belongs to source Burrow at depth (falling.rs:10-61).

Eligible dry supporting surfaces at the body's fixed bottom z are:

- The authored floor when body.z == battlefield.floor_z, with the existing
  complete-body battlefield-bounds checks. This supplies support over all [0,1].
- Movement-blocking obstacle tops whose volume.max.z == body.z.
- Terrain tops with `!water && (supports_top || burrowable)` and
  volume.max.z == body.z. Merely difficult, obscure or climbable terrain is not
  a platform. A water-tagged volume supplies no ground-support authority.

Retain ordinary embedding/solid/sweep/corner checks independently. A supporting
top does not allow entering its solid volume, standing below the authored floor,
or using the other creature as a platform. Each body may have a different fixed
supported elevation; neither may rise or descend in this slice.

## Exact interval reuse and bounded extension

Reuse the existing exact rational arithmetic in spatial/geometry.rs:14-37:
Fraction has i64 components, normalizes denominator sign and compares with i128
cross-products. Its existing private segment_interval:48-85 clips a line to an
open box, but returns only endpoints and drops singleton contacts. It does not
preserve the endpoint-inclusion information needed for support coverage. Keep
that old function and public segment_intersects behavior unchanged.

Add an internal XY **positive footprint-contact interval** helper alongside it,
using the same Fraction arithmetic plus explicit lower/upper inclusion flags.
It is an internal geometric query, not a public movement/forced-authority API.
For body origin a, actual footprint width F and delta d, contact with a surface
rectangle S is the intersection of [0,1] with these strict inequalities:

    S.min.x - F < a.x + t*d.x < S.max.x
    S.min.y - F < a.y + t*d.y < S.max.y

Use i64 before subtraction/expansion, not i32 subtraction followed by a cast.
For a nonzero axis, derive exact entry/exit fractions and reverse them for a
negative delta. Its surface bounds are open. For a zero axis, retain the whole
time range only when the constant body coordinate is strictly inside that
expanded open interval. Intersect both axes and [0,1]; clipping to0 or1 includes
that endpoint only if the strict original contact inequalities hold there.
No epsilon, floating point, integer-grid sample or world-bound clamping is needed
for these internal rational bounds. Authored coordinates/body size remain under
their existing validated limits; all products use the existing i128 comparison.

Compute an interval for every eligible support top, or [0,1] for floor. Sort by
lower bound, with included endpoints first, then union intervals in one pass.
Merge overlapping intervals; at equal bounds merge only if at least one adjoining
interval includes that time. Two open endpoints meeting at one time leave a real
unsupported instant and must fail. The union must contain both0 and1 and leave
no gap between them. Separate top surfaces may jointly maintain continuous
contact; no single surface must cover the whole segment. This is temporal union
of actual footprint contact, not spatial union of endpoint boxes.

Expose only a crate-internal query such as
`ground_translation_supported(encounter, from_body, to_position)` to the coupled
evaluator. It should also apply the water exclusion below before a floor fast
path can succeed. It can return the ordinary nonrevealing unsupported/prerequisite
classification; do not expose surface IDs, interval boundaries or gap locations
through player previews or errors. Recompute at the actual next-step cut and
after reactions; the owner rederives the same geometry during original replay.
Intervals need not be serialized into the movement receipts.

Bounds already cap terrain and obstacles at512 each (domain/spatial.rs:519-520).
Each body therefore has at most1025 candidate support intervals including floor,
plus at most512 water intervals. Use O(S log S) sorting plus O(S) union per body,
not walking an unbounded fine time grid. The existing route and continuation
bounds remain unchanged; this query creates no new work node or bytecode.

## Explicit dry-body exclusion

Before accepting either body's segment, reject any nonempty time interval of
positive body-volume intersection with any water terrain. At fixed z/height,
water is vertically relevant when `body.z < water.max.z` and
`body.z + body.height > water.min.z`; combine that with the same XY contact
interval and reject if any intersection time exists. This excludes a body already
submerged, entering water midway, or partially immersed while touching dry ground.
Apply it to both bodies even at floor_z and even if an independent solid top
would otherwise support them. Water-top contact alone grants no ground support.

A dry solid bridge above water remains eligible when its top continuously
supports the body and the body's volume never positively intersects the water.
Touching the water volume's top face without immersion is not invented submersion;
without an independent dry supporting surface it still fails positive support.
This precise boundary avoids turning all water below a legal bridge into a ban.

Source reason: fall_destination:142-149 returns None for an already submerged
body; :191-192 also returns None for same-height contact. Those cases cannot be
distinguished by None. validate_physical_position:64-89 only rejects embedding
and invalid underground positions. Existing source Fly/Hover, buried support,
liquid landing and other fall behavior remain unchanged and grant no permission
to this GroundDragV1 query.

## Required acceptance additions

Use genuine source body sizes and the actual admitted map producer. Exercise
the query through coupled movement and a real table/persistence case, alongside
small exact-geometry controls where necessary:

- Positive full-floor and continuous solid-top routes for both bodies; dry
  supports_top and burrowable tops; overlapping/adjacent support surfaces that
  jointly cover the full interval; a legal narrow ledge under the existing
  positive-overlap convention; and a dry bridge over water.
- A target-only gap while the holder has a continuous dry strip. Concrete valid
  geometry: target footprint x=[0,10] to[10,20], matching y overlap, at z=10;
  obstacle tops x=[-10,1] and[19,30] at z=10. Both endpoints are supported, but
  contact is absent for t in[1/10,9/10]. The pair must stop without moving either
  body or spending this step. Mirror with the gap under the holder.
- Two intervals whose open bounds meet at one instant, versus intervals with
  actual positive overlap. Check diagonal movement with independent x/y entry
  constraints and reverse/negative movement. These controls rule out endpoint
  sampling, midpoint-only checks and incorrect closed-interval union.
- Already-submerged, partly immersed on a floor, water encountered only between
  dry endpoints, water-tagged supports_top, water surface without dry support,
  wrong-height tops, and non-supporting difficult/climbable/obscuring terrain.
  None may acquire ground authority through fall_destination(None).
- Reached-prefix persistence before the failing segment: prior body poses/cost
  remain committed and the rejected pair remains atomic. Cold reopen and portable
  restore replay the same next input. Hidden gap/water differences must retain
  the existing generic reached-place result and unrelated audience equality;
  no geometry explanation or remote feasibility preview is added.

Ground dragging remains incomplete ordinary drag/carry scope. Carry pose/support,
mass/corpse policy, nested and multi-target transport still need their own decisions
and source work. This amendment adds no implementation authorization.
