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
