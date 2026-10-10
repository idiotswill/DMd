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

## Authored boundary and source handoff — 2026-10-07

The reducer selects a crate-internal creation entry only for its two existing
creation operations. That entry requires the existing trusted issuer/no-actor
shape, absent command/session binding, idle table, and the complete existing
`require_finished_encounter` validation. Only that proof suppresses the ordinary
kernel's tactical-state admission refusal. The common evaluator retains campaign
and sequence checks, complete state/read validation, pending/Inspiration guards,
existing identity/source builders and one-time mechanical insertion. Standalone
`resolve` and every other ordinary table rules operation retain original admission.
Semantic table replay uses the same reducer; no new event or save version exists.

Three new application cases use ordinary transport commands and accepted physical
dice. Two establish owned current-source Mage Armor, genuinely conclude and close
the session, then finish through the supported closed-session host action before
creating a legacy or explicitly pinned character. They compare every original
rules field and world/character entry, complete encounter/history, item custody,
scene/location and clock, plus journal/projection prefixes and retained bindings.
They exercise cold reopen, portable file restore/reopen/replay, exact retry across
a subsequent session, changed-body whole-store refusal, normal physical equipment
preparation and new-session selection. Refusal controls cover active/aftermath and
Finished active-session states, stale revision, wrong session/channel/player,
reused identity, current source mismatch and standalone rules entry. Invalid
completion images are negative restore inputs only; each leaves an unrelated
destination database's complete rows unchanged. The third case preserves a real
pending Grapple roll under creation/session refusals and resumes through its
actual controller with the original accepted raw result and grip provenance.

The original Mage helper, original failing case, all prior cases and immutable
captures remain unchanged. The new cases supplement that unchanged regression;
they do not substitute for running it on the frozen successor. Direct rustfmt
parsing/check and full source/body preservation audit are the only current
validation. Canonical fast/full, focused gameplay cases, strict Clippy and native
acceptance remain UNRUN for this source. Independent review must precede root's
publication and allocation of those checks. Root's original integration checkout
and runner remain untouched.

Next: independently review the frozen full delta and preservation receipt, then
allocate canonical checks and the unchanged original-Mage case together with the
three new `finished_character_creation::` cases. Any observed runtime failure is
to be preserved and corrected against its exact source; no inherited pass applies.
