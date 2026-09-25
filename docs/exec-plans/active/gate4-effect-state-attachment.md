# Gate 4 effect authority and condition queries

Writer: root. Branch: `codex/gate4-effect-state-attachment`, from
refreshed main `beaad44c459a65ba674eeca747096a406eab0d78` after verified NPC setup PR30. Integration reference: `df75fb8`;
the reviewed pure lifecycle reducer is already on main through PR24.

## Objective and boundaries

Connect source effect lifecycle authority to the ordinary mechanical state and its
condition queries, retaining concentration identity, source actors, suppression,
expiry and recovery provenance. This is the next coherent prerequisite for real
initiative and shared turn work. It advances product source fidelity, authoritative
conditions, persistence and Gate4 tactical timing under AGENTS, product definition,
Gate4, ADR026 and the pinned SRD5.2.1. The existing effect-lifecycle plan retains the
full18-family spell obligations and detailed source references.

Add the optional RulesState effect attachment, its structural/source validation,
unified ephemeral condition adapter and tactical condition queries. Preserve legacy
event interpretation when the attachment is absent. Do not persist a second copy of
derived conditions in legacy ActiveEffect. This internal bridge does not expose an
arbitrary install-effect player command or claim an executable spell by itself.

Do not copy the flow-dependent vitality drop hook, spell slot reservations, encounter
queue or raw initiative controls into this slice. Inspection confirms tactical_spells
depends on tactical_budget and encounter.flow, so it cannot be extracted as a pure
independent prerequisite without temporary adapters. The private test_outcome helper
belongs with its actual callers in the subsequent queue slice, not an unused export.

## Acceptance and planned work

1. After PR30 merges, refresh main and inspect the current adapter/condition dependency
   closure. Retain all earlier physical-reference/table-composition regression fixes.
2. Attach effects with omitted/default historical state semantics, typed legacy save
   rejection, original recovery anchor and complete source-operation origin collection.
   Keep shipped SQL migration checksums and full app commit validation unchanged.
3. Preserve one concentration pointer and source-specific charmer/fear/grapple queries.
   Verify replacement, suppressed effect reappearance, independent targets, time/turn
   expiry, duplicate identity rejection and malformed post-anchor source rejection.
4. Resolve the observed adapter bug: installing Unconscious currently sets Prone
   unconditionally, and attachment validation requires it even for a Prone-immune
   target. Add a source-condition immunity regression and preserve historical legacy
   semantics without allowing impossible new grouped-condition state. A later vitality
   or falling hook must not be used to hide the incorrect condition bridge.
5. Test actual domain/rules/app validation and restore boundaries appropriate to this
   internal attachment; do not substitute direct fixture mutation for future source
   spell admission or a packaged encounter. Run focused checks, canonical verification,
   independent full-diff review and exact-head Linux/Windows CI before protected merge.

## Following production slices

The next larger atomic slice owns real initiative and shared turn work together:
final typed flow/resolution contracts, budgets, vitality/KO proof, pending raw roles,
source creature hooks, table controls and cold reopen/semantic replay. Keep the single
production serde shape and fixed roll-role tags. Until a feature is activated, any
nonempty unsupported work/cursor must fail closed and no UI may advertise it.

Then add complete vertical paths for physical combat, movement/OA/falling and casting,
using current reviewed privacy/source fixes rather than historical incomplete versions.
All remaining Gate4 source mechanisms, enemy behavior, improvisation and full packaged
multi-round encounter acceptance remain active. No Gate5 work is authorized.

## Status and exact next action

PR30 merged with expected-head protection after independent full extraction/correction
and final evidence-head reviews. Final head `78166c6` passes all six checks: Linux
`36111205928` and Windows `36111206026`, including offline installer. Canonical source
`6136aeb` passes356 Windows Rust tests and all guards/lints; final source/lock/frontend
parity was verified. Fetched merged main `beaad44` has exact full-tree parity with that
head. Post-merge checks are running.

This fresh branch starts from that fetched main. The adapter/condition closure is
extracted from `16f97e7`, including the previously reviewed internal held-cast duration
operation and its tests. No new player effect-install endpoint exists. Legacy commands
fail closed only when grouped authority is attached; ordinary existing campaigns and
the PR30 PC/NPC preparation path retain absent-attachment behavior.

New regressions cover grouped Unconscious with/without Prone immunity, inability to
install an immune condition, malformed missing Prone, retained posture after expiry,
suppressed Unconscious reappearance and historical legacy query parity. Only grouped
conditions receive the new immunity consequence; legacy HP-zero/ActiveEffect behavior
keeps its old interpretation. Source-authorized vitality/held-item dropping is not
copied without its queue dependencies or claimed as a complete condition path here.

Typed legacy save and transactional database preflight reject non-null effect authority
and both duplicate-null orders. Real table export/restore rejects invented current,
backfilled-anchor and operation-origin authority before writes, while an untouched
export restores identically. Existing physical-reference/composite table regressions
remain intact. Rust formatting and whitespace checks pass; all new runtime tests are
written but uncompiled. Next commit/open the draft, obtain full exact-source review,
run focused/canonical verification when the serialized compiler reaches root, and
verify final-head Linux/Windows CI before merging. Gate4 remains active.
