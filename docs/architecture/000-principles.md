# DMd Architecture Principles

## Product goal

DMd is a local-first tabletop RPG runtime intended to run a complete, persistent, enjoyable campaign for human players at a physical table.

It is not an Asterra-specific engine and it is not an LLM prompt wrapper.

## Non-negotiable boundaries

1. **Game truth is authoritative and deterministic.** Language models may interpret or narrate, but they do not directly mutate campaign state.
2. **Campaigns are data.** The engine must boot with no campaign present and must support unrelated campaigns without code changes.
3. **Local play must not depend on GitHub, Google Drive, or cloud services.** Source control and backups are outside the gameplay transaction path.
4. **Player-character agency belongs to the human player.** The engine may resolve consequences but must not invent a PC's voluntary choice, dialogue, thoughts, or feelings.
5. **Unknown remains unknown.** Missing facts are not filled merely for narrative convenience.
6. **Knowledge is not truth.** World truth, observations, claims, beliefs, evidence, and player/PC knowledge must be representable separately.
7. **Routine play may be compressed.** The engine should support standing intentions such as “keep following the tracks until something changes.”
8. **Clarify only when ambiguity materially changes state or consequences.** Do not require command-language repetition for high-confidence natural declarations.
9. **State changes are transactional and auditable.** Every material mutation must be attributable and recoverable.
10. **Core gameplay works without AI.** Removing STT/LLM/TTS may reduce natural-language quality, but must not invalidate rules, persistence, simulation, or campaign state.
11. **Completed checkpoints are production architecture.** Avoid disposable core implementations that must later be replaced.
12. **Enjoyment is a system requirement.** Latency, pacing, spotlight, narration length, interruption handling, and friction are tested alongside correctness.

## Asterra boundary

Asterra campaign materials and transcripts may be used only as research, migration fixtures, and interaction-regression input. Production engine modules must not contain assumptions about Asterra locations, PCs, quests, factions, dates, or IDs.
