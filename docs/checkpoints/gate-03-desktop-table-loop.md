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

## Integration evidence in progress

[PR #20](https://github.com/idiotswill/DMd/pull/20) merged the table foundation at
`ac900fb5c5603f26bd3f3108aecf81bf597eadae`. Its tree matches reviewed head
`52ffab23c5e67db7d6ee40a622533b08d44f10fd`: full local verification passed 207
Windows/GNU tests; [CI run 318](https://github.com/idiotswill/DMd/actions/runs/36016090350)
passed 208 Linux tests, Rust 1.88 compatibility, formatting, strict Clippy and guards.

The foundation provides durable Session Zero agreements, player/PC bindings, source-derived
supported creation, current sheets, bounded local text proposals/questions/corrections,
physical dice, accepted-outcome recaps and exact pending-state recovery. Composed replay
validates table/rules provenance and preserves immutable recovery anchors. These application
tests are evidence for the foundation, not a substitute for the packaged desktop scenario.

[PR #21](https://github.com/idiotswill/DMd/pull/21) supplies the desktop. At intermediate
head `51d2962d1a9bd7e1bae792ade7bc203e44e3d262`, Linux CI and 207 local Windows/GNU
tests pass; [Windows run 6](https://github.com/idiotswill/DMd/actions/runs/36017036314)
passed Rust 1.88/stable native checks, strict desktop lint, three native host tests,
frontend checks and 12 frontend tests. Release executable and offline NSIS construction
succeeded, but the post-build clean-source guard rejected packaging. That run is failed,
has no accepted artifact and proves no native UI acceptance. The integration plan tracks
the fix and remaining scenario; Gate 3 remains active.
