# ADR 024 — Durable table commands and bounded text interpretation

Status: **Accepted — Gate 3; see the [integrated checkpoint](../checkpoints/gate-03-desktop-table-loop.md).**

## Context

Gate 2 supplies deterministic mechanics but no ordinary desktop table flow. Gate 3 must
connect setup, character creation, attendance, questions, proposed actions, raw physical
dice and exact restart without giving language or UI code authority to alter a save.
Broad autonomous interpretation remains Gate 9; the desktop must preserve unresolved
decisions until supported context exists.

## Decision

`CampaignRuntime` owns the desktop table application service. The desktop host supplies
trusted command metadata from explicit local host/player selection; text supplies only
an application-defined proposal. A name, claimed role, claimed DC or claimed success in
prose never changes the issuer, actor, modifier or outcome.

`table.action_resolved@1` contains the original command metadata, typed table action,
derived player-safe outcome and, where relevant, the nested rules event. Replay reruns
the entire table resolver and compares the derived event. Nested rules use the same
audited command identity and trusted metadata. A host-issued adjudication consumes a
player's pending proposal and creates its rules request in one transition; a player's
raw-dice submission consumes that request and applies its consequence in one transition.
Neither operation is a pair of independently committed writes.

Session start/end composes the state/journal commit with the session ledger compare-and-
swap described in ADR 021. A current bounded session binding is retained in table state;
historical sessions and conversation history remain in their separate ledgers (ADR 008).
An unresolved decision stays in the active session when the application closes. Ending
a session requires finishing or explicitly withdrawing its pending work.

Command IDs survive uncertain replies. Identical retries return the accepted receipt;
reusing an ID with different metadata or payload fails. A correction retains the pending
proposal identity, increments its revision and never rewrites an accepted world outcome.
Correction, proposal and withdrawal transcript entries are labeled as table activity.
Only `TableRejected` proves that input was rejected and may be revised. Storage failures,
failed accepted-receipt lookups/decoding and observation write/readback errors do not prove
non-acceptance; the desktop retains the original request for safe retry.

The deterministic local text adapter recognizes questions, table chatter, corrections,
Second Wind and a single goal matching current host-established context. Negated,
conditional, conflicting or unknown intent remains unresolved. Normal prose needs no
formal commit word. Generic host situation forms establish ability-check context,
including a bounded DC and success/failure descriptions; player prose cannot establish
these. This is a supported Gate 3 path, with autonomous adjudication/interpretation still
explicitly deferred to Gate 9 and noncombat breadth to Gate 5.

Questions and chatter append versioned observations with visibility and the exact head
they observed. They do not change campaign state, command audit or event sequence. A
retry returns its original answer, including after that session ends. Views project
only public situation text and accepted public outcomes; they exclude hidden DCs and
unrevealed consequences. Private character details and private question/answer history
are returned only to their controller or the local host. Recaps derive from accepted
outcomes, never proposed actions, speculative answers or unaccepted backstory.

Character profiles retain licensed creation choices and source-derived grants. The
source kernel rebuilds those profiles for validation; the table cannot supply HP, AC,
proficiency or other arbitrary starting totals. Player-authored backstory is descriptive
input and does not enter world facts. Installed creation catalog bytes must match the
exact compiled source catalog in addition to ordinary manifest integrity checks.
Restored earliest anchors must also match every immutable source-derived mechanical grant.
Only explicit mutable play fields (health, resource counters, death/condition state and
rest/Inspiration history) may differ from the reconstructed initial sheet. Future
advancement/equipment features must deliberately extend this supported-state boundary.

## Verification and consequences

Acceptance requires application tests covering ownership/absence, immutable queries,
stale corrections, raw faces, idempotence, visibility, semantic replay and restore,
plus a real packaged desktop scene and quit/restart at pending work. These tests are
integration evidence; they do not replace the desktop gate review.

Transcript reads currently assemble campaign history before returning the latest 500
entries; recap returns the latest eight accepted outcomes. This bounded presentation
does not delete history. Indexed cursor-based reads and measured long-campaign costs
remain receiving work for Gates 7/13/14 under the existing persistence/performance debt.
