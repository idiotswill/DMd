# Gate 3 supported character creation and feature primitives

Status: active. Sole writer: source/character agent on `codex/gate3-character-creation`, based on `fff6900`.

## Objective and boundaries

Create source-faithful, validated level-1 Human Fighter Soldier characters through a public deterministic rules API for the production desktop/table layer. User choices include standard-array assignment, background boosts, skills, size, languages, style, equipment and mastery grants. Persist source-backed profile/grant records; never accept caller AC, HP or bonuses. Backstory remains unaccepted player narrative.

Gate 3 advances supported creation, recognizable sheets and physical-dice/resource use. Full tactical/mastery/property execution remains Gate 4; complete noncombat/tool/item gameplay Gate 5; complete catalogs Gate 6. No normal-player workflow may require editing state or JSON. Root owns table/application integration; this branch owns creation/domain contracts, feature primitives, source catalog and ADR 023.

## Sources and decisions

Read root AGENTS, Gate 3 checkpoint, product character/table/natural-input requirements and ADRs 017–020. Exact licensed source is the existing SRD 5.2.1 pin: creation pp19–22, Fighter47–48, Soldier83, Human86, feats87–88, equipment89–97. Tough is not an SRD feat. Human extra feat is Skilled with three skill choices; gaming-set tool grant is preserved with explicit later execution scope. Initial equipment uses Fighter155GP + Soldier50GP source alternatives, with bounded source-priced purchases.

## Slices and acceptance

1. Domain profile, pure creation API/catalog, authoritative validation and source/numeric regression tests.
2. Defense/Archery, Second Wind raw d10+level with two uses and short-rest +1/long-rest all recovery, Human long-rest Inspiration with optional persisted transfer choice, and bounded Savage Attacker if safely integrated.
3. ADR, complete local checks and independent root review; root integrates and verifies real table/desktop flows.

Each new feature must retain raw rolls, fail without mutation, replay identically and survive serialization. Legacy mechanics with absent new optional fields retain meaning. Use source-derived positive/negative fixtures rather than snapshot-only assertions.

## Verification and next action

Implemented the public creation/profile contracts and source catalog, authoritative creation validation, selected feature primitives, raw-roll recording and replay tests. Creation can initialize mechanics or add a character without replacing existing entities, roll history or completed short rests. Savage Attacker records both physical damage sets, the player's selection and an optional Inspiration replacement. Duplicate Human Inspiration awaits an explicit controller choice.

Local verification on the coherent implementation: `cargo test -p dmd-rules -p dmd-domain` passed 90 tests (49 domain, 41 rules, including 10 creation/feature regressions); `cargo clippy -p dmd-rules -p dmd-domain --all-targets -- -D warnings`, `cargo fmt --all`, both boundary/genericity guards and `git diff --check` passed. A separate Cargo target directory avoided shared build locks.

Root integration remains next: inherit the table contract's explicit house rules on first character initialization after schema 3 lands, compare installed creation catalog bytes with the embedded source asset, and exercise the production table/desktop/restart flow. The standalone slice intentionally compiles without the parallel table contract. Root will run integrated full verification and publish a coherent PR; this branch does not independently claim Gate 3 acceptance.
