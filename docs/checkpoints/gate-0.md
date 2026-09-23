# Gate 0 — Architecture Foundation

Gate 0 exists to prevent expensive architectural rewrites before gameplay implementation begins.

## Acceptance criteria

### Repository and build
- Rust workspace is split into domain, core, rules, persistence, and conversation boundaries.
- CI runs formatting, compilation, Clippy, and tests.
- Production Rust crates contain no current-campaign names or assumptions.

### Architectural decisions
- Local-first runtime is documented.
- SQLite transactional persistence + append-only event journal is documented.
- AI is an adapter and never authoritative state.
- Current campaign material is explicitly research/regression input only.

### Genericity
A clean campaign can be created conceptually without any imported campaign content. Production types use stable generated IDs rather than character names as identifiers.

### Conversation requirements
The transcript regression corpus must cover messy natural tabletop interaction, including compound declarations, corrections, roll results, split scenes, macro travel instructions, and genuine ambiguity.

### Human review
Before Gate 0 is accepted, a human reviewer should be able to answer yes to:

1. Could the current campaign be deleted from all development machines without breaking the engine architecture?
2. Could four entirely different characters be used without changing production code?
3. Could another fantasy setting be added as content rather than as an engine fork?
4. Can GitHub/Drive be unavailable during play without stopping the game?
5. Can the language model be replaced without rewriting rules or persistence?
6. Is there a clear place for rules, persistence, conversation, simulation, Director, audio, and UI responsibilities without circular ownership?
7. Are correctness and table enjoyment both explicit test concerns?

Gate 0 should not be marked complete merely because the code compiles.
