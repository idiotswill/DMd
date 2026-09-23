# Gate 0 — Architecture Review and Risk Register

Status: **Accepted — independent final review completed 2026-09-23**

Gate 0 is accepted as a production architecture checkpoint. This does **not** mean DMd is a playable or finished game; it means the foundation is sufficiently coherent, generic, and mechanically guarded to begin Gate 1 without knowingly building gameplay on a contradictory base.

## Final review baseline

The independent acceptance pass reviewed PR #1 against `docs/checkpoints/gate-0.md`, `docs/product-definition.md`, ADRs 000–010, the complete PR change set, production Rust boundaries, persistence constraints, transcript fixtures, CI, and PR review state.

The accepted foundation provides:

- separate Rust crates for domain, core/application command handling, rules, persistence, and conversation boundaries;
- typed campaign/world identities with structural validation;
- `CharacterId` separated from world-facing `EntityId`;
- split-scene and persistent coarse-location primitives;
- truth / claim / belief / knowledge separation;
- stable `KnowledgeId` identity with duplicate knowledge IDs rejected;
- typed `GameCommand<C>` rather than string + arbitrary-JSON command dispatch;
- trusted `CommandIssuer` separated from the optional in-world actor;
- typed event metadata with multi-parent causality;
- physical/digital dice requests and raw-result provenance;
- a play-session ledger persisted beside, not inside, `CampaignState`;
- versioned SQLx migrations for implemented SQLite state;
- versioned `Cargo.lock`, locked dependency resolution, and Rust 1.88 MSRV verification;
- CI for formatting, compilation, Clippy, tests, and campaign genericity;
- concrete transcript-regression fixtures for all interaction categories required by Gate 0;
- a durable product definition that prevents later gates from redefining the finish line as a prototype.

## Independent final-review findings

The final pass found two checkpoint-level gaps and repaired them before acceptance.

### 1. Transcript coverage was partly documentary rather than executable fixture data

The corpus README listed physical-roll association, split scenes, and genuine ambiguity, but the concrete JSON fixtures did not instantiate those cases. Gate 0 requires the corpus to cover them, not merely list them.

**Resolution:** added campaign-agnostic fixtures for physical-roll association, split-scene routing, and material ambiguity requiring clarification.

### 2. Play-session database isolation/order needed stronger final guards

The public writer generated participant ordinals correctly, but SQLite did not independently enforce ordinal uniqueness. Also, an existing `PlaySessionId` could theoretically be updated with a different `campaign_id` through the upsert path if presented with a separately valid campaign state.

**Resolution:** migration `0002_session_integrity.sql` now enforces unique `(session_id, ordinal)` values and makes a persisted session's campaign immutable. Integration tests exercise both constraints.

No other Gate 0 blocker was found. There are no submitted PR reviews, inline review threads, or discussion comments outstanding.

## Boundary review

### Authoritative world state

`CampaignState` is materialized world state, not transcript/session/provider history. Historical tabletop sessions remain external and correlate by stable ID.

**Result:** accepted and regression-covered.

### Language / AI and player authority

Provider output may classify and propose typed candidates but cannot directly write authoritative state. Trusted issuer identity comes from the application/input channel and is separate from the in-world actor.

**Result:** accepted. Concrete gameplay authorization policies remain intentionally deferred to the commands that need them.

### Campaign isolation

Mutable world records are campaign-scoped, cross-campaign references are invalid, names are labels rather than keys, and session persistence now prevents a durable session ID from changing campaigns.

**Result:** accepted.

### Information provenance

Facts, claims, beliefs, and knowledge remain distinct. NPC/faction knowledge can differ from truth. Durable knowledge records have stable identity.

**Result:** accepted. Propagation/trust/evidence and repeated holder/target merge policy remain later work.

### Dice

A roll request owns dice shape, modifier, mode, visibility, roller, and reason. A physical/digital result supplies raw faces and provenance, not an authoritative final total.

**Result:** accepted as the Gate 0 primitive. 5e-specific rules remain deferred.

### Persistence direction

Gate 0 implements only the session ledger. It does not pretend the general save/journal system exists. The implemented SQLite boundary is compatible with ADR 002's required future atomic state + event transaction.

**Result:** accepted with Gate 1 blockers explicitly retained below.

## Open risk register

| Risk | Severity | Gate impact | Required next action |
| --- | --- | --- | --- |
| General atomic state + event-journal persistence is not implemented. | High | First Gate 1 blocker | Implement one transaction that validates/allocates sequence, appends events, updates materialized state, preserves command/resolution authority metadata, and survives crash/recovery tests before gameplay depends on it. |
| `CampaignState.schema_version` exists, but snapshot codec/migration/replay is not implemented. | High | Gate 1 blocker | Add explicit snapshot codecs/migrations and replay/recovery tests before a save format is considered durable. |
| Ruleset/content-pack references exist, but manifests/schema validation/dependency resolution/licensing boundaries are not implemented. | High | Gate 1/content foundation | Define validated versioned content-pack boundaries before importing large rules/content datasets. |
| Transcript fixtures are behavioral specifications, not yet an executable interpreter evaluation harness. | Medium | Language-layer blocker | Wire them to deterministic/mock and local-provider evaluations when the intent interpreter is implemented; do not weaken fixtures to fit a provider. |
| Knowledge records have stable IDs but repeated holder/target merge semantics are intentionally not frozen. | Medium | Information persistence | Decide materialized merge policy when the persistence use case requires it; keep historical observations in the journal. |
| `CommandIssuer::Admin` is an authority category, not a distinct local operator identity. | Low | Later admin/audit design | Add trusted operator identity only if multiple human admin identities become a product requirement. |
| World/entity location is deliberately coarse; tactical geometry, routes, portals, reach, LoS, and AoE are absent. | High | Gate 3 blocker | Introduce a dedicated spatial subsystem before full tactical combat content. |
| Director, simulation, audio, and UI crates are not implemented. | Medium | Later gates | Add them only with their first production slice; do not fold them into domain/conversation as shortcuts. |
| `main` repository protection/ruleset remains a manual GitHub setting. | Low | Process hardening | Apply `docs/runbooks/github-settings.md` when convenient; connector permissions cannot administer it. |

## Acceptance matrix

| Gate 0 question | Final result |
| --- | --- |
| Can engine/domain code exist without the current campaign? | **Yes.** Production crates are generic and mechanically guarded against current-campaign names. |
| Can unrelated/replacement characters exist without code changes? | **Yes.** Stable IDs are separate from names; character/world identity and lifecycle support death/retirement/history. |
| Is party size hardcoded to four? | **No.** Four is a product UX target, not an engine cardinality. |
| Can split-party scenes exist? | **Yes.** Multiple scenes coexist and invalid simultaneous physical participation is rejected. |
| Can a witness lie without changing world truth? | **Yes.** Claims and facts are separate. |
| Can NPC/faction knowledge differ from truth? | **Yes structurally.** Beliefs/knowledge are explicit; renderer filtering is later implementation. |
| Do durable knowledge records have stable identity? | **Yes.** `KnowledgeId` is explicit and duplicate IDs are rejected. |
| Can tabletop session history grow without bloating world snapshots? | **Yes.** Session ledger is external to `CampaignState`. |
| Can physical dice remain first-class and auditable? | **Yes.** Raw faces, request ID, source, kept dice, modifiers, and total are preserved. |
| Can provider/LLM output mutate state directly? | **No.** Core accepts typed commands through a validation boundary. |
| Can a provider gain authority by naming another PC? | **No.** Trusted issuer metadata is separate from proposed/in-world actor identity. |
| Are live saves architecturally independent of GitHub/Drive? | **Yes.** SQLite is runtime authority; general save/journal implementation is Gate 1. |
| Are builds reproducible and minimum Rust supported? | **Yes.** Lockfile + locked CI + Rust 1.88 MSRV check. |
| Does the interaction corpus concretely cover Gate 0-required messy-table cases? | **Yes after final review fixes.** |
| Has the architecture/acceptance review been completed? | **Yes.** Independent final pass completed and human merge approval provided. |

## Gate 0 acceptance decision

**Accepted.** The known high-severity items are deliberate Gate 1/later implementation blockers, not contradictions in the Gate 0 foundation. Product-definition requirements for a complete playable game, voice interaction, living-world simulation, emergent play, UI, and endurance remain explicitly unfulfilled and may not be represented as complete merely because Gate 0 passes.

The next production implementation must begin with Gate 1 persistence/replay foundations, not combat content or AI narration.
