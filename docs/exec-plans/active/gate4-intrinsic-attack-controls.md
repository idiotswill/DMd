# Gate 4 own-turn intrinsic attack controls

Status: authored under the source-only allocation on 2026-10-08; independent
review and all compilation/runtime verification remain UNRUN.
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

## Authored source and static review, 2026-10-08

Plan-first commit: `9c900c6b05b21a20f891a193f4e0de176f73b622`.

The new strict v1 `table_intrinsic_attack_options` read uses the existing snapshot
export/history reconstruction and an owned `TableRead`. Its rules query checks the
active actor, session attendance, source ownership and pending state. The scheduler
provides a new pure candidate reader sharing its existing `can_act`, current-turn
and source-availability predicates. It does not call the mutation, create command
metadata or forecast an attack. Host can direct Host/Autonomous sources; enabled
source access preserves player-owned decisions even when the player is absent.

The desktop uses a lazy “Choose creature attack” read, accepted by the parent to
avoid repeating the expensive full-history read on unrelated table refreshes.
Choices are transient, actor-perceived and separate from the retained view. The
panel clears on context/lock changes and ignores obsolete success and failure.
Submission uses the existing `CreatureAttack` and unchanged save/load/retry path.
No old serialized structure, source content, accepted executor or test body changed.

Three new Rust tests are authored: a genuine Chimera history with Host/source-owner
commands, release during pending raw work, ownership reassignment, closed-session
read refusal, immutable repeated reads, portable restore/retry, and lawful G3/G4
then Finished-to-M+G4 progression; four distinct source cases; and strict independent
read schema. Every existing application test body remains unchanged; the existing
test file only includes the new support module. The new frontend file declares
26 cases for lazy submission, unchanged outbox reload, context/lock success/error
races, response binding/version, empty/error states and rendering authority guards.
These are authored cases, not executed passes or extra mandatory native scenes.

Source inspection corrected one example in the preserved external proposal: Air
Thunderous Slam has one Thunder component. Young Red Dragon Rend supplies the
two-component case; Warhorse supplies conditional effects, Air supplies its current
immutable source pin, and Ogre confirms the physical route remains separate.

Self-review covered the complete added code and test bodies against the existing
admission/Charge/source-controller rules, desktop retry and source perception paths.
The static diff is additive: existing test bodies and old DTO/command implementations
have no removed or replaced lines; no content/corpus files changed. The parent
explicitly allowed direct `rustfmt.exe --edition 2024 --config skip_children=true`
for the touched Rust files. That syntax/format pass and `git diff --check` pass;
they are not typechecking or execution evidence. The formatter did not reformat old
test bodies. No Cargo/npm/build/native/database/runtime, push, PR or merge ran here.

All tests, builds, required CI, packaged/native and production acceptance are UNRUN
for this new source. Parent's frozen 52ae run is not this source's verification.
Next action: parent independently reviews the coherent source commit, schedules
focused tests plus fast/canonical/exact-head CI and packages the verified receiver
for the bounded native witness. Resolve any actual findings before acceptance;
Gate 4's broader obligations and final human encounter remain pending.
