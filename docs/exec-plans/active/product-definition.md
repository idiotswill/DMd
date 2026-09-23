# Product definition hardening

Status: in progress
Branch: `lab0/product-definition`
PR: pending
Base: `lab0/dev-lab` @ `1b393f7f3c4e6cf7f5634276835eaa5a86339338`
Verified head: pending

## Objective

Make the repository unambiguous that DMd must become a fully functioning, production-intended tabletop RPG application with a causally evolving living world, not a prototype, proof of concept, framework, scripted campaign demo, or collection of disconnected subsystems.

## Scope

- add a durable product-definition contract;
- encode product-completeness and living-world invariants in `AGENTS.md`;
- link the product contract from the README/repository map;
- define final end-to-end and endurance acceptance, including autonomous world motion and emergent situations/rumors/leads/opportunities;
- forbid throwaway/mock/demo paths from satisfying production gate acceptance when real product behavior is required.

## Non-goals

- implement gameplay systems;
- implement world simulation/Director/procedural generation;
- change Gate 0 domain semantics;
- merge Gate 0 or Lab 0.

## Relevant durable context

- `AGENTS.md`
- `README.md`
- `docs/checkpoints/gate-0.md`
- `docs/architecture/`

## Acceptance criteria

- [x] Repository explicitly states that DMd's target is a complete installable/playable game, not a prototype or proof of concept.
- [x] Completion is defined in player-facing/end-to-end terms, not only architecture/API/test terms.
- [x] Living-world forward motion is a hard product requirement.
- [x] New rumors, leads, requests, conflicts, discoveries, quests/opportunities, and situations must be able to emerge causally from changing world state rather than only scripted content.
- [x] Off-screen actors/processes can advance when justified; the world does not freeze awaiting player triggers.
- [x] The Director may surface/select meaningful developments but may not invent unsupported world truth merely for pacing.
- [x] A campaign remains capable of meaningful new play after authored starting material is exhausted.
- [x] Endurance acceptance covers multi-session persistence, unscripted divergence, PC death/replacement, prolonged in-world time, explainable world changes, and a second unrelated campaign.
- [x] Experiments/prototypes are permitted only as isolated research; production gates cannot be satisfied by throwaway paths.
- [ ] Final branch passes CI on the exact head.

## Planned slices

1. Add product definition and final/endurance acceptance contract. — complete
2. Strengthen `AGENTS.md` and README navigation. — complete
3. Inspect complete diff, verify CI, update PR/plan. — in progress

## Decision log

- 2026-09-23 — Product completeness and living-world motion are product invariants, not implementation details.
- 2026-09-23 — No numbered future gate document is added yet; `docs/product-definition.md` is the cross-gate contract and later gates must trace acceptance back to it.
- 2026-09-23 — Quests are not required to originate as scripted quest records; player-facing quests/leads may emerge from simulated situations and information propagation.
- 2026-09-23 — Code/tests define current implementation truth; the product definition defines the required end-state. An incomplete build cannot redefine the finish line.

## Validation

- `./scripts/verify-fast` — pending
- `./scripts/verify` — pending
- CI — pending

## Risks / blockers

- Product acceptance is intentionally demanding; later gate plans must decompose it without weakening the end-state contract.
- Some living-world mechanisms remain architecturally deferred; this document defines required behavior, not a premature implementation schema.

## Next action

Open a stacked PR against `lab0/dev-lab`, inspect the complete diff, and verify CI on the exact head.
