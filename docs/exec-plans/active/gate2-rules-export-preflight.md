# Gate 2 rules export preflight

Status: implemented, awaiting compilation with integrated kernel on
`codex/gate2-rules-export-preflight`, based on `ebaef360`.

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

Implemented helper and five internal regression tests for valid immutable
preflight, poisoned current/latest snapshots, pre-anchor envelope/audit parity,
unsupported events/forged outcomes, and invented ruling provenance. Origin audit
checks also cover pending requests, accepted roll records and nested permissions.
ADR 020 records the earliest-anchor trust boundary and failure policy.

Standalone rustfmt and `git diff --check` passed. Compilation awaits the kernel
types/dependencies/module wiring; no full verification is claimed for this
unlinked helper. Next: commit this isolated helper, bring kernel commit `77b229c`
into the local verification branch, and compile/test through a temporary harness
without altering application dependencies. Parent cherry-picks only helper commits.
