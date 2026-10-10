# Gate 4 — Lawful owned-source rolls and readable replacement setup

Status: source correction authored after plan-firstd1c6d4e1, 2026-10-08;
compilation/runtime UNRUN. Branch
`codex/gate4-family-receiver`, baseline396530d14135d376e352332129fd7dd6da41574d.
Root owns this plan and independent review. Delegated sole writer
`v5_capture_plan_review_oct8` returns source ownership to root at this handback.
No Gate5 work is allocated.

## Evidence and objective

PR77 Linux run37782376756, grapple job113328427178, actually finished84 passed
and five failed in12,048.49 seconds. Its Rust source is unchanged from familyb62;
this remains PR77 evidence, not a transferred parent pass. Original artifact
SHA2567837e31eb05be9600f1a101c3a8c46451cd4dd89275ec6a0c086a7791cf12c22;
independent auditf64270cb8a15c68e6f6542860ffe33b0018ee9cc2c85f87d9260890d88ed7722.
The original failure helpers print neither the rejected request nor the offered
choices. Distinguish source diagnosis below from directly observed output.

Four existing source save/Escape cases (G3, G4, M+G3, M+G4) fail to find
`Grapple Small armored figure with left hand`. Their fixture assigns that source
to Player1, then asks Player0's PC to grapple it. The existing opposition rule
correctly refuses interactions between two player-controlled bodies. Preserve
that rule and the intended owned-source saving/Escape coverage. Add a genuine
Host-controlled opponent through existing production fixture construction;
carry it through replacement placement and initiative, and advance actual turns
before its attempt and the owned target's Escape. Never inject state or change
the table's PvP policy to manufacture the positive route.

The existing intrinsic attack/release/retry case fails at shared cold acceptance
with `initiative has not begun`. Source tracing identifies a separate read-path
defect: retained G4 asks for ground-drag offers after replacement Establish has
created a legitimate battlefield with no initiative flow. The query calls
`flow(state)?` before its Active-phase check. No-flow setup should yield no drag
offers; it should remain readable so normal Begin can follow. The precise
failing request is an inference until request diagnostics or runtime confirm it.

This advances [Gate4](../../checkpoints/gate-04-tactical-encounters.md) and the
[product definition](../../product-definition.md)'s player authority, real
continued play and exact recovery requirements. Preserve ADR024/026/027/028,
[gate protocol](../../checkpoints/gate-execution-protocol.md), the Finished
Inspiration correction and all existing acceptance obligations.

## Bounded implementation and acceptance

- In the ground-drag offer query only, return no offers when no encounter flow
  exists. Keep original-history/guarded admission and attendance checks before
  that result, and all Active/version/owner/grip eligibility checks unchanged.
  Do not relax movement execution, live grip validation or replacement admission.
- Correct the shared source roll-details fixtures/helpers in their two existing
  support modules. Preserve all four named tests, ability/save/Escape paths,
  owner/Host/unrelated privacy, read-only rows, cold/portable/retry and M/G proofs.
  Keep PC-only producer behavior unchanged. Assert the prohibited PC-to-owned-
  source choice is absent before taking the lawful Host-to-owned-source route.
- Extend the existing intrinsic replacement witness at its actual prepared-map
  boundary: no flow, G4 retained, readable audience views with empty drag offers,
  unchanged read rows, then genuine Begin/physical initiative and current read.
  Add request/error detail to both shared cold acceptance panics only; keep all
  acceptance, retry, replay and refusal assertions.
- No DTO, schema, corpus, dependency, rules-content, public authority, persistence
  normalization, profile/stack/thread or canonical command changes. No test is
  removed, ignored, filtered out of acceptance or relabelled as passing.
- Format touched Rust files and inspect the complete diff independently. Derive
  all final inventories from frozen source. Existing test names remain; any new
  names must be explicitly attributed. Runtime is UNRUN until actually executed.

## Verification and next action

After source review, add the five actual failed cases as exact early diagnostic
stages beside the three new Finished controls and unchanged v5 mass case. Each
stage must verify its complete selected names and expected filtered count on
the same frozen source and fresh dedicated target. These focused stages never
replace the unchanged unfiltered full canonical suite, all27 CI controls and
ordinary frontend verification. Preserve old failures and every old target.

Require fresh Linux/MSVC seven-allocation unions, prerequisites, aggregates,
ordinary Windows package, combined native acceptance and expected-head merge.
Later literal-main verification and the separate human Gate4 encounter remain
required. Root must independently review the frozen source/test corrections,
then allocate actual verification. No prior result passes this correction.

## Authored correction and source-only checkpoint

The production change is only the no-flow guard in
`tactical/grapple/transport.rs::choices`. Original-history, current session and
attendance checks precede it; all existing flow/version/Active/owner/live-grip
checks and movement execution remain unchanged. The actual failed request is
still source-inferred from the preserved shared panic, not newly reproduced.

Only source-owner fixtures add the existing genuine Host goblin opponent. Before
advancing PC -> owned source -> Host holder, the fixture explicitly asserts that
the PC has no attempt against the owned target. Host owns its actual Grapple and
after-equipment completion. Escape advances the intervening PC turn before the
source owner's action. M replacement preserves the third actual placement,
source profile/control origin and mutually opposed relationships, then uses a
separate Host initiative group/physical roll. All source turn identities are
asserted. PC-only producers still select the original two-body constructor,
placements, rolls and commands. The four source strata retain their complete
Strength/Dexterity Save and Athletics/Acrobatics Escape branches and every
existing read/private-authority/retry/cold/portable/M/G assertion.

The existing intrinsic helper now checks the genuine replacement Prepared map:
no flow or pending roll, exact retained G4 attachment, empty raw/presented drag
offers for Host and both players, refused premature intrinsic read and unchanged
export/all-table rows. Original Begin and physical initiative then proceed to an
asserted owned active actor, with the existing current intrinsic read afterward.
Both shared cold acceptance panics now name their request/error. No original
acceptance/retry/refusal body or test name is removed, and no new test is added.

Direct configured Rustfmt/check (edition2024, skip_children=true, five touched
Rust files) and `git diff --check` pass. Full source diff/self-review and bounded
preservation audit are complete. There was no Cargo/compiler/frontend/test,
application/native/database execution, push or workflow action. All corrected
runtime evidence remains UNRUN; the five existing failure witnesses must pass
on the final frozen head before the unchanged full acceptance sequence.

## October 10: replacement initiative grouping correction

The actual 63b48d50 Linux Grapple output names the existing M+G3 owned-source
case FAILED before its six-hour cancellation. The buffered final panic summary
is absent; do not claim its exact assertion was observed. Original audit SHA256:
`83739f8a59316c643d51554f0818685a4893cfb7d74d77242e6abff7cad80378`.

Independent static investigation found a definite invalid setup in the repaired
M source fixture. `start_mass` assigns the owned Goblin and opposing Host Goblin
separate initiative groups, although both use the same immutable Goblin source,
surprise and initiative circumstances. `tactical/validation.rs` rejects precisely
that state: identical creatures must share a roll. Ownership is not an initiative
circumstance. The initial `Fixture::with_opponent` already uses the lawful shared
group and actual Host tie decision. The same defect affects the M+G4 setup even
though that case has no terminal failure output. This is a source diagnosis of
a definite defect, not a reconstructed missing panic.

Root allocates a bounded correction only in
`tests/support/table_grapple_roll_details_mass.rs`, after this plan amendment:

- Use one replacement initiative group `[owned Goblin, Host Goblin]`, retaining
  the distinct PC group. Do not change combatants, source pins, positions,
  controller provenance, real intervening turns or opposing relationships.
- Submit one real shared physical source roll through the owned source channel;
  remove the invented separate Host roll. Preserve the PC-only fixture path.
- Inspect the resulting genuine unresolved tie and use the existing Host
  `ProposeInitiativeTie` command to order source then holder. This is not a
  character-only tie, so do not invent player acceptance or change production
  authority. Keep the command-v5/G3-or-G4 distinction and cold/portable/exact-retry
  helper for the tie transition.
- Assert exact shared group/request/roller, actual owned acceptance and physical
  face, two new rolls with unchanged old prefix, shared total, pending-free tie,
  Host decision and final participant order. Preserve all original source-roll
  tests, four ability branches, private reads, no-write and recovery assertions.
- No production, source content, schema, corpus, snapshot, controller policy,
  resolver or canonical verification change is allocated by this correction.

The independently reviewed test-profile candidate 5b9da641 has not compiled or
run. Its external runner/manifest preparation is preserved without launching it.
Correct this definite setup defect before spending a fresh compilation; then
independently review the full source delta and bind a new clean head to a fresh
target. Keep the same seven focused stages with M+G3 first and unchanged complete
canonical/Python/frontend/hosted/package/native acceptance. The original actual
failure remains recorded until the corrected case really executes successfully.
