# Gate 9 — Autonomous DM and Director

Status: **Planned**

## Product requirements advanced

- messy language interpretation;
- minimum-needed clarification;
- DM adjudication/presentation;
- NPC knowledge-constrained dialogue;
- listening/silence/table-floor management;
- spotlight;
- recaps/memory queries;
- emotional register;
- Director pacing without truth mutation.

## Objective

Deliver the autonomous DM behavior: generous interpretation, conservative authority, rules competence, evidence limits, natural NPC play and the judgment to know when not to speak.

## Entry conditions

Gates 2–8 accepted. Text-first production path is mature enough to isolate language/Director failures from core rules/state defects.

## Acceptance criteria

Implement/evaluate:
- question vs declaration vs hypothetical vs chatter;
- dialogue/action/multi-action utterances;
- corrections;
- speaker/player/PC authority routing in text context;
- material ambiguity detection;
- minimum clarification;
- no magic activation keyword;
- rules query without action commit;
- when-not-to-roll;
- hidden/passive checks;
- player-to-player conversation recognition;
- DM silence/listening;
- spotlight/split-scene management;
- player narrative delegation;
- NPC dialogue restricted to knowledge/beliefs;
- player-safe recaps and campaign memory queries;
- emotional register/humor restraint;
- running-joke continuity without overuse;
- scene zoom/compression;
- Director selection/pacing from supported developments only;
- realistic messy-input behavioral evaluation corpus.

## Architecture invariants

Provider output never authoritative. Director cannot invent unsupported truth, rewrite dice, retroactively create causes or force outcomes. Narrative renderer sees only permitted resolved facts/knowledge.

## Explicit non-goals

Voice/STT/TTS, phone clients, Asterra port.

## Required failure/recovery behavior

Malformed/ambiguous provider output yields clarification/retry/manual fallback without state mutation. Provider outage does not corrupt campaign. Narration failure can be retried from structured outcome without replaying authoritative mutation.

## Merge/pause boundaries

Codex may merge in-scope Gate 9 DM/Director/provider work autonomously after exact-head verification. No change may weaken the AI authority boundary, player agency, evidence discipline, or product behavior merely to improve model convenience. Pause at the end of Gate 9 with behavioral-evaluation evidence and Gate 10 handoff.
## Candidate workstreams

Conversation interpretation, DM policy/adjudication, NPC dialogue context, Director, renderer, behavior corpus/evals, memory query UX.

## Deferred requirements

Speech Gate 10; Asterra full stress test Gate 11; private mobile surfaces Gate 12.

## Open questions

Exact model(s) remain open and must later meet reference-hardware constraints. Avoid optimizing for benchmark prompts at cost of real table behavior.

## Production integration acceptance

Run messy transcript-style sessions through the production text path. Demonstrate generous intent parsing, evidence-conservative rulings, minimum clarifications, no repeated forced prompts, PC-to-PC conversation where DM stays mostly silent, NPC continuity, player failure without script rescue, and Director surfacing only causally supported developments.
