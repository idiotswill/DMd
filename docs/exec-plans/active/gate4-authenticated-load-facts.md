# Gate 4 — Authenticated body and equipment load facts

Status: planning and source inventory, 2026-10-07. Implementation is not yet
allocated. Sole branch writer is root. Branch
`codex/gate4-authenticated-load-facts`; checkout `gate4-authenticated-load-facts`.
Baseline is `b77b1597fb99d18643ed76f43202156f3bcf0fe0`, tree
`f56acf15ef3552436ccfbd8bdc2a199457dae7b3`. Fetched main is
`1a9de14c8a4418893b6664b89f99f0a0c0225ce1`. The baseline's full verification
is pending; inheriting it is not an acceptance claim.

## Objective, scope and product traceability

Supply the real table and desktop with authenticated body and item mass facts
and a custody-derived load view, as a prerequisite for faithful tactical carrying
and dragging. A normal user must be able to supply a missing physical fact
through an authorized form and resume it after restart. A type or source table
without that application path does not satisfy this slice.

Binding sources are root AGENTS, the [product definition](../../product-definition.md),
[Gate 4](../../checkpoints/gate-04-tactical-encounters.md), the
[gate execution protocol](../../checkpoints/gate-execution-protocol.md), the
[active tactical plan](gate-4-tactical-encounters.md), the public Grapple plan,
and the held-movement and combined-receipt plans. This advances source-faithful
mechanics, supported character interaction, conservative world facts, actual
physical inventory, player authority, private information, local play and exact
recovery. ADRs 009, 011, 012, 020, 024, 025, 027, 028 and 029 remain binding.

This is a coherent prerequisite within Gate 4. It does not implement carrying
support, change existing movement, complete ordinary drag/carry, or waive any
tactical acceptance. Broad shopping, travel, downtime and economy remain Gate 5;
complete catalog enumeration remains Gate 6. Any physical item already present
in a supported tactical route still needs an honest load treatment here.

## Source decision and preserved history

Use the pinned English SRD 5.2.1. The PDF SHA256 is
`8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`.
Page 178 supplies Strength/size carrying limits and separate drag/lift/push
limits; page 182 supplies the Grappled per-foot cost and size exceptions. Root
selects the source-conservative interpretation that both constraints apply to
new mass-aware live operations. The text does not expressly waive pounds for a
grappled target. This records engineering interpretation, not an assertion that
an external ruling resolves all ambiguity.

Do not invent body weights from size, height, species or narrative. Source
equipment weights must identify their printed units and quantities. Missing,
variable or unrepresented facts remain unresolved rather than becoming zero.
A Host authorization boolean cannot replace an actual capacity calculation.
Where the source supplies no physical mass, a separately recorded accepted
world fact must be clearly distinguishable from a printed source value.

GroundDragV1's prior behavior, original accepted commands, paid movement prefixes,
audiences, digests, snapshots and byte-identical retries remain exact. Preserving
that history does not make its live mechanics capacity-complete. A subsequent
explicitly versioned operation must integrate capacity into new dragging and
carrying. That integration remains required by Gate 4 and is not deferred away.

External source-only design and independent review identify the prerequisite:

- `tooling/carrying-58fe854-design-proposal-2026-10-07.md`, SHA256
  `4777fc7bdea2ae6eaa14756475c6796744ab2a1842183a00217b8b68e98150c0`;
- `tooling/carrying-58fe854-design-independent-review-2026-10-07.md`, SHA256
  `94f48e26949d6198d37d7f27d037343d20ddd9e3ac4805b1dc2f9937c1fe1898`.

Those reports are navigation/evidence, not substitutes for this checked-in plan.
The baseline's production bytes equal their inspected 58 source; b77 changes
only two test setup files and documentation. Re-pin all inputs for implementation.

## Planned slices and decisions still to resolve

1. Inventory every currently materializable equipment identity against the
   immutable source: ordinary and creature weapons, armor, ammunition, supplies,
   containers, foci and spell materials. Distinguish bundle weight from each
   unit, empty container from contents, and actual coins from an abstract wallet.
   Unknowns must be explicit. Review printed tables visually where necessary.
2. Choose bounded exact arithmetic and catalog representation after that
   inventory. Retain all existing content bytes and fingerprints. A new immutable
   mass catalog/sidecar needs its own source identity and reviewed version; it
   cannot revise previously pinned character or creature definitions in place.
3. Specify the legitimate body/item fact producer, with original command,
   source/entity/item identity and amendment history. Determine who proposes and
   accepts PC facts, who authors NPC/custom-item facts, and the settled boundaries
   for changes. A client-supplied total is never authoritative load. Do not allow
   a new mass form to bypass source ownership or reinterpret old backstory.
4. Integrate optional omitted-when-absent state into the single owned execution
   and typed table path, behind explicit settled activation and the next reviewed
   presentation/transport/audit versions. Confirm all decoder allowlists, old
   producers and immutable retry behavior before selecting the version. This
   activation grants fact functionality, not a carrying pose or support record.
5. Compute load from actual custody, quantities and containment, counting each
   physical object once. Trace worn/held items, destroyed/missing items, containers,
   coins, fluids and unknown material facts. Bound graph work, reject cycles and
   report unresolved inputs honestly. Preserve the distinction between ownership
   and possession. Do not install a cached admitted total as current authority.
6. Expose authorized forms and useful load/source information through the real
   desktop and opaque owned controls. Preserve unrelated audience DTOs, transcript,
   revision and handle behavior; do not expose hidden facts through option ordering
   or detailed errors. Normal users need pounds and physical descriptions, not
   internal proof records, version numbers or implementation terminology.
7. Extend original accepted-history enumeration/replay, close/reopen, portable
   restore and rejection atomicity. Add genuine producer-driven application and
   frontend cases plus focused numeric/custody adversarial cases. Freeze the full
   delta for independent review, then schedule exact-head verification and native
   evidence with root's serial heavy allocation.

No production field, command shape, permission rule or version number is approved
merely by this planning list. Resolve the source inventory and concrete producer
design in a plan update before implementation. Avoid a disconnected catalog-only
completion claim.

## Acceptance criteria

- Source weight rows have exact printed references and unit semantics. Fractional
  ammunition, stack quantities, empty/full container distinctions and no-weight
  entries cannot be silently rounded, double-counted or treated as zero.
- A genuine current PC and an actual source creature can receive an authorized
  authored body fact, with source identity and original command preserved. No
  direct state, SQLite, JSON edit or synthetic positive proof establishes it.
- A real currently owned item uses an immutable printed mass or an explicitly
  permitted accepted item fact. Source facts cannot be overridden by unrestricted
  client values. Equipment transferred in custody changes the real computed load
  without rewriting ownership or the original mass origin.
- A complete supported load and an unresolved load are distinguishable. Missing
  body/item/container/coin facts prevent a claim that capacity is numerically
  satisfied. The app does not quietly remove real objects to obtain a total.
- Mutation timing and controller/session/head restrictions are explicit and
  tested through real controls. Pending attacks, rolls, held work or another
  player's actor do not grant an unintended fact-editing route.
- Historical v1-v4 producers, ordering, bytes and retries remain exact. New
  fact attachments require accepted activation and original fact commands;
  matching forged current/latest snapshots is insufficient provenance.
- Cold file reopen and independent portable restore at activation, fact admission,
  amendment and custody changes accept the same next input with identical complete
  outcomes. Tampered origin/source/value/quantity/custody/activation fails without
  mutating destination typed cells. Preserve all original test bodies and fixtures.
- Independent full source review, canonical `./scripts/verify-fast` and
  `./scripts/verify`, exact-head Linux/Windows checks and applicable native UI
  evidence are required before this slice is accepted. Focused tests supplement
  that evidence and cannot replace it.

## Remaining carrying behavior, unchanged Gate 4 obligations

Future support must persist across idle and turns, with authenticated admission,
pose updates and retirement under actual causal work. Before that implementation,
resolve independent displacement, carrier falling, changing capacity and target
death. Do not automatically drop a target merely to simplify code or reject a
genuine Shove/damage effect to preserve an invariant. Retaining a supported body's
identity/load through target death is the conservative design direction; it still
needs a real death transition and must not fabricate an inventory item.

Release/escape and holder incapacity need real support-loss/fall work even while
idle. Already issued opportunity/damage/concentration children keep their original
authority. Multiple targets/holders, nested carrying/cycles, new corpse pickup,
vertical modes, alternative poses and post-encounter lifecycle remain required
Gate 4 work. The current no-live-grips Finish/session guards remain until a
reviewed persistent-world transition exists. No gate or human playtest criterion
is weakened by this prerequisite.

## Validation status, risks and next action

No implementation, compiler, test, native or database execution on this branch.
Only the inherited clean source and this plan exist. Source inventory and producer
design are in progress outside Git. The original b77 integration worktree is
immutable under its separately scheduled focused run; Held0c60 has completed its
11-case focused selection and linting but its full CI/native scope remains open.

Main risks are invented mass, conflating catalog bundles with physical quantity,
unaccounted container contents, hidden-information leakage, unauthorized fact
amendment, and changing old accepted execution. Resolve those through actual
source and current producer paths rather than weakening load checks.

Next action: receive/review the two exact-b77 prerequisite inventories, record the
concrete arithmetic/catalog/authority/version design here, commit that plan update,
then allocate a single implementation writer. Root continues current runtime and
CI verification independently. No next-gate work is authorized.
