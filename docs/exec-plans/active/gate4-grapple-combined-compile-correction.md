# Combined Grapple/Ground compilation correction

Status: planned before source edits, 2026-10-06. Root is sole writer on
`codex/gate4-grapple-positive-integration`, current local e8bb926 after normal
receipt of Groundb30. Published dae7d0f failed all four compiler jobs; both
architecture/genericity guards passed. Full actual logs are preserved externally
as tooling/ci-oct6/pr70-dae-job-{112209846649,112209846900,112209750161,112209750396}-2026-10-06.log.

## Findings and bounded correction

The combined production opportunity path still calls the removed ordinary
`creature_weapon::validate_source`, although it already owns the admitted attack
read. Pass that exact read to the existing `validate_source_with_read` operation,
preserving source damage and opportunity admission validation.

The complete incoming private Ground mechanism tests still use their old helper
entry points. Public Grapple converted those production functions to explicit
ExecutionContext/ReadContext, and did not need their old names. Restore only
`cfg(test)` ordinary adapters for those names, following the existing ordinary
attack validate and continuation submit adapters. Each delegates to the real
current implementation using the existing ordinary context, which cannot mint
guarded Grapple authority. No production bypass or new context constructor.

Ground's private PreparedPickup tests likewise use the prior eight-argument
new/derive methods and plan method. Keep those exact tests by giving the actual
current methods explicit with-hands names, retaining their exact implementation,
and adding cfg(test) prior-signature adapters that pass no derived hands/read.
All production callers continue passing the actual owned hands/read. The existing
raw/marked-state guards and negative controls remain intact.

## Acceptance and next action

Commit this plan before source changes. Inspect every actual compiler diagnostic,
modify only the named forwarding/adapter boundaries, and preserve every existing
test body, assertion, source/content file and capture. Compare complete inverse
diffs and run direct formatting/whitespace checks, then fresh exact-head CI.
No local competing Cargo run while the canonical Expiry391 package holds the
heavy slot. This is a compile correction; no runtime or Gate 4 acceptance follows
until the original and combined tests execute successfully. Remaining positive
scenarios, native acceptance, full canonical and integration checks remain due.
