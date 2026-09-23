# Gate 1 — Atomic state and event journal

Status: completed and merged
Branch: `gate1/atomic-journal-finalize`
PR: #6
Merged to `main`: `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`
Verified PR head: `b9395320408e6e223737553f1015ef3da36ad6b6` (CI run 147 — pass)

## Objective

Build the first Gate 1 production persistence slice: every accepted material command transition can commit authoritative campaign state, command/resolution provenance, and an append-only event journal atomically in SQLite, with per-campaign sequence allocation and restart/recovery verification.

## Scope completed

- SQLite schema for current materialized campaign state, command audit records, immutable journal events, and causal edges;
- typed pending-event/audit boundary rather than provider-shaped persistence input;
- campaign/state/issuer/session/event-causality and state-held provenance validation;
- persistence-owned contiguous event sequence allocation;
- stale command rejection plus compare-and-set head advancement;
- next `CampaignState` plus audit/events/causal edges in one transaction;
- recovery loading from one SQLite read snapshot, including head/provenance integrity checks;
- tests for success, stale-state rejection, initialization/recovery/current-state provenance corruption, causal ordering/campaign isolation, authority metadata, rollback/reopen behavior, and direct-history-delete rejection;
- mechanical dependency guard preventing interpretation/core/rules layers from acquiring a direct persistence/SQLx write path;
- scoped Gate 1 checkpoint and atomic-journal ADR.

## Non-goals retained

- full snapshot-version migration/replay machinery;
- normalized projection tables for every domain aggregate;
- complete campaign lifecycle/backup/export/purge flows;
- rules/content-pack schema loading;
- combat or gameplay rules;
- world simulation/Director logic;
- voice, local inference, or desktop UI.

## Acceptance results

- [x] Clean campaign initialization at event sequence 0.
- [x] Atomic state + command audit + journal event + causal-edge commit.
- [x] Persistence-owned contiguous event sequence allocation.
- [x] Stale-state rejection without mutation.
- [x] Same-campaign session/causal/provenance validation.
- [x] Issuer preserved separately from in-world actor.
- [x] Restart/reopen recovers only committed state.
- [x] Recovery reads state/head/provenance from one SQLite snapshot.
- [x] Current-state provenance is revalidated before later writes.
- [x] Command/event/causal history rejects direct `UPDATE` and `DELETE`.
- [x] Direct persistence dependencies from domain/conversation/core/rules are mechanically blocked.
- [x] Full repository verification passed on exact final PR head.
- [x] Human explicitly approved the save-format/high-impact slice.
- [x] PR #6 merged to `main` as squash commit `1230fe89829eeb6c2047c75b22dfb8c4b2c39eb8`.

## Important decisions

- The serialized current-state row is a durable recovery/materialized-state artifact, not the permanent query architecture.
- Persistence owns event sequence allocation.
- Language/provider output never gains direct write authority.
- Historical command/event/causal rows are immutable; controlled whole-campaign purge remains a later lifecycle concern.
- Recovery and material commit both fail closed on dangling state-held event provenance.
- Event/provenance metadata lookup is batched rather than N+1.
- The original branch became contested during review; finalization moved to a one-writer branch and PR #6 superseded PR #4.

## Validation history

- CI run 136 — rustfmt failure only.
- CI run 137 — implementation/provenance head passed.
- CI run 138 — full suite passed with architecture guard.
- CI runs 139–140 — batching refactor exposed formatting then Clippy issues; both were corrected without weakening checks.
- CI run 144 — new integrity test formatting failure only.
- CI run 145 on `596675f87931aa539e7c2ae128b470180eff21ce` — full suite passed.
- CI run 146 on `b35856733c65e0d50c6abac60d03356d3917847e` — full suite passed after docs reconciliation.
- CI run 147 on `b9395320408e6e223737553f1015ef3da36ad6b6` — full suite passed on the exact approved PR head.

## Remaining Gate 1 debt handed forward

- snapshot schema migration and typed event replay;
- normalized/query projections and rebuild behavior;
- campaign lifecycle/backup/export/restore/controlled purge;
- versioned rules/content manifest resolution;
- application-level write scheduling/retry policy.
