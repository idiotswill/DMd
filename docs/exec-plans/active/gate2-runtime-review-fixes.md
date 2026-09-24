# Gate 2 runtime review fixes

Status: active on `codex/gate2-runtime-review-fixes`, based on application head
`b74149f`. The rules-architecture agent is the sole writer for this slice.

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

Implementation pending. Verify the focused application tests and strict Clippy
against final foundation `50a86fe` using a verification-only local merge if needed.
Parent cherry-picks only this slice's app/plan commits, owns full exact-head checks,
and integrates the foundation separately. No product scope or gate boundary changes.
