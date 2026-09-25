# Genuine ShieldHitV1 compatibility saves

These are untouched exports from the unchanged flow3 production executor at
`7bc01afc46f3591e38e5168072496e91c115c10d`, production-identical to Shield candidate
`8b5cf52a250c5f381d22977391906cfedc53a630`. The isolated diagnostic generator is
`06045b2b8f6cff6d895a07be46ea7cb1b4087a2f`. Its only changes are an execution plan
and read/export hooks in the existing actual Night Hag and owned Shield scenarios;
it changes no executable production source and must never merge into main.

The generator writes `export_campaign(...).to_json()` bytes with create-new
semantics under `DMD_CAPTURE_MISSILE_DIR`. It does not edit state, commands,
events, source ownership, original envelopes or presentation history. The files
were captured on 2026-09-26 and preserve their original LF formatting. Fixture
attributes disable automatic checkout conversion.

## Imported Magic Missile artifacts

Both files came from
`table_night_hag_cases::night_hag_current_catalog_casts_six_real_darts_through_cold_owned_transport`.
The existing scenario creates actual source actors, casts Night Hag's level4
Magic Missile through transport1, and accepts six individual d4 faces of1 with
sequential HP changes under flow3. The original retained Hag controller is
`Autonomous`; Admin/Host supplies its secret rolls. It is not a Player-owned source
transport2 history. The target is the original source Warhorse at19 maximum HP.

| File | Capture pause | Bytes | Events | Bindings | Projections |
|---|---|---:|---:|---:|---:|
| `shield-hit-v1-missile-after-0-darts.json` | First physical face pending; Warhorse HP19 | 106929 | 15 | 4 | 10 |
| `shield-hit-v1-missile-after-2-darts.json` | Two actual face1 results accepted; HP15; third face pending | 119949 | 17 | 6 | 12 |

```text
1173eed10ff4d80e87b6e3a5b67721a0a9b2c9fcdd1a800c009e1b1734a5b4c1  shield-hit-v1-missile-after-0-darts.json
fd410b848973510230ef5d1a4321636ef5df421ffc0067c19f7c5cfb1fb29cfc  shield-hit-v1-missile-after-2-darts.json
```

The original focused generator scenario passed1/1 in500.88s on the default Windows
test stack. Its command selected the full scenario name from
`cargo test --locked -p dmd-app --test table_loop`, with the external capture
directory selected in `DMD_CAPTURE_MISSILE_DIR`. The actual source log is retained
outside the repository at
`tooling/shield-diagnostic-06045b2-capture-missile.log`; original exports remain in
`tooling/missile-captures-06045b2/`. Import checked both file lengths and SHA256
against those original bytes. This generator result does not claim the new
compatibility module has compiled or run.

## Remaining capture and verification gate

The same generator's owned Shield scenario is still running. The two required
names are `shield-hit-v1-selected.json` and `shield-hit-v1-post-cast-damage.json`.
Their bytes and passing scenario evidence have not yet been imported; missing
files fail explicitly rather than skipping the compatibility tests.

Before flow4 runtime development, both generator scenarios must pass and all four
original artifacts must pass the separately selected baseline import, typed/byte
roundtrip, cold resume and every original request/response exact-retry test.
The independent missile continuations toHP7 and the selected-hit/post-Shield
continuations remain mandatory acceptance. No new runtime behavior may be used
to regenerate these histories or replace their original transport receipts.
