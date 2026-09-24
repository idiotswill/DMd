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

Read root AGENTS, Gate 3 checkpoint/active plan, table protocol/domain and existing Svelte scaffold. IPC contract is being coordinated. No checks or desktop acceptance claimed yet. Next create the typed API/request layer and independent form components while awaiting host command names.
