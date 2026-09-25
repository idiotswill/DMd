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

## Review checkpoint

Extraction `6b1bc541be011e991ece17a44cfb6eabcd0dbcf0` is open as
[PR27](https://github.com/idiotswill/DMd/pull/27). Independent full-diff/source review
confirmed Command p116, Fireball p131, Scorching Ray p159, Chimera p273 and Adult Red
Dragon pp318–319 values, substitutions, usage and material-only waivers. It found one
validator gap: individually valid alternatives and shared limits could require an
impossible overall routine. A bounded capacitated matching now requires a complete
assignment. Regressions cover impossible three-ray/max-one routines and a feasible
case that requires reassigning an earlier choice. Independent review of this correction
found no remaining blocker. Formatting/diff checks pass; Rust compilation, canonical
verification and exact-head CI remain required and are not yet claimed.

## Verified source head

Exact `7210e30175e3cf371809e5cb143ccb92abcfb9b2` passed canonical `./scripts/verify`:
311 Rust tests, formatting, workspace/all-target check, strict workspace/all-target
Clippy, genericity guard and architecture guard (eight tests, one platform-specific
skip). Durable local log: `tooling/gate4-definitions-7210e30-verify.log` outside the repo.
All six exact-head jobs passed in [Linux CI](https://github.com/idiotswill/DMd/actions/runs/36067643556)
and [Windows desktop](https://github.com/idiotswill/DMd/actions/runs/36067643558), including
Rust1.88 compatibility and stable Windows packaging. Two separate reviewers confirmed
the final source/validation diff without remaining blockers.

This final evidence update changes documentation only. Inspect its exact delta, verify
code/content equality with `7210e30` and require its own final-head CI before protected
merge. Then fetch main, compare the merged tree and check post-merge CI. Source execution
and complete Gate4 acceptance remain open as described above.
