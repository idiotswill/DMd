# Gate 4 — Character creation after authenticated encounter completion

Status: source implementation allocated2026-10-07; no runtime executed.
Branch: `codex/gate4-finished-character-creation`.
Baseline: freshly fetched integration `1ffb841f67e87c4239b68dc6ff5e045e0395f5c0`.
Sole writer: ci_oct7, in the reused `gate4-grapple-combined-receipt` checkout.
Root owns independent review, verification allocation, publication and merge.

## Problem and binding scope

The unchanged original-Mage integration test fails by name on both retired573
platform runs. Earlier complete logs report the ordinary kernel's active-tactical
rejection; current573 cancellation omits its precise panic. Source independently
shows the helper genuinely settles its original Shield, finishes the encounter,
ends the session and attempts ordinary character creation. Finish retains the
authenticated Finished flow/history, while the ordinary kernel rejects any flow.
The host can add a player but cannot create the new character after completion.

This is a production lifecycle gap, not permission to alter the original Mage
fixture, rewrite its history or clear an encounter. Product-definition character
experience requires supported creation and campaign continuity through character
replacement; its session/recovery clauses require exact retained consequences.
ADR023 owns source-derived creation and nested table provenance; ADR028 and the
accepted encounter-release plan require validated Finished history, retained
scene spaces/dependencies and atomic replacement without an empty-flow shortcut.
This bounded slice advances those Gate4 requirements. Wider classes, progression,
retirement UX and later-gate systems remain deferred, not declared complete.

## Selected implementation boundary

Admit only the two existing ordinary host table creation operations after a
genuine authenticated current Finished encounter and an actually closed session.
Keep existing trusted issuer/actor, idle, membership, fresh identity, original
versus explicitly pinned source, one-time mechanics and atomic command guards.
The exception must be internal to the table-creation route: standalone rules
commands, ordinary other rules actions, active combat, paused aftermath, legacy
or forged Finished images and pending decisions/work gain no authority.

Use the existing strict Finished/release-history validation and unchanged
semantic replay; do not treat a phase flag or caller-provided permission as proof.
Keep all old flow/history, source owners, HP/resources, equipment/custody, lasting
effects/recovery and accepted raw results unchanged when inserting the new entity.
No new wire/schema, execution version, save rewrite, public bypass flag or extra
gameplay ledger is planned. If the existing boundary cannot safely provide this
exception without expanding that scope, stop source edits and report the conflict.

## Evidence and acceptance

- Preserve every original test body, the original Mage helper and all immutable
  captures byte-for-byte. The failing original test must eventually pass unchanged.
- Add focused production-path cases covering both legacy and pinned-source
  creation after genuine Finish/EndSession, later equipment/session usage,
  file reopen, independent portable mirror/replay, exact retry and changed-body
  whole-store refusal. Preserve retained source/physical/history evidence.
- Exercise active/aftermath/pending/session and wrong-channel/identity refusals,
  preserving every durable row and proving original work can continue normally.
- Refuse forged/unsupported completion history on restore without changing any
  destination table. No synthetic snapshot serves as positive-history proof.
- Independently review the full frozen source and preservation inventory, then
  run canonical fast/full and targeted application cases under root's heavy-slot
  allocation and actual fresh-head CI. Source review does not certify runtime.

## Validation limits and next action

Only source edits and static analysis are allocated here. No local Cargo, npm,
native app, gameplay or database execution; original integration1ff is separately
frozen in `gate4-grapple-positive-integration` for root's running verification.
Do not edit that checkout or the frozen CI diagnostic branch. No push/PR/merge
is allocated to this writer. Direct rustfmt parsing may be used without Cargo.

Next: trace the table/kernel/replay entry points, implement the narrow internal
admission and meaningful application controls, freeze and hand back source-only
evidence for independent review. Record all runtime as UNRUN until executed.
