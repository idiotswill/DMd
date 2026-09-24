# DMd

DMd is a local-first tabletop RPG application and autonomous-DM project intended to become a fully functioning game for sustained open-ended campaigns.

The target is not a prototype, proof of concept, technology demo, or scripted campaign framework. Intermediate gates may be incomplete, but they must advance a production-intended application.

The durable end-state contract is in [`docs/product-definition.md`](docs/product-definition.md), including the living-world requirement: simulated actors/processes continue where causally justified, and new rumors, leads, conflicts, opportunities, discoveries, and quest-like situations can emerge from changing world state without requiring scripted campaign content.

This repository starts from a generic engine architecture. Existing Asterra campaign material is research/test input only and must not become production-engine assumptions.

## Current phase

[Gate 3 — first playable desktop table loop](docs/checkpoints/gate-03-desktop-table-loop.md)
is active. The accepted persistence and rules foundations now support durable table sessions,
source-derived character creation, physical dice and exact pending-state recovery. Windows
desktop integration and packaged acceptance are in progress; this is not a finished game.

See the [roadmap](docs/checkpoints/roadmap.md), [rules coverage ledger](docs/rules/rules-coverage-ledger.md)
and [active execution plans](docs/exec-plans/active/) for verified scope and remaining work.

## Software and content licenses

DMd's original software uses the [MIT License](LICENSE), as declared by the Rust
workspace. Licensed rules adaptations retain their separate [SRD attribution and
CC BY 4.0 terms](content/srd-5.2.1/NOTICE.md). Bundled third-party dependencies retain
their own license notices.
