# Gate 3 — First playable desktop table loop

Status: **Accepted — scoped Gate 3 technical acceptance, 2026-09-24. Owner pause before Gate 4.**

## Product requirements advanced

- normal-user campaign/session flow;
- Session Zero/table contract;
- supported character creation/selection;
- text-first table interaction;
- physical dice reporting;
- transcript/recap;
- exact session start/end/restart;
- rules-question/correction path.

## Objective

Turn the durable rules/runtime foundation into the first application a human can actually use for tabletop-style play, still text-first and visually simple.

## Entry conditions

Gate 2 accepted. Desktop shell/application boundary can consume runnable campaigns without bypassing validation.

## Acceptance criteria

A normal user can:
- start the desktop application;
- create/list/open a campaign;
- establish the table contract;
- create/select supported PCs;
- bind attending players to PCs;
- start a session;
- enter ordinary text declarations and questions;
- receive rules outcomes and roll requests;
- report physical die faces;
- see a transcript and player-safe recap;
- correct a misinterpreted but uncommitted declaration;
- end/save, quit, restart and continue;
- pause at an unresolved material decision and resume without inventing resolution.

## Architecture invariants

All gameplay-facing access routes through the application/runnable boundary. UI never writes authoritative state directly. Manual/text input constructs the same typed proposals/commands eventual voice will use.

## Explicit non-goals

Voice, polished visuals, full tactical combat, living-world simulation, companion phone apps.

## Required failure/recovery behavior

Application/provider/UI crashes or malformed inputs do not corrupt authoritative state. Resume distinguishes accepted from pending work. A normal user receives recoverable errors rather than database/Git instructions.

## Merge/pause boundaries

Codex may merge in-scope Gate 3 work autonomously after exact-head verification. Product/table-contract semantics must remain consistent with the accepted product definition; do not narrow player agency or recovery guarantees to make acceptance easier. Pause at the end of Gate 3 with evidence and the proposed Gate 4 handoff.
## Candidate workstreams

Desktop shell, campaign/session UI, Session Zero, player/PC binding, text input, transcript, roll UI, recaps, correction UX, session lifecycle/recovery.

## Deferred requirements

Full tactical encounters Gate 4; noncombat breadth Gate 5; autonomous intent interpretation Gate 9; voice Gate 10.

## Open questions

Exact UI visual design remains flexible. Determine minimum accessible keyboard/text-only experience.

## Production integration acceptance

Run a short real tabletop-style scene through the packaged/production desktop path without developer tools. Save/restart mid-session and continue with correct transcript, player/PC bindings, resources and unresolved decisions.

## Accepted implementation and exact evidence

[PR #20](https://github.com/idiotswill/DMd/pull/20) merged the foundation at
`ac900fb5c5603f26bd3f3108aecf81bf597eadae`, with the same tree as reviewed head
`52ffab23c5e67db7d6ee40a622533b08d44f10fd`. Full local verification passed 207
Windows/GNU tests. [Linux CI318](https://github.com/idiotswill/DMd/actions/runs/36016090350)
passed 208 tests, Rust 1.88, formatting, strict Clippy and guards.

[PR #21](https://github.com/idiotswill/DMd/pull/21) merged the Windows desktop at
`998eedea92da8aab288e1c586eb4a3e406c74b4c`, with the same tree as reviewed head
`05ff272d16ab7d893e0c39950f07bd5176c82e3e`. That head passed full local verification
(207 tests, formatting, compilation, strict Clippy and guards), independent complete-diff
review, [Linux CI329](https://github.com/idiotswill/DMd/actions/runs/36025008608)
(208 tests and Rust 1.88) and [Windows run10](https://github.com/idiotswill/DMd/actions/runs/36025008694).
The Windows run passed MSVC 1.88/stable checks, strict desktop Clippy, 3 native host tests,
12 frontend tests, 7 notice regressions, zero Svelte errors/warnings, release compilation,
offline NSIS construction, the clean-source guard and artifact upload.

[PR #22](https://github.com/idiotswill/DMd/pull/22) records the exact closeout head and
post-merge main verification. It strengthens Cargo dependency-boundary checks, reconciles
coverage/ADRs/debt and archives the completed plans. The final merged main SHA and its CI
runs are recorded there after merge, avoiding a self-referential commit identifier here.

### Packaged production-path scenario

Native Windows controls exercised the real Tauri executable, embedded frontend and local
SQLite/content runtime. No developer tools, API mocks, internal-state editors or manual
save repairs were used. Installed WebView2 was 153.0.4234.48.

| Acceptance | Observed result |
|---|---|
| Start, create/list/open campaigns | Portable release started normally; Ridge Crossing QA and Harbor QA were created/listed/opened through the UI. The second campaign had no inherited characters, scene, transcript or recap. Switching back retained the first campaign's pending work. |
| Table contract | All agreement fields were present. A custom tone persisted through session end and restart; optional natural-extremes mechanics remained explicitly unchecked. |
| Supported character and attendance | Created Alex and Mira, a Human Fighter 1 Soldier with Skilled. Purchased/wore Leather Armor; the source-derived sheet showed HP 11/11, AC 14, proficiency +2, speed 30 and Second Wind 2/2. Started Across the ridge with Alex present and bound to Mira. |
| Text questions and corrections | A character question returned saved HP/AC without spending resources or adding a recap outcome. Correcting an uncommitted Second Wind declaration to a climb retained 2/2 uses and replaced only the pending intent. |
| Physical dice and outcomes | Host established a generic Strength (Athletics) DC 15 challenge. The player view hid DC and unrevealed consequences. Native validation rejected d20 face 21; raw 12 received the authoritative +5 modifier and recorded one total-17 success in transcript/recap. |
| Unresolved decisions | “I climb or drink a potion” stayed unresolved across normal quit/restart with the same binding, history and resources. Explicit withdrawal reported no world outcome. |
| Crash and duplicate launch | With Second Wind pending and one use reserved, a second launch focused the existing window. Forced termination of only the tested app and normal relaunch preserved the d10 request, binding and 1/2 resource count. |
| Final package continuation | The final build reopened that pending roll with the readable “Second Wind healing” label. Raw 4 plus level 1 resolved once as 5, with 0 actual HP regained because HP was already full; uses remained 1/2. |
| End/save/reopen/continue | End and save closed the session. Reopen showed Between sessions, the two accepted outcomes, original agreement and unchanged sheet. Starting Beyond the ridge with Alex/Mira retained history, situation and resources without replaying either outcome. |
| Basic usability | Labeled forms, visible focus, keyboard selection/submission and recoverable invalid dice were exercised at the normal 1162×831 window size. No database or Git instructions were required. |

Creation, the ability check, ambiguity/restart, crash and campaign-isolation steps used
intermediate packaged head `f4f1ad2e2049aecbc2323252d290dfbf2893de02` (Windows run7,
artifact 10816249118; all 3,318 hashes verified). Artifact inspection and native use found
missing upstream notice files, recursive notice copying and debug-style roll labels. These
were fixed and independently reviewed. The final candidate differs in packaging/notices and
display-only roll wording; saved gameplay semantics are unchanged. It then completed the
pending-roll and session end/restart/continuation steps above. No failed build is counted as
acceptance evidence; the completed integration plan retains the failure investigation.

### Final artifact identity and notice audit

Artifact **10819373695**, built from exact `05ff272d16ab7d893e0c39950f07bd5176c82e3e`,
targets Windows x64 MSVC release with Rust 1.98.1 and Node 24.19.0.

| File | SHA-256 |
|---|---|
| Artifact ZIP | `b2d6928facdab609545007f476189a9b8691b37e25f37b5454bf2a16e6efacec` |
| Portable DMd.exe | `7eaa1b2faa95bd7d52dccd613ffa72cb2c7ef5929fc36d3f6cb6316c38b1a01d` |
| DMd_0.1.0_x64-setup.exe | `499cade1b24fe572b0e022dcd20d7c000878048a44aaa715f36d8473613e2bf6` |

All **1,068** packaged file hashes passed. Independent inspection found 691 dependency
records (550 Rust, 141 npm) and 1,057 unique notice references, all present and nonempty,
with no recursive trees or unindexed files. All three pinned WebView2 MIT supplements,
previously reviewed Tauri/whoami/crc/TypeScript notices, the DMd MIT license and SRD source
pin/CC BY attribution were present. Signing and the full release-distribution audit remain
Gate 13; this is a development package, not a production release declaration.

## Architecture, coverage and remaining obligations

ADRs 021–024 are accepted: schema-3 bounded current table state with separate observation/
session history; atomic table/rules/session transitions and semantic replay; source-derived
character creation; typed outer desktop IPC with trusted local identity and durable original
requests. Player projections filter private and unrevealed host data. Single-instance startup
protects the local retry slot. The shared computer deliberately trusts channel selection;
remote authentication/private companion devices remain Gate 12.

Only the scoped `play-rhythm` and `character-creation` ledger rows advance: 15 families now
have production integration evidence, 40 remain deferred and no human player acceptance is
claimed. Full tactical/mastery/visibility behavior belongs to Gate 4, noncombat breadth and
progression to Gate 5, complete catalogs/templates/adventure to Gate 6, broad autonomous
interpretation/DM behavior to Gate 9 and voice to Gate 10. Session Zero policy notes do not
add unsupported mechanics; backstory remains unaccepted narrative input.

TD-001 remains open for future event families; TD-004 is mitigated for the desktop and open
for future surfaces. TD-008's creation acknowledgement risk is mitigated by identity-based
reconciliation, with raw lifecycle recovery still Gate 13. TD-009 records full-history reads
behind bounded transcript/recap presentation. Existing growth, projection, content-loading
and snapshot-policy debt retains Gates 7/13 measurements and Gate 14 endurance ownership.

No scoped Gate 3 acceptance criterion is waived or outstanding after final closeout checks.
The NSIS installer was built and hashed, not installed on a pristine machine. No network
adapter was disabled, and no air-gapped provisioning, reference-laptop performance, four-human
enjoyment or endurance result is claimed. Clean-machine installation/signing/release work
belongs to Gate 13; reference-hardware acceptance to Gates 10/13/14; mandatory four-human
vertical-slice play follows Gate 6 and sustained endurance remains Gate 14.

## Owner pause and next gate

Pause after final merged-main verification. Gate 4 has not started. If authorized, its
first workstreams are spatial/tactical truth, turn/reaction/effect ordering, visibility and
non-omniscient enemy behavior, all integrated with natural declarations and exact mid-combat
suspension through this desktop path.
