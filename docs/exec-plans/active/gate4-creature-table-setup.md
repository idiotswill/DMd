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

Next: add the source equipment plan and atomic host creation command after creature
scheduler checkpoint `0f388c0`, then source NPC map placement and desktop controls.
Current source-casting, attack and turn continuation integration remains in progress.
