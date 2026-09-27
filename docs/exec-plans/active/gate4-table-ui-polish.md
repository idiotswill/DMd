# Gate 4 — Keep tactical play readable across decisions

Status: active, implementation reviewed; final exact-head verification pending.
Writer: root only. Branch:
`codex/gate4-table-ui-polish`. Reuses the clean, inactive weapon worktree; its
previous branch and commits remain preserved. Development base is missile
candidate `f4890b73814d7c0f3551350352601eb5ed0d19ed`, whose full production tree
matches the source-verified ffc3 package. Fetched main is
`046109cdc849c16100c42588e771f8abe710c787`. This branch must reconcile verified
missile main and pass its own final checks before acceptance; candidate ancestry
is not a substitute for that proof. No PR yet.

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

Next: complete review and publish the bounded draft for exact-head CI while root
serializes the prerequisite's canonical run. Reconcile verified missile main,
then require the canonical suite, native package exercises and complete final
checks before protected merge and literal-main proof.
