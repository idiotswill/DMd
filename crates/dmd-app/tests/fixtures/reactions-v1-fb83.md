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

A fourth fixture, `reactions-v1-upgrade-100c7da.json`, was captured separately on
unchanged source `100c7dabe07b07b7430bcb721b1dd7f48e4792cf`. It is not part of the
three-file canonical-run backup described above. The disposable harness restored
the original `legacy-savage-f960.json` into a fresh file database, submitted the
actual owner's source-admitted critical Savage Attacker sets `[1,2]` and `[4,4]`,
explicitly chose the first, then submitted the host's original `UpgradeExecution`.
That accepted command changed only flow1 to2 and normal event sequencing. The
export was taken immediately after acceptance, at event16:16 audits,11 presentation
records,2 transport bindings and the single unchanged original snapshot.

The first unchanged-source capture run passed without a harness or production fix.
Both accepted requests cold-retried with byte-identical responses; full normalized
exports and all29 SQLite table counts remained unchanged. Independent file-SQLite
restore/export equality also passed. Its original input and preserved copy both
retain SHA256`c9a5f5b542322464ab873e9ad26632d53e02773f875761f1ac391e42e83978ea`.
The104740-byte captured export has SHA256
`0aac1462c6e0d244127a5332e08e5ecb24a88e04c1425bc38a1d09c75a12b69d`.
Capture code, log, metadata, actual requests/responses and databases remain outside
the repository under `tooling/reactions-v1-fixture-audit/capture-upgrade-100c7da-run1/`
(the adjacent log has the same directory name with `.log`). The export's original
bytes are preserved here. It proves the old unit upgrade's meaning independently
of any future targeted upgrade API; never regenerate it with the new executor.

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
