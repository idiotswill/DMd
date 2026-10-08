# Gate 4 — Author and inspect elevated and liquid encounter geometry

Status: planned, 2026-10-08; no implementation or runtime acceptance yet.
Branch: `codex/gate4-battlefield-authoring`, reused clean `gate4-spatial` checkout.
Baseline: freshly fetched `52ae4bf36ce9378eb4ddc48b3bcef8477d2e4580`, tree
`f13e29b7b53b13fcbcda9849e19a632cd36aa094`. Root is the sole writer.

## Confirmed product gap

The combined native acceptance audit found that `BattlefieldForm` always submits
actor and region elevation zero. Its height control changes body extent, not
elevation. Only opaque walls and difficult ground can be authored; every terrain
region has water/support disabled. Existing production rules already resolve
elevated support, falls, liquid landings and continuous dry Ground movement, but
the actual desktop cannot establish the corresponding supported scenes. The map
also omits elevation from its accessible token descriptions.

This bounded repair advances the tactical philosophy, tactical visibility and
environment interaction clauses of `docs/product-definition.md`, Gate 4 spatial
combat and its integrated native acceptance, and ADR025's single spatial authority,
bounded geometry, actor-specific knowledge and durable compatibility. It supplies
ordinary Host setup controls through the current `PrepareBattlefield` command.
It does not manufacture a saved state or add mechanics in the renderer.

## Scope and decisions

- Add actor elevation separately from the existing body height. Keep the floor at
  the existing zero reference and the map ceiling at 40 feet. Make those bounds
  clear; reject a body extending outside the map before proposing setup.
- Add region base elevation separately from its thickness/height, and expose
  water and a dry supporting platform/bridge top alongside the two existing kinds.
  Preserve walls as solid sight/movement obstacles, difficult regions as before,
  and the backend's independent water/support semantics. A platform is a support
  surface with open space beneath; it must not silently become an opaque wall.
- Region labels explain bottom/top heights. Use exact half-foot conversion and
  bounded finite inputs, never rounding an unsupported number or sending NaN/null.
  Preserve valid old ground-level proposals, participant inclusion, source-derived
  footprint, controller/attendance requirements, area policy and GM ruling.
- Display nonzero elevation using only the existing authorized token position,
  including remembered positions. Do not obtain raw Host geometry for a player,
  update a remembered location, or change saved projections. Preserve old
  ground-level descriptions. Explain the top-down view and render the new Host
  water/support distinctions without implying hidden player terrain is known.
- Preserve every backend, content, dependency, schema, transport, saved fixture
  and existing test body. Add meaningful frontend interaction tests separately.
  No general map editor, light/fog authoring, new carry/fall rule or save migration.

User-facing backup/export/restore remains the explicit Gate 13 traceability
requirement. Gate 4 native cold recovery and real application portable restore
remain separate evidence classes, as in `gate4-expiry-native-2026-10-06.md`.
This plan does not claim that a desktop backup UI is a Gate 4 prerequisite.

## Acceptance and verification

1. Real form interaction proposes an elevated PC/source actor on a dry ledge, a
   water volume, and separated support surfaces above water using the existing
   exact typed protocol. Verify bottom/top/footprint arithmetic and unchanged
   old default geometry, inclusion, team relations, ruling and area policy.
2. Invalid vertical extents, nonfinite/fractional-unit inputs, out-of-bounds
   regions/actors and unavailable preparation cannot submit. Existing backend
   validation remains the final authority; the frontend does not decide support,
   flight eligibility, legal movement or final fall consequences.
3. Map tests distinguish authorized elevated actual and remembered positions,
   and remove all prior Host/other-observer details on selection changes.
4. Independently review the complete exact diff and all old-body preservation.
   When root allocates the serial slot, run frontend check/full tests/build and
   canonical `./scripts/verify-fast` / `./scripts/verify`, then fresh Linux/Windows
   verification and actual Windows packaging on the final receiving head.
5. In the exact final package, author real elevated and water scenes, execute the
   selected supported fall/liquid/Ground routes with ordinary controls and real
   saved consequences, and close/reopen at the required pending and settled cuts.
   Controlled QA faces are not human physical-dice acceptance. Source/UI tests
   alone cannot accept the native scenario or the complete Gate 4 encounter.

## Execution allocation and next action

Commit this plan before implementation. Root may now edit only the scoped
frontend and new test files in this checkout. The exact52ae canonical run owns
the local heavy slot in another checkout: no Cargo, npm, native application,
database access, package/build process or competing runtime is allocated here.
Source reads, bounded static analysis and Git operations are permitted. Keep the
running checkout, runner, tools and evidence immutable.

Freeze the source-only repair with a full preservation audit and hand it to a
separate reviewer. After the current run ends and its failures are resolved,
receive this reviewed whole branch deliberately into the main-facing family
before its final full verification/native session. No result on52ae transfers to
that new receiving identity. Capacity transport and the genuine pre-capacity v5
capture remain separately planned work; neither is implemented here. The
independently found own-turn intrinsic creature-attack UI gap is also separate.

## Source implementation — 2026-10-08

The form now distinguishes actor elevation/body height and region bottom/extent,
and proposes water and dry support surfaces through the existing geometry shape.
Each new water/support region has its own surface identity; no disconnected
regions are silently declared one connected material. The old ground-level
proposal is unchanged. Numeric validation preserves exact half-foot units, body
and region extents remain inside the existing 40-foot ceiling, and a disabled
form cannot submit. Map descriptions add only a nonzero authorized elevation;
the player's remembered position remains the original remembered position.

Eleven new frontend cases are authored in a separate file: default compatibility,
ordinary multi-region/elevated setup, six invalid occupied-space inputs, invalid
terrain/correction, pending-operation refusal, and actual-versus-remembered
elevation/privacy. Every existing test file and all Rust/content/corpus bytes
remain untouched. Static whitespace and preservation checks, followed by fresh
independent review, precede any runtime allocation. No test, frontend build,
canonical command, app/database or package has executed for this branch.

Next: freeze and independently review the exact full diff, resolve any finding,
then follow the deliberate receiving and verification sequence above. Native
acceptance and merge remain pending; source authoring is not acceptance.

## Independent capacity finding — before correction

Independent full-source review of `d4ebc263` found one P2 compatibility defect:
the new combined 512-region cap excludes previously valid mixed maps. The
unchanged domain independently permits 512 obstacles and 512 terrain volumes.
Its report is `tooling/ci-oct8/battlefield-d4ebc263-independent-review.md`, SHA256
`2d552f31f4aaa95b03686ba65706911cefa624b21a09a23fa0ef668093fd2f42`.
The original source and report remain preserved; all other bounded source and
preservation checks were clear, with no runtime executed.

Root selects only the exact capacity repair: count wall and nonwall regions
separately at submission, and allow up to 1024 total draft regions so each
existing valid combination can be authored. An excess-kind draft can be changed
to another kind or removed before submission. Keep all geometric/unit checks,
payloads, existing and new tests and backend bytes unchanged. Freeze the small
successor diff for independent closure; project execution remains unallocated.
