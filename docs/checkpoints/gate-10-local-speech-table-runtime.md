# Gate 10 — Local speech and table runtime

Status: **Planned**

## Product requirements advanced

- natural voice;
- speaker identity;
- interruption/barge-in;
- noisy table handling;
- local STT/LLM/TTS;
- sustained reference-hardware performance;
- text fallback.

## Objective

Make spoken tabletop play the normal local-first interface without requiring command syntax or cloud availability.

## Entry conditions

Gate 9 accepted. Text behavior is good enough that speech errors can be isolated.

## Acceptance criteria

Support:
- local STT;
- speaker/player identity mechanism;
- overlapping voices;
- partial/final utterances;
- interruptions and TTS barge-in;
- corrections;
- uncommon fantasy names;
- background chatter/laughter;
- player-to-player conversation that should not trigger DM;
- roll-result association;
- manual/text fallback;
- local narrative generation provider;
- local TTS and stable enough NPC presentation;
- provider selection/failure recovery;
- sustained multi-hour resource/performance monitoring.
Benchmark on Ryzen 5 8645HS / 16 GB RAM / RTX 4050 Laptop 6 GB / Windows x64.

## Architecture invariants

Speech/provider output remains interpretation/presentation, never authoritative state. Cloud optional. Gameplay must remain possible offline.

## Explicit non-goals

Phone companion, Asterra content port, installer/release hardening beyond what is needed to run benchmark builds.

## Required failure/recovery behavior

STT ambiguity/provider failure offers correction/text/manual fallback. TTS interruption must not lose underlying structured response. Resource exhaustion degrades safely rather than corrupting campaign.

## Merge/pause boundaries

Codex may merge in-scope Gate 10 provider/model/audio work autonomously after exact-head verification. Provider choice remains replaceable and cannot redefine authoritative saves/rules. If the execution environment is not the reference laptop, technical work may merge but reference-hardware acceptance must be called out at the end-of-gate pause before Gate 11 progression.
## Candidate workstreams

STT adapter, diarization/identity, conversation streaming, interruption, TTS, local model integration, performance/soak tests, fallback UX.

## Deferred requirements

Phone Gate 12; final installer/config/performance hardening Gate 13.

## Open questions

Exact latency thresholds should be established empirically on reference hardware, then checked in as acceptance budgets.

## Production integration acceptance

Conduct sustained spoken tabletop sessions on the reference machine. Measure memory/VRAM/latency over time. Demonstrate interruptions, corrections, overlapping speech, background chatter, rule questions, physical rolls and provider failure fallback without requiring a magic keyword or cloud service.
