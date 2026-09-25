# Gate 4 source creatures through private table setup

Writer: root. Branch: `codex/gate4-creature-table`, from refreshed main
`ac35c1d65d27239f704cde1422209a0f106fb0e1` after verified PR29 merge. Integration reference:
`220ee804d7344564a265f02c4d42cb57a7e560c2`; existing source and table plans remain the detailed implementation history.

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
   Preserve the existing PC check/Second Wind/character creation path after NPC setup:
   do not copy the integrated branch's blanket legacy-action rejection for source
   creatures when this slice has not yet shipped the replacement tactical path.
4. Port the existing real runtime create/retry/export/replay and private-view tests.
   Exercise independent file reopen, malformed source/gear and forged source origins,
   preserving atomic failure. Verify selected-feature limitations remain explicit.
   Include an ordinary PC table check after NPC preparation to prevent a regression
   in the already shipped table loop.
5. Run focused checks, frontend tests/check/build and canonical verification; obtain
   independent full-diff review, final-head Linux/Windows CI and expected-head merge.
   Check merged tree parity/post-merge CI, then reconcile the encounter integration.

## Status and next action

The fresh branch is created from the fetched PR29 merge. Its final-head checks and
exact merged tree parity passed; post-merge Linux and Windows MSRV are green and
Windows stable installer packaging is still running. The source profile, equipment,
limited-use validation and policy modules have no shared tactical-flow dependency.
Extract their coherent existing module closure; this does not claim that their actions
are exposed through the table. Source NPC prep remains host-only before battlefield
setup, and the existing PC action path stays available afterward.

Next: extract source attachment, setup command and private form, then port and extend
real persisted regressions. Validate this exact branch independently of the encounter
integration. Remaining Gate4 NPC decisions, attack features, shared effects and packaged
encounter acceptance stay active; no Gate5 work or reduced gate acceptance is authorized.
