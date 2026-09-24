# Gate 2 rules export preflight

Status: helper implementation and focused verification complete on
`codex/gate2-rules-export-preflight`, based on `ebaef360`; parent application
wiring and integrated exact-head verification remain part of the Gate 2 plan.

## Objective and boundaries

Add one app-owned, pure export validator before rules-enabled restore commits.
The parent Gate 2 agent owns module wiring, dependencies and integration tests.
This slice changes only a new `rules_restore.rs` helper and its design/evidence
documents. It does not mutate persistence contracts or generic recovery behavior.

Relevant acceptance: Gate 2 invalid/incompatible actions fail without mutation;
durable production-path/replay equality; product exact suspension and provenance;
ADRs 011, 012, 017 and 019.

## Acceptance and implementation

- Validate mechanical state in every decoded immutable snapshot.
- Cross-check each typed rules event against its envelope and command audit,
  including issuer identity and typed action.
- Replay from the earliest available real anchor, compare every later snapshot
  and the final current state, and reject unsupported post-anchor events.
- Check privileged ruling provenance against available audit history; only exact
  provenance already present in the earliest trusted anchor may lack history.
- Preserve all export bytes and avoid database writes.
- Parent wires and tests the helper after the mechanical types land; this branch
  must not merge independently of those types.

## Risks and deliberate limits

An immutable earliest anchor is the historical trust base. The validator cannot
derive history preceding that anchor and must not claim to. Rules-enabled restore
with unknown post-anchor event semantics fails closed; raw generic recovery stays
available separately. Cryptographic authenticity remains outside this gate.

## Validation and next action

Implemented helper and six internal regression tests for valid immutable
preflight, poisoned current/latest snapshots, pre-anchor envelope/audit parity,
unsupported events/forged outcomes, invented ruling provenance, and accepted
prone/initiative/bonus-action histories. Origin audit checks also cover pending
requests, accepted roll records and nested permissions. Ruling extraction uses
an exhaustive action match so future actions require an explicit decision.
ADR 020 records the earliest-anchor trust boundary and failure policy.

Verified with the kernel checkpoint `77b229c` in this isolated branch:

- Standalone `rustfmt --edition 2024 crates/dmd-app/src/rules_restore.rs` passed.
- A temporary external crate imported this exact module by path and depended on
  this branch's domain, persistence and rules crates. `cargo test --offline`
  passed all six helper tests; `cargo clippy --offline --all-targets -- -D warnings`
  passed. The harness changed no application dependencies or repository wiring.
- `git diff --check` passed.

No full integrated application verification is claimed by this slice. Next: the
parent cherry-picks only helper commits, wires the module into rules-enabled app
restore, and runs `cargo test -p dmd-app` plus the required full exact-head checks.
The local kernel cherry-pick is verification-only and must not be cherry-picked
again by the parent.
