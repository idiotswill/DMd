# Gate 4 source creatures through private table setup

Writer: root. Branch: `codex/gate4-creature-table`, from refreshed main
`ac35c1d65d27239f704cde1422209a0f106fb0e1` after verified PR29 merge. Integration reference:
`220ee804d7344564a265f02c4d42cb57a7e560c2`; existing source and table plans remain the detailed implementation history.

## Objective and boundaries

Expose source-derived NPC creation through the actual host table, with canonical
statistics, explicit allowed size/languages, actual gear identities and ammunition,
private setup views, exact uncertain retries and durable restore. This advances the
product's source fidelity, physical equipment, privacy and recovery requirements and
Gate4's source creature participation. AGENTS, product definition, Gate4, ADR024/026
and pinned SRD5.2.1 govern the work.

Extract the already implemented creature profile/limited-use definitions and physical
gear boundary, optional source attachment and validation, typed CreateCreature action,
private host setup projection and form. Preserve explicit selected-feature omissions.
Do not copy the shared tactical queue, casting, battlefield/initiative controls, effects
or vitality attachment into this slice. Listing a source feature does not establish its
playable execution. NPC policy helpers alone do not satisfy autonomous encounter behavior.

## Required work and acceptance

1. Reconcile current main after PR29; inspect the complete dependency closure of
   tactical_creatures, tactical_creature_equipment and the table adapter before copying.
   Retain source profile identity and real physical armor/custody validation.
2. Add optional/default/omitted source attachment and constructor defaults. Extend the
   legacy save preflight, typed duplicate rejection, original-anchor check and exact
   command-origin audit without changing historical SQL migrations.
3. Extract host/session/pending setup checks, canonical source choice projection,
   actual gear identity retention, private creation transcript and desktop form.
   Preserve the existing PC check/Second Wind/character creation path after NPC setup:
   do not copy the integrated branch's blanket legacy-action rejection for source
   creatures when this slice has not yet shipped the replacement tactical path.
4. Port the existing real runtime create/retry/export/replay and private-view tests.
   Exercise independent file reopen, malformed source/gear and forged source origins,
   preserving atomic failure. Verify selected-feature limitations remain explicit.
   Include an ordinary PC table check after NPC preparation to prevent a regression
   in the already shipped table loop.
5. Run focused checks, frontend tests/check/build and canonical verification; obtain
   independent full-diff review, final-head Linux/Windows CI and expected-head merge.
   Check merged tree parity/post-merge CI, then reconcile the encounter integration.

## Status and next action

The source boundary, private form and real SQLite reopen/restore regressions are
implemented as a Rust-unverified candidate. Svelte check reports zero errors/warnings;
all 16 frontend tests and the production build pass. Logs: sibling tooling
`gate4-creature-ui-{check,tests,build}.log`. No Rust build has run on this branch yet because
falling and source attacks hold the serialized compiler queue. Exact-head canonical
verification and independent review are still required.

Next: commit/open a draft PR for exact-head CI, then run
focused Rust and canonical verification when the compiler is released. Remaining
Gate4 NPC decisions, attack features, shared effects and packaged encounter acceptance
stay active; no Gate5 work or reduced gate acceptance is authorized.

## Extraction decisions

- Creature profile/schedule/policy and physical gear module closure is copied from
  encounter integration `220ee80` without shared queue/effect/recovery coupling.
  Pure capability operations remain internal building blocks, not table-playability evidence.
- Retain source-aware level 0/Hit Dice validation, and preserve existing PC tests and
  Second Wind. Legacy RequestTest explicitly rejects source NPC actors because their
  printed modifiers require the forthcoming source tactical path.
- Actual host form requires an existing player character, matching the mechanical-state
  prerequisite; preparation remains private, and copy clearly states encounter play is
  not yet shipped in this bounded build. Full Gate4 acceptance remains unchanged.
- PR29 post-merge main `ac35c1d` checks are all green: Linux `36107189850` and
  Windows `36107189841`, including stable offline installer packaging.

Independent environment reviewer inspected the full extraction and source closure with
no actionable finding before this candidate commit. Exact committed-head review and
Rust/canonical checks remain pending; no gate or slice completion is claimed.

## First exact-head CI finding

Draft PR30 source `53cbbe9` compiled and passed strict Clippy/Linux MSRV/architecture
and genericity checks. Its new actual PC check after NPC preparation failed at physical
roll completion: the inventory validator called the whole CampaignState validator while
the rules child had cleared pending, before its table parent cleared roll_context.
The same dependency existed after PC equipment preparation; this regression exposed it.

Factor identity/world reference checks into `CampaignState::validate_references` and use
those plus inventory/source checks inside the inventory reducer. Full `validate` retains
all existing attachment, encounter and table invariants, and the app validates the whole
completed table transition before commit. Tests retain actual PC check/Second Wind and
add a deliberately incomplete final table context that still fails validation and restore.
This is an application-composition fix; no final invariant or historical event changes.
Correction verification is pending on the next head.
