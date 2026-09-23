# ADR 009 — Typed intent and command boundary

Status: **Accepted for Gate 0 foundation**

## Context

Natural-language providers often emit JSON or other loosely typed data. It is tempting to carry that representation into the game core as a string command name plus an arbitrary JSON payload.

That would make the apparent AI/core boundary cosmetic: core handlers would still need to switch on strings, inspect provider-shaped JSON, and discover schema mistakes at runtime. It would also make it difficult to prove which fields were validated before authoritative state mutation.

## Decision

The conversation edge may consume provider-specific structured output, but provider output must be parsed into an application-defined typed proposal before it crosses into core validation.

Core commands are represented as:

```text
InterpretedUtterance<TProposal>
        │
        │ typed candidate(s)
        ▼
application validation / authorization / disambiguation
        │
        ▼
GameCommand<TCommand>
├── CommandMeta
│   ├── CommandId
│   ├── CampaignId
│   ├── optional PlaySessionId
│   ├── optional AgentRef actor
│   └── expected event sequence
└── typed payload TCommand
        │
        ▼
CommandHandler<TCommand>
        │
        ▼
resolution + domain events
```

There is no generic `kind: String` plus `serde_json::Value` command contract in `dmd-core`.

## Conversation candidates

An utterance may yield zero, one, or several typed candidates.

Examples:

- clear table chatter: zero state-changing candidates;
- clear action declaration: one candidate;
- materially ambiguous reference: multiple candidates or an `Ambiguous` classification;
- correction: a typed correction proposal rather than a free-form patch to state.

Each candidate may carry provider confidence and a requirement for explicit confirmation. Confidence is advisory interpretation metadata; it is never itself authority to mutate state.

## Validation ownership

Typing is necessary but not sufficient. The application/core path still validates:

- campaign identity;
- actor/player authority;
- stale-state sequence;
- referenced entity IDs;
- rules preconditions;
- timing/reaction windows;
- resource availability;
- ambiguity requiring player choice.

A correctly deserialized command may still be rejected.

## Extensibility

`GameCommand<C>` is generic so Gate 0 does not freeze the eventual gameplay command set. Rules, world simulation, admin tooling, and other subsystems may define their own command enums/structs while reusing the same authority metadata and handler boundary.

This keeps extension type-safe without forcing every future mechanic into one giant enum or arbitrary JSON blob.

## Provider isolation

A local LLM implementation may use JSON Schema, grammar-constrained output, or another provider format internally. Those provider schemas are adapter concerns.

Changing an LLM/STT provider must not change:

- authoritative command types;
- save-game schemas;
- rule semantics;
- event journal semantics.

## Consequences

- invalid provider fields fail before core dispatch;
- production handlers do not branch on magic command strings;
- compiler errors expose command-schema changes to affected handlers;
- transcript/provider regression tests can remain adapter-facing while core tests use typed commands directly;
- future non-AI/manual UI clients can construct the same typed commands without imitating LLM JSON.
