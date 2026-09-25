# Gate 4 physical equipment through the desktop table

Writer: root. Next bounded branch: `codex/gate4-equipment-table`, created from refreshed
main after verified PR28 merges. The integrated source checkpoint is `56a8d35`.

## Objective and scope

Ship the already-tested physical starting-equipment path through the actual desktop:
one durable grant per character, current carried items/armor on the sheet, all-source
Fighter mastery choices and exact uncertain-request retry. Attach only tactical inventory
to RulesState with source validation and provenance. This is a production path slice,
not a new attack scheduler or a claim that all Gate4 encounters are playable.

Root AGENTS, the product's current-equipment, source fidelity, authority and recovery
requirements, Gate4, ADR024 and source-pinned SRD5.2.1 govern this work. Preserve the
existing starter shop and source-created character profile as immutable grant evidence.
Do not copy unrelated encounter/casting/creature/effect/recovery authority into this PR.

## Required extraction and verification

1. Add the optional/default/omitted-when-absent inventory field, structural/source
   validation and creation defaults. Reject legacy rules actions that could bypass
   physical custody; retain the existing supported character/check/Second Wind path.
2. Extend codec, database migration preflight and restore audit/anchor checks for this
   recognized authority. Typed preflight must reject duplicate inventory fields even
   when a second null could hide the first. Preserve atomic all-chain migration and
   historical event/snapshot bytes; no historical SQL migration is rewritten.
3. Extract PrepareEquipment, table equipment views and current-armor validation. Keep
   host/session/pending-decision checks, no duplicate grant and exact ItemId retention.
4. Extract sheet/form controls, actual mastery choices, saved-request whitelist and
   uncertain retry coverage without copying tactical maps/actions into this slice.
5. Run real SQLite create/retry/reopen/export/restore and forged-origin/anchor tests,
   schema compatibility, UI checks/tests/build, full canonical verification, independent
   complete-diff review and final-head Linux/Windows checks. Merge with expected head,
   verify the fetched merged tree/post-merge CI and reconcile the integration branch.

## Evidence and next action

Parent `56a8d35` retains the passing59 application tests plus the independent reaction
projection regression and strict workspace Clippy. Its earlier frontend checks include
equipment retry and source mastery selection. This is navigation evidence only: the
new extracted branch must compile, run its own required checks and receive full review.

PR27 is merged with all post-merge checks green. PR28 final evidence head is awaiting
stable Windows packaging; no dependent extraction has started. Next: merge verified
PR28, refresh main, create the branch, copy this plan and extract the listed boundaries.
Shared casting/movement authors retain their branches and Rust compilation is serialized.
