# ADR027 — Durable audience presentation and transport history

Status: Accepted design; Gate 4 implementation and integrated verification in progress.

## Context

Canonical event numbers are necessary for journal ordering, replay and atomic state
transitions. Returning those numbers to players also reveals how many hidden commands
were accepted. The same leak existed in nested pending metadata, observation replies,
transcript entries and deterministic tactical request/work identifiers. Suppressing a
private work card cannot close these other channels.

Replacing the number in a renderer alone is insufficient: a later stale-head check
would still reveal a secret intervening command, and a process crash between command
acceptance and response retention would make safe retry uncertain.

## Decision

Keep CampaignState schema 4 and canonical game event/reducer version 1 unchanged.
Desktop requests use a versioned envelope with the original nonce, selected channel,
session, typed input and an opaque audience revision. The application derives actor
authority from the selected character, independently of the current turn actor.

A SQLite `BEGIN IMMEDIATE` transaction owns original-binding lookup, revision
comparison, canonical-head derivation, gameplay or observation acceptance, presentation
updates and the exact retained response. An identical accepted request is recovered
before current ownership, attendance, pending work or stale-revision checks. A changed
body, channel, session or revision cannot reuse its identity. Storage uncertainty keeps
the original request available for retry.

SQL0011 stores immutable presentation history and transport bindings outside the game
state. Each audience receives a random UUID revision. A revision changes when that
audience's presentation changes, including replacement of a visible prompt; it does
not change for a hidden-only command. A visible A → B → A transition does not revive
the first revision. Canonical projection hashes exclude revision, host diagnostics and
internal ordinal encodings; hashes are validation evidence and are never exposed as
revision tokens. Actual roll/work capabilities map to random audience-specific handles.
Surviving prompts retain their handles; a new prompt receives a new identity.

Canonical `table.action@2` audits and `table.conversation@2` observations retain the
original transport envelope. Their game action, event outcome and observation body
retain existing semantics. These markers make stripped presentation/binding history
distinguishable from genuinely legacy history. Unknown or malformed new envelopes
never fall back to v1 interpretation.

Presentation validation visits the same authenticated anchor-and-journal replay used
by gameplay restore. Mechanical transcript entries are classified against historical
before/after audience projections. Public declarations, corrections and withdrawals
retain their explicit table-utterance audience even when only one player's private
decision controls changed. Party observation membership is evaluated at its original
semantic image. Later discovery or joining cannot reclassify those old records.

An atomic, constrained first-use bootstrap records replay-derived historical visibility
and initial audience revisions. It rewrites no old journal/snapshot bytes. If an opaque
pre-table anchor prevents reconstruction of an older entry, that entry remains a host
diagnostic; the application does not invent historical player knowledge.

The portable envelope advances explicitly to export format 3 and includes both typed
ledgers. Formats 1 and 2 remain importable only without future protocol authority.
Older importers reject format 3 instead of silently dropping retry or privacy evidence.
Restore structurally validates the ledger/canonical-acceptance joins and semantically
replays audience visibility, revisions, capabilities, requests and responses before
writing anything.

Legacy numeric desktop endpoints are recovery-only. They compare a genuine original
v1 acceptance and its historical channel/context, then return a sanitized acknowledgement
without canonical heads or provenance. An unaccepted legacy request requires a refresh;
its identity/head are never translated into a newly accepted command. Trusted internal
v1 table APIs remain available and atomically maintain any initialized presentation
history. Existing raw rules/tactical APIs still reject table campaigns before writes.

## Consequences and verification boundary

### Evolving read-only choices

Version1 presentation bytes are a historical contract, including the originally
embedded creature picker. A later catalog entry must not change old host digests.
Keep that picker as an immutable version1 wire snapshot, authenticated by genuine
pre-change save replay. It is compatibility presentation only, never a rules grant.
Current creature creation choices use a separate host/revision-bound read-only query,
as current Savage Attacker choices already do. Such queries do not create revisions,
rewrite history or prepay authority; the actual creation command still validates the
current source, controller and physical identities. The desktop must show the current
query result and discard replies after a campaign/channel/revision switch.

Any change to authoritative source instances or existing presented game facts still
needs its own compatible version policy; moving a picker does not license changing
saved rules or ignoring historical digest validation.

The renderer keeps its single durable outbox and saves the full original versioned
request before invoking the desktop. Forms use visible revisions; raw game state,
numeric heads, pending CommandMeta and internal work ordinals are absent from player
DTOs. Host diagnostics may retain the canonical head. Local selected-channel trust and
controller ordering agency are unchanged.

This protocol does not promise timing invisibility, hide human dice activity, or grant
remote authentication. Full historical validation currently adds work proportional to
retained history; future optimization must preserve the same authenticated boundary.

At this decision checkpoint, four focused persistence tests and eight application
protocol tests and strict persistence/application all-target Clippy have passed.
Desktop/UI verification is still pending, and the verified area integration must
exercise a real secret tactical
continuation. Native packaged verification and exact gate acceptance remain root-owned.
The active execution plan records current evidence and remaining work.
