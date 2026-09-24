# ADR 019 — Durable rules application runtime

Status: **Proposed for Gate 2; acceptance requires production integration evidence**

## Decision

`dmd-app::CampaignRuntime` composes deterministic `dmd-rules` actions with the existing atomic campaign journal. `dmd-core::GameCommand` preserves the typed command boundary. Persistence does not interpret rules or load content, and the rules kernel does not call providers, SQL or randomness.

`RulesContext` is trusted session/input metadata: campaign, issuer, actor, optional session and observed event sequence. It is supplied separately from the typed action proposal. An application-created command ID identifies the durable audit entry. The kernel enforces action-specific authority and character ownership; persistence repeats its structural authority, session and sequence checks during commit. A failed or stale resolution writes no state or event. Callers reopen and resolve again after contention rather than retrying a cached resulting state.

## Exact content and mechanics

Manifest resolution proves local content integrity and exact version availability. It does not prove that a compiled kernel implements that version. Rules-enabled create/open/resume/restore and every action/query therefore also load the manifest-declared `kernel.json`, verify the exact bytes against its length/checksum, validate the typed definitions, and validate persisted mechanical state against the supported rules identity.

Existing generic recovery/lifecycle composition remains available for campaigns without mechanical state. No unknown rules identity may execute mechanics. A rules state attached to an unsupported identity fails at the runnable boundary. Content unavailability never triggers a fallback to another rules version.

## Pending and completed rolls

Rule resolution creates the authoritative request (ID, dice, modifier, advantage/disadvantage, visibility and purpose). The pending action is committed before asking for physical dice and survives suspension/restart. Physical submission supplies only raw faces for the saved request. Digital dice use the same result validation and record their raw outcomes; replay never rolls again.

Request consumption and mechanical consequences commit atomically with the action/result event and issuer audit. Duplicate, mismatched, impossible or unauthorized submissions fail without mutation. Player-facing query responses enforce visibility; internal runnable state and raw persistence are trusted runtime/admin surfaces, not player response objects.

## Queries and replay

Rules questions use an immutable query API with a trusted viewer. They construct no command and do not advance event sequence, consume resources or commit journal records. Explanation rendering occurs after structured resolution; rendering failure cannot roll back or repeat accepted actions.

`rules.action_resolved@1` stores the typed action, trusted command metadata and derived outcome. The application replay adapter rejects unknown kind/version and mismatched journal metadata, then asks the pure kernel to re-resolve the recorded authoritative inputs and compare the outcome. It does not assign an arbitrary serialized after-state. Persistence retains responsibility for immutable snapshot anchors, contiguous sequences, causal references and final domain invariants.

## Compatibility and scope

ADR 017 defines schema-2 state compatibility; old recovery anchors remain immutable. Rules data is part of the same current state/snapshot/export path, not a parallel store. Whole-state mechanical queries are deliberate at Gate 2; projection/performance tuning remains owned by later measured workloads (TD-002/005/006).

This is the production rules integration boundary before Gate 3 desktop UI, Gate 4 full tactical encounters and Gate 5 complete noncombat play. It does not claim those gates complete.

## Required evidence

File-backed campaign create, pending physical roll, restart, resolution, query and replay must agree. Invalid/unauthorized/stale inputs and content/version failures must leave state and history unchanged. Export/restore must preserve pending rolls and mechanically relevant state; separate campaigns must remain isolated. These checks must run against the real runtime/persistence APIs before this ADR is accepted.
