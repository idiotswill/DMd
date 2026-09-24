# Gate 4 — Starting equipment materialization

Status: **Active; pure inventory slice, production integration pending.**

## Objective and branch

On `codex/gate4-equipment-materialization`, base `580f487`, materialize validated
CharacterProfile starting equipment into stable physical ItemIds exactly once. Retain
immutable provisioning receipts outside encounter lifetime; use live inventory for
custody, quantity and current loadout. Root owns RulesState attachment, versioned
application events, compatibility guards, armor/AC integration and SQLite acceptance.

## Scope and authority

Own new domain/rules `tactical_inventory` modules, exports, focused tests and this plan.
Consume the shared `WeaponLoadout` contract from reviewed type commit `e9f0d8`; do not
edit that module, existing character creation, weapon rules, app, flow or kernel.

Relevant contracts: Gate 4 checkpoint/active plan; product definition inventory,
ownership/custody, provenance and exact suspension clauses; ADRs 002, 009–012, 018,
021 and 025. Primary source is pinned SRD 5.2.1: starting equipment pp19–22/47/83,
weapon identities pp89–92, armor/shields p92 and equipment/ammunition pp94–96.

CharacterProfile remains immutable creation evidence, including equipment, worn armor,
shield and starting money. No current expenditure, drop, borrowing or destruction
rewrites it. ItemInstance is the sole current physical/custody truth.

## Interfaces and decisions

- `starting_equipment_plan` derives source IDs, quantities, allocation order and initial
  loadout requirements from the validated profile and pinned source registry.
- `materialize_starting_equipment` accepts only new caller-supplied ItemIds alongside
  trusted CommandMeta/character identity. It returns a sequence-neutral campaign copy,
  inventory extension and immutable receipt; no random generation or partial mutation.
- `validate_tactical_inventory` reconstructs source grants and checks retained receipt,
  profile evidence, pin, allocations, live item/tombstone identities and current loadout.
- Weapons are separate quantity-one identities. Canonical ammo stacks distinguish
  arrows, bolts, needles, firearm bullets and sling bullets. Registry categories come
  from exact pinned source IDs, never human display-name parsing. Other supported
  equipment has an explicit stackability/category mapping.
- Current loadout references ItemIds. Intact items in actual custody may be borrowed;
  ownership is not permission to ignore physical custody. Initial shield/armor choices
  are source-derived; the initial shield occupies Left and armor uses the first matching
  canonical allocation. No weapon is silently selected into the free hand. Unknown
  campaign objects can occupy hands for subsequent improvised-object adjudication;
  this does not grant source weapon, armor or shield mechanics. Holding a single object
  in both hands requires quantity one; attack grip legality belongs to the weapon planner.
- Receipt presence decides whether starting equipment was granted. Spent/Destroyed/
  Missing items remain referenced tombstones, never a reason to grant again. Removal of
  a referenced item is corruption, not an invitation to rebuild it. Buying no equipment
  still creates an empty durable receipt. Exact same original inputs re-resolve identically;
  applying another grant to the resulting state returns typed `AlreadyProvisioned`.
  Application recovery returns a previously accepted receipt before calling the reducer.
- With no receipt, potentially matching existing gear linked by custody/ownership
  causes a typed reconciliation-required error. This allocator never silently adopts
  or duplicates imported items. An explicit privileged reconciliation event is a
  separate application operation; it is not inferred from names or empty inventory.

## Acceptance and planned slices

1. Serializable receipts, source allocation records and current actor loadout references.
2. Strict source registry and deterministic pure planning/materialization/validation.
3. Tests for exact source grants, multiple distinct weapons, canonical ammo stacks,
   immutable evidence, borrowing, consumption/tombstones, collisions, stale/foreign
   metadata, wrong counts, repeat grants, imported ambiguity and corrupted receipts.
4. Independent read-only review; focused tests + strict Clippy in parent's serialized
   build slot. Root runs canonical verify-fast/verify and SQLite/replay/UI acceptance
   after integrating this module into the real application path.

## Risks and non-goals

This slice does not implement economy, containers UX, equipment timing, attack execution,
armor recalculation, currency migration or a new creation catalog. Source-reviewed
weapon registry entries do not make them new starter-shop purchases. Legacy v1 creation
replay remains unchanged. Root must reject non-null new authority fields in all legacy
decode/SQL-migration paths before schema upgrade, and validate receipt origins against
journal/audit records; a self-consistent initial snapshot cannot authenticate history.

## Verification and next action

Shared type dependency inspected and cherry-picked. Domain contract/structural
validation committed at `4b30722`; `cargo check -p dmd-domain --locked --offline`
passed. Source registry, deterministic allocator and focused corruption/atomicity tests
are implemented. The serialized verification run passed:

- `cargo test -p dmd-rules --test tactical_inventory --locked --offline`: **11 passed**,
  zero failed/ignored. Tests exercise source completeness, two separate daggers, canonical
  ammo, empty grant, deterministic re-resolution, retained command/profile roundtrip,
  borrowing, custody/ownership transfer, all tombstone states, stale/foreign/colliding
  requests, contained imported ambiguity, unknown held objects and 16 corruption cases.
- `cargo clippy -p dmd-domain -p dmd-rules --all-targets --locked --offline -- -D warnings`:
  passed; owned Rust formatting and `git diff --check` passed.

Root's read-only review identified the unknown held-object gap; it is fixed with a
focused regression. No grant/provenance defect was found in that review. Independent
source review remains pending. The build slot was released to the weapon slice.

Next: integrate this reducer into root's versioned application event and live AC adapter,
run strict restore/replay validation there, and complete independent review. Root owns
canonical full verification and real SQLite/UI acceptance. This pure slice alone does
not complete Gate 4. No source/catalog expansion, legacy replay change or PR push here.
