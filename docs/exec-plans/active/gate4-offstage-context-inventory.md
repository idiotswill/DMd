# Released-time guarded checkpoint: consumer inventory

Status, 2026-10-04: source and controls authored; **all 16 new controls UNRUN**.
This inventory accompanies the reviewed offstage plan and guarded implementation.
It is a static source inventory, not compiler, canonical replay, application,
restore, database or native evidence. Public current/historical/restore admission
remains closed. The complete production vertical slice remains required.

## Context construction and mandatory readers

The old internal mandatory triple is represented by
TacticalResolutionContext::Turn(TacticalTurnContext). Its typed explicit wire
adapter emits the original turn_actor, turn_number and boundary at their old
positions. Only the new explicit ReleasedInterval context omits them. No default
actor, synthetic turn or fallback boundary is introduced.

| Consumers | Exact checkpoint treatment |
| --- | --- |
| rules tactical/areas.rs, attacks.rs, attacks/creature.rs, attacks/unarmed.rs, casting.rs, medicine.rs, movement.rs, second_wind.rs, turns.rs | All nine existing resolution constructors create Turn with their unchanged original actor/number/boundary; other fields retain old behavior. |
| rules tactical/areas/validation.rs, attacks/creature.rs, attacks/unarmed.rs, attacks/opportunity.rs | Mandatory actor/number/boundary reads use fallible turn_context and reject ReleasedInterval. |
| rules tactical/casting.rs, creature_bridge.rs, hit_reactions.rs, medicine.rs, missiles.rs, missiles/validation.rs, movement.rs, second_wind.rs, turn_validation.rs, turns.rs | Same fallible turn read. Turn completion, owner-boundary observations, recharge/legendary, movement and next-turn routing stay on Turn. |
| rules tactical/turns::{pump,choose} | Dispatch only explicit ReleasedInterval to the private elapsed pump/choice. Existing Turn routing retains its guards and automatic singleton dispatch. |
| rules tactical/continuations::{start,finish,key,request} | Shared actual work entry/leave dispatches released work to its private executor. Unsupported raw/ordinary execution of new legacy-expiry work rejects. Existing Turn raw continuations and fall queue remain unchanged. |
| rules tactical/work_trace.rs | Unchanged context-independent allocator/ancestry validation. The interval's exact source partition additionally binds every allocated root and completion/cancellation. No second execution queue. |
| app table_tactical_choices.rs | Existing mandatory-actor continuation DTO returns None for ReleasedInterval. Two ordinary local test constructors explicitly create Turn. No new elapsed DTO or UI is claimed. |
| app table_hit_reactions.rs, table_missile_reactions.rs | Existing view returns None when context is not Turn; all actor reads use the successfully obtained turn. |
| app table_source_control.rs | Existing delegate/order actor reads obtain an optional Turn; ChooseTurnWork requires a fallible Turn when existing area-host ordering does not apply. New AdvanceReleasedTime is classified as actorless host input, but the public rules guard rejects it. This classification is not app admission. |
| Existing effect/weapon/spell/creature timing types | Their own turn_number, EffectTurn and EffectTriggerUse.turn_actor fields are unchanged; they are not TacticalResolution context aliases. |

Mechanical ordinary source-compiled helper adaptations are limited to:
app tests/support/table_hit_driver.rs and table_oa_concentration_cases.rs;
rules tests/support/hit_responses.rs, tests/tactical_attacks/casting_timed_expiry.rs
and hit_shield.rs. Their turn-fixture reads explicitly expect Turn. Existing
TacticalFlow fixture in tactical_spells/tests.rs adds omitted None;
TacticalEncounterHistory fixture in tests/tactical_turns/release.rs adds an empty
omitted interval list. These helpers are distinct from the five frozen receiving
suite files, which remain blob-identical.

## Validation, authority and codec graph

| Entry or shared reader | Exact checkpoint treatment |
| --- | --- |
| public kernel validate_state, tactical validate_tactical_state, public tactical current/historical resolve | Explicit rejection of flow7, upgrade/context records, elapsed receipts or historical execution7 completion receipts. Begin7, targeted upgrade7 and AdvanceReleasedTime reject before policy can admit them. |
| private released_time::transition | Only explicit 5→7, existing ConcludeHostilities/FinishEncounter for7, AdvanceReleasedTime and an exact interval ChooseTurnWork. Validates input and cloned candidate; actual active table session, host/system actorless authority, campaign/id/sequence checks. No public reducer caller. |
| ReleasedValidation | Private constructor; lifetime and pointer-bound to the exact immutable candidate. Derives structural command/source/work evidence, then feeds unchanged shared validation composition. It is not serialized or authenticated by an ambient flag. |
| kernel::validate_released_state and tactical::validate_tactical_state_with_released | Check candidate binding. Preserve complete pack, creature, inventory, roll, source, geometry, initiative and turn checks. Only exact due interval ownership is passed into the relevant readers. |
| release retained_dependencies/history/preflight; aftermath; turn_validation | Private scoped readers permit the exact entered interval source partition. Ordinary Finished guards still reject a live resolution. Ready, movement-result and work ancestry validation precede the no-turn branch. Budgets/cursors/retired placement checks remain enforced. |
| kernel legacy AtTime check | Only the exact bound live legacy-expiry work permits that overdue row. Legacy RulesAction::AdvanceTime and its original semantics are unchanged. |
| raw deadline inventory | All groups and target rows including suppressed effects, legacy AtTime and accepted stable delay. Stable records are sorted by actor for deterministic allocation. Expiry work binds source/id/establishment or stable origin/delay/raw request, instant, actual Time stamp, allocated work and actual completion. |
| domain history and missing-completion invariant | Named supports_release handles5/7; old5 receipt remains unchanged on explicit Finished upgrade. Receipt7 is produced by the actual release path after Active5→7. Upgrade and interval provenance/chain/target/command identities are structurally checked. Structural validity does not authenticate history. |
| snapshot codec schemas1–3 | Typed decode rejects duplicate authority before Value migration. Any encounter_history is rejected as future authority. SQL migrations, source checksums and raw histories are unchanged. |
| rules_restore, table transport/UI, repository play-session/retry/privacy | Remain closed through public validation/reducer guards. No accepted offstage journal, exported-history authentication, app command, UI capability or native scenario is claimed at this checkpoint. |

## Authored controls and evidence limits

Eleven new rules controls exercise the actual private producer over authored world
fixtures: two real Mage Armor sources with either expiry order; three genuine Mage
sources retaining two siblings after a choice; a later-round genuine recast and
suppressed old deadline; actual Medicine and accepted physical StableRecovery d4;
Active5→7→Conclude→Finish; authority/candidate/source/stamp/work mutation rejection;
bounds/version/ruling/unsupported-condition/rest rejection; suppressed-condition
revelation rejection; and isolated group cleanup, legacy expiry and capacity
fixtures. Group/legacy/capacity fixtures deliberately test reducer boundaries and
are not represented as real spell or accepted journal evidence. Stable recovery
starts with the actual private damage adapter to create zero HP, not an accepted
attack history; Medicine/d4 are produced through the real tactical command path.

Four domain controls cover original captured Turn bytes, strict duplicate/null/
partial/mixed context rejection, new omission and versions1–5/7/reserved6. One
persistence control covers typed old-schema history/duplicate-shadow refusal.
All sixteen are authored **UNRUN**. Existing assertions and protected original
histories were not replaced. Direct rustfmt and git diff --check are the only
execution-like verification at this handback; no compiler/tests/build/DB/native.

## Remaining acceptance and boundaries

The interval pump advances to each earliest actual absolute deadline, observes
Time once, uses the existing occurrence allocator/frame stack and automatic
singleton dispatch, and resumes the original target. It does not emit Turn or
invent custody/fall/rest handling. It refuses raw tactical/legacy condition
payloads (including suppressed rows), rest/knockout obligations and ungrounded
participants before time changes. Stable zero-HP Unconscious itself remains
positively reachable. Precise group cleanup records sibling cancellation with its
actual applied cause before frame pruning.

Next work requires independent full source/consumer review and root-allocated
verification before app/UI/restore admission. Repository session authority and
original retries, opaque origin/occurrence capabilities, host privacy, original
accepted semantic history, file SQLite/cold/portable/native proof and accepted
expiry/main integration all remain open. Wider Gate4 offstage relative timing,
physical consequences and rest/source dependencies remain open. This checkpoint
is neither feature completion nor a gate acceptance claim.
