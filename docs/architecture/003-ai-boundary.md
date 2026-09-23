# ADR 003 — AI Is an Adapter, Never an Authority

**Status:** Accepted for Gate 0

## Decision

Speech recognition, language models, embeddings, and speech synthesis are replaceable adapters. They may interpret, classify, retrieve, rank, summarize, or render, but they may not directly mutate authoritative campaign state.

## Required flow

1. Audio/text enters the conversation layer.
2. The language layer produces typed candidate intents plus confidence/provenance.
3. The application/core layer validates permissions, actor ownership, preconditions, rules, timing, and current state.
4. The rules/world systems resolve uncertainty and request dice when required.
5. Persistence commits the accepted state mutation and event atomically.
6. A narrative renderer receives only the permitted resolved facts and produces player-facing language.

## Provider interfaces

The runtime must be able to replace implementations of:

- speech recognition;
- intent/language interpretation;
- narrative rendering;
- NPC dialogue rendering;
- embeddings/retrieval;
- text-to-speech.

Provider choice must not change save-game format or rules semantics.

## Failure behavior

If an AI provider crashes, times out, emits invalid structured output, or is unavailable:

- authoritative game state remains intact;
- the invalid output is rejected rather than partially applied;
- the user may retry, choose another provider, or use a more explicit text/manual interaction path;
- the campaign does not require repository repair or state reconstruction.

## Security and knowledge boundaries

AI requests receive the minimum context needed for the task. NPC dialogue must not automatically receive omniscient world truth; it receives the NPC's permitted knowledge/beliefs plus visible scene context.
