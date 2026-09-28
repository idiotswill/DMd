# Gate 4 — Keep tactical play readable across decisions

Status: active; native package `56a2225` exposed a pending-prompt focus gap. The
bounded correction passed independent static review; successor compile, runtime
and package verification remain pending.
Writer: root following the coherent correction commit and explicit 2026-09-28
ownership handback. `gate4_verification_review` implemented the correction under
exclusive ownership; root independently reviewed the complete diff and retains
the heavy verification slot. Branch:
`codex/gate4-table-ui-polish`. Reuses the clean, inactive weapon worktree; its
previous branch and commits remain preserved. Development base is missile
candidate `f4890b73814d7c0f3551350352601eb5ed0d19ed`, whose full production tree
matches the source-verified ffc3 package. Fetched main is
`046109cdc849c16100c42588e771f8abe710c787`. This branch must reconcile verified
missile main and pass its own final checks before acceptance; candidate ancestry
is not a substitute for that proof. Draft PR 49:
<https://github.com/idiotswill/DMd/pull/49>.

## Objective and observed defects

Actual packaged Gate 4 play exposed three interface defects: selecting an owned
source creature combines every owned observer's map/list, duplicating known
creatures; Character Sheet equipment text still describes implemented weapon and
Savage Attacker controls as unavailable; each accepted roll/dart consequence
remounts the whole view, dropping scroll position and keyboard focus.

Fix these through the existing desktop components and transport. Product-definition
clauses for ordinary players, recognizable sheets, player control, information
boundaries and feature completion apply, alongside Gate04 and ADR024/026/028.
Authority, dice, rules, source content, save formats, receipts and outbox retry
semantics remain the existing production path. No tactical family or gate closes.

## Planned changes and acceptance

1. Render the selected actor's observer map, including only its cells/contacts;
   host maps continue to use host truth. Switching controlled actor replaces the
   map. No borrowing another controlled observer's current or remembered contacts.
   A missing selected observer produces no guessed view. Test actual component
   switching with disjoint contacts and verify each visible entity appears once.
2. Replace stale sheet prose with the supported weapon/mastery/Savage Attacker
   behavior and honest limits. Do not claim all mastery effects or rest controls.
3. Preserve position and useful focus during successful same-context tactical
   decisions. Scope changes and errors still clear private content/focus the error.
   Pending and uncertain requests stay locked and retain their exact original
   envelope. Review form-reset effects before deciding whether to retain DOM or
   restore position after the existing refresh; do not silently retain stale form
   choices. Add meaningful consecutive-decision and scope/error regression checks.
4. Native `56a2225` play at the lower-page Cast form produces response ordering and an
   owned Magic Missile response above the current viewport. Both Cast-to-order
   and order-to-response retain the lower viewport and focus the whole encounter,
   leaving the actual decision offscreen. Mark the existing actionable prompt
   groups with typed identities, exclude disabled groups, and focus/scroll the
   next available prompt after successful same-context submission. Cover hit and
   missile order/owned responses, initiative ties, opportunity and liquid-landing
   reactions, attack decisions and optional save/legendary choices. Ordinary
   action forms and foreign/private waiting states must not become prompt targets.
   Preserve the verified reachable physical-roll transition and cleared faces.
   Add real-form transition regressions and repeat the native failing sequence on
   the corrected package. DOM assertions alone cannot close native viewport proof.

## Verification and risks

Root coordinates one shared heavy build slot. Do not run cargo/Node while release
or missile verification owns it. Use frontend checks/tests/build, canonical
`./scripts/verify`, full independent review, all six exact-head jobs, packaged UI
verification, protected expected-head merge, fetched tree parity and literal-main
checks. Native packaged play must demonstrate consecutive decisions staying
reachable, selected-observer visibility and truthful sheet prose.

The main risk is preserving stale viewer data or editable choices during a refresh.
Keep campaign/player/source identity checks and durable uncertain-request handling.
Tests must distinguish retained UI position from retained authority. Source content
and game rules require no changes for this slice.

## Implementation and evidence — 2026-09-27

The map selects the explicit actor's observer before deriving any terrain or
contact tokens. Host truth remains separately supplied by the existing DTO.
The component regression switches between observers with disjoint current and
remembered contacts/terrain, then an unavailable observer; no fallback borrows
another actor's knowledge. Sheet text now describes the supported Nick/Graze and
Savage Attacker controls while retaining the unsupported-rest limitation.

Accepted tactical requests still clear and remount the saved view so physical
faces, payment and other form choices reset. The UI records scroll/focus together
with campaign, viewer, source actor and page, restoring them only after success
in that same context. Concrete physical-roll and consequence groups take
precedence over the generic encounter container when a new decision appears.
Errors retain their existing alert focus and exact uncertain-request envelope.
The regression cases exercise consecutive opaque consequence handles, successive
raw-roll identities with cleared faces, an uncertain response, page navigation
during delivery, and new physical work after consequence/End-turn decisions.

The first implementation passed `npm run check` (zero errors/warnings), all
105 tests in 18 files, and `npm run build` (140 modules). A jsdom-only scrolling
warning exposed that resetting a spy restored the unimplemented native shim;
using a direct inert mock fixed it. The affected 39 tests in two files then
passed without that warning. Those results precede the final generic-encounter
focus correction and its added End-turn case; that final source must run anew.
No result here substitutes for native scrolling geometry or package acceptance.

Independent review by `aftermath_finish` cleared observer privacy, form resets,
sheet scope and the consequence-to-child-roll correction. Root found the same
priority issue when an ordinary encounter control creates a raw request, added
the End-turn variant and requested the follow-up review. `git diff --check`
passes. No rules, source content, dependencies or save formats changed.

## Native follow-up and correction boundary — 2026-09-28

The actual Windows job `108625596375` for source
`56a222571ca3c0edfb727f32dd4b5e7d95f040f6` reports 106 UI tests in 18 files,
zero static diagnostics and 140 built modules. Native QA used downloaded artifact
`10934313845`, archive SHA-256
`6b6e7eb176ad51a391bd426b8945438165add40a287764621c484b1778f89fc0`;
all 1,070 packaged manifest entries matched and the application displayed that
literal source build. These checks qualify only this source and package.

Root completed normal native round-two Hag self-targeted Magic Missile play:
reverse the initially unlisted response order, decline Shield, enter physical
faces `1, 2, 3, 4, 1, 2`, then choose impact buttons `6, 1, 4, 2, 5` before the
remaining singleton 3. The physical-roll transition and all five subsequent
blank forms remained at the same viewport, as did each fresh impact queue. The
Cast-to-order and order-to-owned-response transitions still left the actual
prompt offscreen and focused the whole encounter. UI acceptance is incomplete.

External evidence is retained at `tooling/ui-56a2225-packaged-observations.txt`
and `tooling/ui-56a2225-packaged-state-audit.json` in the parent development
workspace. After normal play and application close, the read-only SQLite audit
(`tooling/audit-ui-56a2225-packaged.py`) passed at event sequence 46: Mage HP 81,
Hag HP 93, twelve distinct physical request and accepted-result IDs across both
casts, flow 4 active, no pending work or effects. This corroborates the observed
saved outcome; it does not repair or waive the failed viewport transitions.

The correction adds typed, stable prompt identities to the existing actionable
groups and excludes disabled scopes when selecting the next focus target.
Response stage, opaque handle and actor distinguish successive decisions; the
saved view still clears and remounts, so roll faces and payments remain fresh.
Six new component cases are authored: actual Cast/order/owned-response/roll
transitions; owned hit and missile offer-to-selected-Shield transitions; successive
Host responders; a failed physical save revealing Legendary Resistance; and a
disabled opportunity prompt yielding focus to its new physical roll.

Root independently read the complete source, tests and plan, including the new
identity helper, and reported no static blocking issues. `git diff --check`
passes. No npm/cargo/build command or successor runtime check ran during this
correction. Earlier source results do not verify the successor. The correction
changes no backend, transport, dependency, source content or authority behavior,
and is handed back as one local commit without pushing or dependency merges.

During normal preparation of the next native QA encounter, root also observed
obsolete character-creation copy claiming that weapon properties, ammunition and
hands were not resolved by the table. The creation form now points players to
the actual character-sheet equipment preparation step. This is a text-only
correction; the fresh QA character's purchased armor was prepared through that
normal control. No new mechanic or extra copy-only test is introduced.

The separate campaign `Prompt focus QA Sep28` and session `Pending prompt
transitions` are saved on the verified56 package before initiative. They contain
the normally created Fighter, fresh Host-owned Hag and player-owned Mage, with
unspent source resources. External `tooling/ui-prompt-focus-fresh-qa-preparation.txt`
retains the actual setup observation. This prepares the failing-sequence rerun;
it is not successor package acceptance. The earlier QA campaign is preserved.

Next: root runs the frontend checks and required verification, publishes the
reviewed successor for exact-head CI, and repeats packaged prompt reachability.
Reconcile verified missile main,
then require the canonical suite, native package exercises and complete final
checks before protected merge and literal-main proof.
