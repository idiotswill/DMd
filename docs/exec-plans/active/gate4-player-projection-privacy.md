# Gate 4 durable player projection protocol

Writer: environment_audit. Branch `codex/gate4-player-projection-privacy`, base
`9fd73da`. Root owns canonical integration and PRs; area owns combat ordering and
shared tactical work. This branch owns transport/presentation persistence, application
projection/retry composition, desktop IPC and the corresponding UI boundary.

## Objective and authority

Remove canonical event numbers and hidden continuation counts from player requests,
views, receipts, observations and transcript entries while preserving exact accepted
retries, canonical game history, local recovery and the selected player's authority.
Product definition private-information, identity and recovery clauses; Gate 4 hidden
information/UI/log and exact mid-combat resume requirements; ADRs008/012/020/021/024
apply. This does not change tactical ordering agency or claim timing invisibility.

## Audited current defects

The current desktop submits `expected_event_sequence`; TableView, TableReceipt,
TableObservationBody.meta and transcript entries return the canonical number. Every
table event also produces a transcript entry, including hidden tactical continuations.
Filtering only work cards cannot hide the number of secret resolution steps.

The current table service resolves outside its commit transaction, and observations
use a separate write transaction. Replacing a number with a token at the IPC boundary
would still create a stale-response oracle and a crash window. A complete fix must
bind revision validation and the derived canonical head to the same SQLite writer
transaction as acceptance and durable retry/presentation history.

## Design and boundaries

1. Keep canonical CommandMeta, event sequence, game resolver/replay and internal
   accepted receipts intact. Introduce explicit transport DTOs and audience identity;
   no raw CommandMeta or canonical sequence is serialized in a player response.
   Host-only diagnostics may retain the canonical head. New requests retain a typed
   opaque audience revision and original channel/session/action or text.
2. Persist versioned audience revision/presentation history and accepted transport
   bindings outside CampaignState. These are presentation/idempotency records, not
   a second game-state authority. A revision is an opaque UUID, never a counter or
   hash of hidden state. It advances on an actual audience-visible presentation
   change; A -> B -> A receives distinct revisions. Hidden-only commands retain the
   existing player revision. Private observations update only their actual audience.
3. Reserve the SQLite writer before reading accepted bindings or revisions. Recover
   an identical accepted nonce/body first, even after session closure or identity
   projection changes. For new input compare the audience revision, derive the exact
   current canonical head, validate channel/actor, resolve and atomically commit the
   game or observation together with the transport result and presentation updates.
   Reuse of a nonce with different body, revision or channel is rejected unchanged.
4. Retain acceptance-time transcript visibility per audience and validate it by
   replaying the original historical before/after state. Hidden-only tactical steps
   are host-only; later discovery cannot reveal their old entries. Legacy journal
   bytes remain immutable. Bootstrap old histories through authenticated replay,
   preserving accepted legacy retries but rejecting unaccepted numeric-head input
   with a refresh requirement. Never reinterpret an old nonce against a new head.
5. Extend the portable envelope deliberately for typed presentation/transport rows
   and validate the whole ledger against canonical events/observations before any
   restore write. Keep the state schema at 4: no new game-domain field is needed.
   Export-format versioning and SQL migration are part of this slice, with legacy
   format import tests; old software must not silently discard new retry authority.
6. Factor connection-scoped persistence primitives so application composition uses
   one transaction, including single-connection SQLite pools. Do not acquire a second
   pool connection or introduce a process-only mutex as the atomicity guarantee.
7. Reuse the existing renderer outbox. Keep the original request until a typed proven
   rejection or accepted response. Key forms by visible revision, not canonical head;
   strip event numbers, raw provenance and ordinal-derived identifiers from player
   serialization. Do not broaden local selected-channel trust or grant NPC control.

## Implementation slices

- Define typed protocol/storage contracts and SQL/export compatibility, then factor
  transaction-scoped journal/observation helpers without changing legacy behavior.
- Extract a pure audience presentation builder, historical transcript classification
  and durable revision derivation/validation. Bootstrap existing authenticated history.
- Compose atomic request/observation acceptance and exact response recovery. Migrate
  desktop IPC and outbox DTOs; preserve legacy accepted request recovery explicitly.
- Add paired-history privacy, crash/reopen/export/retry, ABA, concurrent-write,
  foreign-audience, changed-body and historical-visibility corruption regressions.
- Run focused persistence/app/frontend checks, strict lint, independent full/exact-head
  review. Root owns canonical verification/CI and packaged acceptance.

## Acceptance evidence required

Two histories sharing the same initial visible state and request identities but
differing only by secret continuations produce identical serialized player views and
unchanged revision. A request made against that revision remains valid despite the
secret commits. A later visible change makes it stale; visible A -> B -> A cannot
revive the older token. Another player's private answer does not rotate this view.

Accepted action and observation responses remain exactly recoverable after process
restart/session end; a retry precedes current stale/attendance checks. Changed payload,
audience or revision with the same nonce fails. Crash/failure injection proves no
state/event without binding/revision and no binding without accepted state/observation.
Portable restore preserves response/revision history and rejects tampered historical
visibility, token transitions or request provenance atomically. Legacy accepted v1
requests recover; unaccepted legacy requests explicitly require refresh.

## Status, risks and next action

Current table/runtime/persistence/IPC paths and bootstrap's privacy design are read.
The transaction boundary issue above is concrete and reported to root. No production
code or schema has changed yet. Next settle the exact storage/transport types, then
implement the first coherent persistence slice. No Rust compiler is authorized until
the area app and root turn-core checks release the shared slot. This plan and later
unit proofs do not establish full Gate 4 or timing-channel acceptance.
