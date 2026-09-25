# Gate 4 initiative and durable turn work through the table

Writer: root. Branch `codex/gate4-table-turn-core`, from fetched merged main
`f8e02c9ab20e73636f82a3414bfd12831b5cd880` after verified PR31. Reference integration
is `9fd73da`; later reviewed area/shield changes
remain separate. This plan is checked in before extraction.

## Objective and boundaries

Expose the actual source-aware initiative, tie agreement, turn/budget and raw save
continuation through the real table and durable SQLite/replay path. Advance product
complete tactical timing, authoritative conditions, player agency, physical dice,
privacy and exact suspension under Gate4, ADR024/026 and pinned SRD5.2.1. This slice
is a prerequisite within the active gate, never an accepted complete encounter.

The integrated encounter branch now has more than thirty thousand Rust lines beyond
main across attacks, movement, falling, casting and their tests. Extract one coherent
turn objective rather than submitting those independent features in one review.
Retain the final shared domain/serde shape and fixed raw-role namespace so later
vertical paths activate the same production queue. Do not create a second scheduler
or temporary authoritative adapter.

## Extraction contract

- Include source-derived battlefield preparation, PC/NPC initiative and surprise,
  identical-source grouping, mixed/monster ties and explicit player-only tie agreement.
  Keep original issuer/actor/session boundaries and physical input.
- Include shared turn frames, deterministic work identity, controller choice, start/
  end source hooks, recharge and Legendary Resistance, condition queries, vitality
  recovery and current action/reaction/movement budgets. Existing source effects
  remain authenticated through PR31 and cannot be installed by arbitrary UI payload.
- Attach recovery and flow with historical omitted/default semantics, typed legacy
  preflight, original anchor, every source/operation origin and semantic replay.
  Keep PR30 composite validation and PR31 immunity/legacy-condition fixes intact.
- Preserve final typed placeholders for later attack/movement/casting/falling state
  only where needed for the single durable shape; reject every nonempty unsupported
  cursor/work record or action before any mutation. Do not accept inactive features
  merely because their enum/record is serializable. No fallback or silent skipping.
- UI exposes only implemented initiative/turn/budget/controller/raw choices. Later
  physical attacks, movement/OA/falling and casting activate in separate complete
  vertical slices on refreshed main. Their existing integration code stays preserved.
- Any physically unsupported initial map must fail closed before play; do not expose
  flight/falling or occupied-space movement before its actual source resolver ships.

## Verification and review

1. Inspect each module dependency and port only the bounded execution closure. If
   faithful source behavior cannot be separated without a disposable implementation,
   expand the coherent slice and record the concrete dependency before copying it.
2. Port actual table initiative/tie/start-end/dash/dodge and source modifier scenarios.
   Exercise fresh SQLite close/reopen at decisions/raw input, same-ID retry, concurrent
   stale/foreign rejection, original-source audit and forged current/anchor/history
   rejection before writes. Test unsupported serialized future work explicitly.
3. Run focused rules/app/frontend checks and canonical verification under the global
   single Rust compiler. Request independent full-diff review; fix findings and rerun
   only appropriate checks. Verify all six final-head CI jobs, protected merge,
   fetched merged-tree parity and post-merge checks.
4. Update this plan and the central Gate4 evidence. Continue remaining same-gate
   source mechanisms, knowledge/morale, improvisation and real packaged encounter.
   No Gate5 or owner acceptance waiver is authorized.

## Current state and next action

PR31 merged with expected-head protection at `61739b48a753f71047996a3483a84c5afadd2ed4`
after fresh final evidence-head review, source/lock parity and all six final checks
(Linux `36122086428`, Windows `36122086408`, including offline installer). Fetched
main `f8e02c9` has identical full-tree content. Post-merge runs `36122812303` and
`36122812557` are in progress. This fresh branch has its plan committed before code;
no source/code/test acceptance exists yet for the extracted slice.

Read-only dependency review confirms the turn closure is separable while retaining
the existing scheduler. Keep vitality/drop-held/live-AC and knockout-rest proof,
EndOccupiedSpace, source effect saves/damage/concentration, Legendary Resistance,
recharge and after-End choice/decline. Floor-only initial placement is insufficient
unless enforced on every accepted/restored tactical image. Reject unsupported
position-changing effects and nonempty future cursor/history authority, including
last_movement, movement_progress/origin, weapon_history, attack_window, remaining
attacks and other_slot_casters. Preserve historical non-tactical recorded rolls.
No legendary payload execution is claimed by a decline-only timing boundary.

The preserved integration source at `d046810` now passes the Svelte check (zero
errors/warnings), all38 UI tests with one Vitest worker, and the Vite production
build. These are integration-branch frontend checks, not turn-core extraction or
backend acceptance. Logs: `tooling/gate4-integrated-ui-tests-r2.log` and
`tooling/gate4-integrated-ui-build-r2.log` outside the repository. Earlier UI/Rust
attempts interrupted by system memory exhaustion remain failed attempts.

PR31 source verification is now complete:375 Windows Rust tests and all canonical
guards/lints; evidence head `61739b4` has fresh independent review and final CI is
green. Integration imports its immunity fix, source regressions and legacy null
shadow coverage. The combined condition query preserves legacy ActiveEffect posture
even after recovery attaches, while new grouped/recovery conditions respect immunity.
The historical query regression now covers that attachment too. This combined
production change is formatted/reviewed locally but still needs focused runtime
verification with the extracted turn slice; PR31's simpler source pass is not that
evidence. Automatic duplicate enum/module declarations were removed during merge.

Next extract the bounded source closure, adapt existing real table scenarios and add
explicit unsupported authority/restore regressions. Reuse the reviewed pure physical
drop geometry for held-item consequences; do not activate a fall or movement resolver.
Then request full independent extraction review and wait for the serialized compiler
after shield/area checks. Final code still requires canonical and exact-head CI.

Initial extraction is now present. It preserves final wire types and fixed raw-role
tags, but explicitly rejects inactive vertical actions, cursor/work records, turn
budgets and future raw-history roles. Pure spatial drop geometry is reused for
Unconscious held-item custody/AC; every tactical image requires dry floor positions.
The existing complete source turn/death/recovery/concentration/recharge/LR tests are
ported; the previous ordinary Attack admission assertion becomes explicit rejection
in this slice, with its full source attack test retained on the integration branch.
New serialization regressions cover unopened work kinds, budgets, fall cursor and
elevation. Rust formatting succeeds; source is not compiled yet. Svelte check passes
with zero errors/warnings; the focused frontend test run is in progress. Next add the
real cold SQLite round/retry/restore case, obtain independent full extraction review,
then verify source on the queued compiler and CI. No intermediate test fixture or
interface result is claimed as complete encounter play.

## Review correction and durable round regression

PR32 initial source `d475b5e` received independent full extraction review. It found
one UI blocker: map existence incorrectly selected the tactical raw-dice handler
for ordinary checks and Second Wind before initiative. The correction derives a
`TableRollChannel` from the actual visible pending purpose. A private/absent roll
has no channel; projection does not expose an otherwise hidden pending purpose.
Three UI regressions cover the two ordinary dice types and tactical initiative.
The new disk-SQLite round exercises ordinary check/Second Wind after map setup,
initiative, Dodge/Dash and round advance. Each accepted phase is also executed on
an export-restored mirror; original disk reopen and exact command retry must retain
state, journal and audit, while new-nonce stale and foreign input fail. Forged
future budget authority is rejected before writes in both current and backfilled
snapshot forms. This new Rust regression is not yet compiled or accepted.

Initial CI Linux `36123469257` failed on actual E0599 output: the extraction omitted
the existing UUID v5 workspace feature and its locked sha1_smol dependency while
retaining deterministic tactical identity methods. Copying the exact integration
manifest/lock restores that required dependency without a version update. Final
verification must run again on the corrected head; no old green check applies.

PR31 post-merge verification now passes all six jobs on `f8e02c9`: Linux
`36122812303`, Windows `36122812557`, including the offline installer. Svelte
check on the PR32 routing correction passes with zero errors/warnings; frontend
tests/build are running. The serialized Rust compiler remains with the area slice
after completed shield verification, with turn-core next. Next compile this source,
fix any actual failures, obtain correction review and canonical/exact-head CI.
