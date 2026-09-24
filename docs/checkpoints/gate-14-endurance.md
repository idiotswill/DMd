# Gate 14 — Endurance acceptance

Status: **Planned**

## Product requirements advanced

All finished-product requirements. This is the product-completeness gate, not a new subsystem gate.

## Objective

Prove the same production build can sustain a long, enjoyable, unscripted campaign and a second unrelated campaign without developer intervention.

## Entry conditions

Gates 1–13 accepted; no known critical blocker deferred into endurance.

## Acceptance criteria

The endurance campaign must exercise all existing product-definition requirements and the extended acceptance list, including:
- clean install/campaign/PC creation;
- Session Zero/table contract;
- multiple real sessions;
- natural speech + physical dice;
- combat/exploration/social/travel/downtime/economy/rest/advancement;
- repeated save/restart;
- divergence/ignored hooks;
- off-screen world motion;
- revisit changed areas;
- emergent situations after authored hooks are insufficient;
- PC death/retirement/replacement;
- hidden/private/incorrect information;
- rules questions without action commit;
- evidence-limited checks;
- new method/vantage revealing new evidence;
- enemy retreat/surrender/negotiation;
- tactical effects/visibility/environment;
- save/resume during combat;
- player-created persistent goals/projects;
- compressed travel/downtime with continuing simulation;
- durable clues/handouts;
- recurring NPC memory;
- player cosmetic narration;
- serious emotional scene without inappropriate humor;
- running joke continuity without overuse;
- settlement-constrained availability;
- meaningful failure without script rescue;
- abandonment of prepared adventure;
- repetition review;
- long-session performance;
- supported recovery/admin correction;
- second unrelated campaign with no leakage.

## Architecture invariants

No test harness, developer state surgery, manual quest feeding, secret prompt/source editing or external cloud dependency may substitute for the production product.

## Explicit non-goals

None. Failed acceptance creates concrete defects/debt that must be fixed and re-tested; it does not get reclassified as future work if required for DMd 1.0.

## Required failure/recovery behavior

Any failure discovered during endurance must preserve campaign integrity and be recoverable through supported product/admin paths. A failed acceptance scenario becomes a tracked blocker until fixed and re-tested on the production path.

## Merge/pause boundaries

Codex may merge fixes and release-readiness work needed to satisfy Gate 14 after exact-head verification. Codex may not waive a product-definition requirement or narrow the 1.0 finish line on its own merely to declare success. Gate 14 necessarily requires sustained human endurance evidence; pause with the complete final acceptance record and any unsatisfied criterion rather than manufacturing a pass.
## Candidate workstreams

Endurance test planning, playtest execution, telemetry/performance observation, defect correction PRs, final release review.

## Deferred requirements

Post-1.0 enhancements only, after all 1.0 requirements are genuinely satisfied or explicitly re-scoped by human approval.

## Open questions

Determine minimum duration/session count sufficient to expose drift/repetition/performance issues; prefer enough play that authored starting hooks are no longer sufficient.

## Production integration acceptance

A human group can regularly forget about the software and simply play. The same production build remains authoritative, recoverable, performant and coherent across the entire campaign. The test fails if continued meaningful play requires a developer to add quests, repair state, edit internal files/prompts/source or decide outcomes behind the system.
