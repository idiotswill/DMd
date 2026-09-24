# Gate 4 table encounter integration

Status: active. Branch `codex/gate4-encounter-execution`; writer root.

## Objective and boundaries

Connect the pure encounter authority to the existing durable desktop table path. This
advances the Gate 4 checkpoint's production encounter, physical dice, agency, hidden
information and exact suspension requirements, under product-definition tactical timing
and offline play clauses and ADR 026. It does not replace full encounter acceptance or
move broad natural-language interpretation from Gate 9 into this slice.

## Acceptance and slices

1. A typed table encounter envelope enforces active session, attendance and player/host
   channels, reuses the pure resolver and retains its exact nested event. Historical table
   event bytes stay readable with absent optional tactical fields.
2. Host setup creates a bounded scene from explicit map geometry and source-derived actor
   placement. It does not accept arbitrary mechanical sheets, hidden-state patches or
   caller-supplied combat totals. Source NPC construction joins the reviewed creature slice.
3. Table projections give each player only their actors' tactical knowledge, plus their
   own actionable choices. Host map truth is absent from player DTOs. Saved transcript
   messages cannot expose secret actors, rolls, targets or locations.
4. The desktop uses the same nonce-retaining retry path and raw physical dice. Narrow
   ordinary-language tactical declarations become typed proposals and preserve material
   choices; controls remain available for inspecting/correcting them.
5. SQLite tests cover authority, mid-roll restart, exact replay/export, forged nested
   events and filtered views. UI checks test retained retries and player channel changes.
   Full verify and packaged desktop acceptance follow integrated mechanics completion.

## Current decisions, risks and next action

Equipment preparation `b6d887a` is a dependency, with independent review in progress.
The turn scheduler is under development in its separate single-writer worktree. Avoid
duplicating its continuation authority in TableState. New optional nested tactical events
will use deterministic replay; no tactical state is trusted in an imported initial anchor.

Next: implement and test the typed table envelope, then source setup and visibility DTOs.
Nothing in this plan claims a playable complete encounter until integrated acceptance.

Current bounded work: expose retained simultaneous-work choices and voluntary saving
throw failure to their authorized player/host channel, without serializing hidden effect
payloads or target identities. Normal turn controls must remain unavailable while a
continuation is outstanding. Add projection and UI regressions, then validate through
the existing durable command path. Exact failed-save provenance is included in restore
audit collection. SRD 5.2.1 p257 explicitly makes Multiattack part of the Attack action;
the weapon window documentation is corrected accordingly before creature integration.

Implementation checkpoint: typed Tactical and PrepareBattlefield actions are written with
active-session/attendance checks, source-derived PC speed/size, exact nested event replay
and secret-safe shared messages. Players receive actor knowledge only; full map/participant
truth stays in the host view. Budget presentation excludes other actors' private records.
Ending the session during initiative is rejected while ordinary quit/resume remains valid.
A real SQLite test covers pending initiative export/restore, continuation equality,
wrong-channel rejection, hidden map filtering and forged nested provenance. This checkpoint
now compiles and that integration test passes. Its first run correctly rejected an absent
fixture player; the test now starts a real session with both players attending. No attendance
rule was weakened. The 13 schema compatibility tests also pass with recovery/creature
authority rejection for old saves and retained atomic migration rollback.

Damage `dbe8cf7`, turn domain `615676a`, compiling turn reducer `4d2704f` and source creature
construction `9f5527a` are integrated. Turn focused tests and creature scheduler/deep resource
validation remain under independent development. Optional recovery/creature state uses the
same pre-tactical anchor, typed legacy preflight and audited retained-command checks. Only
source-pinned creatures receive the level-zero/real-Hit-Dice validation exception.

Next: review/check the combined runtime, connect source creature setup and complete turn
choices/UI and spell/movement/reaction execution. Full gate acceptance remains pending.

Desktop checkpoint: host terrain/placement form, filtered SVG map, initiative/tie controls,
physical tactical roll reporting and initial turn controls are connected through the
existing retained-command path. Svelte reports zero errors/warnings; 13 existing tests
passed and the two added regressions pass (host-to-player map replacement; unchanged raw
disadvantage faces and original command after uncertain failure/restart). Production Vite
build passes. Native packaged encounter acceptance and the full final-head checks remain
outstanding. No complete-combat claim is made by these initial controls.

Turn-choice checkpoint `1c7dac5`: controller-filtered simultaneous work and voluntary
save-failure choices are projected without hidden effect source/target IDs, payloads or
DCs. Existing turn controls stay disabled during retained continuation. Sixteen frontend
tests pass together, Svelte has zero errors/warnings and the production build passes.
The two Rust projection regressions are written but await the serialized build slot;
the new recharge/Legendary Action continuation variants must also be integrated.
