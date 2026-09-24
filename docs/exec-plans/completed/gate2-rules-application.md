# Gate 2 application integration

Status: complete; merged in [PR #18](https://github.com/idiotswill/DMd/pull/18).

## Objective and boundary

Integrate the rules foundation with `CampaignRuntime`, atomic command/journal persistence,
content validation, restore and deterministic replay. This is slice 3 of the checked-in
Gate 2 plan, based on the foundation branch and source PR #16. Gate 3 UI and complete
Gate 4/5 encounters/adventuring are not part of this slice.

The Gate 2 checkpoint and product clauses for local authority, trustworthy suspension,
physical dice, query/action separation and source fidelity govern acceptance. ADRs 017–019
document schema compatibility, pure mechanics and this application boundary.

## Acceptance and work

- Actions use trusted context separately from typed proposals; committed state, event and
  audit share one transaction. Invalid, unauthorized and stale actions write nothing.
- Pending physical/digital rolls, damage, concentration, timing, effects, resources, spells
  and rests survive reopen and replay through production APIs.
- Read-only rules questions enforce viewer ownership and secret-roll visibility.
- Installed content is revalidated before every gameplay entrypoint. Unsupported versions
  and changed definitions fail before writes.
- Restore checks every rules snapshot and event/audit relationship before installing an
  export; semantic replay uses the earliest available immutable recovery anchor.
- File-backed integration tests cover those paths, two unrelated campaigns, malformed
  exports and replay envelopes, optional house rules and actual inspiration rerolls.
- Run the full repository verification, independently review the complete diff and final
  exact head, require green CI/MSRV, then merge with expected-head protection.

## Decisions and findings

The pure kernel leaves event sequencing to persistence. The app proposes exactly one
next sequence for atomic commit; replay leaves sequence advancement to the replay engine.
Digital randomness is generated only outside the kernel and recorded as raw faces.
Current state returned to trusted runtime/admin code is not a player response object.

Review found actor/provenance mismatch on privileged roll submission and overly strict
read ownership for dead characters. The foundation writer fixed both; integration tests
exercise their public application paths. Review also requires malformed saved requests to
retain enough origin data for the kernel to rederive their authoritative dice and purpose.

## Verification and next step

The reviewed foundation and restore helper are integrated. The initial application run
passed 13 runtime scenarios, six restore-helper tests and eight existing runnable-campaign
tests. Independent review then found two boundary defects: historical rules lineage could
be bypassed by relabeling current/event data, and create/restore reread content after commit.
Both are fixed in `ce5b67e`/`916ca49`, with regression coverage for audit/snapshot lineage,
post-resolution session rejection and deterministic content changes during a database wait.
The isolated fixes passed 15 runtime, six helper and eight existing application tests.

Final head `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c` received independent full/delta
review and passed `./scripts/verify` (172 Windows workspace tests), formatting, checks,
Clippy and both guards. [CI #305](https://github.com/idiotswill/DMd/actions/runs/36003158276)
passed all four jobs, including Rust 1.88 MSRV. Expected-head squash merge produced
`3ad3885559073e6748d3f17c2172be9ff2a99f52`; its tree equals the reviewed head and full
local verification passed again. Gate closeout owns final publication; no owner blocker
remains. UI, geometry, complete content and measured endurance remain deferred.
