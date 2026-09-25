# Gate 4 source creatures through private table setup

Writer: root. Prospective branch: `codex/gate4-creature-table`, from refreshed main
after PR29 equipment preparation is verified and merged. Integration reference:
`cd3ba7a`; existing source and table plans remain the detailed implementation history.

## Objective and boundaries

Expose source-derived NPC creation through the actual host table, with canonical
statistics, explicit allowed size/languages, actual gear identities and ammunition,
private setup views, exact uncertain retries and durable restore. This advances the
product's source fidelity, physical equipment, privacy and recovery requirements and
Gate4's source creature participation. AGENTS, product definition, Gate4, ADR024/026
and pinned SRD5.2.1 govern the work.

Extract the already implemented creature profile/limited-use definitions and physical
gear boundary, optional source attachment and validation, typed CreateCreature action,
private host setup projection and form. Preserve explicit selected-feature omissions.
Do not copy the shared tactical queue, casting, battlefield/initiative controls, effects
or vitality attachment into this slice. Listing a source feature does not establish its
playable execution. NPC policy helpers alone do not satisfy autonomous encounter behavior.

## Required work and acceptance

1. Reconcile current main after PR29; inspect the complete dependency closure of
   tactical_creatures, tactical_creature_equipment and the table adapter before copying.
   Retain source profile identity and real physical armor/custody validation.
2. Add optional/default/omitted source attachment and constructor defaults. Extend the
   legacy save preflight, typed duplicate rejection, original-anchor check and exact
   command-origin audit without changing historical SQL migrations.
3. Extract host/session/pending setup checks, canonical source choice projection,
   actual gear identity retention, private creation transcript and desktop form.
4. Port the existing real runtime create/retry/export/replay and private-view tests.
   Exercise independent file reopen, malformed source/gear and forged source origins,
   preserving atomic failure. Verify selected-feature limitations remain explicit.
5. Run focused checks, frontend tests/check/build and canonical verification; obtain
   independent full-diff review, final-head Linux/Windows CI and expected-head merge.
   Check merged tree parity/post-merge CI, then reconcile the encounter integration.

## Status and next action

Planning only. PR29 remains under canonical/Windows verification. No extraction branch
or implementation exists yet. Next: merge verified PR29, refresh main and inspect the
source/profile/gear dependency closure to create the coherent implementation branch.
Do not infer this slice's evidence from the broader integration branch. Remaining
Gate4 NPC decisions, attack features, shared effects and packaged encounter acceptance
stay active; no Gate5 work or reduced gate acceptance is authorized.
