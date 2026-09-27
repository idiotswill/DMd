# Genuine closed source-only aftermath save

`reactions-v1-aftermath-be544.json` is an unchanged official export of a consistent
SQLite online backup from the real application test
`aftermath_source_only_mage_retains_owner_and_real_armor_through_session_resume`.
It was captured during canonical verification of source
`be5442af8b24f8ee24d2c0549ea8dac452d44489`, complete tree
`6b497034863a6bd79d250be98d2f3f6b2d02d7af`. PR44 subsequently merged as
`f441adedcf490504b6f1e3db1a964c023c511e47` with that identical complete tree.

The actual scenario creates and assigns a source Mage, accepts physical initiative,
casts real Mage Armor through its player-owned source channel, concludes hostilities
on the retained cadence and closes the session. The capture is after that accepted
close: execution version2, no active session, original armor and paid action retained,
15 events,10 presentation projections and9 original transport bindings. Its sequence
is15 and its campaign ID is `b280b864-d368-4c68-8eab-0617ffa9da75`.

The external watcher opened the live database read-only with `query_only=ON` and an
explicit read transaction, then used SQLite's online backup. It changed no test
inputs, rows, commands or source files. Capture time was
`2026-09-26T07:07:23.774967+00:00`. The original499712-byte backup has SHA256
`4e0b499a229cff217d6fe732090cb2cd3380e954301b24c20ba17fad3e6d669c`; its integrity
check passed and foreign-key violations were zero.

Only after the complete original canonical run passed did an isolated diagnostic
helper open that backup read-only and call the official `export_campaign().to_json()`
under the same unchanged production source. Its create-new output is98963bytes:

```text
4d9075b1566a0279717d9eb617ff4651576187fbbbbc07bf8a053916413d96b5  reactions-v1-aftermath-be544.json
```

The separately selected diagnostic test passed1/1 in31.55 seconds after26.10 seconds
compilation, on the default Windows GNU stack with jobs1/incremental0. It independently
restored into a new file database, closed/reopened/resumed, retried every original
request and response byte exactly, rejected all nine changed-body retries, and checked
the complete normalized export after each operation. The backup hash was unchanged.
The temporary helper was removed and the production checkout remained clean.

Original canonical evidence:705 Rust tests across54 suites, all49 table cases
(1520.76seconds), including all three aftermath cases; full verification passed.
All six final-head CI checks also passed706Linux/708native Rust and81UI.
This baseline evidence does not claim the later current-executor continuation test
has passed; that is verified separately on the implementation branch.

Original backup, watcher, generator, logs and provenance are preserved outside the
production checkout under `tooling/aftermath-flow2-be5442a/`,
`tooling/capture-aftermath-flow2.py`, `tooling/aftermath-flow2-baseline.rs`,
`tooling/pr44-be5442a-canonical.log` and `tooling/pr44-be5442a-flow2-baseline.log`.
Watcher SHA256 is `06f8195f4c5a9072eb1566c4fd3a57d9a8fe8d71fa92ebb1563d208362b98e69`;
generator SHA256 is `aff4cc968a4169c4d65c9e6896c8333694a44b0c18b36c248f1eeda9ab54e874`.
The existing `reactions-v1-*.json -text` attribute preserves these exact bytes on
every checkout. Never regenerate this history with a newer executor.
