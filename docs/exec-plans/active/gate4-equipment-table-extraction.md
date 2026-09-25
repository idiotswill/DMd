# Gate 4 physical equipment through the desktop table

Writer: root. Next bounded branch: `codex/gate4-equipment-table`, created from refreshed
main after verified PR28 merges. The integrated source checkpoint is `56a8d35`.
PR: https://github.com/idiotswill/DMd/pull/29 (draft).

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

PR27 and PR28 are merged with all six post-merge checks green. PR28 merged main is
`d12b2a68b82592cc59622062d6ab1668defa20ed`; its full tree exactly matches reviewed final
head `1c5ebb9bf0268e565412804a7a6a1271be2eb3be`. Post-merge CI36103736797 and Windows
36103736775 passed, including the offline installer build.

The branch is based on that refreshed main. The inventory attachment, host command,
source validation, private views, UI controls and exact uncertain retry are extracted.
Pending-work checks conservatively require equipment before battlefield setup because
this bounded main-based slice does not yet attach the tactical flow. Schema regressions
cover nonempty authority and both duplicate/null-shadow orders. The application regression
now closes/reopens a physical SQLite file before retry and rejects a replacement anchor.
Fresh independent complete-diff review and branch-specific checks are pending. Rust stays
serialized behind shared casting and movement. Next: complete review, run application and
schema tests plus frontend checks/build, then canonical verification and exact-head CI.

The first extracted head `63e6351` compiled and passed MSRV, architecture and genericity
CI, but Linux run36104674881 failed the real equipment export/restore regression:
the origin collector knew about inventory commands while the audit validator still
required a nested RulesEvent. The correction recognizes only the genuine typed
PrepareEquipment table action; all metadata/audit and semantic replay checks remain.
A real unrelated AddPlayer command is now also tested as a forged grant origin.
The UI retry regression begins with the real preparation button and preserves its
generated identities across uncertain delivery/restart, instead of seeding a request.
The original extracted UI passed13 tests, zero diagnostics and production build;
the strengthened13-test rerun now passes. Its first run exposed missing saved-campaign
selection in the new fixture and retained mock responses leaking after that failure;
the fixture now selects its campaign and resets mocks between tests. Production UI
bytes remain those of the passing build. Corrected Rust/canonical evidence is pending.
