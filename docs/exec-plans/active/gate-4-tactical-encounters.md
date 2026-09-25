# Gate 4 — Authoritative tactical encounters

Status: **Active — owner authorized continuation after Gate 3, 2026-09-24.**

## Objective and baseline

Complete the twelve Gate 4 rules families through the real desktop application, with
source-faithful mechanics, player agency, filtered perception, durable consequences and
exact mid-combat suspension. Fetched main is `afcbe108c57d13322a32260325a5fc0133c7240c`;
Gate 3 is accepted there. Its final local and Linux/Windows checks remain recorded in PR #22.
Initial branch: `codex/gate4-spatial-foundation`; [PR #23](https://github.com/idiotswill/DMd/pull/23).

## Scope, authority and non-goals

The [Gate 4 checkpoint](../../checkpoints/gate-04-tactical-encounters.md) is binding.
The [product definition](../../product-definition.md) supplies tactical philosophy,
complete timing, non-omniscient enemy behavior, information boundaries, physical dice,
player agency, general environment interaction, local play and exact suspension.
All twelve ledger families retain their full existing scope: combat sequence/actions,
movement/geometry, targeting/areas, visibility/stealth, complete combat conditions,
death/knockout, grapple/unarmed, mounts/underwater, weapon properties/masteries, monster
running and reusable tactical spell effects. Catalog enumeration remains Gate 6; this
does not defer Gate 4 mechanics required to execute those catalogs.

Gate 5 exploration/economy/progression, Gate 6 complete catalogs/adventure, Gate 7 living
world, Gate 9 autonomous language/DM and Gate 10 voice remain deferred. No cinematic UI
or replacement persistence/rules implementation is planned. No gate criterion is waived.

Relevant existing ADRs: 002 state/events, 003 AI boundary, 005–007 domain/placement/
knowledge, 009–010 authority/dice, 011–012 atomic persistence/replay, 014–020 pinned
content/runnable rules/restore and 021–024 table/session/character/desktop integration.
The only mechanical source is the pinned SRD 5.2.1 and recorded campaign rulings.

## Planned coherent slices

1. **Spatial and durable encounter foundation:** bounded map/size/elevation/movement,
   geometry/targeting, sensory and knowledge projections; explicit state compatibility,
   validation and regression fixtures. Record architecture choices and source anchors.
2. **Encounter resolution:** initiative/ties/surprise, turn and movement budgets, full
   actions, interrupted movement, reaction/ready windows, weapon properties/masteries,
   unarmed/grapples, conditions/effects/concentration/death, mounts and underwater play.
3. **Spell and creature execution:** reusable target/effect/trigger/summon/transformation
   machinery, monster feature scheduling and perception/capability/morale-based decisions.
   No actor may use hidden knowledge or manufacture unsupported abilities.
4. **Application and desktop:** typed encounter commands and conservative natural text,
   setup/map/player views, raw-dice and material-choice windows, environment adjudication,
   session continuation and provenance-preserving transcript/recap integration.
5. **Integrated acceptance and closeout:** source/coverage audit, independent full-diff
   review, packaged native encounter, fault/restart checks, debt/checkpoint/ledger evidence,
   final merged-main verification and owner pause before Gate 5.

Each coherent slice gets a fresh branch/PR as needed, with one writer per branch. Parallel
work uses isolated worktrees; root coordinates interfaces, integrations, builds and merges.

## Acceptance and verification

- Each applicable selected-source clause in the twelve families has implementation and
  meaningful boundary/interaction evidence, not merely a marker or interface.
- Invalid actions, movement and stale/foreign requests leave authoritative state untouched.
- Pending physical dice, reactions, material choices, effects, initiative, visibility and
  remaining budgets survive restart, export/restore and semantic replay without duplication.
- Player map, transcript and decisions never reveal hidden actors, DCs, plans or outcomes.
- Run a multi-round encounter through the packaged desktop, including physical player dice,
  a reaction, concentration/ongoing effect, restricted visibility, blocked/difficult movement,
  an improvised environmental action and a fleeing/surrendering/negotiating opponent.
  Save/quit/resume with initiative active and finish with correct durable consequences.
- Use `./scripts/verify-fast` while iterating and `./scripts/verify` on each final reviewed
  head; add focused source/mechanical/application/frontend tests appropriate to the changes.
  Linux and native Windows MSRV/stable CI, packaging and exact-head review must pass before
  expected-head merge. Verify final merged main again. Do not start Gate 5 before the owner
  receives the required evidence-based gate summary and authorizes continuation.

## Decisions, evidence and risks

- Initial read-only audits separately examine rules/timing, spatial/perception and desktop/
  persistence integration. Existing mechanics are reusable primitives, not complete combat.
- Final-main baseline is clean; no repository movement was found on entry.
- PR #23 merged as `580f487944608d8c7c7220c7a410a779386ad615` with the schema-4 spatial/perception/content foundation and installed-byte
  pin. It does not claim encounter execution or Gate 4 acceptance. Independent reviews
  cover source definitions, migration/recovery and spatial privacy/geometry. Review fixes
  include Unaware perception, selected-speed Dash, darkness, cover and bounded projections.
- The initial integrated `./scripts/verify` passed 237 Rust tests plus architecture checks;
  subsequent focused spatial tests cover the review fixes. Final-head evidence is recorded
  in PR #23 after full verification and Linux/native Windows checks. The first Windows run
  exposed SQLx Acquire lifetime inference at the desktop boundary; direct acquired-connection
  migration plus a spawned-future regression addresses it without weakening transactionality.
- Exact PR head `0b910b8` passed canonical verification with 249 Rust tests and all six
  CI jobs. Merged-main Linux run 36044061166 and Windows run 36044061140 also passed.
- `codex/gate4-encounter-execution` owns versioned encounter commands, persisted initiative,
  player-approved ties and upcoming interrupts; `codex/gate4-effect-lifecycle` owns grouped
  concentration, suppression and ordered trigger tickets. Both are unaccepted followups.
  The full source audit retains summons, transformations, interrupted casting, forced actions,
  zones, source-linked damage and monster feature limits within this gate.
- Existing TD-001/002/004/006/007/009 remain visible. New compromise must name its receiving
  gate; it may not silently move unfinished Gate 4 rules to a later checkpoint.

## Exact next action

### 2026-09-25 integration checkpoint

Main is now `ac35c1d65d27239f704cde1422209a0f106fb0e1`. Bounded prerequisite
PRs24 (effects),25 (damage/vitality),26 (physical inventory),27 (source creature
definitions) and28 (physical weapon plans) passed exact-head verification and merged.
Their detailed exact-head/local/post-merge evidence remains in their execution plans.
PR29 (physical equipment through the table) also merged: Windows canonical verification
passed331 Rust tests on source `88b468a`; Linux runs332 because of its Unix-only symlink
case. The final docs-only head `c4d992a` retained all source bytes, received independent
review and all six final checks. Merged main `ac35c1d` has exact full tree parity with
that final head; all six post-merge jobs passed in Linux36107189850 and
Windows36107189841, including stable offline installer packaging. Root reconciliation
`18330c5` retains the integration's wider source/flow authority and ports PR29's real
file-reopen, forged-origin/null-shadow and actual equipment-button regressions.
These source/reducer slices do not establish playable encounter acceptance by themselves.

The separate encounter integration branch includes source creature setup/gear,
initiative/turns, physical attacks and Light/Nick choices, KO recovery, raw dice,
movement proposals and owned source/weapon opportunity choices. All59 application
tests passed at `e5d1c2d`'s code (including17 table-loop and3 recovery cases); the
subsequent main merge `ea21c76` preserves the catalog's matching validator fix.
The movement UI passed30 tests, Svelte validation and a production build. A fresh
review found the legal two-handed reaction projection gap; its fix is present and
the dedicated12-comparison grip regression and strict workspace all-target Clippy
passed at `56a8d35`. Miss/decline restore tests do not prove
the still-required damaging reaction and concentration sequence.

Casting checkpoint `81c1a4e` passed96 integration tests and strict domain/rules
all-target Clippy, including the source spell-attack adapter and EndTurn occupied-space
consequence. Movement `4a6f879` plus departure helper `5435743` are integrated; the source
branch passed42 attack/OA,18 movement and24 turn tests plus strict Clippy. Source Charge
`2a16e37`/evidence `bf2e38d` passed94 attack/movement/turn cases and strict Clippy, with
fresh review, and is integrated as `1132e48`/`dc62e56`.

The table-casting checkpoint `9fe100a` passed61 app tests,35 frontend tests, zero Svelte
diagnostics, production UI build and strict domain/rules/app all-target Clippy. Its real
SQLite scenario covers Cultist Hold Person, player save, Dragon rays, separate
concentration children, hidden target exclusion, file reopen and exact retry/restore.
Final copy/Charge-origin changes have the focused rerun evidence recorded in its plan;
independent exact-head app/UI review is clear. Root merged it cleanly as `220ee80` after
PR29 reconciliation. This integrated combination still requires its own full verification;
it is not interchangeable with the separately verified source heads.

Root's fresh main-based `codex/gate4-creature-table` is draft PR30, candidate `53cbbe9`.
It extracts source profiles and actual NPC gear, host-only setup, optional state attachment,
legacy/replay origin guards and the real form. Existing PC checks/Second Wind are retained;
legacy source-NPC checks reject the wrong PC formula. New SQLite tests cover creation,
file reopen, identical retry, eight restore corruptions and ordinary player activity afterward.
Frontend16 tests/Svelte/build and independent full extraction review are clear; Rust and
canonical/exact-head CI remain pending. Do not infer verification from the integration.

Falling queue attachment and actual source physical weapon attacks are separate active
branches using the serialized compiler. New source SaveArea breath work has a plan on
`codex/gate4-area-resolution`; it does not claim area execution yet. The18 spell mechanism
families remain active where unfinished, including Ready, interruptions, zones, barriers,
summons, forms and source-linked effects; do not promote pure catalog coverage to playability.
Remaining Gate4 mechanisms, NPC knowledge/morale, improvisation, encounter completion
and the full packaged desktop scenario remain open. No Gate5 work is authorized here.

Continue encounter execution and the effects/equipment/damage integrations from merged
foundation main. Complete all twelve ledger
families and packaged encounter acceptance before the Gate 4 owner pause; do not enter Gate 5.
