# Gate 11 — Asterra production world port

Status: **Planned**

## Product requirements advanced

- first substantial real world;
- generic engine/content proof;
- authored + procedural + living simulation interaction;
- real messy play patterns from Asterra reference experience.

## Objective

Port Asterra into production content/world architecture as the first substantial real setting without adding Asterra-specific engine assumptions.

## Entry conditions

Gates 2–10 accepted. Asterra source/content licensing/provenance reviewed for intended distribution/use.

## Acceptance criteria

Prove:
- Asterra represented as content/world packs;
- engine remains generic;
- independent Asterra campaigns diverge safely;
- authored and procedural material coexist;
- important relationships/locations/factions are representable;
- living simulation acts on Asterra entities;
- autonomous DM handles investigation, uncertainty, PvP, evidence, NPC negotiation, split scenes, travel, ridiculous plans, running jokes and player-created activities similar to observed real sessions;
- any private/non-distributable source material is handled according to explicit policy.

## Architecture invariants

No Asterra name/ID/setting logic in generic engine crates. Content does not gain authority over core validation. Licensing boundary explicit.

## Explicit non-goals

Phone companion, release packaging finalization.

## Required failure/recovery behavior

Missing/corrupt/incompatible Asterra packs fail through existing content boundary. Port errors cannot damage unrelated campaigns.

## Merge/pause boundaries

Codex may merge in-scope Gate 11 content/architecture work autonomously after provenance/licensing and exact-head verification. Asterra must remain content rather than an engine assumption. Pause at the end of Gate 11 with genericity/content evidence and Gate 12 handoff.
## Candidate workstreams

Content conversion/authoring, world validation, simulation hooks, autonomous-DM playtests, genericity regressions.

## Deferred requirements

Phone Gate 12; release hardening Gate 13; endurance Gate 14.

## Open questions

Decide what portion of Asterra is distributable versus private/test-only and create original replacements where necessary.

## Production integration acceptance

Run multiple sessions in Asterra through the same production executable while also maintaining a second unrelated campaign. Demonstrate no setting leakage into engine/other campaign and that unexpected player-created activity remains supported.
