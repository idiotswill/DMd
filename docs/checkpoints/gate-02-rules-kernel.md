# Gate 2 — Commercial fifth-edition rules kernel

Status: **Integrated implementation verified — closeout review and publication in progress**

## Product requirements advanced

- commercially distributable first rules pack;
- Rules Coverage Ledger and licensing/provenance;
- deterministic application-owned rules resolution;
- character mechanics and derived values;
- physical/digital dice integration;
- rules questions/adjudication hierarchy;
- primitives needed by tactical and noncombat gates.

## Objective

Establish a legally distributable, auditable and production-intended fifth-edition mechanics foundation. The gate must prove completeness accounting, not merely representative mechanics.

## Entry conditions

- Gate 1 accepted.
- roadmap/product expansion bootstrap merged.
- exact legal rules source/version verified from current official sources and recorded before content ingestion/implementation that depends on it.

## Acceptance criteria

At minimum:
- create the Rules Coverage Ledger from the exact selected legal source;
- account for every rules/content family with gate ownership;
- implement the core resolution model required by later gates;
- ability checks, saves, proficiency, advantage/disadvantage;
- attack/AC/damage/healing foundations;
- conditions/effects/resource expenditure/recovery primitives;
- initiative/timing/reaction/concentration primitives needed later;
- rest/resource primitives;
- spellcasting primitives;
- character-derived modifiers and validation;
- rule-driven RollRequest/Result/ResolvedRoll through existing dice boundary;
- explicit rules-query path that does not commit an action;
- campaign house-rule representation and ruling provenance;
- passive/secret resolution primitives;
- deterministic tests and invalid-action tests;
- no provider/LLM decides authoritative rules outcomes.

## Architecture invariants

- AI/provider output never authoritative.
- trusted issuer/actor separation remains.
- rules/content remains versioned and content-isolated.
- raw persistence remains content agnostic.
- rules behavior must be reproducible from authoritative inputs.
- legally non-reusable D&D material must not enter shipped content by convenience.

## Explicit non-goals

- complete tactical map/battlefield;
- autonomous DM;
- voice;
- living-world simulation;
- Asterra port;
- polished desktop UX;
- implementing every ledger row that properly belongs to Gates 4/5/6, but those rows must already be assigned.

## Required failure/recovery behavior

Invalid, impossible, stale, unauthorized or incompatible rules actions fail without authoritative mutation. Rules/content version mismatch fails closed through the runnable-campaign boundary. A failed provider/helper explanation cannot change state.

## Merge/pause boundaries

Codex may merge Gate 2 PRs, architecture records, and checkpoint updates autonomously after exact-head verification when they satisfy the approved Gate 2/product contract.

Do not silently narrow the product or claim proprietary/unlicensed material is distributable. If official licensing evidence is genuinely ambiguous enough to threaten commercial distribution and no clearly safe implementation exists, surface that as a real blocker.

Pause for the owner at the end of Gate 2 with the complete rules-source/provenance decision, merged PRs, verification evidence, ledger state, remaining debt, and Gate 3 handoff.

## Candidate workstreams

- legal rules-source/provenance inventory;
- Rules Coverage Ledger;
- character/rules kernel;
- effects/timing/resource model;
- rules query/adjudication model;
- integration with existing dice/app/persistence boundaries;
- mechanical + property/regression tests.

## Deferred requirements

Tactical geometry/combat completion (Gate 4), full noncombat systems (Gate 5), world/adventure content (Gate 6), autonomous DM (Gate 9), voice (Gate 10).

## Decisions resolving entry questions

- The official English SRD 5.2.1 (2025-05-01) is pinned under CC BY 4.0. See
  [provenance](../rules/srd-5.2.1-provenance.md) and the shipped
  [notice](../../content/srd-5.2.1/NOTICE.md). PDF SHA256:
  `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
- The ordinary-check/save natural 1/20 house rule is disabled by default and requires
  explicit campaign configuration. Source-defined attack/death-save extremes remain
  distinct. Trusted rulings retain source/house-rule/adjudication provenance.
- No proprietary book content is included. Gate 6 owns any original compatible content
  needed beyond the selected source; omission from this legal inventory does not grant
  permission to copy an unavailable work.

## Production integration acceptance

Through the real `dmd-app` runnable campaign path, create/load a representative legal campaign/character, execute core rules actions using the real dice boundary, durably commit results, restart/reopen and prove identical authoritative mechanics. Demonstrate invalid actions and content/version failures do not mutate state. The Rules Coverage Ledger must show no unassigned family.

## Integrated acceptance evidence

The following evidence uses the real `CampaignRuntime`, SQLite atomic journal and shipped
SRD content manifest. Test names below are in
[`crates/dmd-app/tests/rules_runtime.rs`](../../crates/dmd-app/tests/rules_runtime.rs)
unless another file is named. The [machine-readable ledger](../rules/rules-coverage-ledger.json)
provides the per-family pure-mechanics tests and exact production evidence.

| Gate criterion | Verified production evidence |
|---|---|
| Legal source and complete ownership | `pinned_source_has_complete_owned_inventory` in `crates/dmd-domain/tests/rules_coverage_ledger.rs`; real distributed pack loads in `distributed_rules_pack.rs`; 55 families, all source chapters/catalogs assigned. |
| Character validation, checks, saves, proficiency | `pending_physical_roll_survives_shutdown_and_replay_matches_committed_mechanics` checks saved request, raw 13 + derived 5 = 18 and command audit; `invalid_mechanical_initialization_and_stale_competing_action_leave_no_partial_writes` rejects invalid HP; concentration scenario covers a saved saving throw. |
| Advantage, Inspiration, passives, house rules | `inspiration_advantage_passive_queries_and_house_rulings_are_durable` checks cancellation, two raw dice, actual one-die replacement/resource consumption, passive answers and optional natural-extreme behavior; export/restore/replay agree. |
| Attack, AC, critical damage, HP/healing | `initiative_attack_damage_reaction_and_effect_timing_use_the_durable_path` authorizes context, resolves a critical, restarts with pending damage and applies the expected 8 damage; spell/rest scenario checks healing; dead-character scenario checks read/action lifecycle distinction. |
| Conditions, effects and resources | Initiative scenario checks condition/ruling provenance and turn expiry; Inspiration scenario checks timed effects; spell/rest scenario checks resource expenditure/recovery. Source-specific variants are mechanically tested in `crates/dmd-rules/tests/mechanics.rs`. |
| Initiative, timing, actions, bonus actions, reactions | Initiative scenario checks ordered rolls, repeated-budget rejection, turn expiry/reset, six-second round advancement and combat end through committed actions. |
| Spellcasting/concentration and rest primitives | `spells_concentration_damage_healing_and_rests_survive_reopen_and_replay` casts from typed content, restarts a concentration save, checks failure, physical healing, slot cost/recovery, short-rest hit dice and long-rest recovery; full sequence exports/restores/replays. |
| Physical/digital dice and secret visibility | Pending physical scenario and `secret_digital_rolls_do_not_leak_and_replay_never_rerolls` use the existing raw-face boundary; wrong issuer/actor rejected, players cannot query the secret request, repeated replay preserves exact faces. |
| Questions do not become actions | `rejected_rolls_and_questions_leave_durable_state_and_history_unchanged` compares authoritative state, sequence, audit and journal before/after queries and invalid actions. Structured query/resolution contains no provider call; later explanation adapters cannot commit from prose. |
| Invalid/stale/unauthorized/content failures | Rejection tests cover pending IDs, faces, ownership, stale competitors, malformed initialization and cross-campaign isolation; `persistence_session_rejection_preserves_resolved_rules_state_and_history` proves transaction rejection after valid resolution; content/version tests fail before gameplay writes. |
| Recovery, compatibility and hostile exports | Pending export test, `legacy_rules_campaign_restore_upgrades_before_mechanical_preflight`, 11 semantic-corruption variants, nine replay-envelope variants and six restore-helper tests validate snapshots/audits/outcomes before destination writes. Five persistence compatibility tests retain immutable schema-1 anchors. |
| Content handoff across database waits | `create_and_restore_reuse_preflight_content_after_waiting_for_database` deterministically changes kernel bytes after preflight, proves the committed operation returns success, and proves the next open rejects changed content. |

The production tests create campaign entities and validated imported mechanical sheets
through the supported host API. That is the Gate 2 application boundary; it is not the
normal-user character creation/desktop workflow owned by Gate 3. Broad runtime scenarios
check explicit numeric expectations and real database state, then reopen, export/restore
and replay. Pure kernel equality alone is not the integration evidence.

## Merge and verification record

| Slice | Reviewed exact head | Merge / CI evidence |
|---|---|---|
| Roadmap bootstrap [PR #15](https://github.com/idiotswill/DMd/pull/15) | `5a6aab55aa83a156c48feeab45a9c634f9b78f78` | `414040b33d58701cec81e6d73b791347c6d43ca1`; [CI #297](https://github.com/idiotswill/DMd/actions/runs/35995428478) |
| Source and ledger [PR #16](https://github.com/idiotswill/DMd/pull/16) | `bbf4d78f052bb9d6ec35553dc5a020ea1021defd` | `a0fb8762f3256e384c9bd1ee1e20b0d4acd4d4c3`; [CI #300](https://github.com/idiotswill/DMd/actions/runs/35998593520) |
| Kernel/schema foundation [PR #17](https://github.com/idiotswill/DMd/pull/17) | `ca54b8215258468db4fbac7b5c15a648fdbb98d7` | `f3222e72b970708517965306dc5bfffbe74eb414`; [CI #303](https://github.com/idiotswill/DMd/actions/runs/36002412992); 151 Windows tests |
| Application/recovery [PR #18](https://github.com/idiotswill/DMd/pull/18) | `3346699d5b8047ad5232199c4ad1c2c3e8d5c72c` | `3ad3885559073e6748d3f17c2172be9ff2a99f52`; [CI #305](https://github.com/idiotswill/DMd/actions/runs/36003158276); 172 Windows / 173 Linux tests |

Each implementation head received complete/delta independent review and passed
`./scripts/verify` locally plus all four CI jobs: Rust formatting/check/Clippy/tests,
Rust 1.88 MSRV, genericity guard and architecture guard. The final merged implementation
`3ad3885559073e6748d3f17c2172be9ff2a99f52` also passed local full verification with
172 tests and [CI #306](https://github.com/idiotswill/DMd/actions/runs/36003489271).
The closeout adds one ledger evidence-regression test, without changing
runtime code. Its PR records the final exact-head checks and merged-main follow-up.
Earlier formatting/Clippy failures were read, fixed and rerun; no failed head is acceptance
evidence. Tests exercise Windows locally and Linux in CI; this is correctness evidence,
not reference-laptop performance acceptance.

## Architecture and precise scope

- ADR 017: schema 2 current-state upgrade with immutable historical snapshots.
- ADR 018: deterministic typed kernel, raw dice, one-use trusted context, explicit rulings
  and semantic replay; no provider, SQL or RNG in the kernel.
- ADR 019: trusted issuer separated from actor/proposal, one atomic state/event/audit
  commit, version-checked content and read-only queries.
- ADR 020: restore checks all versioned snapshots and rules lineage, binds command
  provenance and replays from the earliest actual anchor before installation.

Gate 2 delivers limited executable definitions: Club, Dagger and Shortbow single attacks;
Cure Wounds healing/upcasting; Fire Bolt creature attack damage/scaling; Dancing Lights
concentration/duration only. Typed support metadata identifies the partial spell behavior.
Geometry, visibility, all condition clauses, complete combat actions and combat spells
remain Gate 4; exploration, full rest scheduling, ritual/long casting and noncombat spells
remain Gate 5; complete legal option/catalog coverage remains Gate 6. A concentration
marker does not implement lighting. No later requirement is accepted by this checkpoint.

## Debt and external acceptance

All retained debt is recorded in [the debt ledger](../tech-debt.md):

| Debt | Receiving work |
|---|---|
| TD-001: replay semantics for future event families | Each owning gate, especially 3–10; Gate 2 rules events now covered |
| TD-002/006/007: whole images, snapshot cadence, contention, retained mechanics history | Measurements in Gates 7/13 and endurance in 14 |
| TD-003: corruption checks/earliest anchor do not prove authenticity | Gate 13 content/recovery trust policy |
| TD-004: trusted runtime needs player-facing adapters | Gate 3, extended for Gates 9/10/12 |
| TD-005: repeated full content verification | Gates 6/13 volume measurements |
| TD-008: inherited lifecycle readback can fail after a committed installation | Gate 13 acknowledgement/reconciliation contract |

Gate 2 requires no physical-human or reference-hardware acceptance beyond its technical
scope. No playability/enjoyability or laptop-performance result is claimed. The mandatory
four-human vertical slice after Gate 6, reference-machine evidence at Gates 10/13/14 and
the sustained human campaign at Gate 14 remain required.

## Gate 3 handoff

On owner continuation, create a fresh branch from verified main for the first playable
desktop table loop: campaign/session shell, Session Zero, supported PC creation/sheets and
player binding, trusted text/physical-dice adapters, player-safe transcript/recap, correction
before commit and exact pending-decision restart. Route gameplay through `CampaignRuntime`;
never accept issuer identity from interpreted prose or expose raw secret state. Gate 3 is
planned and has not begun.
