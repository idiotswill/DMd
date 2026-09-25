This fixture is a genuine legacy tactical damage pause from source commit
`f9602a8454fe09958ef04ac3d6fc26ba8410d485`, before the reaction foundation.

Source: `table_savage_cases::tactical_savage_survives_owned_opaque_dice_cold_retry_and_hostile_restore`
during its canonical local run on 2026-09-25. SQLite online backup captured event14
from the actual file database: normal campaign/character/source Goblin creation,
initiative, a paid source dagger attack and its physical natural20. The two critical
d4 damage dice are pending; the Fighter still owns Savage Attacker's choice.

The export copies the original state, snapshots, accepted payloads and event rows
without rewriting their JSON. Its nine original presentation records are retained;
there are no observations or transport bindings at this pause. The regression checks
restore/export equality, accepted legacy retry, owned two-set completion, cold exact
retry and independent replay. It keeps fresh legacy actions blocked until a settled
explicit upgrade. Do not regenerate this history using the newer executor.

`src/table_creature_catalog_v1.json` preserves the eight-entry creature picker wire
image serialized after this untouched export successfully restored under the
pre-Mage catalog. All original host presentation hashes were checked before capture.
That snapshot is compatibility data for presentation v1, not a current rules catalog;
new creation choices are read from current sources by `table_creature_options`.

Git chronology confirms all eight entries predate presentation v1: Chimera was
added at0ff676d, before storage969f936 and application7aff85c; main introduced v1
atc9b8207. Catalog-producing code/data are unchanged from that v1 introduction
through f9602a8. No supported earlier seven-entry v1 catalog is being overwritten.
