# Genuine ReactionsV1 compatibility saves

These are untouched exports of actual file-SQLite campaigns running source
`fb83db7821adfce39f24f4de0707e95a3b49df34`, before live reaction responses.
All three retain tactical flow version2, original game/audit JSON, presentation
history, audience handles and accepted transport bindings. They were captured by
SQLite online backup during the canonical source run on 2026-09-25; they were not
created by modifying a snapshot or replaying commands through a newer executor.

| Fixture | Actual source scenario | Event | Projections | Bindings |
| --- | --- | ---: | ---: | ---: |
| reactions-v1-attackroll-fb83.json | `table_unarmed_cases::actual_unarmed_attacks_and_knockout_survive_owned_cold_dice_retry_and_semantic_restore`, first owned unarmed AttackRoll | 13 | 8 | 1 |
| reactions-v1-knockoutchoice-fb83.json | Same campaign, actual later KnockoutChoice after accepted physical natural20 | 22 | 17 | 8 |
| reactions-v1-paid-ready-fb83.json | `table_ready_cases::owned_ready_abandonment_survives_cold_retry_without_changing_foreign_presentation`, paid movement Ready | 13 | 8 | 1 |

SHA256 of the original exported files (original CRLF container formatting retained;
fixture attributes disable automatic line-ending conversion):

```text
a67212067075ceb4afabf6aef726b6e7370ef010808382b3c8a8cb6396b79e92  reactions-v1-attackroll-fb83.json
e37202249bc46916d984fc429e1f75f4332426ea31b0f30cf9195ee8a313bb5b  reactions-v1-knockoutchoice-fb83.json
08c8aea9578ab28acbe0fb40638e7ea213159a6ebe2ca4098e1e459118ae6629  reactions-v1-paid-ready-fb83.json
```

The pre-change independent audit ran the production libraries at
`ae2cd1c35e7bce585e4200d183afb373ce6dd9a4` (identical code/content to fb83db7;
later100c7da changes only Markdown). It restored each export into a separate file
database, cold-reopened it, and retried all ten original transport requests.
Every response serialized identically; complete exports differed only in the
normalized export-request timestamp, and all29 SQLite table counts stayed fixed.
Original fixture hashes remained unchanged. Audit source/log/report are preserved
outside the repository in `tooling/reactions-v1-fixture-audit/` in the development
workspace; the checked-in tests make the important compatibility guarantees portable.

The attack fixture must retain its original paid Action and physical raw roll,
complete fixed unarmed damage without a newly invented response pause, and preserve
the original request's recovery. The knockout fixture must retain the accepted die
and meaningful owner's choice. The Ready fixture deliberately has no pending roll
or resolution despite owning a paid declaration: an empty resolution alone cannot
authorize a newer executor to reinterpret it. Tests for a new explicit upgrade must
pair refusal while Ready exists with success after legitimate owner abandonment.
Rejection by the older Legacy1-to-ReactionsV1 upgrade does not prove that safeguard.

Do not regenerate any of these files with future reaction/source-control behavior,
rewrite their stored digests, or replace their transport history with newly issued
tokens. New compatibility scenarios require separately named, provenance-recorded
fixtures. Source NPC control and live Shield/Counterspell/Ready release are separate
Gate4 acceptance work; these exports do not prove those features.
