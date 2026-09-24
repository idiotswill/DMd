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

The independently reviewed ordinary attack checkpoint `334d82b` is integrated by
`1325da9`. Its own focused 19 attack, 20 turn and 33 vitality tests and strict rules/domain
Clippy passed. Root adapted its allocation to the verified boxed cursor, added generic
attack/damage roll labels and retained attack/equipment/knockout-rest origins to the
restore audit. Owned knockout and Graze controls are projected without damage/AC values.

The first physical form uses canonical weapon/ability/grip/delivery choices, real carried
ammunition and explicit equipment changes. Its targets come from only the acting
creature's current perception, including when viewed by the host. Three initial UI tests
passed; Svelte caught four unsupported test-query typing options, which were corrected
and now check cleanly. A fourth choice/channel regression is added for the next run.
All 25 frontend tests now pass together, with Svelte zero errors/warnings and a successful
production build on `624c642`. Independent review of the physical projection, form and
decision controls found no defect. These UI checks do not verify backend integration
yet. Root's Rust build is queued after inventory and casting. Real SQLite scenarios are
now drafted for physical attack/damage/knockout suspension and restore, source NPC shield
drop and live AC, outsider rejection, truthful player labels, host-versus-NPC perception
and loss of a thrown weapon from subsequent physical choices. They are not yet compiled.
Concentration follow-up through table spell casting remains pending. Light/Nick follow-up choices and source feature controls
remain active work; the first form currently submits the ordinary Attack action only.

Both new SQLite scenarios now pass with all 15 `table_loop` tests on the normal test
thread stack. The knockout case also executes the identical pending choice after a fresh
database restore and compares the complete resulting state against uninterrupted play;
independent review of this stronger continuation assertion found no defect. The focused
hidden-target attack regression and strict workspace Clippy are running next. Evidence:
`tooling/gate4-attack-table-tests.log` outside the repository. This is focused integration
evidence; full canonical verification and packaged encounter acceptance remain pending.
