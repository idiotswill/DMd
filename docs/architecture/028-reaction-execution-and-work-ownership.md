# ADR028 — Versioned reaction execution and owned nested work

Status: Accepted architecture; implementation and acceptance in progress.

## Context

Mandatory Shield/Counterspell opportunities and Ready responses add real pauses to
an accepted combat action. Replaying an old Begin and attack with those new pauses
would invent a different historical outcome. A caller must also not suppress a
mandatory source reaction by omitting a new request field. Area invocation consent
currently applies to a dedicated resolution; it must not authorize ordering an
unrelated action which later interrupts that resolution.

## Decision

`TacticalAction::Begin.execution` is a typed semantic execution version. The old
wire omits it and decodes to `Legacy`; its retained `TacticalFlow.version` is 1.
`ReactionsV1` retains flow version 2. New live admission requires the current
version. The first reaction foundation explicitly encoded `ReactionsV1`; the
accepted-hit Shield slice advances fresh requests to `ShieldHitV1` (flow version 3).
Accepted retry
lookup precedes new admission, so accepted old requests recover their original
response first. An existing canonical `table.action@2` envelope is interpreted
under its original request semantics; replay never fills an omitted execution
field from today's default. Any future normalization change needs its own retained
envelope version rather than silently reinterpreting old requests.
Historical tactical replay has a separate internal policy which accepts the old
version and reproduces its state image. A public payload cannot select that policy.

An active legacy encounter upgrades through the journaled `UpgradeExecution`
action, authorized to the host/system only and admitted when its current source
work and raw request are settled. It preserves budgets, inventory, HP, initiative,
concentration and past events. Until then, saved continuation choices and rolls can
complete under the original semantics, but a new ordinary action cannot proceed.
There is no silent upgrade and no public switch back to a weaker executor.

`UpgradeExecution` remains the original 1-to-2 transition forever. The separate
typed `UpgradeExecutionTo` action advances a settled active older flow to the
current version only when no resolution, raw request or paid Ready declaration
would be reinterpreted. Flow 3 adds accepted-hit Shield windows for physical,
intrinsic and spell attacks, including opportunity attacks; it does not change
Magic Missile, add Counterspell or execute Ready releases. Those required Gate 4
families need their own verified implementation and a further version when their
execution changes. A player-owned source must have explicitly activated source
access before a fresh flow 3 encounter or upgrade can require that owner's response.

Ready declarations live on the flow across other actors' turns. Their paid Action
and original command are distinct from the later trigger and Reaction payment.
Held spell identity and expenditure survive between resolutions. A local work key
is always `(resolution origin CommandId, occurrence)`; original source/declaration
identity is separately retained when it attaches to a later resolution. All raw
requests continue to use deterministic source roles and identities.

Nested responses use the existing resolution and frame stack. A typed parent
attachment preserves any interrupted physical action; it is not a second command
queue. Current-turn ordering authority, responding actor control and source rolling
actor remain separate. An area delegation may cover only its own admitted source
work and authenticated descendants. It cannot become resolution-wide authority
because an area record is present. An independent Ready/reaction action establishes
its own control boundary even when caused by a delegated area occurrence.

The production layer rederives actual source grants, components, perception,
range, reaction/slot availability and current source timing before accepting a
response. A string circumstance, opaque transport handle or retained offer alone
does not grant permission. Optional responses remain explicit controller choices.

## Same-trigger timing and private decisions

SRD p.10 places a Reaction immediately after its trigger unless its description
specifies otherwise; Ready acts after its perceptible trigger finishes (pp.186–187).
Counterspell interrupts a spell still being cast (p.120), while an Opportunity Attack
precedes departure (p.15). These are distinct retained timing boundaries. SRD p.187
assigns the order of things happening at the same time to the current-turn controller.
The source does not prescribe a network response protocol; the following collection
and ordering stages are the application's interpretation of those timing rules.

Every eligible controller receives only its own source-safe offer and retains an
explicit accept/decline decision against the same canonical trigger occurrence.
Command arrival does not select which accepted reaction happens first. Every public
trigger presents a uniform current-turn decision stage for zero, one or many private
competitors. Optional delegation to the host is offered independently of private
eligibility, and retains authenticated consent for that exact trigger and turn. A
prompt introduced only after discovering private competitors would itself leak their
presence through the audience's DTO or revision and is forbidden.

Once the same-occurrence decisions are collected, competing accepted responses are
ordered by the current-turn controller, or the host under that explicit delegation.
The application neither reveals hidden offers nor silently substitutes host or
transport priority. Consent covers only the identified trigger's simultaneous set;
it does not transfer to a child-created trigger. Hidden-only intents must leave
unrelated audience DTOs, revisions and transcripts unchanged.

After each chosen response and its nested consequences finish, remaining responses
are rechecked for current source timing, resources, perception and other prerequisites.
A retained acceptance is intent, not prepaid permission: an invalidated response
cannot spend resources or act on an expired target. A new trigger caused by the
selected child response gets its own nested window and its own decisions before the
parent competitors resume. It does not join the parent's simultaneous set merely
because both windows were transported during the same wall-clock interval. Existing
area-ordering consent does not delegate these independent reaction choices.

### A total order without disclosing private competitors

The uniform turn-controller decision records an explicit ordering instruction over
potential respondents, independently of which respondents accepted. It is not a
list of the private accepted set. The controller may rank any currently known
participants and place the unlisted participants before or after that list, using
either forward or reverse established initiative order within the unlisted set.
The existing initiative order is already total, including resolved ties. Selecting
this fallback is an actual controller choice; the application supplies no automatic
default and never uses UUID or network arrival as fictional priority. Exact-trigger
host delegation remains a separate explicit option.

This lets the controller make a material ordering choice even when some competitors
are unseen, without revealing their count, identities or offers. Ranked identities
are admitted against the controller's existing knowledge, not reaction eligibility;
the same known-participant controls appear with zero, one or many hidden responses.
Once all decisions are collected, apply the recorded total-order instruction to
the accepted set and revalidate each selected response as above. The instruction
may be supplied before private collection completes; it is immutable after its
acceptance and applies only to this occurrence. Each child trigger requires its own
explicit instruction, even if its parent chose the same initiative direction.

This is an application interpretation of the controller's source ordering authority,
not an automatic host override. Tests must show that reversing the instruction can
change the first eligible responder, that opposite network arrival produces the same
selected order, and that unknown or ineligible respondents never alter the ordering
controls, unrelated audience digest or revision during private collection.

## Magic Missile target responses and same-time impacts

This decision is approved for the next bounded Gate4 executor, provisionally
`ShieldMissileV1` (flow4), but is not an implementation or acceptance claim. The
current flow3 behavior remains unchanged until that versioned runtime is verified.
The detailed plan is `docs/exec-plans/active/gate4-shield-missile-runtime.md`.

SRD5.2.1 p146 gives each Magic Missile dart its own 1d4+1 Force damage and requires
the darts to strike simultaneously. Shield responds to being targeted by that spell
(pp161–162), and p187 assigns same-time order to the current-turn controller. The
p16 shared-roll rule addresses simultaneous saving-throw targets; it does not by
itself establish one shared Magic Missile d4. The source does not fully specify
network collection or the grouping of repeated-target damage/consequence work.
The following is the application's explicit adjudication of those boundaries.

The real cast command binds every dart occurrence and its chosen target, including
repeated targets, before any target response or damage face. There is one private
Shield respondent per distinct targeted creature. Every target owner acknowledges
collection even if no response is available; this is cost-free coordination, not a
fictional action or spent Reaction. The uniform current-turn ordering stage is
independent of private eligibility/acceptance. It uses the same explicit potential-
participant order and exact-trigger delegation rules as other reaction collection.
Each selected respondent casts or declines through a fresh current-head command;
permission and costs are rederived from its actual source, components and budgets.
All selected pre-impact Shield responses finish before amount collection proceeds.

Collect and retain a separate actual d4 face for every admitted dart before the
first missile vitality change. The approved first implementation retains all faces,
including darts whose damage Shield prevents. No shared roll or invented zero face
replaces those records. A deterministic order of requesting amounts is bookkeeping;
it does not choose impact order. Only one existing raw request is pending at a time.

After collection, put every committed dart impact in one sibling work frame. The
actual current-turn controller selects individual occurrences, with meaningful
source-safe labels that distinguish darts even when their target actor is the same.
Neither transport arrival nor an actor-only ranking can decide that material order.
The earlier response-order delegation and an unrelated area's delegation do not
authorize the impact set or a child-created response window.

Each selected dart is a separate Force damage instance, using ordinary defenses,
temporary HP and vitality. Each actual damaging instance can produce its own
concentration save or damage-at-zero failure under SRD pp17–18/179. It is not an
attack, critical hit or melee knockout source. Normal nested consequence frames
drain before selecting the next sibling impact; concentration loss can therefore
remove dependent effects before the next dart resolves. This is not a global HP
aggregation or a requirement to defer all consequences until every HP mutation.

Committed strike identity survives those consequences. Do not re-run the original
cast's range/perception/source permission or erase/retarget a remaining dart because
a prior child changes position, incapacitates/kills the caster or ends concentration.
An already-dead recipient gets a retained no-effect completion for its selected
occurrence, without revival or invented corpse HP. Preserve every original target,
accepted face, impact occurrence and causal receipt; damage prevention does not open
a new target-trigger window after the amounts are known. Distinguish original cast,
trigger, amount acceptance, selected response and later execution metadata rather
than retiming a command or fabricating an issuer.

These records live inside the existing resolution and frame stack. A typed parent
distinguishes hit-Shield from missile-target Shield; no fabricated attack or second
queue stands in for missile targeting. The complete partition must prove that every
dart occurs once across amount collection, pending impact/children and completion.
Independent response casts keep their own authority boundary in retained ancestry.

Flow1, flow2 and flow3 accepted behavior remains frozen, including flow3's sequential
Magic Missile and saved hit-response pauses. Numeric flow0 remains invalid, absent
pre-tactical authority remains absent, and the original unit upgrade remains1→2.
New live Begin/forward upgrade admission and feature support for hit-Shield are
explicitly versioned; source program fingerprints, existing roll-role tags and old
transport/presentation bytes are not rewritten. Before flow4 runtime work, genuine
unchanged-flow3 missile and owned Shield scenarios must pass and their actual pause
exports, accepted bindings and pre-tactical anchors must be restored and frozen.
The new slice may develop after that capture gate while PR45 verification finishes;
its acceptance/merge requires reconciling verified PR45 main and checking the exact
integrated head. Counterspell, Ready release and off-turn missile producers remain
separate required Gate4 work with their own source and ownership acceptance.

## Compatibility and verification obligations

### Authenticated encounter release (accepted through PR48)

PR48 is accepted at `dbf1d633460473183324b4ec519e8d1980884b5c`; the
[release plan](../exec-plans/active/gate4-encounter-release.md) records separate
successful literal-main checks. The source8c paragraph below preserves its
historical candidate evidence and then-pending integration requirements.

`EncounterReleaseV1` owns flow 5. Counterspell requires a later boundary. The
initial domain/rule checkpoint defined this executor and inherited hit/missile
predicates. The subsequent application checkpoint enables current admission only
alongside authenticated finish, closed-session upgrade, replay, replacement and
desktop controls. Integrated8c03f9f passes canonical/CI historical continuations and
actual packaged two-encounter recovery; the encounter-release plan records precise
scope and remaining merge/main proof. Flows1–4 keep their accepted meanings and
the historical unit upgrade remains 1→2.

Explicit conclusion is narrative evidence that fighting stopped. A separate host
release may retire initiative only after a pure whole-campaign scan proves no owed
work or stranded timing dependency. It may also admit a narrowly explicit host
forward upgrade of an already concluded, closed legacy aftermath using the same
complete preflight. Neither command is a rest, clock advance, resource refund,
pickup, effect dismissal, synthetic Start/End or proxy decision by another owner.

Persist an omitted-when-absent encounter history containing ordered immutable
completion receipts and separate old scene spaces. A receipt retains encounter,
scene and location IDs, original setup/initiative/conclusion metadata, actual
release metadata, predecessor release ID, unchanged instant and final global
turn/actor. No HP, resources, concentration or equipment are duplicated. Every
encounter/scene identity remains unique across all receipts. Strict journal replay
must reconstruct every field; structural checks alone cannot authenticate a receipt.

Each scene space preserves its battlefield, participant identities for attached
light validation, exact loose-item positions and original drop commands. Custody
and ownership stay authoritative in the existing item store. Release marks the old
scene Closed while retaining its presence history, and keeps the Finished encounter
attached until atomic replacement. A new scene may share the location but must use
a fresh scene ID. A physical item cannot occupy both old and new spaces.
Replacement stages its new scene Closed without moving old participants, validates
the retained Finished state, then changes selected actor locations and activates
the new scene atomically with the new encounter. Failed placement preserves the
entire old state. Source/equipment preparation and reduced session attendance are
available only after validated Finished state. The new host-only finish capability
is omitted from old execution projections and every player projection.

The next initiative starts at checked last-final-turn plus one, with round one.
Savage Attacker's existing scalar remains unchanged and is validated against the
authenticated completion highwater while timing is absent, including Finished,
replacement setup and pending initiative. Historical states without receipts keep
their previous rejection. Retained source recharge requests/results must remain
linked to their original authoritative recorded rolls outside Active combat too.

The scan includes all table/raw/central/source work, global Inspiration transfer,
paid attack grants, Ready, Reaction expenditure, active and suppressed relative
effects, Turn/per-turn/zone clauses, orphaned Casting groups, dying actors, recovery
dice and due absolute deadlines. Surviving effect/group/legacy sources, targets,
concentration owners and recovery/rest actors must collectively fit the actual next
setup route and its current source/equipment eligibility. A dead Mage's surviving
nonconcentration Mage Armor deadline blocks this bounded release; a missing corpse
timing route cannot be discovered only after initiative has already been removed.
An offstage dependency in another Active scene also blocks this bounded route.

Full acceptance requires genuine unchanged-source flow 4 exports and baselines,
two real encounters through app/UI, retained original byte retries, independent
SQLite/portable continuation and negative current/retired snapshot proofs. The
details and deferred Gate 4 timing work are in the encounter-release execution
plan. Pure fixtures or newly declared records do not meet that acceptance.

Old Begin JSON must round-trip without an added default field, replay to the old
flow image and reject as a fresh weak live request. A saved legacy pause must still
finish; non-idle and foreign upgrades must fail without writes. New typed fields
fail closed in older binaries through existing strict tactical-state decoding.

Reaction tests must prove actual source admission, one queue, cost and raw-face
preservation, nested cancellation/resumption and source expiry. Ordering tests must
include delegated area descendants alongside unrelated Ready/OA parent or sibling
work and forged scope/ancestry. Application tests must exercise genuine creation,
physical source resources, cold SQLite reopen, exact accepted retry, independent
semantic replay and hostile current/historical anchors. Until those pass, the new
contracts and source helpers are implementation milestones, not Gate4 completion.
Timing regressions must accept the same private decisions in opposite arrival orders,
preserve the current-turn controller's material order choice, and distinguish a
Counterspell of a selected child spell from another response to the original spell.
Player DTO/revision tests must compare zero, one and two private competitors and prove
that a child window cannot reuse its parent's delegation.
