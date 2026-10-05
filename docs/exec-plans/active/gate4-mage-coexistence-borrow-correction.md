# Gate 4 — Mage coexistence async borrow correction

Status: planned before source edits, 2026-10-05. Root is sole writer on
`codex/gate4-mage-ordinary-hands`, PR65. Fetched local and remote head both equal
`cae385425ab67b5bd891a318e7b464b1ec255c32`; main remains
`32c0c682c4dbb235e1f9a119643c5d8626d5cb71`.

## Finding and scope

Windows run37311869032 jobs111768966893 (Rust1.88) and111768967472 (stable)
both fail all-target compilation with three E0308 diagnostics in the new
`tests/support/mage_source_coexistence.rs` at lines363,371,459. The unchanged
legacy fixture returns `Box<CampaignState>` asynchronously. Direct references
to the awaited expression do not coerce at these three typed function calls.
The actual diagnostics request explicit dereferencing. Complete raw responses
are retained outside the checkout in tooling/ci-oct5 at the1255 evidence cut.

Change only those three `&f.state().await` expressions to `&*f.state().await`.
Keep all test declarations, scenarios, setup, assertions, legacy helpers,
production source, content identities and original capture bytes unchanged.
This repairs test compilation; it changes neither feature scope nor acceptance.
The original Mage plan, Gate04, ADR028/029 and product recovery/source-authority
requirements continue to apply without amendment.

## Verification and acceptance

1. Commit this plan before editing the three expressions.
2. Review the complete literal delta and reproduce that removing only the three
   dereferences yields the original complete file. Compare every other prior
   tracked blob and protected history byte against cae.
3. Use the already configured direct Rust formatter for this one file and check
   the resulting diff. No concurrent compiler/test/native/DB work is permitted;
   Shove canonical session83971 retains the sole heavy slot.
4. Obtain independent correction review, freeze the source successor, publish
   normally and require fresh exact-head CI. Retarget the prepared focused run
   with every original selected test retained and fresh unused evidence paths.
5. Keep the old compile failures and unrun preparation. No old pass or static
   review establishes a compiler/test/runtime pass for the corrected head.

Next action: commit this plan, make the three-expression correction, and review.
Gate4 remains open; no native, canonical or integrated acceptance is claimed.
