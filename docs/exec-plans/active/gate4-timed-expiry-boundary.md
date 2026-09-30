# Gate 4 — Timed effect expiry at turn boundaries

Status: **Plan only, 2026-09-30. Implementation and executable verification have
not started. Root review of this plan is the next checkpoint.**

## Objective, ownership and baseline

Repair the supported case where a still-live timed tactical effect prevents normal
turn advancement at its deadline. Preserve every previously accepted history's
operation stamps, expiry tickets, work ordering and durable continuation.

Branch: `codex/gate4-timed-expiry-boundary`. Sole writer: the assigned expiry-repair
agent, coordinated by root. No PR exists. Base is freshly fetched `origin/main`
`c4d8c34c19b5c92eca789f292f99632a0107d861`. The clean, unused historical turn-core
checkout was reused; `codex/gate4-table-turn-core` remains preserved at
`25be7f6c5f31e60a065f009c52ab44c852639aed`. No reset or force operation was used.

Root retains the single build/verification slot and the original-source capture,
baseline, UI, Air, Shove and encounter-release integration work. This branch must
not modify those checkouts, captured histories or immutable content.

## Product and architecture contract

This advances the [product definition](../../product-definition.md)'s complete
tactical timing, player control, explainability/provenance, real production path,
and session start/end and exact suspension requirements. It addresses the
[Gate 4 checkpoint](../../checkpoints/gate-04-tactical-encounters.md)'s durations,
concentration, start/end triggers, ongoing saves and exact mid-combat recovery.
The [gate protocol](../../checkpoints/gate-execution-protocol.md) and
[active gate plan](gate-4-tactical-encounters.md) remain binding.

Relevant accepted contracts are [ADR 018](../../architecture/018-rules-kernel-and-semantic-replay.md)
(pure atomic resolution and semantic replay),
[ADR 020](../../architecture/020-rules-restore-history-validation.md)
(original-anchor and complete history preflight), and
[ADR 026](../../architecture/026-interruptible-encounter-resolution.md)
(durable interruptions, controller-owned simultaneous ordering and historical
semantics). The [effect lifecycle plan](gate-4-effect-lifecycle.md) explicitly
retains suppressed effects and their durations. SRD 5.2.1 timing remains the
pinned rules source; this repair adds no adjudication or house rule.

This bounded repair does not complete Gate 4. Other tactical families and packaged
encounter acceptance remain open. Noncombat elapsed-time scheduling belongs to
Gate 5, broad catalogs to Gate 6, living-world simulation to Gate 7, autonomous
language/DM to Gate 9, and voice to Gate 10. No product scope is reduced.

## Source finding and reachability

At the base, `tactical/turns.rs::begin_boundary_from` performs existing Ready and
legacy expiry handling, then observes Time and immediately Turn, before collecting
effect work. `tactical_effects::observe(Time)` queues due AtTime group/target expiry
tickets. Turn refuses a nonempty pending list. Consequently the normal scheduler
cannot offer the newly due work. Domain `expiry_matches` already matches a due
AtTime under Turn as well as Time.

A genuine source path exists: create/control Cultist Fanatic and Mage through the
normal table catalog, cast Hold Person, fail its actual initial and repeat saves,
and retain concentration through ten natural round advances. The source installs
a 60-second deadline. The End-to-Start transition that would reach it queues expiry
under Time, then fails at Turn. EndTurn itself is available while Paralyzed. If an
accepted End was suspended on child work, the command completing that work can be
the rejected clock-crossing command instead. The resolver clone and application
transaction make rejection atomic; earlier accepted costs and raw rolls remain.

This is source analysis, not an executed reproduction. The independent external
memo `tooling/timed-expiry-boundary-independent-audit-2026-09-30.md` outside the
repository contains the reviewed source chain and alternatives. The same sequence
is present in release `44ae95053d5f94d16a64e5f0684c605a4a8b2c87` and integrated
Shove `3f8024bde89f8878bcc42197e42d60169182cceb`; their inclusion is not required
to repair the base.

## Bounded design

1. After the existing Ready/legacy handling, inspect **all raw stored groups and
   effects**, including suppressed entries, for AtTime due at authoritative now.
   Reuse the domain expiry predicate; do not infer expiry from visible conditions,
   active-overlap projections, source names or wall-clock time.
2. If no AtTime is due, execute the current **Time then Turn** sequence unchanged.
   Empty Time operations still carry an operation stamp; they are not removable.
3. If any AtTime is due, observe **Turn once**, omitting the preceding Time only on
   this formerly refused path. Existing Turn observation collects timed expiries
   and matching turn consequences together. Use existing Turn ticket causes and
   normal step allocation; do not fabricate a no-op Time receipt.
4. Keep pending guards strict. Then use the existing shared boundary work frame,
   `new_effect_work` and pump. Simultaneous work remains the current turn owner's
   explicit choice; singleton handling and cancellation stay unchanged. The raw
   roller may differ from that ordering owner and retains its own authority.
5. Preserve End effect work before the separate Legendary window, all End work
   before initiative/time advances, Start cursor/per-turn reset exactly once,
   Ready's current expiry order, and source recharge/death/recovery and nested
   concentration/fall continuation boundaries.

No persisted schema, command, observation or transport variant is planned. Do not
globally remove Time, reverse Time/Turn, resolve expiries automatically before
collecting other boundary work, relax pending guards or add a second scheduler.
A dedicated atomic boundary adapter is a larger alternative only if a concrete
counterexample defeats this design; stop and record that evidence before expanding
scope. No implementation of such an alternative is currently authorized.

## Replay proof obligation

Every previously accepted boundary must retain its complete operations and tickets.
The intended proof is that, after existing preliminary cleanup, any raw due AtTime
record would have queued at least one ticket under Time and failed Turn. Observation
does not remove records. Therefore every historically successful boundary takes
the unchanged no-due branch. Suppression does not exempt a record from this proof.

Review must check this argument against queue deduplication, group/target membership,
pending guards and preceding Ready cleanup. Tests must verify actual historical
operation step/command stamps, ticket IDs/causes, work occurrences, request identities,
source pins, outcomes, projections and receipts, not only final HP/conditions.
Preserve successful history prefixes and all original capture bytes. If a previously
accepted history changes, this design has failed its acceptance criterion; fixture
regeneration or relaxed equality is not a repair.

## Acceptance and planned slices

1. **Review this plan.** Confirm the due-only branch and unchanged accepted-path
   proof before source changes. Only this document may change at this checkpoint.
2. **Minimal resolver repair and meaningful rule regressions.** Use an actual
   installed timed spell and normal round advancement to cover both immediate End
   completion and completion after an accepted raw/owned interruption. Assert one
   clock advance and ordinary legal expiry work. Add focused invariant coverage
   for simultaneous timed expiry, owner-relative expiry and turn-trigger work in
   both selected orders, suppressed raw entries, group/target cancellation, ticket
   deduplication and unchanged rejection while prior work is pending. Any synthetic
   pure lifecycle fixture must be labeled as such and cannot substitute for the
   real-source path. Verify End effects still precede Legendary windows.
3. **Real persistent Hold Person regression.** Extend/reuse the existing genuine
   table creation/control/cast route in `table_missile_cases`, with file SQLite and
   normal physical failed saves, without injected effects, manual time changes,
   manufactured actors or database edits. Keep the effect alive to the natural
   deadline. Cold reopen before crossing and at the resulting expiry choice; also
   restore an independent export and continue through the same commands. Exercise
   exact accepted retries, changed-body/stale/foreign no-write rejection, selected
   expiry completion and final restore. Compare the original cast cost, all prior
   raw roll objects and durable journal/projection/binding prefixes; concentration
   and Paralyzed clear without unrelated damage or resource changes. Check the
   ordering owner's capabilities separately from target raw-save ownership.
4. **Historical compatibility.** Run unchanged no-due boundary controls, including
   Shield owner-start expiry, Ready, recharge and End/Legendary work. Replay genuine
   original-source histories once root's producer/baseline work is accepted, including
   the seven flow4 captures. Root coordinates their integration; this branch neither
   generates them with the repaired source nor rewrites their fixtures or producers.
5. **Independent full-diff review and exact-head verification.** Resolve findings,
   run the appropriate focused checks and canonical verification in the assigned
   serial slot, update actual evidence here, then let root coordinate exact-head CI,
   protected merge and merged-main verification. No old branch pass is acceptance
   for this repair. Gate 4 remains active after this bounded slice.

Planned focused commands, only after root releases the verification slot:

- `cargo test --locked -p dmd-rules --test tactical_turns`
- `cargo test --locked -p dmd-rules --test tactical_effects`
- `cargo test --locked -p dmd-rules --test tactical_attacks`
- `cargo test --locked -p dmd-app --test table_loop` with the new case's focused
  filter during iteration, then the relevant existing controls.
- Root's unchanged original-source baseline/compatibility target after integration.
- `./scripts/verify-fast`, then `./scripts/verify` on the complete reviewed source.

Use the root-configured toolchain and one compiler. Do not start another build,
change test stack limits, weaken source/persistence assertions or edit immutable
captures to obtain a pass.

## Validation, risks and exact next action

Completed: source-only audit, product/ADR/checkpoint review, clean unused-checkout
check, fresh main fetch and exact-base branch creation, and this plan. No production
or test source changed. No executable reproduction, compiler, test, npm, build, UI,
database inspection, push, PR or merge was performed for this repair.

Risks: a raw due record missed by the predicate would preserve the bug; unconditional
Time removal would change accepted provenance; expiry-first automatic cleanup would
steal a simultaneous choice; an imprecise app fixture could end Hold Person early
and never exercise the deadline. Rule-only success would not establish file-SQLite
retry/restore correctness. Source analysis does not establish full Gate 4 acceptance.

**Next action:** root reviews this exact plan commit. Until root authorizes source
work, keep the branch clean and do not implement, compile, test, run npm/builds or
push. After authorization, implement only the bounded due-only observation selection
and its tests, preserving every no-due execution path and reporting any counterexample
to the historical proof before changing the design.
