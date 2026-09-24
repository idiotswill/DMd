# Gate 2 runtime review fixes

Status: complete; integrated and verified in [PR #18](https://github.com/idiotswill/DMd/pull/18).
The isolated `codex/gate2-runtime-review-fixes` branch was based on application head
`b74149f`; the rules-architecture agent was its sole writer.

## Objective and boundary

Resolve the application review findings without changing persistence or kernel
contracts. This slice owns `dmd-app/src/lib.rs`, runtime integration tests and this
plan. ADRs 019/020, Gate 2 failure/recovery acceptance, and the product's durable
authority and exact suspension requirements govern the work.

## Acceptance and implementation

- Detect rules lineage in current state, command audits, event kinds and every
  version-aware historical snapshot before choosing the generic restore path.
- Load and validate content before create/restore writes, then reuse that validated
  content and pack to validate the returned state without another file read.
- Preserve strict content validation on every later open/resume/action/query.
- Prove relabeled event/audit histories cannot bypass restore preflight.
- Prove a valid mechanics transition rejected by persistence for an invalid session
  leaves authoritative state, audit and event journal unchanged.
- Adapt the existing explicit circumstances fixture to the final kernel API.

## Validation and next action

Implemented both review fixes. Restore inspects all current/history signals before
choosing generic recovery. Create/restore retain the preflight catalog and rules
pack, then perform only pure validation after persistence. Open/resume still reload
and verify content; retaining a pack for one response grants no later capability.

An additional deterministic regression exhausts the target connection pool and
polls create/restore through preflight to their database wait. It changes kernel
bytes before releasing the connections: the operation must return its successful
commit, and the next open must reject the changed content. This avoids sleeps or
production test hooks. The session regression asserts the concrete persistence
`MissingSession` error after otherwise-valid rule resolution, then compares current
state, command audits, events and snapshots unchanged.

Verified against foundation `50a86fe` through a verification-only local merge:

- `cargo test --offline -p dmd-app`: 6 helper, 15 runtime integration and 8 existing
  runnable-campaign tests passed.
- `cargo clippy --offline -p dmd-app --all-targets -- -D warnings`: passed.
- Scoped Rustfmt and `git diff --check`: passed.

Final integrated head `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c` passed complete
workspace verification (172 Windows tests) and
[CI #305](https://github.com/idiotswill/DMd/actions/runs/36003158276), including MSRV.
Independent final-head review found no blocker; PR #18 merged as
`3ad3885559073e6748d3f17c2172be9ff2a99f52`. No remaining slice action or product/gate
boundary change. Gate closeout owns final publication and owner pause.
