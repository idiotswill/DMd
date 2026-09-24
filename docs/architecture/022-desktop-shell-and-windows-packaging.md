# ADR 022 — Desktop shell and Windows packaging

Status: proposed for the owner-authorized Gate 3 desktop slice.

## Decision

Retain ADR 001's Tauri 2, Svelte, TypeScript and Vite stack. The desktop crate is
an outer adapter over `dmd-app`; it must not write SQLite or authoritative domain
state itself. Narrow typed commands delegate to the production table service.
The host constructs trusted issuer/player/session metadata from explicit local
channel selection, never from prose or a renderer-provided rules context.

Static frontend assets are embedded in the release executable. No Node process,
development server, cloud connection or browser installation is required for play.
The package includes the exact pinned rules directory and license notices. Writable
campaign data belongs in the OS application-data directory, outside installed assets.
The frontend receives player-safe projections, never raw campaign state or secret
host adjudication context in player mode. A restrictive CSP denies remote resources;
no general filesystem, shell, SQL or HTTP plugin is exposed to the renderer.
The native single-instance plugin registers first. A second launch focuses the existing
window and exits before opening another renderer that could overwrite its saved request.

## Build and package

The current supported desktop target is `x86_64-pc-windows-msvc`. Windows dependencies
are target-gated so the existing Linux/GNU core verification does not require GTK or
pretend to build the Windows desktop. Unsupported targets produce an explicit error
when the shell executable is run; their core crates remain supported independently.

Windows CI checks the entire graph with Rust 1.88, the workspace MSRV, and stable.
Stable builds the release executable and a per-user NSIS installer. The installer
embeds offline WebView2 provisioning; a supplemental portable directory requires
an existing WebView2 runtime. The host development environment can retain GNU Rust
for core tests and portable Node/npm for frontend work without installing MSVC.

Artifacts include the exact commit/build target, installer, executable/resources,
dependency license declarations and available notice files, and SHA-256 checksums.
The Windows workflow checks out the exact PR head, not a moving branch. CI downloads
are development/distribution infrastructure; they are not runtime dependencies.
Packaging always performs a fresh release build from a clean source tree and checks
the commit and source cleanliness again afterward. It never stamps a new head onto
executable/installer files left by an earlier build. SHA-256 hashes cover the copied
executable, installer and resources together.
Publisher signing and broader installer/release hardening remain Gate 13 work.

## Verification and limits

Frontend checks/tests test rendering and recovery behavior; they are not tabletop
acceptance. Startup initializes the production table service before mounting the table
UI, with a recoverable startup error otherwise. Typed commands include campaign and
contract setup, source character choices, sessions, player text and physical dice.
No placeholder screen or mocked backend is claimed as a usable table loop.

Run the packaged executable through its normal UI, including keyboard navigation,
labels/errors, offline campaign creation, physical dice, quit/restart and exact pending
decision continuation. Native Windows inspection through the computer-use skill is
available on this host. WebDriver may drive the same production executable for CI;
an API mock or Vite-only page is insufficient evidence. Record artifact commit/hash
and actual visual/interaction evidence before claiming the gate complete.

## Sources

- [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Tauri installer and offline WebView2 configuration](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri Vite static build configuration](https://v2.tauri.app/start/frontend/vite/)
- [Tauri packaged-application WebDriver setup](https://v2.tauri.app/develop/tests/webdriver/manual-setup/)
- [Tauri single-instance initialization and focus](https://v2.tauri.app/plugin/single-instance/)
