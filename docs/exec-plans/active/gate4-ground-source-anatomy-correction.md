# Gate 4 — Current source anatomy registry expectation

Status: plan before source, 2026-10-05. Root is the sole writer on
`codex/gate4-ground-production`, beginning at reviewed corrected
`819765bda18ee0dc82c2f5efcacc72a8eff8bfd8`, tree
`98058f2ea6cd057320ba0dcf4925e85b62701dae`. This is a test expectation correction
within the active Gate4 Ground/Ogre receipt, not a production or content change.

## Finding and intended behavior

Independent source review found a concrete stale assertion in
`tactical_grapple_sources::immutable_goblin_revision_changes_only_the_reviewed_typed_anatomy`.
It checks every current source except the Goblin ID has absent ordinary anatomy.
The reviewed current registry now includes the immutable Ogre source with explicit
`TwoHandsV1`; therefore that old blanket assertion contradicts the current source
contract. This is a static finding, not an observed failure from the pending819 CI.
The older055 package-test failure and its819 correction retain their original
evidence and attribution.

Keep the complete original Goblin fingerprint, old/new normalized equality,
optional-absence wire, immutable coexistence, exact current selection and forged
pin controls. Replace only the obsolete registry-wide expectation. Derive exact
full Goblin and Ogre pins from their respective immutable definitions, require
exactly one current Ogre full-pin entry, and require TwoHandsV1 only for these two
full pins. Every other current source must still have absent anatomy. Never use a
loose ID exclusion or allow an arbitrary new source to acquire ordinary hands.

This advances source fidelity and Gate04 creature/grapple prerequisites while
preserving ADR029 immutable source meaning, historical compatibility and the
product's authoritative rules boundary. No source asset, registry, gameplay,
fixture, old capture or independent original-history body changes are allowed.

## Verification and receiving scope

Commit this plan first, then make the one bounded expectation correction. Inspect
the complete changed function and independently verify all other source and test
bytes against819. Direct standalone rustfmt and static Git checks are permitted;
Shove canonical83971 retains the sole local compiler/test/native/DB slot.

The corrected exact head requires fresh focused137 preparation preserving all137
literal functions and test arguments, with both documented expectation migrations
explicit. The actual package regression stays near the start. Strict affected
all-target Clippy, meaningful focused tests, canonical and required exact-head CI
remain mandatory; no old result verifies the new head. The819 preparation remains
preserved as superseded/unrun. Publish the corrected PR61 head after source review.

Root must normally receive this correction into consequences and later combined
Ground/Graze source; the independently reviewed33570 Physical union and its active
Graze writer stay frozen to their current parent until a deliberate receiving
checkpoint. No branch is merged to an unaccepted development base for acceptance.
Gate4 remains open. Next action: commit this plan, correct the precise assertion,
then obtain independent source/preservation review before publication.

## Authored correction checkpoint

Plan `64eced1b04f25c7819ce3512485ccac6e14b5f87` preceded the source correction.
The test now requires one current exact Ogre pin and checks TwoHandsV1 for only
the exact current Goblin/Ogre pins. Every other current source retains the
absence assertion. The remaining original function, all three sibling functions,
production, immutable source definitions and historical artifacts are unchanged.
Standalone configured rustfmt and whitespace checks pass; no compiler/runtime
result exists for this correction. Independent review and fresh preparation,
publication and receiving verification remain next.
