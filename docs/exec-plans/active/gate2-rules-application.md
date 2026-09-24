# Gate 2 application integration

Status: active; primary agent is sole writer on `codex/gate2-rules-runtime`.

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

The application scenarios are drafted and formatted; compilation awaits the reviewed
kernel foundation. No integration success is claimed yet. Incorporate foundation and
export preflight, generate/verify the distributed content manifest, execute tests, repair
findings, then update this plan with actual head and CI evidence. No owner blocker known.
