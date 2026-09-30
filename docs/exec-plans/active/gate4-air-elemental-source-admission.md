# Gate 4 Air Elemental immutable source admission

Status: implementation authored; static checks only, no executable verification or acceptance yet.
Branch: `codex/gate4-air-source-admission`, based on `c4d8c34`.
Writer: coordinating root, 2026-09-30; the source agent has handed back the clean branch.

## Objective and authority

Admit the genuine SRD 5.2.1 Air Elemental through current creature creation while
preserving every historical source identity, command and presentation. This is a
bounded prerequisite for the reviewed Shove Prone-immunity case, not Shove or full
Air Elemental gameplay. Product-definition source fidelity, controlled entities,
durable play and complete tactical timing remain binding, as do Gate04 and
ADR026/ADR028. The accepted immutable-registry direction is recorded in
[Counterspell source preflight](gate4-counterspell-source-preflight.md).
Magic Missile has merged; that preflight's ordering prerequisite is satisfied.

## Accepted scope and non-goals

- Freeze `tactical.json` and `table_creature_catalog_v1.json` byte-for-byte. Add one
  complete immutable Air Elemental definition in separately installed content.
  Resolve profiles by the full ruleset/version/definition/fingerprint pin; freeze
  ID-only/profileless fallback to V1. Existing source and spell tuple fingerprints
  retain their meaning. Do not reconstruct V1 by removing new fields.
- Add optional, omitted-when-absent creation/source-option pins. Missing pins mean
  V1 forever, and are accepted only for historical replay. New live creation must
  supply an exact current admission pin, including for unchanged V1 definitions.
  Normalization, accepted retries, old requests, exports and the V1 picker never
  acquire a default pin. No existing profile upgrades or general migration API.
- Use pinned SRD 5.2.1 pp258–259 (PDF SHA256
  `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`). Preserve
  the complete statistics, languages, Fly/hover, resistances, damage/condition/
  Exhaustion immunities, Air Form, both attacks and recharge details. Air Form
  authorizes entering/stopping in another creature's space and one-inch passage;
  it does not authorize crossing solid walls or ignoring all difficult terrain.
- Represent Air Form and Whirlwind faithfully as typed source data. Ordinary
  open-space movement may use the existing footprint. Operations that need Air
  Form's shared-space or narrow-passage exception must fail explicitly as
  unsupported before an ordinary-body rule can falsely decide them. No global
  footprint shrinkage, grid rounding, teleport or forced-movement wall bypass.
  Placement must not create an unsupported overlapping Air Form state.
- Refuse Whirlwind and Air Elemental Multiattack at authoritative scheduling
  boundaries before spending action/recharge. Expose execution limits to the Host
  through the current source catalog, without modifying historical presentation.
  Thunderous Slam may use the existing intrinsic attack executor only if source
  identity and implement resolution are exact; otherwise it is explicitly refused.
- Audit profile creation/validation, statistics/saves/initiative, policy,
  equipment/weapons/opportunities, spell planning/components/reconstruction and
  actor-to-program source equality. New full-pin actors must not encounter an
  ID-only V1 lookup or borrow another revision's spell tuple.

No Counterspell, Magic Resistance, corrected Hag, Shove continuation, paid-Shove
ruling, shared-space/narrow-gap executor, Whirlwind executor, general registry
framework, or Gate4 acceptance reduction is included. Source representation and
an explicit unsupported operation are not feature completion. Opponent-facing
refusals must not reveal hidden traits or immunities.

## Planned slices

1. Complete source payload, small immutable registry, explicit current admission
   and full-pin profile/build/equipment resolution. Keep legacy helpers frozen.
2. Audit and repair all source resolver boundaries; add spatial/action guards.
3. Current Host DTO/UI exact pins and execution-limit copy; live versus replay
   admission without normalization or receipt changes.
4. Meaningful tests for immutable V1, source fidelity, forged/missing pins,
   legacy absence/retry, normal Air creation/initiative/save/reopen, and unsupported
   action/spatial atomic refusal. Static review, then root-owned verification.

## Acceptance and validation

Required before acceptance: exact-head source/content tests and application tests;
canonical verification and independent review; genuine historical export/retry
continuation unchanged; new admitted Air Elemental survives normal persistence and
reopen with exact statistics, hover and immunities; unsupported actions/geometry
cannot silently execute or consume resources. Future Shove must separately prove
paid Prone against this source completes without effect, including retry/reopen.

No tests/builds may run on this branch while the coordinating writer owns the
heavy slot. No push or merge is authorized until root integration. `rustfmt`,
static inspection and `git diff --check` are permitted. Authored tests are not
passing evidence. Preserve source PDFs, historical exports, capture producers and
the root's baseline. No other worktree is writable by this agent.

## Decisions, risks and next action

- Full V1 definitions remain immutable current admissions; this additive source
  does not supersede any V1 revision. Missing creation pins do not select current.
- New source installation is verified separately against its declared bytes;
  compiled content alone cannot authorize a missing or changed installed entry.
- The narrow spatial guard intentionally leaves the Air Form execution obligation
  open. Its exact scope and privacy will receive independent review.
- Added [ADR029](../../architecture/029-immutable-creature-source-admission.md).
  The coordinating writer approved package compatibility: older installed bundles
  lacking Air content receive a recoverable content error until normal package
  update, with no database mutation. Old campaigns replay using current verified
  content with byte-identical V1 definitions, not a save migration.
- Shared-space guard refinement: ordinary permitted allied/size-based intermediate
  transit and ordinary involuntary overlap need no Air Form permission and retain
  normal behavior. New placement and voluntary final sharing remain unsupported.
  Bounds and a destination wholly enclosed by authored solid remain ordinary
  obstruction; partial body/sweep/corner contact is conservatively unsupported,
  not proof that a one-inch passage exists. Opponent errors name no hidden source.
- Thunderous Slam uses the existing intrinsic backend attack/implement path; this
  does not add a new desktop own-turn intrinsic action form or claim full creature
  play. Multiattack and Whirlwind are refused before payment. Source page anchors
  were independently checked: traits/first two actions p258, Whirlwind p259.
- Resolver audit: profiled statistics, saves, initiative, attack/Pack Tactics,
  weapon/equipment planning, policy and opportunity labels now use exact source;
  spell planning/components/program reconstruction, retained feature activation
  and actor binding compare the complete tuple. Frozen ID-only profileless/build/
  equipment helpers remain V1. Normalization and request receipts are unchanged.
- Authored tests cover Air source/build/immunity/hover/saves, forged pins and V1
  absence, pre-cost unsupported action refusal, open movement/allied transit/
  forced overlap/special geometry, current owned transport creation with live
  missing/forged pin refusal, cold reopen/exact retry/restore, installed missing/
  undeclared/rehashed source no-write refusal, frozen picker serialization and UI
  pin/limit handling. Existing live constructors now carry current pins; these are
  new producers, never replacement historical evidence. Genuine exports and old
  producer/baseline files remain untouched.
- Static checks: direct rustfmt parse/format and `git diff --check` pass. Static
  Python manifest inspection confirms all six declared lengths/FNV checksums.
  Frozen tactical SHA256 remains
  `5cb3e2e6965489c4c8e4f2f0ca643e0e3634ee84e2aed455260f1589fd330548`;
  frozen picker SHA256 remains
  `adca5f6c4e7d5f316f9a8958c93e35a411493d201fac95f6938734e385e63dac`.
  Portable script and Tauri resource configuration already include the entire
  source directory. No Rust/JS compilation, test, packaging or runtime execution
  has run on this branch. Source tests are authored, not passing evidence.
- Recovery review on 2026-09-30 rechecked the complete source block against the
  pinned extracted SRD text. It corrected a non-UTF-8 byte in the new distribution
  notice and refreshed its declared checksum; expanded authored assertions for
  the actual Slam/Multiattack clauses and healthy airborne Hover; and recorded
  qualified scope in the Rules Coverage Ledger. No evidence status was advanced.

Next action: coordinating writer reviews the exact unpublished commit, then owns
focused and canonical verification after the existing heavy/capture slot is free.
Run rules source/profile/schedule/spell/spatial tests, application content-integrity
and table-loop source creation tests, frontend tests/check/build, and unchanged
historical replay/retry suites. Resolve actual failures without weakening evidence.
New Air tabletop placement/initiative and the eventual genuine Shove immune
no-effect retry/reopen proof remain required before claiming those gameplay paths.
No merge or Gate4 completion is implied by this checkpoint.

### Draft verification checkpoint — 2026-09-30

Root reviewed the independent complete 51-file static review of source
`4a27cb0776f1ba5efe0bc2c9fa4a3e782093039e`, full tree
`e1e90f9fa046505cd79dfd0275cc00acface1e00`. No actionable static defect was found;
source fidelity, all six installed manifest lengths/checksums, unchanged V1 Git
blobs, exact source resolution, live/replay admission and explicit unsupported
boundaries were checked. Formatting and whitespace checks pass. This adds no
passing Rust, frontend, application, native or compatibility result.

Root now authorizes draft publication to run remote CI while the local heavy slot
continues the original flow 4 capture and baseline. This supersedes the earlier
agent publication hold only. Main remains fixed at verified c4d8 until that
original-source proof completes; neither this draft nor a green CI result can
waive the remaining acceptance above. This checkpoint changes documentation only.

Next action: inspect actual draft CI output and fix concrete failures; run local
focused/canonical and frontend checks when the heavy slot is available. Preserve
the exact old exports and prove their continuations under this source before
acceptance. Reconcile accepted dependency movement, review the final full diff,
and complete native source/placement evidence before any protected merge.

### Manifest tooling correction — 2026-09-30

The later same-ID creature revision work found two omissions in the Air change:
the manifest regeneration script still enumerated only the five legacy files,
and the existing distribution test expected that same five-file set. Running
the old script would drop the required Air entry. The script and exact-set
assertion now include the installed Air payload; the assertion remains strict.
No source payload, current manifest, legacy picker or historical fixture changes.

Root ran the corrected actual script against an isolated copy of all six declared
files. Its output is byte-identical to the installed manifest, SHA256
`63196eccc85324e248973fb65741ada9b02404ce9e0b74939ddbc8dc0fd0a5bd`.
The script SHA256 is
`792f232ed554a9fe649338bd8c9147e278f33c7ff16d4806b02ce64c419a613e`.
External evidence is `tooling/air-manifest-regeneration-2026-09-30/result.json`.
This verifies regeneration only; the Rust assertion has not executed locally.
Original 1a0a5e7 runtime CI remains separately under audit, and no green result on
that head verifies this correction. The corrected head requires fresh checks.
The original-flow baseline still owns the local heavy slot. Preserve the prior
remote runs until normal completion before publishing the corrected source.
