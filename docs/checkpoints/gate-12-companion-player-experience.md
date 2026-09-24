# Gate 12 — Companion player experience

Status: **Planned**

## Product requirements advanced

- phone character sheets;
- player-specific maps/logs/private information;
- private DM delivery;
- player identity support;
- central-authority client model.

## Objective

Provide useful player-side phone surfaces while retaining the desktop runtime as authoritative campaign host.

## Entry conditions

Gate 11 accepted; stable player/session/auth interfaces exist.

## Acceptance criteria

Companion clients can appropriately expose:
- player identity/session join;
- character sheet;
- HP/resources;
- inventory/equipment;
- abilities/features/spells;
- conditions;
- maps/visibility;
- personal logs/notes;
- private character knowledge/handouts;
- advancement choices where supported;
- rules reference.
Actions submit validated commands to central runtime.

## Architecture invariants

Phone is never authoritative game state. Server/desktop validates ownership, campaign/session identity, stale state and rules.

## Explicit non-goals

Cloud account ecosystem, unrelated social features, making phone mandatory for play.

## Required failure/recovery behavior

Disconnect/reconnect does not duplicate commands or lose authoritative state. Stale/offline client submissions fail safely. Private information cannot leak to wrong player.

## Merge/pause boundaries

Codex may merge in-scope Gate 12 network/security/privacy architecture autonomously after exact-head verification and explicit validation of authority/privacy boundaries. Phones remain clients, never authoritative state. Pause at the end of Gate 12 with security/authority evidence and Gate 13 handoff.
## Candidate workstreams

Companion API/protocol, authentication/session pairing, mobile UI, private visibility, reconnect tests.

## Deferred requirements

Release packaging/hardening Gate 13; final endurance Gate 14.

## Open questions

Exact mobile framework can be chosen when active; avoid constraining desktop core to one client technology.

## Production integration acceptance

Run a session with multiple companion clients, private information, character updates and map visibility. Disconnect/reconnect one client. Verify all authoritative state remains central and the main app can still play without companions.
