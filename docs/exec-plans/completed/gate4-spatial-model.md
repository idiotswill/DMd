# Gate 4 spatial foundation

Status: **Completed foundation slice — merged PR #23; Gate 4 remains active.**

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

Standalone model plus pure implementation passes `cargo check --locked -p dmd-rules`.
After correcting two test fixture compilation errors, all 15 focused tests pass:
`cargo test --locked -p dmd-rules spatial::tests --lib`. Strict
`cargo clippy --locked -p dmd-rules --lib --tests -- -D warnings` passes. Formatting
and diff whitespace checks pass. Builds were serialized with the other gate writers.
No full workspace/desktop verification or production acceptance is claimed here.
Existing main was read at `afcbe108c57d13322a32260325a5fc0133c7240c`; the gate plan
creates base `14a94d6`. Next: independent review and root integration, then integrate
accepted encounter transitions and exact suspension through the real application.

## Geometry and information decisions

- All coordinates/lengths use half-feet, with 5ft grid squares; Small and larger
  anchors align to full squares and Tiny anchors to half squares. Integer i128
  products/rational segment intervals avoid platform-dependent floating point.
- AABB volumes are half-open for occupancy. Line traversal through their interiors
  blocks; merely touching their boundary does not. Movement separately rejects
  solid diagonal corners. Vertical grid steps use the same declared grid metric.
- Source cover bonuses are exact (+2/+5, highest only). Authored partial-cover
  volumes carry a grade. Partly clipped Total-Cover geometry returns an explicit
  adjudication requirement; ray counts do not invent an SRD percentage rule.
- All six source area shapes have continuous point predicates; bounded integer direction
  vectors are normalized algebraically. Cube origins may lie anywhere on a selected face.
  The spatial grid uses an explicit square line-prism and voxel-center rasterization
  convention. A point predicate remains available for boundary adjudication.
- Perception separates physical cover, opaque terrain, ambient/emitted light,
  magical darkness, invisibility and special senses. Tremorsense locates; it does
  not provide sight. Remembered contacts keep old positions and never gain current
  HP, true names, positions or unknown entity identities through projection.
  Concealed terrain and invisible obstacles have explicit observability separate from
  their physical properties; the actor view cannot reveal those properties by sight.
- Enemy/UI view DTOs are capability-limited. The host-only geometry/path validator
  may use truth; planners must not query it repeatedly to infer unknown obstacles.
  The runtime must validate an accepted attempt and reveal only its observable result.
- MovementAllowance is a query input derived by the runtime, not player authority
  or a second saved budget. MovementPlan proposes segments/costs and reaction
  boundaries; the runtime must suspend before a reaction and rederive the remainder.
- Hide/Search dice/DCs, noise/contact acquisition, spell-origin clipping, source
  feature grants, timing/interruptions, falling/check resolution and audience-aware
  transcript integration remain the next encounter slices. Pure geometry alone
  does not claim these accepted gameplay transitions exist.

## Source anchors

Pinned SRD 5.2.1: vision/light p.11; grid p.13; movement, creature occupancy and
unseen targets p.14; cover/range/opportunity attacks p.15; spell target path p.106;
area blocking and Blindsight p.177; climb p.178; Cone/Cube p.179; Cylinder/Darkvision
p.180; Emanation p.181; Frightened/Grappled/Flying p.182; Hide p.183; Invisible/Line
p.184; Passive Perception/Prone p.186; Search p.187; Speed/Sphere p.188;
Stunned p.189; Tremorsense/Truesight p.190. Stunned/Incapacitated do not themselves
reduce ordinary movement to zero; unsupported flight still cannot continue.

## Independent review followup

The source-content review found only the root-owned Fire Bolt page-reference correction.
The spatial review found that Unconscious observers could receive fresh perception and
that a global Dash count granted extra movement to unselected speeds. The followup owns
only this slice's spatial module, tests and plan:

- Suppress all fresh senses and terrain for Unconscious/dead observers, including special
  senses, retaining only remembered contacts/cells. Hide their current self-position so
  unconscious forced movement does not disclose a new location. Keep Stunned and
  Incapacitated awareness, as those conditions lack Unaware (SRD pp.184,189,191).
- Reject voluntary dead-creature movement while preserving externally forced movement.
- Replace the query's global Dash count with selected-speed counts, with shared spent
  movement and fallback climb/swim using ordinary Speed. Match the root's ADR 026
  documented cross-mode interpretation (SRD pp.180,188), without storing a second budget.
- Remove the 26-direction restriction in favor of bounded arbitrary integer vectors;
  test non-cardinal orientation, scale invariance, and maximum-product bounds.
- Require proof from one convex solid before returning Total cover. A union of sampled
  blocked rays can conceal an opening and instead requires an explicit geometry ruling.
- Preserve dim-only emitter output at its exact origin. Cull zero-output/out-of-radius
  lights before occlusion. Share an 8,000,000-operation deterministic geometry-work budget
  across a projection; pathological combined workloads return `SpatialError::Capacity`
  without mutation or a fallback that reveals truth. Storage limits alone do not imply
  every Cartesian query is affordable. The maximum sparse floor remains supported.
- Block ordinary sight and Darkvision across magical-darkness regions; Truesight must
  reach the far edge of the obscured ray portion. Independent heavy fog still blocks it.
  `magical_darkness: true` represents the darkness cause; `obscuration: Heavy` represents
  a separate fog/foliage cause and must not be redundantly added for darkness alone.

Followup verification after root released the serialized build slot: all 25 focused
spatial tests pass, strict `cargo clippy --locked -p dmd-rules --lib --tests -- -D warnings`
passes, and formatting/diff whitespace checks pass. The first test run found two fixture
identifiers containing unsupported spaces; those fixtures were corrected before the
successful run. No full-workspace verification was rerun here. Build slot is released.
Next: independent exact-commit review, then root integration/combined runtime verification.

### Composite-cover review followup

Independent review of `bedc6c1` found a regression: Blindsight interpreted an uncertain
composite cover grade as proof of an opening. It now requires an actual clear effect ray
for each candidate target point. A regression distinguishes a closed two-panel wall from
a sampled opening; an unsampled opening still requires adjudication and cannot justify
revealing an actor. The change also removes an unnecessary aggregate cover calculation
from perception. Independent working-diff review found no remaining blocker. After the
serialized build slot was released, all 26 focused spatial tests, strict focused Clippy,
and formatting checks passed. Root retains full integration/CI verification ownership.

## Merged evidence

PR #23 merged as `580f487944608d8c7c7220c7a410a779386ad615`. The exact reviewed
head `0b910b846199f975baa763f6ed9bbc3e88474049` passed canonical `./scripts/verify`
with 249 Rust tests and all six PR CI jobs, including native Windows MSRV/stable
checks and packaging. Merged-main Linux run 36044061166 and Windows run 36044061140
also passed. The merge retained exact reviewed tree `dd26b376192a6202e4f9d83fed02f5b42b1b0a71`.
The next action is the active encounter-execution plan; historical next-action
notes above describe this slice before its final integration. Complete encounter
execution, source-family coverage and packaged desktop acceptance remain Gate 4.
