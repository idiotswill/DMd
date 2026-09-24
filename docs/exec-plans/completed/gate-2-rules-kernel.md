# Execution plan — Gate 2 rules kernel

Status: **Implementation complete — merged and verified; gate closeout records final publication.**

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

The implementation slices are complete. The portable Rust GNU toolchain is installed outside the repository under `../tooling`. Baseline full verification passed after correcting Windows-only unused test helpers and reducing a platform-dependent large error variant. Shell guards now fail closed if their required host utilities are missing; an empty-PATH negative run exited unsuccessfully as intended. Content text uses pinned LF bytes for cross-platform manifest integrity.

Source/inventory [PR #16](https://github.com/idiotswill/DMd/pull/16) merged as `a0fb8762f3256e384c9bd1ee1e20b0d4acd4d4c3` after final full/delta review and [CI #300](https://github.com/idiotswill/DMd/actions/runs/35998593520) passed on `bbf4d78f052bb9d6ec35553dc5a020ea1021defd`. It accounts for 55 rules families and the source catalogs with explicit gate ownership.

Schema 2 compatibility includes five migration/restore regressions and landed atomically with typed rules state. Independent review fixed authority, retained request derivation, rest interruption, condition interactions and effect timing. Final foundation `ca54b8215258468db4fbac7b5c15a648fdbb98d7` passed full local verification (151 Windows tests) and [CI #303](https://github.com/idiotswill/DMd/actions/runs/36002412992), then merged in PR #17 as `f3222e72b970708517965306dc5bfffbe74eb414`. Final regression inputs prove adjacent Blinded/ranged-threat and separate Invisible/Incapacitated initiative behavior.

Application review fixed historical-lineage bypass and post-commit content rereads. Final application `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c` passed 15 runtime, six restore-helper and eight existing app tests within 172 Windows workspace tests; [CI #305](https://github.com/idiotswill/DMd/actions/runs/36003158276) passed all jobs. PR #18 merged as `3ad3885559073e6748d3f17c2172be9ff2a99f52`, which also passed full local verification. Earlier foundation CI #302 formatting and local Clippy failures were inspected and fixed; failed runs are not acceptance evidence. The checkpoint maps every criterion and records TD-001–008 and later-gate limits. No owner blocker remains.

## Exact next action

Follow the Gate 2 closeout plan/checkpoint for final exact-head publication and merged-main verification. At gate completion pause for the owner; do not begin Gate 3 until told to continue.
