# Gate 0 — Architecture Review and Risk Register

Status: **Draft checkpoint review — not yet accepted**

This review exists to keep a green build from being mistaken for a completed architecture checkpoint. Gate 0 is acceptable only when the foundation is generic, its authority boundaries are explicit, and remaining risks are consciously deferred rather than hidden in placeholders.

## Review baseline

The current foundation provides:

- separate Rust crates for domain, core/application command handling, rules, persistence, and conversation boundaries;
- typed campaign/world identities with structural validation;
- `CharacterId` separated from world-facing `EntityId`;
- split-scene and persistent coarse-location primitives;
- truth / claim / belief / knowledge separation;
- stable identity for durable information records, including `KnowledgeId`;
- a typed `GameCommand<C>` boundary rather than string + arbitrary-JSON command dispatch;
- typed `EventEnvelope<T>` metadata with multi-parent causality;
- physical/digital dice requests and raw-result provenance;
- a play-session ledger that is persisted beside, not inside, `CampaignState`;
- versioned SQLx migrations for the implemented SQLite schema;
- CI for formatting, compilation, Clippy, tests, and campaign-name genericity checks.

## Boundary review

### Authoritative world state

`CampaignState` is materialized world state. It must not accumulate transcripts, tabletop attendance history, provider output, or other operational/audit history.

**Review result:** boundary represented in code and protected by regression coverage.

### Language / AI

Provider output may classify and propose typed candidates. It cannot directly write authoritative state. Core commands have typed payloads and separate authority metadata.

**Review result:** foundation boundary is sound. Concrete gameplay command payloads remain intentionally deferred.

### Tabletop sessions

Attendance and session history are durable campaign records but not world truth. Commands/events may correlate to a `PlaySessionId`; between-session simulation may omit it.

**Review result:** boundary represented in domain + SQLite persistence with database constraints.

### Information provenance

Facts, claims, beliefs, and knowledge remain distinct. NPC/faction knowledge can differ from world truth. Materialized knowledge records have stable identity; historical acquisition/observation detail belongs in events rather than being inferred from prose.

**Review result:** foundation boundary is sound. Propagation/trust/evidence mechanics remain deferred.

### Dice

A roll request owns dice shape, modifier, mode, visibility, roller, and reason. A physical/digital result supplies raw faces and provenance, not an authoritative final total.

**Review result:** primitive boundary is sound. 5e-specific checks, saves, attacks, critical rules, reactions, and effects remain deferred.

## Open risk register

| Risk | Severity | Gate impact | Current mitigation / required next action |
| --- | --- | --- | --- |
| `Cargo.lock` is generated in CI but is not yet versioned in the branch. Dependency resolution can therefore drift between machines or dates. | High | Gate 0 release-hygiene blocker | Commit the generated lockfile, then run Cargo CI with `--locked`. The current CI artifact preserves the exact generated candidate until this is done. |
| ADR 002 specifies one transaction that appends events and updates normalized state, but the general state/event-journal store is not implemented yet. | High | Gate 1 implementation blocker, not a reason to add more Gate 0 schemas | First Gate 1 persistence slice must implement atomic state + event commit, sequence allocation, causal-reference validation, and crash recovery tests before gameplay systems depend on it. |
| `CampaignState.schema_version` exists, but snapshot migration/replay code is not implemented. | High | Gate 1 blocker | Add explicit snapshot codecs/migrations and replay tests before any released save format is considered durable. |
| Ruleset/content-pack IDs are versioned references, but content schemas, validation, dependency resolution, and licensing boundaries are not implemented. | High | Gate 1/content foundation | Define content-pack manifests and validators before importing large rules/content datasets. Do not copy campaign/rulebook prose into engine code. |
| Transcript fixtures are behavioral specifications, not yet an executable end-to-end interpreter evaluation harness. | Medium | Later language-layer blocker | Preserve fixtures now; wire them to deterministic/mock and local-provider evaluations when the intent interpreter is implemented. Do not weaken fixtures to fit a provider. |
| Knowledge records now have stable identity, but uniqueness/merge semantics for repeated acquisition of the same target are intentionally not frozen. | Medium | Gate 1 information persistence | Decide whether current state permits multiple provenance records per holder/target or canonicalizes to one relation; keep repeated historical observations in the event journal either way. |
| World/entity location is deliberately coarse; tactical geometry, routes, portals, reach, LoS, and AoE are absent. | High | Gate 3 blocker | Keep them out of the base `Location` type. Introduce a dedicated spatial subsystem and reaction/timing architecture before full combat content. |
| The foundation has no Director, simulation, audio, or UI crate yet. | Medium | Later gates | Current ADRs define ownership boundaries. Add these as separate modules/crates only when their first production slice is implemented; do not fold them into domain or conversation as shortcuts. |

## Acceptance matrix

| Gate 0 question | Evidence/status |
| --- | --- |
| Can engine/domain code exist without the current campaign? | **Automated structural pass.** Production Rust crates have a campaign-name guard and use generated typed IDs rather than current-campaign names. |
| Can unrelated character names and replacement characters exist without code changes? | **Automated pass at domain level.** Names are labels; character/world identities are separate; dead/retired state remains representable. |
| Is party size hardcoded to four? | **Pass.** Collections/session participants/scenes are variable-sized; four players remains a target table UX, not an engine constant. |
| Can split-party scenes exist? | **Automated pass.** Multiple scenes are supported while simultaneous active participation of one entity is rejected. |
| Can a witness lie without changing world truth? | **Automated pass.** Claims and facts are separate and independently validated. |
| Can NPC/faction knowledge differ from truth? | **Structural pass.** Knowledge/belief holders and targets are explicit; renderer filtering is still a later implementation task. |
| Can tabletop attendance/history grow without bloating world snapshots? | **Automated pass.** Session ledger is external to `CampaignState` and correlated by ID. |
| Can physical dice remain first-class and auditable? | **Automated primitive pass.** Raw faces, request identity, source, kept dice, modifiers, and resolved total are preserved. |
| Can an LLM/provider mutate state directly? | **Boundary pass.** Core accepts typed commands; arbitrary provider JSON is not the authority contract. |
| Are live saves independent of GitHub/Drive? | **Architecture pass; partial persistence implementation.** SQLite is the runtime authority; general state/event transaction implementation is Gate 1 work. |
| Are saves reproducible across dependency resolution? | **Not yet.** `Cargo.lock` must be committed and CI switched to locked mode. |
| Has a human architecture/acceptance review been completed? | **No.** PR must remain draft. |

## Gate 0 merge posture

Do **not** mark this PR ready merely because CI is green.

Before Gate 0 is accepted:

1. version the Rust dependency lockfile and make CI use locked resolution;
2. reconcile this review with the main Gate 0 checklist and PR description;
3. perform the human architecture/acceptance review against the questions in `docs/checkpoints/gate-0.md`;
4. keep deferred gameplay/simulation schemas deferred unless a concrete Gate 1 requirement proves they belong in the foundation.

The next production implementation after Gate 0 should begin with persistence/replay foundations, not 5e combat content or AI narration.
