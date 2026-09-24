# Gate 3 desktop shell and packaging slice

Status: active. Sole writer: desktop shell agent on `codex/gate3-desktop-shell`,
based on `fff6900` (Gate 3 plan following accepted main `397f5bb`).

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

Verification planned: frontend `npm ci`, check/test/build; existing
fast/full checks as applicable; Windows MSRV/check/release/package CI; artifact hash
verification and real executable launch/restart. Failures and unexecuted checks must
remain explicit. No implementation success is claimed at plan creation.

## Risks and next action

The actual Windows MSRV and release compilation now pass. Artifact packaging after
the source-cleanliness fix, local materialization and real packaged-app acceptance
remain unverified. Next: rerun PR 21's Windows workflow on the corrected head and
hand the verified artifact to root for native launch, table play and restart tests.
