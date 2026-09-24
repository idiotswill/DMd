# Gate 3 — First playable desktop table loop

Status: **Active — owner authorized continuation after Gate 2 on 2026-09-24.**

## Product requirements advanced

- normal-user campaign/session flow;
- Session Zero/table contract;
- supported character creation/selection;
- text-first table interaction;
- physical dice reporting;
- transcript/recap;
- exact session start/end/restart;
- rules-question/correction path.

## Objective

Turn the durable rules/runtime foundation into the first application a human can actually use for tabletop-style play, still text-first and visually simple.

## Entry conditions

Gate 2 accepted. Desktop shell/application boundary can consume runnable campaigns without bypassing validation.

## Acceptance criteria

A normal user can:
- start the desktop application;
- create/list/open a campaign;
- establish the table contract;
- create/select supported PCs;
- bind attending players to PCs;
- start a session;
- enter ordinary text declarations and questions;
- receive rules outcomes and roll requests;
- report physical die faces;
- see a transcript and player-safe recap;
- correct a misinterpreted but uncommitted declaration;
- end/save, quit, restart and continue;
- pause at an unresolved material decision and resume without inventing resolution.

## Architecture invariants

All gameplay-facing access routes through the application/runnable boundary. UI never writes authoritative state directly. Manual/text input constructs the same typed proposals/commands eventual voice will use.

## Explicit non-goals

Voice, polished visuals, full tactical combat, living-world simulation, companion phone apps.

## Required failure/recovery behavior

Application/provider/UI crashes or malformed inputs do not corrupt authoritative state. Resume distinguishes accepted from pending work. A normal user receives recoverable errors rather than database/Git instructions.

## Merge/pause boundaries

Codex may merge in-scope Gate 3 work autonomously after exact-head verification. Product/table-contract semantics must remain consistent with the accepted product definition; do not narrow player agency or recovery guarantees to make acceptance easier. Pause at the end of Gate 3 with evidence and the proposed Gate 4 handoff.
## Candidate workstreams

Desktop shell, campaign/session UI, Session Zero, player/PC binding, text input, transcript, roll UI, recaps, correction UX, session lifecycle/recovery.

## Deferred requirements

Full tactical encounters Gate 4; noncombat breadth Gate 5; autonomous intent interpretation Gate 9; voice Gate 10.

## Open questions

Exact UI visual design remains flexible. Determine minimum accessible keyboard/text-only experience.

## Production integration acceptance

Run a short real tabletop-style scene through the packaged/production desktop path without developer tools. Save/restart mid-session and continue with correct transcript, player/PC bindings, resources and unresolved decisions.
