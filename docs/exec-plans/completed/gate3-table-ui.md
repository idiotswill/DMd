# Gate 3 desktop gameplay interface

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
