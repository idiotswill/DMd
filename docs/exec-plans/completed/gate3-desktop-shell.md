# Gate 3 desktop shell and packaging slice

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


## Objective and contract

Establish the accepted Tauri 2 + Svelte/TypeScript/Vite production desktop boundary,
Windows release packaging, offline resources and accessible keyboard/text controls.
Follow ADR 001, Gate 3, the root Gate 3 plan, and the product's normal-user access,
offline core, authoritative application boundary and exact-restart requirements.

Own frontend scaffold/configuration, `crates/dmd-desktop`, desktop build/package scripts,
a dedicated Windows workflow and ADR 022. A separate UI agent owns `TableApp.svelte`,
`table-api.ts` and table components; root owns application/domain contracts. No gameplay
fixtures, parallel state or mocked backend count as
acceptance. Until root's typed table service is connected, show an explicit unavailable
status; this scaffold alone cannot satisfy Gate 3.

## Acceptance and slices

1. Bootstrap verified portable Node/npm outside the repository; keep local GNU core
   testing and use Windows MSVC CI for the desktop executable.
2. Build static Svelte assets with type/accessibility checks and focused UI tests.
3. Configure typed IPC, restrictive CSP, packaged content/license resources, a per-user
   NSIS installer with offline WebView2 provisioning, and a portable resource bundle.
4. Record exact commit/build/checksums in artifacts; test the actual packaged application
   using native Windows inspection after the production table service is integrated.

## Decisions and validation

Host audit: bundled Node 24.19.0 lacks npm; WebView2 153.0.4234.48 is installed.
MSVC, Visual Studio installer and Windows SDK were not found in normal locations.
Tauri officially supports the MSVC Windows target. CI builds prevent a heavy system
toolchain installation. Workspace MSRV remains Rust 1.88 and must include the Windows
desktop dependency graph. The `@oai/sky` native UI runtime initialized and enumerated
windows successfully; no desktop acceptance scenario has run yet.

The scaffold now exists: static Svelte status/error/retry screen, narrow status IPC,
Windows-only Tauri configuration, offline NSIS resources, build/package scripts and
Windows stable/MSRV workflow. Rust formatting, PowerShell script parsing, notice-script
JavaScript syntax and diff whitespace checks pass. The first portable Node download
was blocked awaiting network escalation and aborted; root subsequently provisioned
the verified Node 24.19.0/npm 11.17.0 ZIP. npm registry access also fails in this agent's
sandbox, so root supplied dependencies separately. The Node/npm and Cargo lockfiles
are now copied from root's isolated provisioning workspace. The official
`svelteTesting({autoCleanup:false})` Vite plugin selects browser exports while local
test setup performs cleanup; the integrated frontend now passes check (zero
errors/warnings), all 12 tests and the production static build.

The shell branch incorporates the production table runtime through `b9dbf7c` and
exposes the agreed typed commands. Runtime startup uses app-owned `open_local` with
native paths; player authority is derived from the selected persistent character and
controller, preserving the original request's command/head/session metadata. Storage
initialization errors remain recoverable. At exact head `51d2962`, Windows workflow
`36017036314` passed the full native workspace check on Rust 1.88 and stable, desktop
strict Clippy and three native host tests. The release executable and offline NSIS
installer built successfully, but the packaging source guard rejected Tauri's CRLF
to LF rewrite of its manifest. A detached reproduction confirmed the normalized Git
blob was unchanged; a fresh checkout with the narrow LF attribute remains clean
after the same CLI rewrite. The guard remains enabled and now prints diagnostics.
Independent review also found omitted upstream license filename/directory variants;
the collector now preserves those files and declared Cargo license files, with three
copying/fail-closed regressions passing locally. A fresh exact-head package run and
actual packaged UI acceptance remain required. Root's full GNU verification at
`51d2962` passed 207 tests, formatting, check, strict Clippy and both guards.

At `f4f1ad2`, Windows workflow `36020516064` passed both jobs including packaging
and artifact upload; Linux CI `36020516060` also passed. Root verified all 3318
artifact checksums and began native UI acceptance. Independent artifact inspection
found WebView2's published crates omit the upstream MIT text and noticed repeated
collection copied the desktop crate's generated output into itself. The followup
adds exact archive/revision/license/hash-pinned upstream text for the three crates,
clears only the validated generated output directory, excludes that subtree from
collection, and tests copying, repetition, safe output cleanup and changed pins.
The corrected final artifact still requires fresh CI and notice inspection.

Verification planned: frontend `npm ci`, check/test/build; existing
fast/full checks as applicable; Windows MSRV/check/release/package CI; artifact hash
verification and real executable launch/restart. Failures and unexecuted checks must
remain explicit. No implementation success is claimed at plan creation.

## Risks and next action

The actual Windows MSRV and release compilation now pass. Artifact packaging after
the source-cleanliness fix, local materialization and real packaged-app acceptance
remain unverified. Next: rerun PR 21's Windows workflow on the corrected head and
hand the verified artifact to root for native launch, table play and restart tests.
