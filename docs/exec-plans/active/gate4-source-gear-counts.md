# Gate 4 — Fixed quantities in immutable creature gear

Status: plan-first checkpoint, 2026-09-30. No implementation or runtime evidence.
Root is the sole writer on `codex/gate4-source-gear-counts`, checkout
`gate4-source-gear-counts`. No PR yet. The local heavy verification slot remains
reserved by the release verifier; this branch must not start concurrent builds,
tests, database or native work.

## Objective and dependency boundary

Extend the existing source equipment producer so a fixed source count can create
several distinct physical weapons. This is a prerequisite for a complete Ogre
source: the pinned SRD5.2.1 page312 supplies one Greatclub and three Javelins.
Three Javelins must be three quantity-one item identities, not one invalid
quantity-three Individual item, recurring ammunition, or duplicated source IDs.

Starting development dependency is reviewed foundation
`fa4ea903dfe44333c2cefb1b1b1acc91ba160f8f`, tree
`5922405605b45d00a0e5993b1fd4e7e43df2d7b1`. It contains the additive source
registry and Goblin V2 needed by the later Ogre path. Fresh main remains
`d88a69232c0b9d7f44fa6d3a1437dfe5e18f56a7`; the foundation's published older
head is still completing verification. This is an explicitly tracked development
dependency, not accepted main. Never merge a stacked PR into its development base.
Reconcile normally with the accepted prerequisite before final acceptance.

Read root AGENTS, product-definition, Gate4 checkpoint, gate execution protocol,
ADR024 and ADR029. The product requirements advanced are source-faithful physical
equipment, exact identity/custody, ordinary creation, authoritative resource
changes and save/retry continuity. This prerequisite alone does not complete a
new source or a player feature. Complete Ogre attacks/OA/UI, genuine Grapple/LR,
the remaining Gate4 mechanics and later product gates remain required.

Design input: external `tooling/ogre-source-preflight-2026-09-30.md`, SHA256
`5019edbbbc997a8b44dac333fbe96a308493e84fe867654501e40710accf5b66`.
Root read the full memo and the pinned source text on pages255/312. Independent
preflight review is in progress; resolve its findings before implementation.

## Scope and canonical representation

Add `CreatureStatistics.gear_quantities: BTreeMap<String, u32>`, default empty
and omitted on serialization when empty. Existing unique `gear` IDs remain the
membership list. Omitted/empty quantities retain exactly the old count-one
meaning and serialized fingerprints. Only counts at least two belong in the map;
reject zero and redundant one. Every key must occur in the unique gear list.
Reject duplicate JSON keys in this new map instead of silently keeping the last
value. Retain strict unknown-field handling and deterministic key ordering.

Do not modify V1 tactical bytes, frozen picker bytes, Air/Goblin source files,
old pins, current admission lists or original accepted histories. No Ogre file,
new source anatomy, printed-attack override, transport or UI producer is added
in this prerequisite. The later coherent Ogre slice must implement its complete
printed attacks, finite thrown gear, physical source OA and UI before admission.

Separate structural source validation from actual equipment classification:

- `CreatureDefinition::validate` can check map shape, count and gear membership
  using its supplied data. It must not call the global equipment registry.
- `equipment_registry::build_registry` calls `bundled_tactical_definitions`;
  calling it during that same OnceLock's definition validation would recursively
  initialize the lock. Preserve this acyclic initialization boundary.
- The existing post-load source equipment planner resolves every real equipment
  definition and its stacking/kind policy after immutable source lookup. Unknown
  source pins fail before count expansion. No caller supplies fixed counts.

For each source gear definition, Individual creates N allocations of quantity1;
Stack creates one allocation of quantityN. Order is exact definition ID, then
ordinal for distinct Individuals. Preserve every existing source's exact plan,
including ammunition and specified spell materials. Use checked size arithmetic
and fallible allocation where expansion requires it; do not introduce a new
arbitrary campaign/item lifetime cap or untrusted-source admission escape hatch.

Repeated Armor/Shield quantities must refuse at the authoritative planning
boundary in this bounded extension. The current materializer automatically
equips its single source armor/shield; iterating several would silently choose
the last identity. Supporting several requires a separately explicit equipped
selection and stowed-item policy. Count-one existing armor/shield behavior stays
exact. This limitation must be reported honestly for any future affected source;
it is not permission to omit printed gear from an admitted source.

Fixed source gear must not collide with generated ammunition or generated
`spell-material:` allocations. Detect the collision and reject before allocation
or state mutation; do not overwrite a fixed count, sum namespaces or treat a
Javelin as ammunition. Preserve existing legitimate deduplication among generated
requirements: two ammunition weapons can require one shared matching stack;
several source grants can require the same one physical material. Do not change
the existing host-selected ammunition policy or infer unspecified quantities.

## Production path and planned implementation

1. Commit this plan, resolve independent preflight comments, then implement one
   coherent source-schema/producer/test checkpoint. Keep one writer per branch.
2. Add the optional canonical field and scoped duplicate-key handling. Update
   literal Rust constructors as necessary without altering old serialized data.
3. Extend `tactical_creature_equipment::creature_equipment_plan_from_source` using
   the existing real equipment definitions. A small private pure planning helper
   may take the already resolved source/vocabulary for meaningful source tests;
   production public entry still resolves an immutable full pin first. No alternate
   materializer, runtime flag or test-only public admission is allowed.
4. Keep `materialize_creature_equipment` atomic and one-time. Its existing identity
   validation must compare against expanded allocation count. Every Individual
   remains intact, quantity1 and assigned to the actual new source actor, with
   the original creation command. No ID generation belongs in the rules layer.
5. Verify `table_creatures::current_catalog` derives item_count from the actual
   expanded plan; the existing desktop creation path allocates that many IDs.
   Existing sources must expose identical counts/DTOs and original retries. A
   new count-bearing source is deliberately not admitted by this prerequisite.
6. Review the complete diff, verify the exact final head, integrate accepted
   dependencies, and merge with expected-head protection only after all required
   evidence. The later Ogre branch supplies the real count-bearing app/native
   positive; do not relabel internal source-planning tests as that evidence.

## Required discriminating evidence

| Boundary | Evidence required |
| --- | --- |
| Canonical input | Missing/empty map retains old serialized bytes; count3 succeeds for actual Javelin vocabulary; zero, one, missing membership, duplicate keys and unknown definition reject. |
| Correct identities | Validated count-bearing source planning yields one Greatclub plus three distinct allocation positions of quantity1; real Stack count yields one stack. This is source-planning evidence until an immutable source is admitted. |
| Namespace separation | Fixed/generated ammunition or material collision rejects; ordinary generated dedup remains identical; Javelins require no ammunition. |
| Equipped gear | Repeated armor/shield refuses before mutation; original count-one armor/shield grants and loadout identity remain unchanged. |
| Atomic materialization | Existing production controls retain rejection for duplicate, missing, extra, nil and already-used IDs. No partial source/profile/items/journal writes, and accepted retries never regrant. The expanded count-bearing app positive belongs to the later admitted Ogre integration. |
| Immutable compatibility | Compare every original source's complete serialized fingerprint and allocation vector with the baseline, including Goblin V1/V2, Air, components and ammunition. Preserve every frozen source/fixture/picker byte and original receiving suite. |
| Application compatibility | Existing source creation/catalog, exact accepted schema/transport retries, cold restore/export, source gear custody and private projection regressions pass unchanged. |
| Verification | Direct static formatting/diff checks during authoring; focused definition/equipment tests when assigned the slot, `scripts/verify-fast`, `scripts/verify`, all six exact-head CI checks and independent full-diff review. Separate merged-main verification. |

## Decisions, risks and current evidence

The optional count map, explicit namespace rejection and Individual expansion
extend the real existing producer. They do not change ordinary item definitions,
source combat damage, current player inventory quantities or source grant history.
The most important review risks are source fingerprint drift, reentrant catalog
initialization, overwritten generated quantities, silently selecting equipped
armor/shields, and mistaking helper evidence for an admitted-source positive.

No source/test code has been changed or executed on this branch. All new behavior
and acceptance remain pending. The separate Grapple core correction writer owns
its branch; this branch must not modify its source or fixtures. No positive LR
claim follows from this plan. Record exact findings, commits, commands, failures
and evidence here as work proceeds.

Next action: finish independent preflight review, resolve its canonicalization
and source-boundary findings in this plan, then implement the bounded quantity
producer while release verification retains the sole local heavy slot.
