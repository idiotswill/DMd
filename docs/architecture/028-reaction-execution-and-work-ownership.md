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
