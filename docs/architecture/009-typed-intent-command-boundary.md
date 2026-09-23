# ADR 009 — Typed intent and command boundary

Status: **Accepted for Gate 0 foundation**

## Context

Natural-language providers often emit JSON or other loosely typed data. It is tempting to carry that representation into the game core as a string command name plus an arbitrary JSON payload.

That would make the apparent AI/core boundary cosmetic: core handlers would still need to switch on strings, inspect provider-shaped JSON, and discover schema mistakes at runtime. It would also make it difficult to prove which fields were validated before authoritative state mutation.

A second boundary is equally important at a physical table: the entity attempting an action is not the same thing as the authority principal that issued the command. A player may control one PC while naming another creature in the same sentence. Speaker/issuer identity must therefore come from a trusted input/session channel, not from the language model's interpretation of words or names.

## Decision

The conversation edge may consume provider-specific structured output, but provider output must be parsed into an application-defined typed proposal before it crosses into core validation.

Core commands are represented as:

```text
trusted table/client identity ───────────────┐
                                             │
InterpretedUtterance<TProposal>              │
        │                                    │
        │ typed candidate(s)                 │
        ▼                                    │
application validation / authorization ◄─────┘
        │
        ▼
GameCommand<TCommand>
├── CommandMeta
│   ├── CommandId
│   ├── CampaignId
│   ├── optional PlaySessionId
│   ├── CommandIssuer
│   │   ├── Player(PlayerId)
│   │   ├── System
│   │   ├── Admin
│   │   └── Import
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

## Issuer vs actor

`CommandIssuer` is trusted authority metadata attached by the application/input layer. It identifies the authority context under which the command is being attempted.

`actor` is an optional in-world `AgentRef` describing the entity or faction performing the action.

They are deliberately separate. For example:

- Alice's table client may produce `issuer = Player(alice)` and `actor = Entity(alice_pc)`;
- a world-simulation tick may use `issuer = System` and `actor = Faction(merchant_guild)`;
- an explicit recovery correction may use `issuer = Admin` and an actor only when the correction represents an in-world actor;
- import/migration work uses `issuer = Import` rather than inventing a player.

The language/STT provider may propose which actor a sentence refers to, but it does **not** establish the issuer. Speaker identity should come from trusted local-client/session metadata such as a player-specific push-to-talk channel. A provider cannot gain authority by emitting another player's name or ID.

`CommandIssuer` is an authority category at this foundation layer, not a complete authentication/account model. If future admin tooling needs multiple operator identities, that is added without collapsing issuer and actor back together.

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
- issuer identity/category;
- actor/player authority (for example, whether a player may control the referenced PC);
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
- issuer/authorization semantics;
- save-game schemas;
- rule semantics;
- event journal semantics.

## Persistence implication

Events may reference their originating `CommandId`. When the general command/event journal is implemented, the durable resolution/audit record must preserve command authority metadata—including issuer—so a later causal explanation can distinguish who authorized an action from which in-world actor performed it.

## Consequences

- invalid provider fields fail before core dispatch;
- provider output cannot manufacture player authority by naming another player or PC;
- production handlers do not branch on magic command strings;
- compiler errors expose command-schema changes to affected handlers;
- transcript/provider regression tests can remain adapter-facing while core tests use typed commands directly;
- future non-AI/manual UI clients can construct the same typed commands without imitating LLM JSON;
- player-agency checks have an explicit trusted principal to compare with character ownership.
