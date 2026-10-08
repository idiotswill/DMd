# Gate 4 own-turn intrinsic attack controls

Status: planned, source-only allocation on 2026-10-08; runtime verification UNRUN.
Writer: `v5_capture_plan_review_oct8`. Branch:
`codex/gate4-intrinsic-attack-controls`; no PR yet. Parent fetched and allocated
base `52ae4bf36ce9378eb4ddc48b3bcef8477d2e4580`, tree
`f13e29b7b53b13fcbcda9849e19a632cd36aa094`; writer independently checked clean
branch/head. Parent owns integration, compiler/runtime scheduling and acceptance.

## Objective and governing requirements

Expose the existing supported own-turn `CreatureAttack` through normal desktop
controls. The backend already executes intrinsic source melee attacks, but the
desktop has only source weapon and reaction attack forms. The missing route
prevents the intrinsic-family native witness in the combined Gate 4 acceptance.

This advances product-definition's normal application play, feature-completion,
physical dice and exact save/resume requirements, Gate 4's authoritative attacks,
player-safe visibility and recovery, and ADRs 024 (table authority), 029 (immutable
source admission), and 020 (restore/history validation). It preserves the existing
source-action/Charge plan. It does not accept Gate 4 or the human encounter.

## Scope and non-goals

- Separate strict version-one advisory current-read request/response, authenticated
  from a single owned execution/history snapshot and exact audience revision.
- Actual active source actor, current controller/attendance, current timing and
  pending guards; source-derived supported intrinsic features and actor-perceived
  targets. Shared pure eligibility helpers only where needed and sound.
- Transient desktop loader/form and the existing `CreatureAttack` action through
  the existing persisted outbox. Late responses cannot cross campaign, channel,
  selected source, revision, encounter or turn contexts.
- Additive application/frontend regression tests and exact source evidence.

No new mutation, source content, save schema, durable presentation field, archive
format, transport version or authority bypass. No fake attack origins, synthetic
forecast execution, hidden mechanics prediction, physical placeholder items,
client source catalog enumeration, NPC/reaction permission substitution or promise
that every offered feature/target pair is legal. Existing accepted attack execution,
old test bodies, content and corpus bytes remain unchanged. Multiattack, legendary,
ranged source expansion, capacity/v5 capture and portable desktop UI are excluded.

## Acceptance and planned slices

1. Commit this plan before implementation.
2. Add owned read admission/source candidates and independent application DTO/API;
   preserve stored projections and every accepted attack cost/proof path.
3. Add Tauri bridge, transient desktop choices and intrinsic form, retaining exact
   saved request retry behavior.
4. Add genuine application tests for source choices, ownership, no-write reads,
   privacy, pending/unsupported/stale cases and accepted intrinsic recovery; add
   frontend form/outbox/context-race tests. Keep exhaustive historical tests intact.
5. Self-review full diff and static `git diff --check`; commit coherent source and
   hand exact head/tree/diff to parent for independent review and scheduled checks.

Application acceptance covers genuine ordinary intrinsic, conditional source and
multi-component source choices, Host and selected-source authority, stable raw
attack across real relation release, independent restore/retry, and byte-identical
durable history before/after reads. Feature menus use pinned source IDs/names and
actor perception. They do not reveal hidden targets or replace final command checks.
Frontend acceptance covers late success/failure, source/PC/campaign/revision/turn
switches, unavailable/version/error states, locks, and complete original request
preservation after lost acknowledgement/reload. These are application/frontend
requirements, not a new native Cartesian matrix.

Parent must later schedule appropriate focused tests, `./scripts/verify-fast`,
`./scripts/verify`, required exact-head CI, full independent review and packaged
native evidence. An older base/head pass is not evidence for this new source.
Native witness: normal source selection and own-turn intrinsic attack, owned public
dice, a meaningful close/reopen at pending attack, actual off-turn relation release,
completion without duplicate cost, and later current-state action. Host must wait
when a player owns the source. Controlled QA faces do not prove human physical dice.

## Decisions and verification

The parent accepted the bounded design review, SHA256
`13dd55a302c68f17fc3253c80fcc4b218e1c1fd8383e1c9b245ca164fa4d78ff`,
retained externally at
`tooling/ci-oct8/own-turn-intrinsic-attack-route-proposal-2026-10-08.txt`.
The independent read remains advisory; final admission uses unchanged
`CreatureAttack`, owned `attack_current` and `intrinsic::plan_with_read` before cost.
Do not fabricate `CommandMeta` to make the planner produce UI choices: Charge
validates actual accepted movement and attack origins.

All build, test, CI, native, database and production acceptance states are UNRUN
for this branch. Source-only authoring is allocated while the parent owns a frozen
52ae canonical run. No Cargo/npm/build/native/database/runtime, push, PR or merge is
allocated here. Static source review and `git diff --check` only.

Next action: commit the plan, report its identity/scope to parent, then implement
the bounded read/form with additive tests. Material design changes require parent
review. Gate 4's broader obligations and final human acceptance remain pending.
