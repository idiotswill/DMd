# Gate 4 — Owner-controlled excess Heroic Inspiration

Status: **PLAN COMMITTED BEFORE SOURCE; IMPLEMENTATION AND VERIFICATION PENDING**.
Date: 2026-10-06. Branch: `codex/gate4-inspiration-transfer`.
Sole writer: private_grapple_oct6, allocated by root. No PR/publication allocated.
Clean starting head and fetched Host parent: `54b674234011da4b8aee800fde74ad92d7df266d`;
tree `3edeb062b866c0590974879d3ebdb7af7f2f3143`.
The existing `gate4-host-inspiration` checkout remains immutable to this author.

## Objective and source

Complete the still-required optional excess-Inspiration choice through genuine
Host awards, the owning Player, authoritative table reduction, original replay,
file persistence, portable recovery and the desktop durable outbox. This advances
the product's source-faithful mechanics, PC agency, private information, physical
dice and real save/restart requirements. It does not close Grapple or Gate4, move
remaining work to Gate5, waive canonical/native acceptance, or promise future
unsupported character profiles.

Binding references: AGENTS, product-definition, Gate04, gate-execution protocol,
ADRs020/024/027 plus their atomic persistence/typed authority dependencies, and
the existing Host Inspiration/public Grapple/held movement plans.

The actual pinned SRD5.2.1 pp8/183 and Human Resourceful source were read.
PDF SHA256 `8974902d109d6e63672d7c490bde9ccf052410503d9cfa768237154fbc5e3d87`;
text SHA256 `d277d934b775e8f1edb3bf8f0241559840e4388083f3b49a6a7e09718c202197`.
The already-inspired PC decides whether to give the excess to another eligible
PC; otherwise the excess is lost. The original resource stays. Host adjudication
awards the excess but never chooses or declines for the Player.

## Approved bounded design

Preserve the existing successful `AwardHeroicInspiration` path and its duplicate
refusal. A distinct new outer excess-award action records the real Host p8 ruling
and requires the current active session/head, actual source-created inspired PC,
present correctly bound owner, no existing transfer, and settled mechanics.
Keep the user-facing Host award flow simple; the internal action distinction is
not a gameplay concept the user must understand.

Reuse the existing `inspiration_transfer_pending` flag as the single decision.
An optional omitted-when-absent companion in table state records its actual Host
award command and PC identity. It is producer provenance, not another resource or
queue. The companion requires the matching flag, existing Inspiration, actual
source profile, and original owned history. Existing Resourceful long-rest
production, flag validation and raw ResolveInspirationTransfer semantics remain
unchanged. The current supported source-created PCs satisfy the Human feature
invariant; do not relax it or invent a Long Rest to produce this path.

The new outer resolution requires the current attending Player controlling that
PC, independently of the kernel's broader Admin/System authorization. Share the
old resolver's exact recipient mutation/flag clearing primitive without changing
old behavior. Another active, living, controlled, source-created PC in this
campaign may receive the gift if lacking Inspiration; the active controlled
campaign roster is the current supported group. Receipt is passive and does not
require recipient attendance or a combat turn. Decline clears only the choice.
Neither operation spends an Action, advances time, alters a grip or issues dice.

The existing flag blocks raw rules work, initiative and encounter release, but
does not globally block ordinary actions in an active tactical turn. Therefore
the owned table reducer must refuse unrelated fresh gameplay/session mutations
while the new Host-origin choice exists. Only its owner's resolution may mutate
game state. Reads, reopening, conversation observations and exact accepted
retries remain available. Ending/changing the session requires deciding first;
requiring the owner present at award avoids creating an absent-owner deadlock.

The new award and owner decision are actual outer table events with no invented
nested rules event. The Host ruling joins its exact award. The optional origin
joins that same accepted typed command. Original-anchor replay must reconstruct
the producer, each choice, all intermediate snapshots and final state; snapshots
or flags alone never authenticate it. Reject companion records in original
anchors. Do not append privileged rulings under Player decisions.

Use new actions on the still-unreleased shared v4 surface without changing old
accepted action meanings, audit schema5 or existing marker authority. Refuse
old transports, direct canonical resolution and legacy input. Optional fields
and capabilities remain absent on old histories; preserve their exact bytes,
digests and ordering. Existing accepted retries resolve before new guards.

Privacy decision: only the actual pending-choice owner receives recipient names
and opaque selection handles. Eligibility for this one gift is the intentional
minimum disclosure of another PC's resource eligibility; no sheet, resource
value, reason for exclusion or canonical provenance is exposed. Host can see who
must decide but cannot choose. Unrelated Players retain exact projection,
revision, capabilities and transcript. Transfer becomes visible to its giver and
receiver through their permitted state; decline changes only Host/giver views.

## Implementation and additive acceptance

1. Add source/provenance shapes and strict owned producer/resolver/global guard.
2. Extend exact outer-event/ruling/origin replay validation and opaque presentation
   capabilities, maintaining historical omission and current-authority checks.
3. Extend the existing Host form, add the owner's choice/decline control, and
   explicitly validate both saved-request shapes. Never regenerate a saved input.
4. Add genuine application producer/cold/portable/full-destination hostile cases
   and frontend behavior/uncertain retry/draft reset cases. Freeze for review.

Required cases include first award then excess transfer/decline; no stacking or
automatic recipient; recipient physical Grapple use; wrong/absent owner and
Host/System/Import/source channels; stale/copied/raw capabilities; self/unknown/
foreign/dead/already-inspired recipients; all unrelated fresh gameplay/session
mutations blocked; full typed-row unchanged rejection; exact retries after later
resource use/session changes; forged flag/origin/ruling/event/audit/binding/current/
latest/anchor failures; complete unrelated-audience equality; correct narrow
eligibility disclosure; and old-version omission/refusal.

Preserve every existing test/helper/fixture body, including successful and
duplicate Host awards, saved-award failures, Resourceful transfer/decline, release,
normal/Savage rerolls and all source/Grapple scenarios. Document any genuinely
necessary absent optional-field constructor migration and audit it explicitly.
Use new support modules/helpers; never inject resources or source profiles into
accepted setup. Hostile images are negative recovery inputs only.

## Scheduling, risks and next action

Root exclusively owns the heavy slot. No Cargo/compiler/tests/npm/database/native
execution or push is allocated. Direct Rustfmt/check on changed Rust files is
allowed. Record exact formatter paths, full source preservation and clean freeze.
All new cases remain UNRUN until explicit later allocation. Required future
checks are affected strict all-target Clippy, focused/new plus protected replay
suites, full frontend check/test/build, canonical verify-fast/verify, exact-head
CI, independent review and genuine packaged physical dice/save/reopen evidence.

Risks to test explicitly: global-pause omissions, treating Host authority as
Player agency, confusing Resourceful with Host provenance, resource banking,
capability/recipient privacy leaks, and a saved-request allowlist omission.
Next: commit this plan, implement the bounded production path and real tests,
then hand a clean formatted source freeze and preservation audit to root.
