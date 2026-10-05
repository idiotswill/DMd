# Gate 4 — Immutable Mage ordinary-hand source

Status: plan before source, 2026-10-05. Sole writer ci_oct5 on
codex/gate4-mage-ordinary-hands, exact development parent
a7d40848c8b8588261444bdb5af19f13389b5336/tree91914f6cdf78a955bff6fc64eb83aabd771945bb.
Root owns all other branches, verification, publication and later normal intake.

## Objective and authority

Provide one bounded immutable Mage revision with authenticated ordinary anatomy,
unchanged source spells/gear/statistics, current creation and original-history
coexistence. This advances Gate04 source-faithful creature play, product rules
authority/recovery/privacy, ADR029 immutable identity and ADR028 historical work.
It does not complete Grapple, Counterspell execution, the Mage stat block or Gate4.

Root reviewed preflight81d7f744 and source audit755f469b, personally inspected the
pinned PDF page305 and explicitly approved this normalization in external
tooling/mage-twohands-root-design-decision-2026-10-05.md. The official PDF SHA256 is
8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87.

Page305 lists Wand Gear and spellcasting but NO hand count. TwoHandsV1 here is an
explicit engineering ordinary-body interpretation for this immutable revision,
not a printed quotation or authority inferred from Humanoid/Wizard, names, empty
slots, spell labels or Wand alone. Unlike Goblin's Shortbow and Ogre's Greatclub,
the Wand does not independently establish two hands. No global anatomy rule.

## Scope and acceptance

- Add complete mage-v2.json equal to frozen Mage plus only ordinary_hands.
  Keep SelectedFeatures and every omitted clause unchanged. No new attack,
  spell, immunity/resistance, mastery, focus waiver, extra uses or prepared AC.
- Exact full-pin immutable lookup retains V1 and V2. Only reviewed V2 is current
  Mage admission; ID-only V1, profileless fallback, omitted historical creation
  pins, old picker/presentation digests and accepted retries remain unchanged.
- Original Mage pin srd-5.2/5.2.1/mage/af0f81ba7833b9c4 remains exact.
  New fingerprint may be provisionally derived by serializer review; actual Rust
  verification must be separately run before any correctness/acceptance claim.
- Both revisions retain ordered [spell-material:mage-armor1,wand1], real custody,
  both Small/Medium choices, unbuffed creation and genuine Mage Armor preparation.
- All three source spell tuples (Mage Armor, Shield, Counterspell) remain exact
  to actor revision, including cross-revision refusal and retained reconstruction.
- Required installed payload/manifest/byte checks reject missing, undeclared,
  altered and altered-plus-rehashed content without destination mutation.
- Add genuine original Mage capture continuation and new current creation in the
  same campaign, exact retries, cold file and independent portable continuation,
  hostile revision swap refusal with full populated-destination equality.

## Planned changes and existing-control migrations

Production: tactical_definitions loader/constant; immutable/current registries;
rules_runtime required package entry; content manifest generator, NOTICE and
distribution entry set. Existing optional anatomy field and exact spell tuple
resolver should need no new execution semantics.

The following deliberate current-only controls require migration, inventoried
before source. Preserve all other original assertions and compare literal diffs:
1. table_missile_cases current setup reads options but explicitly pins V1;
   select the actual matching current option pin.
2. table_release_recharge_cases mixed Mage/Dragon setup and table_shove_ledge
   fresh Mage setup explicitly use V1. Derive the actual current option pin and
   its exact source equipment allocation. Keep every existing gameplay assertion.
3. tactical_grapple_sources current anatomy allowlist extends exact Goblin/Ogre
   pins with the candidate Mage pin; singleton Mage current assertion is additive.
   Do not skip arbitrary sources or authorize via ID.
4. tactical_ogre_source immutable cardinality13 retains its original13 exact
   entries plus one explicitly asserted Mage candidate; current count12 stays.
5. gear_count_tests original12 allocation controls and tactical_definitions
   original12 omitted-gear controls exclude only the exact new Mage pin from
   their original subset. Add separate candidate allocation/serialization
   assertions. Existing Ogre control/body otherwise stays.
6. distributed_rules_pack required path set adds mage-v2.json, with all original
   paths retained. Package hostile/recovery tests are additive.
7. Add child module registration to legacy_shield_hit_v1_replay only if needed
   to reuse the genuine original driver. Every existing helper/test body and all
   original fixture/capture/raw bytes remain byte-for-byte unchanged. New child
   cannot modify original captures or synthesize positive history.

Inspect every caller affected by current Mage admission; reconcile an additional
concrete current-selector migration in this plan before editing that caller.
Original source/history producers stay V1; current fixtures do not silently
receive original pins from a changed ID-only helper.

## Non-goals and verification boundary

Actual outgoing-grip Mage Armor/Shield component matrix belongs to later complete
Grapple integration. Counterspell's live trigger remains a later ADR028 execution
boundary; source tuple controls do not claim a current live Counterspell window.
No production semantics fix without a concrete finding and root design review.

No Cargo/compiler/test/npm/DB/native/publication allocated. Content maintenance
and external static hash/JSON/Git audits are allowed; request a brief standalone
rustfmt window before formatting. Preserve prior files and all original failures.
Root later requires full independent source review, strict lint, meaningful focused
controls, canonical/CI exact-head logs, recovery and native integration evidence.
All new implementation/tests are UNCOMPILED/UNRUN until then. Gate4 remains open.

Next action: commit this plan, author the coherent bounded revision and controls,
update exact migrations/evidence, then hand back a clean frozen head/tree and
complete source/hash/test inventory for root review.

