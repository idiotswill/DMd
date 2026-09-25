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

Main is now `f8e02c9ab20e73636f82a3414bfd12831b5cd880`. Bounded prerequisite
PRs24 (effects),25 (damage/vitality),26 (physical inventory),27 (source definitions),
28 (physical weapon plans),29 (actual equipment preparation) and30 (private NPC
preparation) passed exact-head verification and merged. Detailed evidence remains in
their plans. PR29 canonical source88b468a passed331 Windows Rust tests (332 Linux);
its finalc4d992a and postmergeac35 passed all six checks, including offline installer.

PR30 canonical source6136aeb passed356 Windows Rust tests (357 Linux), strict full
workspace Clippy and both guards. The actual PC-roll regression exposed and fixed
composite validation calling the table parent before its child transition completed;
full final-state invariants and foreign physical-reference rejection remain intact.
Final evidence-only78166c6 received independent full-source/correction/final review,
source-byte parity and all six checks (Linux36111205928/Windows36111206026). Expected-
head merge producedbeaad44; fetched full-tree parity and all six postmerge checks
(Linux36112003062/Windows36112002720) pass. Real form retries/SQLite reopen/export/
restore and PC checks/Second Wind after NPC setup are verified; encounter execution
is not claimed by those slices. Root reconciled current main asd046810.

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

PR31 is merged as `f8e02c9` after375 Windows Rust tests and full canonical checks at
source0486c77, exact final review at61739b4 and all six final CI jobs
(Linux36122086428/Windows36122086408). It attaches lifecycle authority/condition
queries/concentration, legacy typed save guards, recovery origins and original-anchor
enforcement. Review fixed suppressed immune conditions that could strand stronger
effect removal. Expected-head merge, fetched full-tree parity and all six post-merge
jobs pass (Linux36122812303/Windows36122812557, including offline installer).
Integration6796581 preserves the source fix with tactical recovery/legacy posture;
5127efc strengthens that query regression. Combined backend verification is pending.

Fresh main-based PR32 (`codex/gate4-table-turn-core`) extracts initiative, durable
start/end turn work and budget control with inactive future actions/cursors explicitly
rejected. Exact source70b8333 passes full local canonical verification:424 Windows
Rust tests, strict workspace Clippy, formatting/check and both repository guards.
Linux36124985437 passes425 Rust tests and all four jobs; Windows36124985423 passes
MSRV/stable including offline installer. Independent full extraction and correction
reviews are clear,22 UI tests pass, Svelte has zero diagnostics and build succeeds.
Review fixed ordinary dice after map setup routing through the wrong handler. The
new disk-SQLite test executes ordinary checks, initiative and a whole round on both
real and independently restored runtimes, then verifies cold retry and forged restore
rejection; it passes the default Windows stack. Initial extraction compile/lint
failures are fixed and retained in the slice plan. Followup9c22b74 changes only docs
and broadens native Windows CI from desktop-only to the full workspace; Linux is
green there, with the expanded Windows test/packaging step still pending. Final
evidence-head review/CI and expected-head merge remain required.

The next physical encounter slice includes reviewed Immediate-only source casting:
a genuine damaging opportunity-attack/concentration case needs a legitimately cast
ongoing spell. No arbitrary effect installer or synthetic state substitutes for that
prerequisite. Exact combined source/app recovery review remains mandatory; areas,
Ready/reaction magic and privacy protocol remain separate active Gate4 obligations.

Source weapons1ab9e7 passed104 focused cases (62 attack,18 movement,24 turn) and
strict domain/rules Clippy, with independent exact review. Integrated as8fa4484.
Falling sourceebbff59 has final12 falling+43 spatial cases and strict Clippy; earlier
95-case combined evidence predates its last two source fixes and is not final-head
proof. Root integrated its seven-source chain throughb4360d2, then app/UI27c458
as4fa4212. App/UI37 tests/check/build passed on its source branch; actual SQLite
falling verification exposed an oversized async test future. Test-only32ad5be boxes
independent phases and now passes all62 app tests, strict three-crate Clippy and the
liquid landing/recovery cases on the default Windows stack. Integrated as9fd73da.
Earlier runs exhausted system committed memory; those remain failed attempts. After
owner freed memory, integrationd046810 passed38 UI tests/check/build. No user
applications were closed.

Area source2703f96 passed131 distinct rules cases (10 leaf,67 attack including14area,
30 movement,24 turn) and strict domain/rules Clippy, with exact review. Source-faithful
breaths use explicit GM occupied-cell sample/cover policy, shared amount and all saves
before damage, settled fall state before admission and retained source/geometry proof.
Area app integration and explicit controller delegation for hidden simultaneous work
are active on a separate branch. Its697a238 source passes17 current area rules cases;
actual SQLite app verification is running. Privacy audit exposed canonical sequence
and generic transcript command-count leakage. A separate owned branch plans durable
per-audience revisions and acceptance-time presentation history, without reducing
ordering agency or claiming timing invisibility.

Shield slice16ff104 has independent exact review and121 affected rules tests, all65
app tests,42 UI tests, zero Svelte diagnostics, production build and strict
domain/rules/app all-target Clippy. Real SQLite cases retain physical source weapon
identity, paid shield transitions, AC/training, pending raw dice, restart/retry and
hostile restore rejection. Its pre-code plan and source are integrated as ee13cff/
e64c53f. Combined integration and eventual main PR verification remain pending.

The reviewed PR32 visible-purpose dice routing fix is also reconciled into this
integration branch. The only overlap was additive TableApp test insertion; both
shield and dice scenarios are retained, and the older initiative fixture now declares
its actual Tactical roll channel. This combined frontend passes45 tests and Svelte
check with zero diagnostics. Backend evidence still needs the combined source run;
the separate turn-core cold-round regression is preserved on its bounded PR branch.

The18 spell mechanism
families remain active where unfinished, including Ready, interruptions, zones, barriers,
summons, forms and source-linked effects; do not promote pure catalog coverage to playability.
Reviewed spell training correction1b39980 is integrated as456272e: an untrained
Shield alone does not prohibit casting, while actual occupied hands and untrained
worn armor remain enforced. All43 spell tests and strict domain/rules all-target
Clippy pass; paired genuine component and prepared-caster regressions have independent
review. Full combined/app verification remains required. PR32 final25be7f6 contains
only a test comparison correction beyond its canonical production source: export
request time is normalized while every persisted field still compares exactly.
The expanded native Windows test step exposed that second-boundary test defect;
fresh final CI is required before merging.

PR32 subsequently passed fresh final review and all six checks at25be7f6: Linux
36127754177 (425 Rust tests), Windows36127754144 (427 native Rust tests and installer).
Protected merge produced12ed29a72eb3d66ad8565caf358ae573a91aa0dc, with fetched
full-tree parity. Post-merge CI is pending. Fresh codex/gate4-physical-encounters
starts there, with plan5d250ba preceding source extraction from reference0fd7ddc.
Its actual damaging OA/concentration continuation and combined full verification
remain required; prior leaf/source checks do not establish acceptance of this tree.
Separate area checkpointa45fe57 has current17 source and65 app tests, strict3crate
all-target Clippy and clear review. It remains outside this physical PR; protocol
and Ready source branches continue independently, with serialized local builds.
Remaining Gate4 mechanisms, NPC knowledge/morale, improvisation, encounter completion
and the full packaged desktop scenario remain open. No Gate5 work is authorized here.

PR32 post-merge checks subsequently all pass on main12ed29a (Linux36129020814,
Windows36129020852). PR33 initial combined extractionb7f5dfb has all six CI checks,
590 Linux and592 native Windows Rust tests, with independent source/app review.
Review fixed ordinary/intrinsic dead-target admission before resources; the common
guard checks actor knowledge first and does not affect retained lethal validation.
All71 affected attack tests pass, including living zero-HP, hidden target and lethal
OA completion. Genuine damaging-OA/concentration/cold-recovery testf13224e passes
and is independently reviewed, integrated as5e3f89a. Actual lethal/dead-target
SQLite regression9910ce7 also passes after a two-line fixture mutability correction,
retaining zero durable writes and valid lethal replay. Full combined canonical/UI/
final-head CI and protected merge remain pending. The12 Gate4 ledger families now
say implementing, with scopes and evidence arrays unchanged and no gate acceptance.

Continue encounter execution and the effects/equipment/damage integrations from merged
foundation main. Complete all twelve ledger
families and packaged encounter acceptance before the Gate 4 owner pause; do not enter Gate 5.
