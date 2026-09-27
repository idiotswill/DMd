# Gate 4 — Keep tactical play readable across decisions

Status: active, implementation not yet verified. Writer: root only. Branch:
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

Next: inspect existing refresh/reset and observer tests; implement the bounded UI
changes, then request independent review while the shared slot remains serialized.
