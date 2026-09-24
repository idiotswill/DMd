# Gate 4 physical inventory reducer

Writer: root on `codex/gate4-inventory-verified`, starting from verified main
`da964869bfc3ebf61b7b1a46d9204f0ff55c6693`. This is a bounded dependency slice
of the active encounter integration; it does not claim a completed playable encounter.

Status: reducer implementation verified. [PR26](https://github.com/idiotswill/DMd/pull/26)
is the review/merge surface; final documentation-head checks precede protected merge.

## Objective and boundaries

Materialize source-derived starting allocations as real physical ItemIds with bounded
quantity, custody and loadouts. Validate every recognized loose/held item consistently,
including NPC gear, and retain original grant identities without recreating expended or
transferred supplies. Unknown campaign objects remain available for later source-bound
improvised adjudication. No label proves proficiency or a source capability.

The pure reducer takes an explicit inventory attachment and returns validated state and
inventory together. Main's RulesState/save interpretation, table actions and legacy
events remain unchanged in this PR. The parent integrates the attachment, durable
authority/replay and desktop preparation; later attack work consumes the same ItemIds.

Relevant authority: product-definition rules fidelity, durable state/provenance and real
player equipment; Gate4 physical attacks/improvisation; source-pinned SRD5.2.1 equipment.
The parent's proposed ADR026 defines the shared resolution path. No alternate inventory authority
or parallel attack engine is introduced here.

## Acceptance and verification

- Source allocations, prices/quantity multiples and one identity per ordinary weapon
  agree with the pinned catalog; finite ammunition uses explicit physical stacks.
- Initial provisioning rejects existing allocations, duplicate/nil IDs, incorrect source
  choices and invalid custody without partial mutation.
- Hand/armor/shield references describe real carried intact items; invalid unheld known
  quantities/states are rejected too. A holy symbol alone grants no casting focus.
- Preserve source field names and provenance. Inspect the full exact diff separately
  from implementation; run focused regressions, canonical verification and final-head CI.
- Merge only with expected-head protection and record post-merge tree/CI evidence.

## Implementation and evidence

The four implementation files exactly match parent `37d8044`; only module exports were
adapted for main. The original twelve regressions were retained without fixture changes.
A thirteenth regression independently covers loose NPC gear without a PC receipt or
loadout, legal spent ammunition and unrelated custom campaign objects.

Independent full source review at `ff08f26` and exact test-delta review at
`fdd4a48e9e8bb47292df346cf9ca793979e658d9` found no blockers. On that code head,
canonical `./scripts/verify` passed: formatting, workspace/all-target check, strict
workspace/all-target Clippy, **309 Rust tests**, genericity guard and architecture guard
(eight tests, one platform-specific skip). Local evidence is retained outside the repo
in `tooling/gate4-inventory-fdd4a48-verify.log`.

All six jobs passed on that exact head in [Linux CI](https://github.com/idiotswill/DMd/actions/runs/36062884008)
and [Windows desktop](https://github.com/idiotswill/DMd/actions/runs/36062884120), including
Rust1.88 compatibility and stable Windows packaging. This final documentation delta has
no code changes; rerun canonical verification and require its own CI/review before merge.

## Next action and remaining gate work

Merge PR26 with expected-head protection after final checks, fetch main and verify exact
tree equivalence and post-merge CI. Reconcile the dependency into the parent encounter
branch. Table creation/replay, NPC preparation, current armor, attacks and source spell
components already consume these ItemIds there, but their integrated acceptance remains
separate. Full Gate4 movement/reactions/casting and packaged encounter acceptance are
still required; this pure reducer does not satisfy them by itself.
