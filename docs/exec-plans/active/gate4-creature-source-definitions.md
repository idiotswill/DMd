# Gate 4 creature source definitions

Writer: root on `codex/gate4-source-definitions-verified`, from main
`6d14d81027995f1d880853ec1dfdfdedbc1c9875`. This is a bounded prerequisite for
physical weapon and source creature execution, not encounter acceptance.

## Objective and boundaries

Extract the source catalog/schema changes already used by the encounter integration:
explicit creature statistics and gear, mixed multiattack selections and replacement
limits, selected spell/legendary features, and exact source omissions. Retain the pinned
SRD5.2.1 attribution and manifest integrity. No RulesState attachment, action dispatcher,
save interpretation, runtime behavior, or presentation capability is added by this slice.

Authority: root AGENTS, product-definition rules fidelity and commercially distributable
content, Gate4 checkpoint, source-pinned SRD5.2.1, ADR024 source authority. The established
separate tactical catalog preserves the historical kernel's meaning. Definitions describe
source rules; they do not authorize commands or imply execution support.

## Acceptance and planned work

1. Copy only tactical definitions, their catalog/notice/manifest and definition tests
   from reviewed parent `de40b2b`; inspect the exact dependency diff.
2. Independently check source values and substitution/usage constraints against the
   pinned source; retain explicit omitted-feature coverage.
3. Run focused definitions and distributed-content integrity checks, canonical
   `./scripts/verify`, fresh full-diff review and exact-head CI.
4. Merge with expected-head protection, verify merged tree and post-merge CI, then
   reconcile main into the active encounter branch.

## Status, risks and next action

Plan created before extraction. No checks are claimed for this branch yet. Pure weapon
planning can follow this slice; creature profiles/physical gear then depend on that
planner. Shared scheduler, journal/codec guards, table execution and packaged Gate4
acceptance remain active work. Content completeness is not inferred from these selected
stat blocks. Next: extract the bounded five-file dependency and review source evidence.
