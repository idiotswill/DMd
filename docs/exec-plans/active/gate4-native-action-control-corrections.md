# Gate 4 native action-control corrections

Status: plan before implementation, 2026-10-10. Root is the sole writer on
`codex/gate4-family-receiver`, draft PR #76. Freshly fetched baseline is
`f7b8d0ee8b695931970e531104c86742eac8c3ae`, tree
`2dd83dea568ad3c686d66266a6759dbb704e2e51`; accepted main remains
`c101c202a9d7302e5ac60d791d3d6c5a4ed30e84`. Gate 4 remains active.

## Observed problems and authority

The verified f7 portable package exposed two actionable presentation defects
during the combined family native ledger. Neither was exercised as an invalid
native command, and neither observation proves an extra Action or owner bypass.

1. At QA2 sequence 100, Dain withdrew the actual pending Grapple save and finished
   the owed equipment choice. His Action remained spent and attacks remaining
   zero, but fresh Grapple buttons were offered. Cora independently reached the
   same defect at sequence 106 after a real Glaive miss, Graze and after-equipment
   stow. The shared pure planner proposes a new AttackAction whenever remaining
   attacks are zero without checking Action affordability. The accepted producer
   still calls the ordinary budget guard, so the trace predicts refusal rather
   than a free attack. Fix the offer/admission discrepancy; retain that guard.
2. At sequence 105, Host could read Cora's actual after-attack equipment operation
   and saw enabled decision controls. Host read visibility is intentional, but
   the rules require the PC's actual controlling player to make this decision.
   Dara's unrelated view correctly omitted the card. Preserve Host read access
   while making only this player-owned equipment form read-only for Host.

The external findings have SHA256
`afe21f17ad62a0f21880092471fb585a60b2cb8dbc86839b564d10fb77390e54`
and `521e244ad69a76312e992b2f4fb07c9b36a9d1fdb4852ec86d27f7bb59a0aef8`.
Both belong to current PR76 integration, not the recent test-only correction.
The exact source, UI observations and copied campaign evidence remain outside
the repository under `tooling/ci-oct10` and the native operator journal.

This advances the product definition's usable production path, action economy,
player agency and exact suspension; the Gate4 tactical checkpoint; ADR020
original-history integrity, ADR024 trusted table commands, and ADR028 owned work.
The family receiver and Ground/Grapple plans retain their full acceptance scope.
No rules source, save format, DTO, issuer policy or gameplay cost is redesigned.

## Bounded implementation and regression criteria

- In `grapple/admission.rs::plan_attempt_with_read`, reject a proposed new Attack
  Action if timing already records its Action spent. Keep the separate branch
  for paid remaining attacks, all ordinary target/hand checks and free Release.
- Extend the existing public withdrawal regression through actual equipment
  Finish and cold reopen. Require no fresh Attempt, zero attacks, the exact paid
  budget/timing, canceled original request and unchanged roll receipts. Advance
  two real EndTurns and cold-accept a legal new Grapple on the next owned turn.
  Keep all existing assertions and the established spent-action Release controls.
- In `EncounterPanel.svelte`, disable only Host's player-owned PC after-equipment
  form and explain that the player must choose. Preserve its readable operation
  labels, original handle, Host-controlled NPC decisions and existing filtering
  for player-controlled source creatures. Do not broaden the shared controls
  predicate or change the rules/transport authority.
- Add frontend regressions for Host PC read-only/no submitted callback, actual
  owner Apply, unrelated absence, Host NPC Decline and owned-source filtering.
  Preserve all existing tests. Expected Rust test-name counts are unchanged;
  frontend increases by two to 298 in 37 files, subject to actual discovery.

## Preservation and verification allocation

The f7 native run stopped at a settled real boundary: QA2 sequence 106, Cora's
round2 turn13, Glaive put away, incoming grips retained. A normal close removed
PID12080; both the native window list and independent process query found no
DMd process. Closed capture SHA256:
`e24d86e1b171c83264d6000789723f71e86c76b2e56c7351007ca813aeb9be67`.
Root archived the exact owned native lock after all captures stopped. Preserve
all nine campaigns, old packages, captures, logs, runners and failed attempts.
Never rewind the live campaign, replay a copied paid image as new play, or edit
its database to arrange the new checks.

Root reviews the complete proposed delta, commits this plan first, implements
only the four named code/test files, and obtains a fresh independent review.
Review the complete final diff and preservation against f7 before freezing.
The two external patches are proposals only, not tested source.

This explicitly amends the capacity plan's seven preliminary diagnostic stages:
run the changed exact withdrawal case first, then unchanged full canonical
verification, all 27 CI Python controls and ordinary frontend install/check/test/
build. The full canonical run still covers every earlier diagnostic case, so
there is no exclusion, changed oracle or partial suite. `scripts/verify` includes
`verify-fast`; a second identical fast run is unnecessary. No test profile,
stack, threading, fixture history, CI allocation or accepted guard is altered.

Prepare a separately reviewed runner and manifest bound to the new clean head,
actual source inventory, fresh absent target and new output paths. Keep GNU
1.98.1, jobs1/incremental0, the committed opt1 test profile with debug assertions
and overflow checks, original executors/configuration guards and one exclusive
local heavy allocation. Only the independently controlled Python split-outcome
parser correction may enter the successor runner; retain its original rejected
receipt and positive/negative controls. Never modify or resume the f7 runner.

Publish the reviewed head for fresh hosted prerequisites, all seven allocations
per platform, complete Linux/MSVC unions, MSRV/lint/source guards and ordinary
Windows release/NSIS packaging. Bind actual named cases, compiler graphs, Linux
synthetic parents/tree and Windows literal head. Expected catalogues remain
GNU1150/Linux1151/MSVC1153 across 58 executable targets plus six doctest groups;
fresh evidence must establish the actual counts and outcomes.

The new verified package must satisfy the combined inherited native ledger and
Finished addendum. Reopen the preserved settled106 boundary first and confirm
exact campaign persistence plus absence of unaffordable Attempt offers. Verify
fresh next-turn admission and free Release, and a genuine later PC after-equipment
choice with Host read-only, owner Apply/Decline and unrelated absence. Resume the
remaining production-path obligations using the corrected package. Earlier f7
captures remain explicitly attributed to f7 and cannot be relabeled as runs of
the new package or a completed native ledger. Record any further required
revalidation explicitly; no acceptance criterion is waived by this correction.

## Risks, limits and next action

Risks are hiding legitimate paid attack slots/free Release, suppressing Host NPC
decisions, stale owner selection, and confusing visible Host information with
decision authority. The regressions and targeted native cuts distinguish them.
Unrelated source/content/corpus/schema/workflow files must remain exact.

The earlier f7 canonical, hosted and packaging evidence remains historical; the
new production change invalidates a docs-only closeout. Both findings are OPEN
until corrected-head checks and native witnesses pass. The combined family,
separate human physical-dice encounter and whole Gate4 acceptance remain pending.
Gate5 is not allocated.

Next: independent plan/proposal review, then the bounded implementation and
fresh source-bound verification. No routine owner approval is needed.

## Authored correction checkpoint

Plan-first commit `d9548f76` preceded all four code/test edits. Independent
proposal review found no actionable issue; report SHA256
`f35a015dbfe97f987238250e230c04dfd2d7e1e80d3ac194f559caa1a0e761d2`.
Root read that review, applied both complete bounded patches and formatted only
the two edited Rust files with the configured standalone formatter. Full source
diff review and whitespace checks pass; the formatter only expanded new chains.
All original test statements remain in order. No compiler or test result is
claimed yet. Exact frozen-diff review and all verification above remain required.

The final f7 copied Graze/equipment audit has SHA256
`2edd21ca41cc66306f0b9c7a9292639028bc2a861eca1fc6d2fa7c5e9d44352e`.
It confirms all nine campaign images across pending/closed/reopened105 and
completed/closed106, one original miss, Graze44-to41 and one owner stow. The
Host observation specifically had a settable selector and enabled Decline;
Apply was disabled before selection. No unsupported command was attempted.
Both findings remain open until corrected-package verification.

Next: freeze/review this correction, bind the new runner and publish for fresh
hosted checks. Native remains closed at106; preserve it for corrected-package
reopen. Recheck memory/process/disk headroom before allocating local compilation.
