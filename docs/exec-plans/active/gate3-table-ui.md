# Gate 3 desktop gameplay interface

Status: active. Sole UI writer on `codex/gate3-table-ui`, based on `2354c9f`.

## Objective and scope

Implement normal-user campaign setup, full Session Zero agreement, source-derived supported character creation, attendance/session bindings, generic host-established situations, ordinary player text, corrections, raw physical dice, sheets, transcript and recap in Svelte 5. The root application service remains authoritative. UI storage retains only selection and unconfirmed request identities for exact retry after restart. Product requirements and Gate 3 acceptance require real production-path use; UI mocks are verification tools only.

Own only `apps/desktop/src/TableApp.svelte`, `table-api.ts`, `components/**`, adjacent UI tests and this plan. The shell agent owns App/scaffold/package/configuration/desktop Rust/CI. Root owns application DTOs and integration. Do not bypass runtime validation, invent outcomes, duplicate source prices, expose raw JSON forms, or portray deferred tactical/autonomous features as working.

## Implementation and verification

1. Agree typed IPC adapter and durable request retry behavior with desktop host writer.
2. Build accessible setup/creation/session/situation forms and safe table play views.
3. Test retry identity, source choices, player visibility and representative UI interactions. Run scaffold-provided Svelte/TypeScript checks and UI tests/build after integration availability.
4. Commit coherent UI slice for independent root/shell review; verify packaged scenario and actual restart in integrated production path before Gate 3 acceptance.

## Current state and next action

Implemented the agreed `desktop_*` IPC adapter with snake-case request bodies and explicit Host/Player channels; the backend derives entity authority. Runtime defaults initialize every agreement field, and runtime catalog/options supply source choices and equipment prices. Separate forms cover creation, sessions, generic situations and raw dice; sheets include current derived bonuses, hit dice, Inspiration, conditions and death state. Host-only check context and transient private answers clear when changing player views.

The complete original unconfirmed request is stored before invocation and retained on uncertain/transport failures. A typed host `retryable: false` rejection releases it for correction without remounting the form. Exact accepted replies clear it; restart offers the same request again. No game state is written to browser storage.

Local direct Svelte compilation passed every new component with zero warnings; TypeScript `tsc --noEmit` passed. Root then ran the isolated check tree outside the native esbuild sandbox restriction: `svelte-check` reported zero errors/warnings and all ten UI tests passed (six TableApp retry/privacy/recovery tests and four creation/agreement/dice tests). The isolated scaffold build passed, but that scaffold did not yet mount TableApp; the integrated shell's production build remains required.

The review follow-up validates malformed saved request context/channel/head/action shape before dereferencing it, normalizes malformed selection data, keeps corruption recoverable through campaign navigation, and uses user-facing retry descriptions. Switching player channels clears private answers and restores keyboard focus after the new view is ready. Tests wait for an enabled channel selector before interaction, preserving the actual projection/privacy assertions. This is verified component/application-adapter evidence, not native desktop acceptance. Next independently review the integrated shell/IPC path and exercise the packaged production scene and quit/restart behavior.
