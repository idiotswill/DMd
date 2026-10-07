# Gate 4 — Genuine pre-change v4 compatibility captures

Status: source-only harness authored; runtime pending, 2026-10-07. Sole writer is
`native_capture_audit_oct7`; branch `codex/gate4-v4-compatibility-captures`,
checkout `gate4-v4-compatibility-captures`. Base is fetched exact
`58696ac1d0c55ef7f71cfb546fb92e97747ee437`, tree
`ddd6d01102c5ecbb2b6df77c633336203f582d78`. Fetched main is
`1a9de14c8a4418893b6664b89f99f0a0c0225ce1`. No runtime result on this branch.
Root's separate exact586 focused runner owns heavy execution.

## Objective and boundaries

Produce reviewable, genuine earlier-version application artifacts before the
authenticated-load-facts work changes v4 consumers. This advances exact combat
recovery, player authority and private information from the product definition,
Gate 4 and ADRs009,011,012,020,024,027,028,029. Root AGENTS, the gate protocol,
roadmap, relevant current producer tests and ADR027 were read. The current
combined-receipt and Inspiration/Ground work remain authoritative behavior.

Existing same-build cold/portable tests prove replay but discard their temporary
databases; they are not a frozen previous-version oracle. Existing checked-in
captures concern earlier reaction/Flow4 routes, and the retained Physical native
copies contain v1/v2, not v4. This bounded harness fills that evidence gap. It
must not fabricate an archive or claim that a source-only test body has run.

No production, content, dependency, source pin, previous test body or existing
fixture bytes may change. Add new capture support/tests and module registrations
outside existing bodies only. Reuse the actual Fixture/cold/restore helpers and
real accepted inputs; do not insert positive state, SQL rows, rulings, resources,
grips, source grants, projections or history. No new game rule or mass feature.
No native database or native UI access. No push/merge during this allocation.

## Artifact and harness design

Add a common test-only capture writer and two child route modules, registered
beneath the existing Ground and Inspiration-transfer helpers so their private
genuine producers are reused without changing visibility or implementation.
Normal tests execute the route assertions. An explicitly supplied external output
directory additionally persists artifacts; absence of that directory never causes
a skipped/ignored test or a false captured-result claim.

Each named cut captures the authoritative state *before a concrete next request*,
then accepts that request using the unchanged `Fixture::cold`, obtains its exact
retained response and captures the after image. Thus every advertised next input
has actually succeeded when an archive is eventually generated. The after image
is an expected result, not another advertised resumable cut lacking a next input.

Save complete typed CampaignExport before/after, all original audit/event strings,
snapshots, projection records and transport bindings. Separately retain the exact
original request/response JSON strings from the binding; do not substitute a
reconstructed lookalike. Save the complete actual TableEvent/outcome, next request,
Host and every fixture Player raw/presented DTO, transcripts, revisions, handles
and roll-option responses. When a real source controller exists, record the actual
SourceCreature channel alias and its same Player audience explicitly rather than
pretending that it is an independent fourth audience.

Source manifests identify exact executing Git head/tree, fixed pre-change base,
tracked file/blob identities and source/harness hashes; the separately reviewed
runner also pins toolchain, command, environment, output path and complete logs.
Archive files use create-new semantics in fresh per-scenario subdirectories and
SHA256 manifests. Reject output inside the checkout, dirty source or an existing
scenario archive. Failed runs may leave partial files, but only a final manifest
written after all route assertions constitutes a completed scenario. Never edit
or overwrite a prior capture.

Read snapshots must not mutate durable history. Compare exports around DTO/options
reads, normalizing only the export timestamp. Each accepted request's saved binding
must match the actual input and retry result. Preserve complete prefix arrays;
do not normalize command IDs, source IDs, audiences, revisions or capabilities.
Random IDs from independently accepted mirror branches are not expected to match;
the existing cold helper checks canonical outcomes/state and each branch's own
exact retry. The archived original branch retains its actual random identities.

## Exact planned routes

### Ground, v3 upgrade and a paid source opportunity

Use `ground_transport::opportunity_fixture`, whose real Host battlefield gives
the required reciprocal hostile relationships. Activate Grapple v3 and retain
the original activation request/binding. Capture the genuine v4 Ground transport
activation, then the current AttackEquipment activation through its real Host
action, before spending the Action. Establish and finish the PC's real left-hand
grip through the unchanged actual choice/raw/cold helpers and explicit v4 inputs.

Assign the independent reactor to Player1 through genuine settled source control;
the grapple target remains Host-controlled. Capture Host/PC0/PC1 projections and
the actual source-channel view/control. PC1 is an uninvolved audience before that
assignment; its later source authority must be clearly labeled.

Move the pair through `(0,10)` then `(0,0)` as in the existing nonempty-prefix
control: first step commits, second departure offers the independent reactor.
Assert the one-step prefix, both endpoints and 20 half-feet paid cost. Capture
the actual source OpportunityAttack acceptance, its pending physical raw input,
and explicit holder release while that raw is issued. The same issued child and
paid prefix must survive release; accept a controlled physical miss and capture
the stopped result through a legitimate next EndTurn. Retain one Reaction,
original raw/provenance and completed prefix. Replay the original v3 request after
v4 activation and confirm its exact response without durable changes.

### Inspiration award, recipient/decline and physical reroll

Use the genuine Fixture and existing Inspiration helpers in two separate scenarios:
recipient and decline. Enable v4 and current AttackEquipment through real inputs.
Capture first Host award, duplicate/excess award with its actual pending owner
choice, and the actual opaque recipient or decline control. Assert one original
resource, the correct recipient result and cleared pending state.

Use the existing genuine target-PC sequence to create a PC Strength save against
the source Goblin's Grapple. Capture the pending roll/options with an actual
SubmitRollWithInspiration request (controlled original 1, replacement 20); then
capture the consumed resource, original/replacement raw evidence and real outcome
through the legitimate Host after-equipment completion input. Finish with a
settled next turn input and retain all prior accepted retries. These are authored
QA faces; no human physical throw is claimed.

Both scenario families preserve actual source/creation history and all existing
opaque authority. Host/owner/uninvolved audience information remains available
in the full original projection history even when a later participant becomes
involved. Capture failures must identify a real producer gap, not patch it away.

## Acceptance and future consumer

- Full source review proves production/content and all 1109 prior Rust test bodies
  exact, plus existing frontend/fixture entries; only reviewed additions exist.
- The harness compiles and its actual routes pass on its exact frozen head under
  root's serial heavy allocation. No such result exists at plan time.
- Each completed archive has all required cuts, complete hashes, accepted next
  requests/results, actual v3/v4 bindings and genuine original replay provenance.
  Independent artifact audit verifies every hash/prefix/source identity and absence
  of overwritten history before using it as a compatibility oracle.
- A future new-version consumer restores each frozen export normally, compares old
  persisted records/DTOs/retries exactly, then completes pending work legitimately
  before new activation. It compares new acceptance canonical state/outcome and
  per-store exact retry without requiring independent random allocations to match.
- Canonical verification and CI on changes that use these fixtures remain required;
  source review and a capture pass do not establish full Gate 4 or native acceptance.

## Execution sequence and current status

1. Commit this plan before harness edits.
2. Author the common writer and three positive capture cases using real helpers;
   read their entire paths and verify every advertised next input in source.
3. Coordinate any direct rustfmt with root so it does not overlap root's verify-fast
   formatter. No compiler, Cargo, npm, tests or database execution in this turn.
4. Freeze a coherent code/status commit, full diff and preservation/source manifest.
   Request fresh independent review through root, not routine user approval.
5. Root schedules exact-head compilation/runtime and capture output later, after
   the current heavy runner is released; preserve all failures and partial output.

Current blockers/limits: source authoring only; no generated archive, runtime,
native or compatibility acceptance. Remaining mass/carrying and Gate 4 human
playtest obligations are unchanged. The authored freeze below records the next
source-review and execution steps.

### Source-derived route ordering correction before route code

`attack_equipment_access::activate_with_context` requires an unused settled turn,
including no spent Action or retained attack window. Therefore establish the grip
after, not before, the v4 transport and GroundEquipment activations. Retain the
genuine v3 Grapple activation request, capture v4 activation, capture GroundEquipment
activation, then use the unchanged actual choice/raw/cold helpers with explicit
v4 envelopes to establish and finish the grip. This preserves the original
producer and avoids activating a feature through paid work. No production or old
helper changes are needed. Inspiration captures use the actual current Physical
Glaive creation helper, whose real source pin/purchase/materialization history is
then retained with their awards, transfers and PC saves.

## Authored harness freeze, 2026-10-07

The authored delta adds only `v4_capture_archive.rs`, `v4_capture_ground.rs` and
`v4_capture_inspiration.rs`, plus three module registrations outside original
bodies. It uses three positive test cases with 22 planned named cuts: 8 Ground and
7 in each Inspiration branch. Ground includes a genuine Player-controlled source
reactor and a released-but-still-issued raw child over a paid one-step prefix.
The Inspiration branches use the actual current Physical Glaive creation helper.
Every cut calls the unchanged cold helper with its actual next input; no existing
helper or prior test body was edited. The baseline helper's separately reported
negative-Dodge issue is preserved; these advertised next inputs are never Dodge.

Each cut writes 8 artifacts when capture output is enabled: complete before/after
exports and audience DTO/options, original request/response JSON from the saved
binding, actual original TableEvent JSON and the complete binding. Additional
retained retry records preserve the v3 activation and prior awards/transfer/reroll.
Final manifests identify the exact source tree and hash every artifact. Dirty or
changed-production source, a compiled capture/source mismatch, an in-checkout
output parent or an existing scenario directory is refused. A failed partial
directory has no completed manifest and must never be relabeled successful.

The writer compares complete exports around reads/retries, ignoring only export
timestamp. Audit/events/projections/snapshots retain exact chronological prefixes;
transport bindings are sorted by command UUID, so removing only the new identity
must reproduce the complete prior binding vector. Cause rows receive the same
identity-aware comparison. No UUID, original JSON string or audience evidence is
normalized away. SourceCreature snapshots explicitly alias their controlling
Player audience; there is no invented separate source projection audience.

Both Inspiration routes retain genuinely uninvolved Player 1 across the first
private Host award and the duplicate award's pending owner choice; the decline
route also retains that audience after the owner's decline. Each checks the
unchanged original private-history helper and exact serialized presented DTO
bytes, including revision and opaque handles, against the pre-award cut. The
internal raw view's globally advancing canonical sequence is treated exactly as
the original helper specifies; all audience-visible fields remain equal. The
recipient becomes involved when the gift is accepted. Ground's Player 1 becomes
the actual reactor controller and is never labeled uninvolved at its OA cuts.

Later runner entry points in the existing `table_grapple_public` target are:

- `ground_transport::v4_capture::capture_v4_ground_upgrade_paid_prefix_and_issued_source_raw`;
- `inspiration::transfer::v4_capture::capture_v4_inspiration_recipient_and_physical_reroll`;
- `inspiration::transfer::v4_capture::capture_v4_inspiration_decline_and_physical_reroll`.

Run these exact cases serially only after source review and root's heavy allocation.
Set `DMD_V4_CAPTURE_DIR` to an existing absolute external parent, with the three
scenario child names absent. The tests always execute their assertions; without
that variable they simply produce no persisted capture. The runner must pin the
frozen source/tree, toolchain, executable hash, command/environment, complete log
and external output manifest. No Cargo/test command has been run by this author.

Direct changed-new-file rustfmt and Git whitespace checks pass; these are not
compiler or runtime verification. Source preservation audit and fresh independent
review accompany the frozen handback. No archive exists yet. Root must inspect
the complete delta, schedule exact-head execution, audit actual output and then
record its precise results. Future-version consumer verification, canonical checks,
CI, native behavior and Gate 4 physical-human acceptance remain pending.
