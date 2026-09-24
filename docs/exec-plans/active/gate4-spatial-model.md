# Gate 4 spatial foundation

Status: in progress. Branch: `codex/gate4-spatial-model`, based on `14a94d6`.

## Objective and boundary

Introduce reusable authoritative battlefield data and pure deterministic spatial,
perception and area queries. Advance Gate 4 movement/geometry, targeting/areas and
visibility/stealth, and the product's spatial truth, non-omniscient combatants and exact
suspension clauses. This slice does not claim production combat completion: schema
migration, command integration, timing/interruption, enemy policy and desktop UI are
separate root-owned slices.

Relevant contracts: ADR 006 (placement versus presence), ADR 007 (knowledge versus
truth), ADRs 018/020/024 (rules authority, semantic recovery, trusted table commands),
Gate 4, and the pinned SRD 5.2.1. Source anchors: pp. 11, 13–15, 106, 177–190.

## Scope and acceptance

- Domain `TacticalEncounter`, battlefield, participant, movement, sense and remembered
  knowledge types; structural validation against existing campaign identities.
- Pure checked geometry for bounds, occupancy, directional cover, sight obstruction,
  illumination, movement modes and all six area shapes.
- Current perception derived from truth; remembered contacts never follow hidden actors.
  Actor projections omit unknown identities/positions and private mechanical state.
- Explicit provenance for authored geometry where SRD leaves GM discretion. Engineering
  rasterization choices are documented separately from source rules.
- Bounded inputs, deterministic order, no ambient randomness, no query mutation.
- Focused source/boundary tests including malformed inputs and hidden-state differences.

No edits to CampaignState/schema/persistence, kernel mechanics, table/runtime/UI or the
global gate plan. Timing and HP remain solely in RulesState. The schema writer will call
`TacticalEncounter::validate(&CampaignState)` through its normal validation hook.

## Planned slices

1. Commit this plan, then model/export/structural validation contract.
2. Geometry, path legality, cover, perception, filtered view and area APIs.
3. Source interaction and hostile-boundary tests; focused verification after coordinating
   build serialization with root; independent review and integration handoff.

## Verification and next action

No builds or implementation tests run yet. Existing main was read at
`afcbe108c57d13322a32260325a5fc0133c7240c`; the gate plan creates base `14a94d6`.
Next: implement the model and communicate its exact API to the schema writer.
