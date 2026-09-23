# Gate 0 — Architecture Foundation

Gate 0 exists to prevent expensive architectural rewrites before gameplay implementation begins.

## Acceptance criteria

### Repository and build
- Rust workspace is split into domain, core, rules, persistence, and conversation boundaries.
- `Cargo.lock` is versioned for reproducible application builds.
- CI uses locked dependency resolution for compile, Clippy, tests, and the declared Rust 1.88 minimum-version check.
- Production Rust crates contain no current-campaign names or assumptions.

### Architectural decisions
- Local-first runtime is documented.
- SQLite transactional persistence + append-only event journal is documented.
- AI is an adapter and never authoritative state.
- Current campaign material is explicitly research/regression input only.
- The generic domain/state model is documented and represented by serializable Rust types.
- Tabletop play-session history is documented as a durable ledger separate from the materialized world snapshot.
- Conversation/provider output must become typed intent candidates and typed core commands before authoritative resolution.

### Genericity
A clean campaign can be created without imported campaign content. Production types use stable generated IDs rather than character names as identifiers.

The following must hold:
- duplicate human-readable character names remain legal and distinct;
- no production type assumes a four-person party;
- every mutable runtime record is campaign-scoped;
- cross-campaign records are detected as invalid state;
- ruleset and content-pack identity/version are campaign data, not engine constants.

### State and provenance
- `CampaignState` has an explicit schema version.
- A snapshot records the last applied event sequence.
- world time is setting-agnostic and not hardcoded to one calendar;
- durable events contain campaign, sequence, world time, source, and causal/correlation metadata where relevant;
- language/AI output cannot directly become authoritative state without command validation/resolution;
- core command handlers receive typed payloads rather than dispatching on string command names or arbitrary JSON;
- historical tabletop sessions do not accumulate inside `CampaignState` snapshots;
- commands/events may correlate to a stable `PlaySessionId`, while between-session simulation remains valid without a session ID.

### Tabletop session model
- attendance is session-scoped rather than a permanent character lifecycle status;
- a player may be present without an assigned character during creation/death/replacement flows;
- duplicate player and duplicate character assignment within one session are invalid;
- at most one active tabletop session exists per campaign in persistence;
- session records can be persisted/reloaded without changing world-state serialization.

### World and scene model
- character build identity and world-entity identity are separate;
- dead/retired characters can remain historical entities rather than being deleted;
- a retired character may remain alive in the world;
- more than one scene can coexist for split-party play;
- standing directives can represent macro play such as “continue until danger/decision/discovery” without repeated micro-prompts;
- locations use stable IDs and can form a containment hierarchy without relying on display names.

### Knowledge and mystery model
- world facts, NPC/player claims, beliefs, and knowledge are separate record types;
- a claim does not become a fact merely because an NPC said it;
- beliefs carry a basis and confidence;
- durable materialized knowledge records have stable `KnowledgeId` identity rather than being addressed by display text or collection position;
- knowledge can belong to one entity/faction or be explicitly shared with the table;
- out-of-character chatter is not implicitly promoted to character knowledge.

### Inventory model
- item instance identity is distinct from item definition/content identity;
- ownership and physical custody are separate facts;
- borrowed, stolen, hidden, dropped, container-held, missing, and destroyed states do not require a global `Party` inventory bucket.

### Conversation requirements
The transcript regression corpus must cover messy natural tabletop interaction, including compound declarations, corrections, roll results, split scenes, macro travel instructions, and genuine ambiguity.

Provider-specific JSON or grammar output is adapter data. It must be parsed into application-defined typed candidates before core validation; a provider schema is not the authoritative game-command schema.

### Human review
Before Gate 0 is accepted, a human reviewer should be able to answer yes to:

1. Could the current campaign be deleted from all development machines without breaking the engine architecture?
2. Could four entirely different characters be used without changing production code?
3. Could another fantasy setting be added as content rather than as an engine fork?
4. Can GitHub/Drive be unavailable during play without stopping the game?
5. Can the language model be replaced without rewriting rules or persistence?
6. Is there a clear place for rules, persistence, conversation, simulation, Director, audio, and UI responsibilities without circular ownership?
7. Are correctness and table enjoyment both explicit test concerns?
8. Can two unrelated campaigns coexist without sharing IDs, state, knowledge, inventory, event history, or play-session history?
9. Can a witness lie, a character believe the lie, and the world truth remain unchanged?
10. Can a player character die and a replacement character join without deleting or mutating the dead character's history?
11. Can two simultaneous scenes be represented without pretending the whole party has one location?
12. Can an item be owned by one entity while physically carried by another?
13. Can a long-running campaign accumulate many tabletop sessions without making every world snapshot carry that historical session ledger?
14. Can the language provider be replaced without forcing core handlers to understand a provider's JSON shape or magic command strings?
15. Can durable information/knowledge records be migrated or corrected by stable identity rather than relying on vector position or display text?

The current risk/status assessment is maintained in `docs/checkpoints/gate-0-review.md`.

Gate 0 should not be marked complete merely because the code compiles.
