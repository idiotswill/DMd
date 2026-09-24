# ADR 020 — Rules restore history preflight

Status: **Accepted — Gate 2 addendum to ADRs 012 and 019, [PR #18](https://github.com/idiotswill/DMd/pull/18), exact head `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c`, [CI #305](https://github.com/idiotswill/DMd/actions/runs/36003158276).**

## Decision

Before a rules-enabled application restore opens its destination write transaction,
it upgrades and structurally validates the export, resolves the exact local rules
pack, and validates mechanical history in memory. Generic raw recovery remains
available through persistence and does not interpret rules.

The preflight checks every immutable snapshot through the version-aware codec and
the selected rules kernel. It checks every typed rules event's kind/version,
campaign, actor, session, command identity, event source and one-event sequence
contract against its durable envelope. The typed event command metadata must
exactly match the corresponding accepted audit, including trusted issuer identity.
The audit's typed action must equal the action being replayed. A rules command
cannot masquerade as an opaque generic event to avoid these checks.

Deterministic replay begins at the earliest actual available snapshot, not the
latest snapshot. Each subsequent event must have supported semantics and the
next contiguous sequence. The reconstructed state must match every later snapshot
and the final exported current state. Thus a poisoned latest snapshot cannot hide
an earlier divergence merely because an ordinary replay-to-head would select it.
The preflight has no database writes, randomness or provider calls.

Privileged ruling records must identify the exact originating accepted audit and
match that typed action's ruling. Duplicate or nonprivileged ruling provenance is
invalid. A historical ruling whose audit is unavailable is tolerated only if its
complete record already exists in the earliest anchor, has no future sequence,
and remains unchanged. Later snapshots cannot introduce invented provenance under
that exception.

The retained command metadata on pending requests, accepted roll records and
contextual permissions is likewise compared with its available accepted audit.
Only unchanged origin metadata already in the earliest anchor may lack an audit.
Purpose-specific mechanical validation remains owned by the rules kernel.

## Historical trust limit

Gate 1 backfilled recovery anchors at real then-current heads; it did not fabricate
earlier snapshots. The earliest valid immutable anchor remains the trust base for
history that cannot be reconstructed. Rules events before/at that anchor still
undergo typed/envelope/audit consistency checks, but there is no claim to re-resolve
them without their prior state. Opaque generic pre-anchor events remain available
for raw history. An unknown event after the anchor fails rules-enabled gameplay
restore rather than being ignored. Future production event families need an
explicit composed replay implementation before this policy can admit them.

This provides internal integrity and deterministic consistency, not publisher or
save-file cryptographic authenticity (TD-003). A deliberately rewritten earliest
trust anchor is not disproved by unavailable earlier history.

## Integration

`dmd-app::rules_restore::validate_rules_export` receives an already upgraded export
and validated exact `RulesPack`. The application invokes it before raw restore for
rules-enabled campaigns; generic campaigns continue their existing lifecycle path.
Rules lineage is detected from current state, rules command/event namespaces and
every version-aware historical snapshot. Relabeling only current content or event
kinds cannot bypass semantic validation. Six helper regressions and the runtime's
eleven malformed-export variants, valid pending restore, legacy upgrade and broad
replay/export/restore scenarios passed with the integrated kernel and wiring.
