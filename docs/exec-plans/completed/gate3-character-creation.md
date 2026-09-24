# Gate 3 supported character creation and feature primitives

Status: **Completed — Gate 3 technical implementation and native acceptance.**

## Completion and handoff

Foundation PR #20 merged at `ac900fb5c5603f26bd3f3108aecf81bf597eadae`; desktop PR #21
merged at `998eedea92da8aab288e1c586eb4a3e406c74b4c`, matching reviewed/tested head
`05ff272d16ab7d893e0c39950f07bd5176c82e3e`. Full verification passes 207 local Windows/GNU
tests and 208 Linux tests. Windows run `36025008694` passes MSRV/stable checks, native
host/frontend/notice tests and fresh packaging. All 1,068 final artifact checksums and
the dependency/SRD notice audit pass. Independent source review found no remaining blockers.

The packaged native scene covered creation, agreement, character/attendance, questions,
correction, raw dice, ambiguity, graceful/crash restart, duplicate launch, campaign isolation,
session end and new-session continuation. Final build reopened the pending Second Wind roll,
resolved raw 4 + 1 once with 1/2 uses retained, closed/saved/reopened, and started Beyond the
ridge with Alex/Mira, the original agreement, two accepted outcomes and unchanged resources.
The checkpoint distinguishes intermediate-build scene work from final-build continuation.
No manual save edits or developer gameplay bypass was used.

See [Gate 3 acceptance](../../checkpoints/gate-03-desktop-table-loop.md) for the complete
matrix, exact artifact identities, limits and debt. PR #22 records its final reviewed head,
checks and post-merge main verification. Gate 4 is not started. After final main verification,
pause for owner review; the next action is owner-authorized Gate 4 planning, not another
Gate 3 implementation slice. Human playability and reference-hardware/endurance acceptance
remain assigned to their existing later checkpoints.

## Historical execution notes

The implementation notes below preserve intermediate findings and next actions as history.
Their pending integration/build work is superseded by the completion evidence above.


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
