# Gate 4 physical inventory reducer

Writer: root on `codex/gate4-inventory-verified`, starting from verified main
`da964869bfc3ebf61b7b1a46d9204f0ff55c6693`. This is a bounded dependency slice
of the active encounter integration; it does not claim a completed playable encounter.

## Objective and boundaries

Materialize source-derived starting allocations as real physical ItemIds with bounded
quantity, custody and loadouts. Validate every recognized loose/held item consistently,
including NPC gear, and retain original grant identities without recreating expended or
transferred supplies. Unknown campaign objects remain available for later source-bound
improvised adjudication. No label proves proficiency or a source capability.

The pure reducer takes an explicit inventory attachment and returns validated state and
inventory together. Main's RulesState/save interpretation, table actions and legacy
events remain unchanged in this PR. The parent integrates the attachment, durable
authority/replay and desktop preparation; later attack work consumes the same ItemIds.

Relevant authority: product-definition rules fidelity, durable state/provenance and real
player equipment; Gate4 physical attacks/improvisation; source-pinned SRD5.2.1 equipment.
The parent ADR026 defines the shared resolution path. No alternate inventory authority
or parallel attack engine is introduced here.

## Acceptance and verification

- Source allocations, prices/quantity multiples and one identity per ordinary weapon
  agree with the pinned catalog; finite ammunition uses explicit physical stacks.
- Initial provisioning rejects existing allocations, duplicate/nil IDs, incorrect source
  choices and invalid custody without partial mutation.
- Hand/armor/shield references describe real carried intact items; invalid unheld known
  quantities/states are rejected too. A holy symbol alone grants no casting focus.
- Preserve source field names and provenance. Inspect the full exact diff separately
  from implementation; run focused regressions, canonical verification and final-head CI.
- Merge only with expected-head protection and record post-merge tree/CI evidence.

## Next action

Extract reviewed inventory/equipment contracts, reducer/registry and focused tests from
parent `37d8044`; adapt only module exports and any main fixture differences. Verify
byte parity of implementation, review dependencies, then open the small PR for CI.
Local Rust builds remain serialized with the parent and other agents.
