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

## Imported owned Shield artifacts

Both files came from
`table_hit_cases::owned_source_shield_reopens_each_decision_preserves_attack_cause_and_rejects_changed_hits`.
The unchanged scenario uses a genuinely created Mage delegated to its present
Player, source transport2 and the attacker's own PC channel. Every captured
transport binding is version2. Both snapshots retain Mage HP81.

| File | Capture pause | Bytes | Events | Bindings | Projections |
|---|---|---:|---:|---:|---:|
| `shield-hit-v1-selected.json` | Selected first response, physical attackface10, protective uses0 | 133906 | 20 | 9 | 13 |
| `shield-hit-v1-post-cast-damage.json` | Second Shield committed after physical natural20, protective uses2, original damage pending | 194330 | 28 | 17 | 21 |

```text
b3dc61a39a42f9e1de3bca53633489da457de4e07e5763fc93998f4220b78729  shield-hit-v1-selected.json
65f03e391abaf0a540ef122b74ad69a403e878da4e323a6623adceb61a88a04d  shield-hit-v1-post-cast-damage.json
```

The original focused owned Shield scenario passed1/1 in1427.43s on the default
Windows test stack. Its command selected the full scenario name from
`cargo test --locked -p dmd-app --test table_loop`, using the same external capture
directory. The actual log is `tooling/shield-diagnostic-06045b2-owned-shield.log`.
All four imported files match the lengths and SHA256 of their original exports.

The fourth capture retains the attacker's original accepted-hit CommandMeta as
the pending damage cause; the fresh Shield command is distinct. Its attack stage
is DamageRoll and `attack.damage_roll` is still null: production sets that field
only after accepting the damage result. The raw request is already retained by
the pending roll. Preserve this distinction in future compatibility assertions.

## Passed baseline verification gate

Diagnostic successor `f9698e1761ee0485c4cf0ff593f89a03f016302b` copies all four
artifacts and the exact reviewed compatibility module from85cff17, with -text
attributes. The module's working-file SHA256 is
`41cae152fab4d43bc5e2ce91d6f1bd4e57ddfd433e2ef64be18df5bd03edcee6`;
the unchanged cleanup helper is
`78f204d7c12dad3d234fa3f95d948b9cba29952d084f94d34642eb534d05c49d`.
Complete source/dependency/content comparison confirms production equivalence
with this development branch. The only other code differences are isolated
read/export hooks in the original scenario drivers. Reusing this already built
generator does not claim verification of any later executor or integrated head.

The separate four-file baseline test compiled in25.99s and passed1/1 in299.50s,
exit0, default Windows GNU stack, jobs1 and CARGO_INCREMENTAL=0. The actual log is
`tooling/shield-baseline-f9698e1.log`. It restored all four exports into real file
databases, verified byte/typed roundtrips, cold resume, every original accepted
request/response and unchanged full exports for exact retries and changed bodies.
The exact selected command was:

```text
cargo test --locked -p dmd-app --test legacy_shield_hit_v1_replay genuine_flow3_capture_baseline_roundtrips_and_retries_original_bindings -- --exact
```

Both original generator scenarios and the four-file baseline gate have passed.
Root authorized the approved flow4 domain/rules development after this evidence
and the import-time continuation-test correction are committed.
The independent missile continuations toHP7 and the selected-hit/post-Shield
continuations remain mandatory acceptance. No new runtime behavior may be used
to regenerate these histories or replace their original transport receipts.

Import-time inspection found one unrun continuation assertion in85cff17 that
mistook the fourth snapshot's pending raw request for an already accepted
`attack.damage_roll`. It did not affect the separately selected baseline test.
Correct that assertion against the genuine snapshot in a separate test commit;
all four continuation tests and final integrated checks remain required.
