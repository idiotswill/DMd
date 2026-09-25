This fixture preserves the genuine legacy executor from commit
`f669389204dccd20cd9847ce0eeed2318add01a8`.

Source: `table_turn_core_cases::second_wind_direct_and_declared_actions_survive_cold_dice_retry_and_restore`
during its canonical local run on 2026-09-25. SQLite's online backup API captured a
consistent snapshot at event22: direct Second Wind and its physical roll, a complete
round, then the accepted spoken declaration and host adjudication. The second raw
d10 is still pending. Both source uses and the Bonus Action are already spent.

The JSON maps the original rows to the existing CampaignExport v3 layout. State,
snapshots, accepted payloads and events are copied without reinterpretation or
rewriting. Observations, projection history and transport bindings were verified
empty. The corresponding regression requires restore/export equality, historical
replay and accepted retry, live completion of the old pause, refusal of a new legacy
action, and an explicit idle upgrade preserving mechanics. Do not regenerate this
fixture with current execution semantics.
