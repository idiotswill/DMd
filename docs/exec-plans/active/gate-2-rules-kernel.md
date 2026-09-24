# Execution plan — Gate 2 rules kernel

Status: **Active**

## Objective and baseline

Deliver the commercial fifth-edition rules kernel through `dmd-app::CampaignRuntime`, with deterministic adjudication, durable roll requests/results and replay. Gate 2 ends only after integrated acceptance and exact-head verification; do not begin Gate 3.

- Baseline: `414040b33d58701cec81e6d73b791347c6d43ca1`, roadmap bootstrap PR #15 merged after exact-head CI #297 on `5a6aab55aa83a156c48feeab45a9c634f9b78f78`.
- Foundation: `codex/gate2-rules-foundation`, [PR #17](https://github.com/idiotswill/DMd/pull/17). Application integration: `codex/gate2-rules-runtime`. Source/ledger slice used isolated `codex/gate2-source-ledger` with a separate writer.
- Source decision verified 2026-09-24: official English SRD 5.2.1, published 2025-05-01, Creative Commons Attribution 4.0. Official landing page https://www.dndbeyond.com/srd identifies it as the current release. PDF https://media.dndbeyond.com/compendium-images/srd/5.2/SRD_CC_v5.2.1.pdf, SHA256 `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`. The source/provenance slice records attribution and the complete source-derived inventory before content ships.

## Scope and non-goals

Implement every Gate 2 checkpoint primitive: validated derived character mechanics; checks/saves/proficiency/advantage; attacks/AC/damage/healing; conditions/effects/resources/rest; timing/reactions/concentration/spellcasting foundations; actual physical/digital dice boundary; read-only queries; explicit house rules and ruling provenance; passive/secret resolution. Persist pending and accepted mechanics atomically with versioned events and replay; reject unauthorized, stale, impossible and incompatible input without writes.

Full spatial encounters (Gate 4), full noncombat systems (Gate 5), complete content catalogs/world templates (Gate 6), desktop UX (Gate 3), autonomous language/DM (Gate 9), and speech (Gate 10) remain deferred to their owning checkpoints. Primitives do not claim those capabilities complete.

## Relevant contract

`docs/product-definition.md`: commercial content, rules fidelity/coverage, local authority, player agency, rules questions are not actions, uncertainty, exact suspension/restart and recovery. `docs/checkpoints/gate-02-rules-kernel.md` and `requirements-traceability.md` define acceptance. ADRs 002, 004, 005, 009–012, 014–016 define state/content/command/dice/atomic persistence/replay/application boundaries.

## Planned slices

1. Pin licensed source, attribution and machine-readable Rules Coverage Ledger with explicit gate ownership and completeness validation.
2. Add typed mechanical state and deterministic kernel; document versioning/compatibility and adversarial tests.
3. Connect typed actions, requests/results and queries to runnable campaigns, atomic persistence and versioned replay. Prove restart, invalid-input non-mutation, content mismatch and campaign isolation.
4. Integrated review, full verification, evidence/checkpoint/ledger/debt updates, archive plan and pause for owner.

Each slice receives complete diff review and exact-head green CI before expected-head-protected merge. Refresh main before dependent integration.

## Acceptance evidence required

- No unassigned source family; all shipped content has pinned licensing and adaptation attribution.
- Mechanics are computed from validated authoritative inputs with no provider dependency.
- Real app campaign actions use the existing dice records and durable command issuer provenance.
- Pending rolls/effects/resources survive reopen; replay matches persisted state; invalid/stale/unauthorized/content failures leave state/journal unchanged.
- Rules queries and failed explanations cannot commit actions or expose secret results to players.
- `./scripts/verify-fast`, `./scripts/verify`, CI/MSRV and independent full-diff review pass on final heads; final merged main verified.

## Decisions and inherited debt

- Keep `dmd-persistence` content/rules agnostic and compose gameplay in `dmd-app`; inner rules code has no SQL/provider dependency.
- Extend the production snapshot/atomic journal path rather than adding parallel rules storage.
- TD-001 requires typed replay for every new rules event. TD-004 requires gameplay entrypoints through runnable validation. TD-005 catalog revalidation remains correctness-first. TD-002/006 performance tuning remains Gates 7/13/14.
- Optional rules are disabled unless recorded as explicit campaign configuration; no implicit model ruling becomes authority.

## Validation / risks

No Gate 2 completion claim yet. The portable Rust GNU toolchain is installed outside the repository under `../tooling`. Baseline full verification passed after correcting Windows-only unused test helpers and reducing a platform-dependent large error variant. Shell guards now fail closed if their required host utilities are missing; an empty-PATH negative run exited unsuccessfully as intended. Content text uses pinned LF bytes for cross-platform manifest integrity.

Source/inventory [PR #16](https://github.com/idiotswill/DMd/pull/16) merged as `a0fb8762f3256e384c9bd1ee1e20b0d4acd4d4c3` after final full/delta review and [CI #300](https://github.com/idiotswill/DMd/actions/runs/35998593520) passed on `bbf4d78f052bb9d6ec35553dc5a020ea1021defd`. It accounts for 55 rules families and the source catalogs with explicit gate ownership.

Schema 2 compatibility passed 119 workspace tests before integration (including five new migration/restore regressions). Its typed rules field is now integrated atomically on the foundation branch. The independent migration review found no blocking defect. Kernel/source review produced fixes for authority, retained request derivation, rest interruption, condition interactions and effect timing. Full foundation verification on `50a86fe4be47a8db7ab3339e953b7aa6fdcfff30` passed formatting, workspace checks, all-target Clippy, 151 Windows tests and both architecture guards. The subsequent test/docs-only `7906b06` strengthens adjacent Blinded target/ranged-threat and independent Invisible/Incapacitated initiative regressions and clarifies Gate 3/4/5/6 ownership. Final-head verification and CI remain required before merge.

The integrated app run passed 13 real runtime scenarios, six restore-preflight unit tests and all eight existing runnable-campaign tests before the final review fixes. Review identified two application failure boundaries: relabeled current/event data must not bypass historical rules preflight, and create/restore must reuse validated content after commit rather than report a post-commit file-read failure. Isolated fixes and regression tests are in progress. Foundation CI #302 on `3c16455` passed MSRV and guards but failed formatting of the newly added bonus-action primitive; its actual diff output was inspected. Local full verification also identified three Clippy collapsible-if failures. Both are corrected in the reviewed follow-up and the full run above; neither failed run is counted as acceptance. No owner decision blocker is currently known.

## Exact next action

Run final foundation verification and CI, then merge PR #17 after exact-head review. Refresh main and integrate the isolated application review fixes; verify the complete production scenarios before ledger/checkpoint closeout. Keep this plan current as evidence changes.
