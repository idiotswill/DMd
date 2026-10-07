# Timed expiry native evidence — 2026-10-06

Status: primary native path A passed on source
`391c571fdc488ddaa96292a5d145c2ba641b8d15`, tree
`5b34cdd6aa10154b425038fd2e72df3172f35d7a`. This is bounded PR52 evidence;
Gate4, final reviewed-head checks and merged-main verification remain open.

## Full source verification

Exact391 passed [Linux CI](https://github.com/idiotswill/DMd/actions/runs/37436120537)
and [Windows desktop CI](https://github.com/idiotswill/DMd/actions/runs/37436120513).
Actual logs report808 Linux and810 native Windows Rust passes, zero failed,
ignored or filtered tests, including the real source deadline/restore case.
Linux's synthetic checkout5f73a2465902e45c00221997284724d0d6ee77f0 has exactly
391's tree; Windows checked out literal391. Windows Rust1.99.0 stable also passed
126 frontend tests, zero static diagnostics, notice checks and canonical package
creation. Both platforms passed the declared Rust1.88 minimum check.

Normalized Linux job112178227390 log SHA256 is
`9505e036b80b5eaddbd4490662788fb98fbb75e927f69a12b6fc1aa58ef9dfc5`;
Windows stable112178227674 log is
`05249daa0bcb8ff2681ccdac0e71f474106a167459554e7b6660bb1d2a3674a9`.
The Windows CI artifact was uploaded but not downloaded or used for this native
run. Native evidence below is separately bound to the local Rust1.98.1 package.
Final documentation-head CI and literal merged-main verification remain due.

## Executable and store

The unchanged canonical MSVC desktop packaging command completed locally with
Rust1.98.1 and Node24.19.0: 126 frontend tests, zero static diagnostics and1071
verified manifest payloads. Portable EXE SHA256 is
`be5dbd3beca8505562596812ee96e494212eb7f03c2a68ff14d75b0e1a3dc5e7`;
installer `5e6168718c37cf0425f59eb6db688473db2c49b9c7a5420d128a69db292013e8`.
Completed package provenance SHA256 is
`de1c79af738a7b1deee7248ca944e294686967746fb023d2934113b626f05d7f`;
the complete payload verification report is
`6814603225871941fdbb19f8d2c4de4b48cdcab622512e65ccafc8178aa4da23`.
The incomplete PowerShell transcript and first failed package-verifier attempt
remain preserved. The passing verifier consumed recovered actual build-tool
output, not reconstructed or invented output. The CI compiler version is separate.

All packages share the existing app-data store. Before switching, the real eleven
migration checksums and schema4 headers were checked read-only against the actual
verified binaries. Both Windows builds embed CRLF migration checksums; repository
LF checksums differ. No database checksum was changed. Cross-platform migration
checksum stability remains release/recovery debt and is not proved by this run.

## Real UI path

All creation, source selection, session control, initiative, casting, turns, dice
and expiry decisions used shipped native controls. No application API, database
write, authored clock, injected identity or browser-storage edit created evidence.
QA campaign is `e48574bc-4d06-4d5e-987e-66e7d67b2590` (Expiry391 deadline QA
Oct06 A); active source-only session is `6563cbbd-73d0-4a35-8ac2-05c7bc26604b`.
Two attending players each had normally initialized PCs outside the battlefield;
a third player was absent. The actual creation-form PC choices varied from the
prepared fixture and were recorded before setup; they did not supply combatants.

The four original Medium source actors were two Night Hags (Alpha), Mage and
Cultist Fanatic (Beta). Genuine pins were respectively `90405f1404e04648`,
`af0f81ba7833b9c4`, `719b3c883ae1fcec`. The current picker supplied all pins,
materials and item identities. Initiative reports A20/B18/Mage5/Cultist3 produced
25/23/7/5, with B and Cultist surprised. The source Cultist cast Hold Person once
on the Mage, using the real straight-iron material and sole source feature use.

Cast command `ede8c50d-61fd-4943-b60d-959408ed17a5` led to the owned initial
physical save1+4=5. Paid sequence28/time0 retained group
`d1640afc-f02e-5ad3-8632-c3883c31ec63`, target effect
`ab061f4b-0092-5bd9-af5d-271dbcfbad47`, both AtTime60, and Mage Paralyzed.
Nine further physical saves1+4=5 occurred at times6,12,18,24,30,36,42,48,54 through
normal turns. There were40 accepted End-turn commands from initiative completion,
37 after casting. The five paid-cut raw rolls survive exactly; precisely nine
repeat rolls were added. No extra roll was generated at expiry.

At settled sequence73/time54, Cultist End
`65c18c36-24e9-4450-9009-982245afe303` advanced once to sequence74/time60,
round11/Hag A Start41. The two pending tickets share that command/step2 and Start
cause: ordinal0 expires the concentration group; ordinal1 expires the target.
Alpha/Hag A alone saw the two generic ordering buttons. Host, absent Observer,
Beta/Mage, Beta/Cultist, Alpha/no actor and Alpha/Hag B had no ordering override.
Earlier repeat-save privacy checks likewise withheld Mage's form from other actors.

The first offered button was clicked once. Accepted command
`16f384aa-8154-4c74-b4f3-80ceb59d3918` resolved opaque handle
`418273f9-9875-4bea-a68e-91ff9df75a9f` to occurrence0 and the actual group-expiry
ticket. Sequence75 remains time60/Hag A/round11. Both tickets, group, target effect,
concentration and Paralyzed are gone; no second ordering choice remained. Complete
entity records match the paid state except the expected concentration removal.
Inventory/materials and source profiles match exactly; the source use remains
spent1. Complete source runtime matches after excluding only verified turn and
operation cursors. No use refund, HP change or extra clock advance occurred.

## Cold recovery and preservation

Five normal close/exit/reopen triples cover initial unanswered save, final unanswered
repeat, settled54, due60 and completed60. Full typed rows in all28 tables of all
three captured campaigns, state bytes, schema metadata and migration rows compare
equal across each triple. Screenshots independently showed the same blank pending
inputs and the two due choices. Verified391 process identities after successive
reopens were PID12100 (13:25:35.341481+02),6232 (14:06:56.6040037+02),7156
(14:09:07.8017631+02),13656 (14:12:32.0948680+02),20780 (14:14:15.7490403+02).
Each close used Alt-F4, followed by observed window/process absence; every launch
followed a fresh executable hash check.

The protected bridge `f5da4b10-50f8-47ef-9ad9-e0a921ad0bdc` and existing campaign
`a2858187-20b5-45da-9062-2a9c1d2d7b46` match their initial captures completely.
Finally391 selected the bridge as Host and exited. After another successful actual
migration/header preflight, verified older f9 EXE
`c852e0f6bf35fae8a52759d372b11de244035413a0420ea277c16bcf4f1a1f20` reopened that
remembered bridge without retry/error (PID20064,14:20:00.4022621+02), then exited
normally. Expiry QA was never opened in f9. All three full logical campaigns remain
equal to completed75 before/after this bridge trip.

External evidence is retained under workspace `tooling/native-expiry391-oct06/`.
The read-only capture helper uses SQLite URI mode=ro, query_only and one BEGIN
snapshot. Thirty complete captures and174 immutable-history prefix comparisons
were audited by `tooling/audit-native-expiry391-oct06.py`; report
`tooling/native-expiry391-oct06-audit.json` SHA256 is
`289bb040deff1230d2233dd5473ec17da2930f80578ca32d9eed01093421cd4d`.
Prefixes compare complete rows by real primary keys, not UUID-sort order or counts.
The final f9 captures retain the helper's391 package-file annotation; that annotation
does not identify the running process. Actual f9 identity is recorded above.

Initial audit drafts made incorrect assumptions about omitted resolution, nullable
pending roll and legitimate creature turn cursors. Actual complete values were
inspected; corrected assertions remain strict and the failed drafts are not product
failures. No capture was edited to make equality pass.

This is normal native cold recovery, not forced-crash, native portable export,
arbitrary native accepted-command replay, all-store backup or human enjoyment
evidence. Those unavailable native controls were not fabricated. The separate
application test covers exact retry and portable restore. Complementary native
path B was not run. The source rules case covers both End-turn and final-raw
crossings; synthetic lifecycle tests cover opposite work orderings. The application
deadline/restore test follows primary A. These remain separately attributed.
