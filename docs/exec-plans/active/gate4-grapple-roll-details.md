# Gate 4 — Correct live Grapple roll guidance without changing history

Status: source implemented; independent review and all runtime UNRUN,2026-10-07. Branch
`codex/gate4-grapple-roll-details`, baseline
`573305cd68e71d51d097703f5797379fe40696ce`, tree
`e4b42195eeabff92d132c3cc16632650572827a3`. Root creates this plan; the assigned
sole writer is mass_capability_fix_oct7. Root owns review, publication, project
runtime and merge. Combined573 verification is running separately; no inherited
pass certifies this branch. Current main is4cf815bd0f0d9b128612867ba829c9ac1c2549f7.

## Confirmed problem and product boundary

Actual supported GrappleSave/GrappleEscape producers create owned physical rolls,
but `table_runtime::roll_label` retains the historical literal `Unsupported roll`
and a stale comment saying no producer exists. Both ordinary and source projections
use it; normal owner/source RollForm renders it. Mechanics, raw authority and the
captured original Inspiration reroll still work. Host viewing a PC-owned roll
continues to receive the waiting instruction instead of that PC's form.

The source finding is pinned at
`tooling/grapple-roll-label-0bc-source-finding-2026-10-07.md`, SHA256
`58e47f8a5644a18b473dc1c3828e0d804cb977526efb37a45eb5f514546ee089`.
Root read it and the actual573 roll-options read, DTO, projection and UI paths.
The completed genuine969 recipient/decline corpus independently confirms the old
raw/presented label. Baseline573 has the same relevant mapping and current producer.

This advances Gate4's usable physical-dice interaction, player/source authority,
privacy and exact recovery under AGENTS, product-definition, Gate4 and ADRs
020/024/025/027/028/029. It is a bounded presentation repair, not new Grapple
mechanics, mass/carrying, permission, global version, save migration or gate closure.

## Selected read-only contract

Add a separate explicit schema1 roll-details request/response alongside the
unchanged legacy `TableRollOptionsRequest`/`TableRollOptions` API. The request
contains version1 plus campaign/channel/revision/opaque-roll-ID. The response
contains version1, the existing options value and a separate display reason.
Reject unknown request versions and unknown fields. New desktop registration and
typed frontend API expose this read; it is not a command or durable activation.

Share the existing private owned authentication/query path so options and label
come from one read transaction and the same authenticated pending request. Retain
the CampaignExecution/read lifetime and existing current audience revision,
opaque capability, canonical pending-roll, PC ownership and source-controller
checks. Host retains its existing read visibility without acquiring submission
authority. Do not clone state to authorize later work or accept a purpose/label
from a caller. Reads must not bootstrap/change projections or write revisions,
handles, sessions, bindings, events or game state.

For authenticated GrappleSave display `Grapple saving throw`. For authenticated
GrappleEscape display `Strength (Athletics) Escape` or `Dexterity (Acrobatics)
Escape` from the exact validated current Escape source choice/request, preserving
its owned work identity. Do not infer that choice from modifiers or earlier UI
text. A narrowly verified canonical request reason is acceptable for these two
source-produced Escape texts; a blanket canonical-reason passthrough is not.
Other purposes use the existing audience-safe label. Do not disclose target/DC
or another audience's hidden work. Invalid new purpose/choice must refuse without
changing historical projection or making an unsupported roll executable.

Keep legacy roll-options serialization/omission and semantics exact, including
independent Inspiration/Grapple capability. Keep all old raw/presented DTO bytes,
digests, revisions, opaque handles, envelope versions and accepted retries exact.
Do not edit `roll_label` output globally or conditional on build date/global5;
that would invalidate already accepted historical digests. Its stale comment may
be corrected to explain the retained historical label policy and separate live
details. No durable presentation/request version is consumed by this read API.

## Desktop behavior

At the existing actual roll-options eligibility point, request one details payload
for the exact displayed campaign/channel/revision/roll ID. Preserve generation,
campaign, player/source and revision stale-response guards, and bind the response
to the actual roll ID too. Reset the separate local display label on any selection
or pending-request change. Options and label must come from the same response;
do not perform a second full campaign replay just for wording.

Pass the separate optional display reason to RollForm, rendering it when present
and otherwise retaining `request.reason`. Never mutate `view.roll`, the request
used for submission, stored outbox envelopes or original dice. Preserve current
Host/owner/source eligibility, modifier/dice fields, ordinary/Savage/Inspiration
submission and exact acknowledgement retry. The backend legacy endpoint remains
available for historical callers and compatibility tests.

## Implementation and required evidence

1. Freeze source pins and inspect complete relevant read/transport/UI paths.
   Implement new types, shared owned read helper, desktop command and frontend
   details call/form label as one coherent bounded change. No source catalog,
   mechanics, dependency lock, compiler/test settings or unrelated UI changes.
2. Add real application cases from genuine production setup for Strength/Dexterity
   GrappleSave and Athletics/Acrobatics Escape, with actual PC and source owners.
   Verify label/selected choice, read-only whole logical rows and export, original
   options equality, and actual ordinary/physical Inspiration submit/retry.
   Cover independently enabled G3 and G+T4; a later deliberate mass-branch receipt
   must also test M+G/T5 and cannot infer it from this non-M baseline.
3. Verify stale/foreign handle, wrong PC/source/controller, unsupported version
   and changed current request refuse atomically. Genuine restored pending work
   must retain old raw/presented labels/digests while the new opted-in details
   improve the live form. No archive normalization or injected positive state.
4. Render TableApp/RollForm for actual details labels, unchanged modifiers/dice,
   one unchanged submit envelope, Inspiration availability, source selection,
   cold-pending context, stale asynchronous replies and Host's PC-roll waiting
   message. Existing assertions remain; required old fixture mock updates must
   be explicitly inventoried and preserve the scenario instead of bypassing it.
5. Preserve all1116 existing Rust test bodies. New application coverage should use
   a separate ordinary harness or additive modules without rewriting the old
   producer. Byte-inventory existing history/content. Genuine969 restored details
   checks will use the approved fixture location only after deliberate receipt
   of that separately authored compatibility work; copying a partial corpus or
   making a current producer masquerade as old history is prohibited.
6. Freeze coherent commits for fresh independent whole-diff review. Root assigns
   exact-head affected Rust/frontend tests, canonical verification, full CI and
   relevant desktop checks. Source authoring/formatting/mock tests cannot alone
   establish the production interaction or old-history safety.

## Allocation, risks and next action

mass_capability_fix_oct7 may write only this branch and external source evidence;
commit the implementation and updated status coherently. Static Git/file reads,
direct formatting and lightweight static checks are allowed. No local Cargo,
compiler, npm, gameplay/database or native execution: exact573 session92894 owns
the sole heavy slot. Root controls pushes and merges. Report concrete boundary
conflicts before broadening the selected API or changing production authority.

First action is source verification and complete relevant reads, then source-only
implementation under this plan. All project runtime here is UNRUN. Historical
mass consumers and CI partitioning remain separately owned branches. Carrying,
integrated native encounters, human physical dice and every other Gate4 acceptance
requirement remain open; no Gate5 work begins.

## Source implementation handback, 2026-10-07

The explicit schema1 `TableRollDetailsRequest`/`TableRollDetails` endpoint now
shares the legacy options path's one owned execution/read and transaction. The
legacy endpoint, serde omission rules and all historical `roll_label` outputs
remain intact. The separate live formatter admits only the exact canonical Save
reason and the two source-validated Escape reasons; other purposes retain the
audience-safe old formatter. Desktop registration/API and RollForm now pass an
optional local label separately from the immutable displayed/submitted request.
The single details response supplies both options and label, with generation,
campaign/player/source/revision/actual-roll-ID checks and stale-error suppression.

Four added ordinary `table_grapple_public::roll_details` cases expand to sixteen
genuine PC/source, G3/G4, Strength/Dexterity Save and Athletics/Acrobatics Escape
setups. Each asserts old raw/presented wording and original options serialization,
current owner/Host reads, complete logical SQLite rows/export equality, cold open
and portable restore, unsupported version/unknown field and invalid audience,
handle, character, source/controller refusals. Host submission with its own valid
handle still refuses. Real ordinary or PC Inspiration dice use unchanged cold
accept/retry helpers and assert canonical work/issuer/acceptance, raw faces and
the single new recorded roll. The old read refuses after acceptance. These are
authored assertions, not executed results.

Sixteen added rendered frontend cases cover all three labels for PC and selected
source, exact ordinary envelopes, G4 Inspiration original/replacement and saved
retry after restart, cold source selection with empty unsent dice, Host waiting,
and late success/error replies after five context changes. The overlapping-read
cases explicitly dispatch an already-queued selection/refresh DOM event while
its control is busy, then use ordinary user interaction for source selection;
they do not weaken production guards. The roll-ID guard additionally receives a
changed ID under a retained mocked revision, a defensive stale-UI scenario rather
than a claim that production can publish such a revision. Optional RollForm
fallback and raw submission are tested separately in the same rendered file.

The only existing frontend test adaptations are in TableApp, TableInspiration,
TableInspirationTransfer, TablePhysicalCreation and TablePosition: rename mocked
options calls to details, wrap the unchanged options response in schema1, and
add schema1 to the existing exact read-request assertions. All other assertions,
scenario strings and flow stay exact. All1116 existing Rust test bodies, the
original producer/helper bodies, content, archives and historical source files
are required to match the frozen573 inventory in the external preservation audit.

Only direct changed-file rustfmt, static file/Git inspection and source inventory
auditing were performed. No Cargo, compiler, npm, test, gameplay/database, native,
preflight, push, PR or merge was run by this author. No runtime pass is claimed.
Next: root/fresh independent reviewer inspect the complete frozen source and
external author handback/audit, allocate exact-head affected Rust/frontend and
canonical verification, then perform the deliberate mass compatibility receipt
and M+G/T5 plus genuine969 restored-details checks in its approved fixture path.
Those checks and full CI/native acceptance remain required before merge;
publication for review/CI belongs to root. Source review alone cannot satisfy them.

## Root verification and reviewed fixture receipt, 2026-10-07

Root reviewed the entire17-file implementation delta and both new test files.
The independent full source review is clear at exact0990d320, tree4328d05e;
report `tooling/ci-oct7/grapple-roll-details-0990-independent-review-2026-10-07.md`
SHA256 be6de7795b52120247a2bcb91d39a996accc634ddbcb26d9c9c0269c8cd1a20a.
Its independent731-entry audit retains1116 old Rust bodies and49 protected
files, with exact history/DTO/authentication/submission inverses.

Exact0990 now passes the ordinary full frontend suite:223 tests in32 files,
including all16 new rendered cases, zero Svelte errors/warnings, and the149-module
production build. Source remained clean and byte-identical throughout.
Evidence: `tooling/roll-details-0990-frontend-2026-10-07`, test log SHA256
88cb5c749d56a4fe05d0c5c0d2d12ce94dbb29314fd42be2aba9569cb3a527c2.
Canonical `scripts/verify-fast` also passed at16:56:49 UTC on GNU Rust1.98.1,
one build job, normal profiles and no test-thread/stack override. It checks
formatting and the entire locked workspace/all-target compilation; it does not
execute the four backend cases. Log SHA256
793173b01eb2fcf8bb238d02b8cd10dddefd78fce0b19ca2536fbff604c887a7.
Both original source-specific evidence directories remain immutable.

Before publication, receive the whole reviewed combined successor
`1ffb841f67e87c4239b68dc6ff5e045e0395f5c0`, treef30559a1, after committing
this plan. That successor normally received Heldb97's actual spatial fixture
correction: two legal identifiers, explicit encounter validation and an exact
wall-obstruction refusal, with every existing assertion preserved. The current
0990 inherits the same two invalid old fixture IDs. Root reviewed the actual
failure, full donor delta, source inverse and complete1ff union. No production
or details implementation changes are intended by this receipt.

Expected receiving changes are exactly the corrected test, the complete Held
plan and combined receiving plan, plus this details-plan appendix. All details
production/new scenarios, UI, historical data and remaining1114 original bodies
must stay literal. The two corrected bodies are separately inventoried; do not
claim all1116 remain unchanged after receipt. Inspect the actual merge and
publish a stacked draft for fresh exact-head CI. Earlier0990 passes retain their
source attribution and cannot stand in for successor backend/CI evidence.

The original combined run is separately allocated to1ff. This branch receives
no local runtime slot during that run. Genuine M+G/T5 and immutable969 restored
details require a later deliberate receipt into the separately owned mass
consumer branch. Full Rust execution, strict checks, canonical verification,
native use and the complete Gate4 encounter remain required; no acceptance is
waived by source review or the passing frontend/compilation results.
