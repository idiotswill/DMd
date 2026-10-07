# Physical creation opportunity provenance correction

Status: planned, before implementation; 2026-10-05.

## Objective and scope

Correct the new Glaive application regression on branch
`codex/gate4-physical-creation-source`, PR62, after actual Linux job
111720292708 failed on head e75e6b80fa4d6cb64f6b792b690b8952fdecb9db.
The complete table-loop harness reported62 passes and one failure at
`table_physical_creation_cases.rs:341`. Hands, armor, shield and actor were equal;
the accepted reaction replaced command provenance, as the existing resolver requires.

The production path assigns the accepted opportunity command in
`tactical/attacks/opportunity.rs` and the completing command in
`tactical/attacks.rs`. The new test incorrectly compared both complete loadouts
with the pre-reaction command. Preserve production behavior, all original
historical suites, every other new assertion, and complete loadout comparisons.
This advances Gate4 reaction/cold-recovery coverage and the product requirement
for durable typed command provenance; it changes no product scope or architecture.

## Planned correction and acceptance

1. Build the complete expected reaction loadout from the actual prior equipment,
   replacing only CommandMeta with independently known request, actor, issuer,
   session, campaign and pre-command sequence; compare the whole object.
2. Retain the actual final physical damage request through the existing cold-step
   producer, then require its complete CommandMeta in the final loadout. Keep
   every physical roll, reaction cost, movement, custody and replay assertion.
3. Full root and separate independent diff review. Reproduce all unaffected blob
   identities and original bodies, preserving the actual failed CI evidence.
4. Direct edited-file format/check and whitespace check, then normal push and
   fresh exact-head CI. Reprepare the original186 focused inventory without
   dropping any selected case or command. No verification claim until execution.

## Boundaries, risks and current validation

Only this plan and the one new test module may change. No weakened field-only
comparison, copied actual output as expectation, skip, relaxed compiler/test
configuration, or production resolver edit. The separately running Shove
canonical command owns the sole local compiler/test/native/database slot.

The failed complete log is preserved externally in
`tooling/ci-oct5/job-111720292708-1225-response.json`. It is old-head failure
evidence, not a result for the correction. Exact next action: commit this plan,
implement the narrow test correction, and obtain independent review before push.

## Source checkpoint

Plan95578a9 preceded the one-file source correction. The reaction expectation
independently names the original pending sequence and actual reaction request,
and also checks the retained attack origin. The final physical1d10 face1 command
is now retained directly through the unchanged cold-step helper; its before-state
sequence and request identity define the completing provenance. Both complete
loadout assertions remain. All other test cases, production and helper are unchanged.
Direct configured Rust formatting/check and `git diff --check` passed. No Rust
compilation or execution occurred locally; independent review and new CI remain.
The original186 focused runner is preserved and must be reprepared for this head
with all original tests and command selection retained, before its eventual run.
