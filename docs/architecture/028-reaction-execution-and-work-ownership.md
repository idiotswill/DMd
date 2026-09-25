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
version. New application requests explicitly encode `ReactionsV1`. Accepted retry
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

## Compatibility and verification obligations

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
