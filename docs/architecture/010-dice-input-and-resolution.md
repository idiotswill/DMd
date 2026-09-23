# ADR 010 — Dice input and resolution boundary

Status: **Accepted for Gate 0 foundation**

## Context

DMd is intended to support real dice rolled at a physical table. A player may speak or enter the face shown on a die, while NPC/hidden rolls may be generated digitally.

If the runtime accepts a player/provider-supplied final total, it cannot reliably explain which modifiers were applied, detect stale/mismatched roll answers, or distinguish physical input from engine-generated randomness.

## Decision

Dice resolution is split into three records:

```text
RollRequest
  ├── stable RollRequestId
  ├── optional world-entity roller
  ├── typed die specifications
  ├── engine-owned modifier
  ├── advantage/disadvantage mode
  ├── visibility
  └── human-readable reason
        │
        ▼
RollResult
  ├── matching RollRequestId
  ├── source: Physical | Digital
  └── raw die faces only
        │
        ▼
ResolvedRoll
  ├── raw faces
  ├── kept faces
  ├── engine-owned modifier
  └── computed total
```

A physical-dice report is therefore an external nondeterministic input, not an authoritative precomputed resolution.

## Validation

The rules boundary rejects:

- an empty dice request;
- zero-count or invalid-sided dice;
- advantage/disadvantage on a request that is not exactly one d20;
- a result tied to a different request ID;
- a wrong number/type of returned dice;
- impossible die faces such as 0 on a d20 or 21 on a d20;
- arithmetic overflow while computing the resolved total.

Normal multi-die requests are matched by die sides/count rather than relying on result ordering.

## Advantage and disadvantage

Gate 0 only establishes the primitive: an advantage/disadvantage request is one d20, while the result carries two raw d20 faces. The engine keeps the higher/lower face respectively and then applies the modifier.

The later rules kernel remains responsible for deciding **whether** advantage/disadvantage applies and for interactions such as cancellation.

## Visibility and source are separate

`RollVisibility` describes who is allowed to see the roll (`Public`, `Private`, `Secret`).

`RollSource` describes how the raw faces were produced (`Physical`, `Digital`).

This avoids conflating a hidden digital GM roll with the fact that it was digitally generated.

## Consequences

- the table can use real dice without trusting spoken arithmetic;
- voice/UI adapters only need to capture raw faces and the request ID/context;
- hidden digital rolls use the same resolution path as physical rolls;
- every resolved roll can explain raw dice, kept dice, modifier, source, and total;
- future deterministic replay can treat physical results as logged external inputs and digital results as explicit RNG outcomes.

## Deferred

This ADR does not define 5e checks, saving throws, attacks, damage types, critical hits, proficiency, features, reactions, or spell mechanics. Those belong to the rules kernel and will construct these generic roll primitives rather than replacing them.
