# Gate 1 — Campaign persistence foundation

Status: **Acceptance review complete — pending explicit human approval**

Gate 1 turns the Gate 0 state model into a durable, multi-session campaign persistence foundation. This checkpoint does **not** claim that DMd is yet a playable finished game. The finished-product contract in `docs/product-definition.md` remains unchanged.

## Product-definition traceability

Gate 1 advances the requirements that:

- authoritative campaign truth survives save, exit, restart, and supported recovery;
- accepted state transitions preserve trusted issuer/provenance and append-only event history atomically;
- unrelated campaigns remain isolated in one installation;
- persistence corruption and projection drift fail closed and have replay-backed recovery paths;
- campaign lifecycle operations do not require ad-hoc database surgery;
- exact persisted ruleset/content references must resolve before application gameplay receives a runnable campaign;
- the core persistence/content path remains local-first and network-independent.

Gate 1 deliberately does **not** complete voice/table UX, full gameplay rules, combat/exploration/social/travel/downtime loops, living-world simulation, procedural materialization, Director behavior, desktop UX, content installation UX, or long-horizon endurance play. Those remain later product work.

## Implemented and merged slices

### Slice A — atomic authoritative commit and journal

Status: **Accepted and merged**

PR #6 merged as `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`; ADR 011 is accepted.

Verified properties include atomic material transition persistence of resulting state, trusted command audit, immutable events and causal edges; persistence-owned contiguous sequence allocation; stale-state rejection; campaign/session/issuer/actor/provenance validation; one-read-snapshot recovery checks; and database-level direct update/delete guards for immutable history.

### Slice B — snapshot migration and replay

Status: **Implemented, merged, and technically accepted; ADR-status reconciliation pending human confirmation**

PR #7 merged as `40a7cf001f51028a98195ada28e2e1f7dbd5af84` after exact reviewed head `2e0b2eb931a7586b573275de20e9077c53676633` passed CI #157. The merge commit records “Accept Gate 1 Slice B”.

Verified properties include immutable sequence-keyed snapshots; sequence-0 snapshots for new campaigns; migration-time backfill at the materialized head that actually exists; explicit state-schema migration registration; replay head derived from append-only journal history; typed replay appliers rather than persistence-side JSON inference; and fail-closed handling for journal gaps, missing/future snapshots, unsupported event versions, campaign/schema mutation, and invalid provenance.

`docs/architecture/012-snapshot-migration-and-replay.md` still says “Proposed for acceptance”. The implementation/merge record is consistent with acceptance, but the durable ADR status will not be changed by this closeout until explicit human approval is recorded.

### Slice C — derivative query projections

Status: **Implemented, merged, architecture accepted**

PR #10 merged as `f4473745bb44e6fa112ae87833c9336553304c20`; final PR head `cf2d106ba5a406da375d5a0ab4ff0182859a20cc` passed CI #221. ADR 013 is accepted.

Verified properties include campaign-scoped normalized projections for current `CampaignState` record families; same-transaction projection maintenance with authoritative state changes; fail-closed head/count validation; campaign isolation; replay-backed deterministic rebuild; malformed/stale materialized-state recovery; rollback of the authoritative transition when projection writes fail; and persistence across database reopen.

### Slice D — versioned rules/content manifests

Status: **Implemented, merged, architecture accepted**

PR #11 merged as `88f813e3afc07d51d2d62416a7b11747acc956f7`; final PR head `e3e851e2da1614e3406dd488aeadada8afabf661` passed CI #249. ADR 014 is accepted.

Verified properties include exact `(kind, id, version)` identity; no latest/closest version substitution; closed manifest schema; explicit engine content-contract compatibility; exact content-pack ruleset/dependency checks; deterministic network-independent discovery; duplicate/malformed/missing/incompatible failure; declared-file integrity checks; path traversal and symlink rejection; and provider/LLM non-authority.

### Slice E — campaign lifecycle and portability

Status: **Implemented, merged, architecture accepted**

PR #9 merged as `f3f18568be62b49458f8085d9ecc635ff87162d3`; final PR head `c8b0b25c28c57146bb9706b0c462e4b9b4ac8007` passed CI #259. ADR 015 is accepted.

Verified properties include durable create/open/list/archive; versioned export; transactional restore; admin-only whole-aggregate purge with an exact validated backup; stale-backup detection; database-enforced protection against selective history surgery even when a purge authorization row exists; mechanical aggregate cleanup from the authorized root delete; restore validation before writes; rollback on collision/corruption; restart/reopen; and unrelated-campaign isolation.

### Slice F — runnable campaign application composition

Status: **Implemented, merged, architecture accepted**

PR #12 merged as `cc71e9e16c3e420842c5d13518ef7c20837aed1d`; final PR head `a60bf18fa3f142d431d57f8c4cdda57f8dc1bec6` passed CI #274. ADR 016 is accepted.

Verified properties include `dmd-app` as the outer composition boundary; exact fresh local `ContentCatalog` resolution on create/open/resume/restore; create and restore content preflight before persistence mutation; raw content-agnostic persistence remaining available for recovery/diagnostics; and a sealed `RunnableCampaign` capability whose fields and construction are private to `dmd-app` with borrow-only public accessors.

### Architecture acceptance reconciliation

PR #13 reconciled ADRs 013–016 after explicit human approval and merged as `33bf7b26520446d4fd8e29b14cc6ab8f4cf70639`. Post-merge `main` CI #279 completed successfully on that exact commit.

## Integrated acceptance evidence

The final Gate 1 review re-read the merged production code, accepted/proposed architecture records, and executable regression coverage on `gate1/acceptance-closeout` rather than relying on worker-chat summaries.

### Journal authority and provenance

- `journal_store.rs` tests prove accepted transitions commit state, audit, and events together; stale transitions and invalid causal/provenance references reject without mutation.
- `journal_integrity.rs` proves corrupt current-state provenance blocks later writes and that direct command/event/causal deletion remains forbidden.
- Trusted command issuer identity is persisted separately from optional in-world actor identity.

### Snapshot migration and replay recovery

- New campaigns receive immutable sequence-0 snapshots; periodic snapshots are created at the configured interval inside the authoritative transaction path.
- Replay reconstructs historical/current state from snapshot + journal with typed appliers.
- Recovery succeeds when the materialized current-state row is deliberately corrupted, while journal gaps, missing snapshots, unsupported future schema, unsupported event versions, and malicious identity/schema-changing appliers fail closed.
- Migration tests prove pre-snapshot databases are backfilled only at the materialized head that actually exists rather than fabricating unavailable history.

### Derivative projections

- Projection queries are campaign-scoped and do not require whole-aggregate JSON decoding.
- Missing/count-corrupt projections fail closed and are repaired by replay-backed rebuild.
- Malformed/stale materialized recovery artifacts can be repaired through replay.
- Forced projection failure rolls back the authoritative state/journal transition.
- File-backed close/reopen preserves queryability.

### Campaign lifecycle, portability, and destructive safety

- A realistic two-campaign archive/export/purge/restore sequence proves unrelated-campaign isolation.
- Restore preserves authoritative state, audit/event/causal history, snapshots, and lifecycle state and reopens after database restart.
- Invalid/corrupt restore artifacts are rejected before target writes; global identity collisions roll back the whole restore.
- Manually inserting purge authorization still cannot unlock selective command/event/causal/snapshot deletion.
- Authorized root deletion mechanically removes the complete campaign aggregate, sessions/participants, projections, history, snapshots, lifecycle, and transient authorization rows.
- A backup made stale by later lifecycle mutation cannot authorize purge.

### Exact content authority

- Multiple unrelated rulesets/content packs resolve without campaign-specific assumptions.
- Missing, wrong-version, kind-mismatched, incompatible, duplicate, malformed, dependency-missing, and corrupted content fail explicitly.
- Unknown manifest fields, unsafe relative paths, and symlink traversal fail closed.
- Resolution is local/network-independent and does not treat provider/LLM output as content authority.

### Runnable application boundary

- Real file-backed SQLite plus on-disk manifest tests cover create/open/resume, process-style restart, export/restore, and exact resolved content.
- Unresolved create fails before persistence; runnable open fails after local content removal/version change/corruption while raw recovery remains available.
- Restore content preflight occurs before target persistence mutation.
- `RunnableCampaign` cannot be fabricated by downstream crates because fields are private and the constructor remains internal to `dmd-app`.
- `scripts/check-boundaries` mechanically enforces the accepted dependency direction so lower layers cannot depend on `dmd-app` or route persistence into core/rules/conversation.

## Validation state

Historical exact-head evidence for each merged slice is recorded above. More importantly, the combined post-ADR-reconciliation `main` commit `33bf7b26520446d4fd8e29b14cc6ab8f4cf70639` passed CI #279, covering:

- `./scripts/verify-fast`;
- Clippy with warnings denied;
- full workspace tests;
- Rust 1.88 MSRV workspace check;
- campaign genericity guard;
- architecture dependency guard.

PR #14 must also pass the same repository CI on its exact final documentation head before it may be considered merge-ready.

## Known limitations and debt carried beyond Gate 1

These are explicit limitations of the accepted persistence foundation, not hidden claims of product completeness:

- Full gameplay replay depends on each future durable gameplay event family defining stable kind/version semantics and a typed `ReplayEventApplier`.
- Databases upgraded from pre-snapshot Slice A can replay forward from their real backfilled upgrade-time snapshot but cannot reconstruct earlier state that was never captured.
- The 100-event snapshot interval is an initial policy requiring endurance measurement/tuning.
- Projection maintenance currently replaces the complete projection image; this is simple and safe at Gate 1 scale but unbenchmarked for future high-volume simulation.
- Projection head/count validation detects missing/extra/stale projection state but does not cryptographically detect arbitrary same-count manual in-place SQLite payload tampering; replay-backed rebuild remains the repair path.
- Export format v1 has strong structural/relational/domain validation but no cryptographic authenticity/signature layer and no end-user backup-file retention/encryption UX.
- Manifest v1 uses `fnv1a64` for corruption detection, not publisher authenticity. Publisher signatures/trust stores, installation/distribution UX, licensing policy, and explicit content-version migration are future work.
- `CampaignRuntime` reloads all configured local manifests at each runnable boundary. Caching/change watching is deferred until measured need and must preserve equivalent fail-closed change detection.
- `dmd-app` is the library-level production composition boundary, not yet the finished user-facing executable. Future gameplay/UI/voice entrypoints must route through it rather than treating raw `OpenCampaign` as runnable.
- Raw persistence lifecycle APIs remain public intentionally for recovery/admin tooling; future gameplay-facing surfaces should gain mechanical guards where necessary to prevent accidental raw-state use.
- Application-level write scheduling/retry policy under real concurrent writers remains future operational work; SQLite contention is a non-committed failure that callers must re-read/re-resolve.

## Acceptance decision pending

The integrated technical review found **no unresolved Gate 1 correctness, integrity, replay, isolation, destructive-operation, content-resolution, or architecture-boundary blocker** in the merged production-intended path.

Two final governance/merge gates remain:

1. Explicit human confirmation that ADR 012 is accepted, reconciling its stale `Proposed` status with the human-approval-gated PR #7 merge record.
2. Explicit human approval of this Gate 1 acceptance evidence after PR #14 reaches a green exact final head.

Only after those approvals may ADR 012 and this checkpoint be marked `Accepted`, the closeout plan be archived, and PR #14 be merged. Gate 2 must not begin before that formal closeout.
