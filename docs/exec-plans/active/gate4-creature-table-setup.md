# Gate 4 source creature setup at the table

Status: active. Writer root on `codex/gate4-encounter-execution`; source reducers are
developed and reviewed separately. This integration follows ADR026, Gate4 encounter
acceptance and the product's real inventory, hidden information and exact suspension
requirements. No complete encounter claim is made before packaged acceptance.

## Bounded objective

Create source-pinned NPCs and their real equipment through a host table command, place
them beside PCs on the map, and include them in source-derived initiative. Creation,
physical ItemIds, source profile and current loadout must commit atomically and replay
once. Players cannot submit arbitrary monster mechanics, hidden truth or source grants.

## Decisions and acceptance

- The host selects a source definition, public name, source-permitted size/languages and
  initial placement. Source average HP is permitted; source HP rolling can use the same
  pending raw-dice path when exposed. Statistics never come from UI totals.
- Only Gear entries create ordinary recoverable source equipment (SRD255); an intrinsic
  stat-block attack does not become loot. Necessary ammunition and required spell
  materials are real carried ItemIds (255/257). The host declares a finite initial ammo
  quantity because the source does not specify one; it is not an infinite refill.
- Specified spell materials use exact canonical source IDs mapped to their spell and
  source cost. Human item names, inferred caster class and a generic NPC exemption do
  not prove component access. Actual custody, intact state and hands govern later use.
- Source armor/shield equipment must affect live AC on loss, while immutable source
  profile validation continues proving intrinsic statistics. Do not retain a frozen
  stat-block AC after dropping a contributing shield or remove natural armor by accident.
- Actor creation rejects duplicate IDs, retained items, missing source choices and
  mid-resolution setup without partial mutation. Source changes cannot silently alter
  accepted profile fingerprints or replenish resources.
- Only host views receive the full NPC catalog/preparation and private actor data.
  Player maps and transcript retain the existing knowledge boundary. Controlled
  companions require an explicitly selected authorized actor, never automatic merging
  of their senses into the PC's knowledge.

## Planned verification and next action

Add real SQLite tests for host/player authority, source construction with physical gear,
repeat delivery, save/export/restore, failed partial creation and filtered map/initiative.
Exercise source shield loss and material/ammo exhaustion through the shared reducers.
Check frontend source choices, retained identities and channel switching. Then include
these flows in full verification and the packaged multi-round Gate4 encounter.

Implemented working integration: atomic host source creation; finite ammunition and
required physical spell materials; source attack implement IDs; live armor/shield AC;
host catalog/preparation, NPC map placement and explicit team/geometry choices; source
initiative grouping with authoritative modifier/mode previews; durable turn ordering,
Legendary Resistance decisions and after-turn decline controls. No source-feature
payload is executed merely by showing those controls.

Independent backend review found two defects, now addressed: a visible NPC could expose
its private world name, so placement requires a separate public descriptor; unheld NPC
weapons/ammunition need the same source quantity/state checks as PC equipment, so all
recognized physical definitions are validated. The SQLite regression now exercises
creation/retry, player rejection, malformed unheld items, restore without regranting,
visible public descriptor privacy and restart during the NPC's secret initiative roll.

The initial compile caught a spell source-page field mismatch and test fixture mutability;
these were corrected without changing assertions. The complete NPC SQLite regression
now passes on the default thread stack after splitting its large async test body into
independent creation/replay and placement/initiative/replay stages. All 21 frontend tests
pass, including actual TableApp uncertain delivery/restart with retained source/ammo/item
identities. Svelte reports zero errors/warnings. A prior production build passed before
the final source-coverage/disabled-placement refinements; its final rerun remains pending.
Independent UI review caught required fields on excluded placements and missing source
omissions; both are corrected, with an exclusion regression. Full verification and
independent final diff review remain pending.

The broader old table suite exposes default-stack pressure as the inline tactical cursor
grows. A debugger backtrace and object-code frame measurements distinguished this from
recursion or catalog initialization. The large legacy async test caller alone reserves
over 1 MB. A separate wire-transparent heap allocation for TacticalFlow.resolution is
being evaluated to keep growing interruption state out of every CampaignState stack
copy. Do not claim the broader Rust suite green until that change is tested.

Memory exhaustion interrupted a separate spell build, including later shell reads. The
owner freed memory; the spell slice then passed all 24 tests and strict Clippy. This is
recorded as an environment interruption, not a successful test or a waived requirement.

Next: finish focused Rust tests and strict lint, review the final UI/backend fixes, and
checkpoint this table integration. Compose actual weapon damage and involuntary NPC
shield drop through the shared resolver (the current isolated armor query is insufficient
evidence). Source casting/attacks, controlled companions, lair selection and the complete
encounter remain active Gate4 work. PR25 merged as `da96486`; refresh this integration
onto it after the local checkpoint and retain exact-main CI evidence separately.

PR25's merged `da964869bfc3ebf61b7b1a46d9204f0ff55c6693` tree exactly matches its
verified head `8cdb0e7`. Post-merge [Linux CI](https://github.com/idiotswill/DMd/actions/runs/36059278204)
and [Windows desktop](https://github.com/idiotswill/DMd/actions/runs/36059277987) both
completed successfully. This is reducer-slice evidence, not full Gate4 acceptance.
