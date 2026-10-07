# Gate 4 — Retain completed spell evidence for Grapple reads

Status: **SOURCE DESIGN; IMPLEMENTATION AND RUNTIME PENDING**.
Date: 2026-10-07. Root allocates source_review_oct7 as sole writer of
`codex/gate4-grapple-public-completion`, checkout
`gate4-grapple-public-completion`. Specific remote fetch and clean local inspection
both confirm `e219ad69a4b9647f630bef85395490e3c2a103fb`, tree
`ac8d94f8322717035a0ce97f68f63366c693c290`, before this plan.

## Actual failure and objective

Exact e219 Linux job112288224236 and Windows job112288225084 each fail the
existing genuine three-ray application case with `invalid mechanics: raw source
cast absent`. Complete logs are retained under tooling/ci-oct7 with SHA256
`5c9fe6084b64b417efdf3fe42fc9e8da6fb243bd01efd0f573ccac54f94c8503`
and `bd4eb1ce9f620381a3f3c0397005a2dfac82a64d381f02df7b0c7bf97b2fbad8`.
The earlier fixture corrections are preserved; this is a production mismatch.

`casting::finish_cast` removes the completed actual source record from live
`resolution.casts`. The final work pump deliberately authenticates Grapple reads
before retiring the resolution. Both domain and rules raw-source readers still
follow each retained SpellProgram ancestor to that cast, so the actual last
consumer cannot authenticate the just-completed spell. Deleting the reads,
skipping their validation or substituting an attack command for its cast-bound
raw origin would weaken the accepted provenance contract.

Advance the product's source-faithful physical dice, tactical interruption,
private state and exact save/recovery requirements. AGENTS, product-definition,
Gate4, the main public completion plan, ADR020/024/027 and the execution protocol
remain binding. This correction neither accepts PR67 nor closes Grapple or Gate4.
Other branches and the inherited Inspiration/Held issues remain root-owned.

## Proposed bounded correction

Retain a complete source-checked `TacticalCasting` as completed evidence inside
the activated Grapple resolution, paired with the exact FinishSpell work key and
finishing CommandMeta. Add an optional omitted-when-empty receipt vector; old
histories, digest bytes and no-authority paths retain the absent representation.
Existing literals receive only an empty vector, explicitly inventoried; preserve
every existing assertion, test behavior, fixture and content asset.

Only the actual entered FinishSpell producer can retain a record, immediately
before ordinary live cast removal, and only when activated Grapple attack reads
refer to that exact cast occurrence. This does not create another cast queue,
payment, source grant, material choice or executable reservation. The complete
record lives only as long as its consuming tactical resolution.

The domain and rules readers use the uniquely matching live or completed record.
Keep full source program, actor/source profile, ordered target occurrence,
completed occurrence and cast-origin checks. Reject duplicate live/completed IDs,
duplicate receipts, incomplete or unreferenced archives, future/noncausal producer
metadata, wrong FinishSpell work, lingering executable work and old/unactivated
authority. Original-anchor detection must see the receipt even without a live
grip. Include its command origins in strict restore and preserve original replay
of all producer transitions and every snapshot; snapshots never mint authority.

Keep live casting partition/cleanup, costs, slots, phases and old executor
semantics unchanged. Preserve the existing per-ray admission distinction and
actual saved transport bindings. Review bounded memory/shape accounting for the
new receipt vector; the existing maximum tactical cast count is its upper bound.

## Verification and allocation boundaries

Read the complete resulting diff and enumerate every absent-field constructor
migration. Prove all other baseline Git entries, all old test bodies after only
those literal insertions, all historical fixtures, source pins, frontend files
and locks unchanged. Direct formatting/check on changed Rust files is allowed.
No Cargo, compiler, Clippy, npm, tests, DB, native, push or merge is allocated.

The existing three-ray case is the primary failing regression and stays intact.
Add new tests only for a substantive receipt/hostile-boundary gap, with genuine
owned producer histories for positive application setup. Synthetic shape probes
must be explicitly negative/unit checks, not claimed native or application proof.
Runtime acceptance must rerun the exact failure, inherited public harness,
protected legacy captures/replay, affected rules/domain suites, strict lint,
canonical verification and exact-head Linux/Windows CI. Root owns the sole heavy
slot, fresh independent review and publication schedule. All runtime remains
UNRUN until actual logs on the corrected head establish otherwise.

Next: commit this plan before production changes; implement the reviewed bounded
retention, freeze a clean source audit and return it for independent root review.
