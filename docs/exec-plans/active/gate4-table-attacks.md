# Gate 4 physical attacks at the table

Writer: root on `codex/gate4-encounter-execution`. Dependency: the separately reviewed
primary attack continuation. This follows the active Gate4 checkpoint, proposed ADR026
and product requirements for real physical dice, player choices and exact suspension.

## Objective and boundaries

Connect physical weapon attacks and their material choices to the existing table command,
retained delivery, roll and recovery paths. Read-only projections may offer only the
authorized actor's actual carried equipment and current knowledge of targets. They must
not expose another actor's HP, AC, source profile, private name or hidden position.
The rules resolver derives legal range, current advantage, costs and consequences anew
when accepting the command. A displayed option grants no mechanical permission.

This bounded UI does not claim complete attacks or complete Gate4. Source monster attack
features, all applicable masteries, movement/reactions, spell attacks and narrow ordinary
language proposals remain explicit active integration work. No unfinished tactical
requirement moves to Gate5 merely because a first control can be shown.

## Planned slices and acceptance

1. Project real ItemIds, canonical weapon choices and finite usable ammunition for the
   current authorized actor. Keep equipment/ability/grip changes explicit player choices.
2. Send typed Attack actions through the existing stable command identity. Surface owned
   knockout/mastery choices and raw physical attack/damage rolls from the same cursor.
3. Collect all retained attack and mandatory-rest command origins during restore audit.
4. Test player/host authority, current channel changes, unknown delivery/restart, damage
   and concentration follow-up, actual NPC shield drop and exact SQLite export/restore.
5. Run focused UI/backend checks, independent review and canonical verification before
   claiming this slice verified; include it in the packaged multi-round gate acceptance.

## Current status and next action

Plan created before implementation. Root cursor heap fix `1237c3a` and focused table,
recovery, turn, spell and lint checks are green. The attack author is validating the
shared resolver and compulsory-rest consequences on its own branch. Inspect that exact
checkpoint before integrating it; develop only independent table projection/UI contracts
until then. Rust builds remain globally serialized.
