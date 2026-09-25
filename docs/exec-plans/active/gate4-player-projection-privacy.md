# Gate 4 durable player projection protocol

Writer: root (taken over after the supporting agent reached its usage limit).
Original writer: environment_audit. Branch `codex/gate4-player-projection-privacy`, base
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
code or schema had changed at the plan checkpoint. The first source slice now factors
journal acceptance, observation append and export onto the caller's transaction while
retaining legacy pool wrappers. A one-connection rollback/commit regression is drafted;
it has not run. Typed immutable presentation/retry storage is the next slice. No Rust
compiler is authorized until
the area app and root turn-core checks release the shared slot. This plan and later
unit proofs do not establish full Gate 4 or timing-channel acceptance.

The surface audit also found nested pending-declaration CommandMeta and numeric work
occurrences. Player presentation must omit pending provenance and use retained opaque
work/roll capability handles; canonical raw roll IDs can otherwise be enumerated from
their deterministic occurrence namespace. Handle translation changes no ordering
authority or source roll facts. The root's newer `roll_channel` projection correction
was integrated from `ba028fe` before updating desktop request routing.

The transaction checkpoint is `4a4f84d` (uncompiled; one-connection atomicity regression
drafted). The next storage draft adds immutable protocol tables in SQL0011 and portable
format 3 with explicit format-1/2 import; state schema remains 4. Generic validation
checks ledger continuity, original canonical acceptance, audience and revision/handle
shape. Application validation must still verify historical visible digests, capability
membership and exact typed response; structural roundtrip alone is not semantic proof.

New protocol requests will also be retained in versioned canonical audit/observation
envelopes. This lets replay distinguish a stripped protocol ledger from genuine old
history without changing the underlying game event or its resolver. Malformed new
envelopes never fall back to legacy interpretation. Initial presentation bootstrap is
a constrained, atomic record of replay-derived historical visibility and current
audience hashes; it changes neither canonical state nor old journal bytes.

The application checkpoint now extracts the trusted view builder and an authenticated
replay visitor. Explicit desktop DTOs remove canonical heads, pending CommandMeta and
work/roll occurrence identities. Versioned request markers are retained in canonical
audit/observation envelopes; a missing protocol ledger cannot masquerade as legacy.
The same SQLite writer handles request comparison, derived head, game/answer commit,
revision/capability updates and exact response. Trusted internal v1 table writes also
maintain an initialized ledger. Desktop migration and its accepted-only legacy bridge
remain unfinished; this is not an externally usable protocol checkpoint yet.

The storage reviewer found and closed a binding-to-presentation-cause gap (including
an ordinal-to-Bootstrap corruption regression). App review found that digest equality
alone suppressed public declarations/corrections/withdrawals; these now retain their
explicit acceptance-time party audience. Mechanical child entries still require an
actual audience-visible state change. Observation party membership is reconstructed
at its historical semantic image, so later players do not retroactively acquire old
party messages. Eight application regressions cover accepted-only legacy recovery,
hidden DC history/private answers, shared utterances, exact session-ended retries,
ABA, rollback, hostile restore and opaque roll handles.

## First verified backend checkpoint

The first persistence batch passed all four tests: three presentation-storage cases
and one caller-owned transaction composition case. The first application batch passed
five of eight cases; three fixtures incorrectly ended a session with a pending
declaration or used an unsupported correction prefix. Fixtures now withdraw before
ending the session and use the actual supported correction grammar. All eight then
passed on the normal Windows test stack. No production assertion was relaxed.

Strict persistence/application all-target Clippy subsequently passed after boxing the
internal action intent and two style corrections. Logs are retained outside the
repository under `tooling/logs/gate4-protocol-storage-tests.log`,
`gate4-protocol-app-tests.log`, `gate4-protocol-app-tests-r2.log`, and
`gate4-protocol-clippy-r2.log`. This evidence is focused; no complete workspace or
packaged desktop claim follows from it.

The desktop transport and renderer migration are authored but unverified. Their
outbox preserves both original modern envelopes and genuine legacy retry bodies;
modern rendering uses opaque revisions and work handles. ADR027 records the durable
protocol/export boundary. Next: independently review and checkpoint this backend,
merge the verified area application, and add a real hidden AreaSave continuation,
file reopen/concurrent acceptance and historical party-membership regressions.
Then check the desktop/UI with a single worker and run the appropriate backend batch
after the shared compiler is released. Native packaged privacy acceptance remains
root-owned and outstanding.

Backend checkpoint `7aff85c` has a fresh independent bounded review of the complete
history/transaction/legacy-recovery paths and eight tests; no remaining finding was
reported. Desktop envelope migration `d6c1e69` remains pending executable checks.
Verified area application `a45fe57` was merged as `7c5abe9`, keeping paid shield controls,
explicit area ordering consent, controller-owned dice and opaque work handles.

The next draft adds three cases (eleven protocol cases total): genuine hidden Wolf
AreaSave selection/raw completion with two accepted private commands; independent
SQLite-connection contention and exact on-disk reopen; and historical Party membership
after a later player joins. Hostile restore additionally changes a returned revision
and canonical roll capability. The three new scenarios have independent read-only
review, but have not run. Desktop tests now call the actual presented application
service instead of retaining a test-only copy of the former identity derivation.
The shared heavy-build slot is with root PR33; this branch is read/edit-only until
explicit handoff. No new test result is implied by the reviewed draft.

Fresh IPC/UI review of `baf5d2f` is clear, including exact modern retry bodies,
recovery-only legacy endpoints and the actual presented-service desktop test.
The concurrent file fixture also now drops every closed runtime/pool before cleanup
and uses the bounded Windows sharing-lock cleanup helper reviewed for PR33. That
CI run exposed OS32 only after all of its gameplay assertions had completed; this
fixture retains every protocol assertion and still fails if deletion cannot finish.
The expanded protocol/UI batch remains unrun pending the shared heavy-build slot.

Before that batch, root requested deliberate integration of physical PR33 source
`54520a9`. Squashed ancestry required resolving equivalent added files against the
independently reviewed area aggregate `c79ddc5`. The combined domain/rules trees
match that aggregate exactly; newer dead-target, shield-training, cold-round and
genuine OA/dead SQLite regressions are retained. The projection extraction already
contains the authoritative roll-channel correction, so the old in-method view body
was not restored over it. Modern revision/handle tests coexist with shield, area
and prepared-map dice tests. Unrelated plan moves/statuses follow current PR33;
ADR027 and this protocol plan remain owned here. Reconcile the eventual protected
PR33 squash/main head before opening a protocol PR. This merge has formatting and
tree-parity evidence only until the next executable batch finishes.

Current-source verification was attempted after root released the heavy slot, on
`a84f8d4` (the reviewed PR33 cleanup followup is included). The first UI check failed
inside PowerShell's binder before a source result. A direct serial retry reached
Vite but its esbuild child could not spawn; Node then reported `Committing semi
space failed` at roughly 68 MB heap. This is observed host commit-memory exhaustion,
not a TypeScript/test assertion failure. Heavy work stopped; no Rust batch or
additional UI retry was started, and no user process was killed. Root and the next
compiler owner were notified. Resume the same frontend/backend batch only once
the shared host has sufficient memory; all current-phase acceptance remains open.

After memory recovered, the serial frontend batch passed on a84f8d4:55 tests,
zero Svelte errors/warnings and production build134 modules. Logs end in
`gate4-protocol-ui-{check,tests,build}-r3.log`. The current persistence batch also
passes all4 protocol/transaction cases (`gate4-protocol-storage-current.log`).
The following app compile exposed one integration fixture omission: the newer OA
scene constructor lacked area's required `area_grid_policy` field. It now supplies
None, like the other non-area scenes; production and all assertions are unchanged.
Root took over the stopped branch and sole compiler slot, and will rerun app tests
and strict lint. Eleven protocol cases, combined app acceptance and native package
verification remain pending until their actual results are recorded.
