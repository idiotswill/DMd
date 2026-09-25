# Gate 4 — Source creature control at the table

Status: planned; implementation and verification pending.
Sole writer: environment_audit. Branch: `codex/gate4-source-creature-control`.
Authorized base: `100c7dabe07b07b7430bcb721b1dd7f48e4792cf`.
Fetch on 2026-09-25 found main `a84c5a1`; the owner of integration deliberately
requested this frozen prerequisite base. No unreviewed main merge is implied.

## Objective and authority

Give an attending player genuine control of an authenticated source creature through
the actual desktop, including actor selection, physical tactical dice and the Mage's
own-turn Mage Armor. Preserve old accepted table requests, historical presentation
hashes and existing player-controlled source data. Never fabricate a Character row.

This advances the product definition's actual application, physical dice, private
identity, save/restart and recovery requirements within Gate 04. ADR026 governs the
shared tactical flow; ADR027 governs audience revisions and exact transport recovery;
ADR028 separates actor control from execution-version and ordering ownership.
`gate4-live-reaction-responses.md` remains root-owned and required Gate 4 work.

No reaction windows, new execution version, spell-grant expansion, NPC autonomy,
full class catalog, or natural-language NPC declarations are implemented by this
slice. Existing own-turn source mechanics and PC conversation remain authoritative.
No build, frontend run, push or PR until coordinated with root; another worker owns
the shared heavy-process slot at plan creation.

## Audited starting boundaries

- Table creation makes source creatures Autonomous. Internal source construction and
  SetContext already support Player ownership; absence of such historical data cannot
  be assumed. CreatureRuntime.controller/control_origin remains the sole owner truth.
- Table transport v1 selects a CharacterId; table tactical admission, projected owned
  actors, pending-roll visibility and desktop controls currently assume PCs.
- Kernel PC sheet/owns logic must not be widened to level-zero source creatures.
- Mage Armor is executable, but frozen table_casting options omit Caster target rules.
  Add its self-target option only under the explicit new presentation semantics.
- SQL0011 accepts JSON and protects immutable rows; it has no record-version check.
  Rust load/insert/portable validation has six version-one checks to version explicitly.
- Existing table.action@2 and table.conversation@2 retain transport-v1 meaning forever.
  Old UpgradeExecution remains Legacy-to-ReactionsV1. Control is not inferred from
  encounter flow version, and source control persists outside encounters.

## Planned coherent slices

1. Capture/locate genuine pre-change presentation and exact-retry evidence before
   changing projectors. Prefer an authentic old Player-controlled source history;
   if unavailable, state that limit and distinguish typed compatibility fixtures
   from genuine old binary output. Do not manufacture or bless old hashes.
2. Add a default/skip-none TableState source-access marker and explicit host table
   activation/adoption. Authenticate all existing Player-controlled source actors,
   their immutable profiles and actual control origins at activation. Retain the
   original host command and adoption evidence; this is provenance, not a new owner
   map. Add journaled controller assignment/revocation using current source ownership
   and preserving lair/context semantics. Reject pending or held work, including Ready.
3. Add deliberately versioned player actor transport, canonical audit interpretation,
   and projection dispatch. Freeze v1 DTO bytes and projectors. New player selection
   is Character or SourceCreature, separately authenticated from player identity.
   A source-only attending player needs no fake PC binding. Exact accepted nonce/body
   recovery precedes current version, actor, session and revision checks. Never silently
   normalize an unaccepted old envelope or rewrite an accepted v1 body.
4. Bind new projection records to the historical marker; preserve before-v1/after-v2
   activation and bootstrap in one BEGIN IMMEDIATE transaction. Authenticate audit
   markers, record/binding versions, origins, capabilities and exact responses during
   replay before restore writes. Legacy schema/export inputs reject future authority.
   Keep SQL0011, schema4 and export3 unless a concrete compatibility reason requires
   a separately documented change; an old binary already rejects unknown record2.
5. Expose host activation/assignment and player actor selection, genuine source-aware
   own-turn options, self-target Mage Armor and actual owned tactical roll forms.
   Preserve the full original outbox across restart/retry and discard stale query
   replies. Text declarations remain tied to actual supported PCs. No implicit host
   reaction control or change to source monster/mixed initiative tie rules.
6. Run focused source/control/transport/restore/UI tests in the granted slot, obtain
   independent exact-diff review, then root integrates and runs canonical/package
   verification. Report limitations and failures accurately; this is not Gate4 acceptance.

## Compatibility and admission decisions

Activation is a new host TableAction, independent of executor upgrade. A future
explicit targeted upgrade could include it only by retaining the table semantic
target and writing the same marker; old upgrade actions cannot acquire this meaning.
The activation boundary must be settled: no table decision/roll, raw rules request,
shared resolution, source routine/recharge ticket or held Ready/cast. Assignment uses
the same restriction, rather than relying on SetContext's narrower local guard.

The activated projector uses source-authenticated actors and current runtime owners.
Other audiences do not receive controller lists, private actor names or hidden roll
work. Source roll admission still requires the current owned opaque capability and
actual roller; secret/host-only source rolls do not become public. Do not globally
change legacy kernel ownership or sheet formulas.

Use explicit transport v2 and projection-record v2, with a distinct canonical audit
envelope version for new requests. Version-one records, responses and numeric legacy
recovery keep their original decoder. Newly submitted v1 requests after activation
must refresh, while genuinely accepted v1 requests still recover exactly. The marker
is mandatory evidence even without source assignments; removing it, its activation
event or the new ledger cannot trigger a legacy fallback.

## Acceptance criteria

- Real host catalog/CreateCreature creates Mage unchanged; explicit activation and
  assignment give one present player the actual actor, without Character/profile/slot
  fabrication or a free change to physical equipment/source uses.
- The player selects Mage, rolls real initiative through an opaque capability, and
  on its own turn selects/executes source Mage Armor with genuine material and source
  cost. Other player and foreign actor/channel/raw identifiers reject without writes.
- Source-only session attendance works under the activated semantics; old PC-only
  session and text paths replay unchanged. A source NPC remains TacticalSource::Creature.
- Activation/adoption retains every genuine prior Player-controlled source owner;
  no old control is automatically projected by merely opening a campaign with the
  new binary. Existing v1 hashes and original request/response bytes stay unchanged.
- Pending/held work blocks transfer; a settled reassignment/revocation is journaled,
  rotates only changed audience presentations and survives reopen. Previously accepted
  exact requests still recover after ownership/session changes; fresh stale/foreign
  requests reject atomically.
- Cold file SQLite at assignment, owned raw pause and casting completion; independent
  portable restore agrees. Tampering actor/source/control_origin/adoption/marker,
  record version, audience, capability, binding body or response fails before writes.
- Stripped new records/markers and legacy envelopes carrying new authority fail closed.
  A genuine old corpus validates byte-preserving compatibility; synthetic tests are
  clearly labelled and never replace that evidence.
- Desktop tests drive actual host assignment and actor selection, then uncertain
  delivery/restart with the full original version/channel/actor/revision/body retained.
- Independent review plus focused Rust/app/persistence/UI checks on the exact final
  source; canonical checks and native evidence remain root-owned and explicit.

## Verification commands and status

Planned focused commands (only with the shared slot): source-control table/transport
tests, persistence protocol/schema tests, all dmd-app tests, strict Clippy for domain,
rules, persistence and app; desktop npm check/test/build. Run repository fmt/diff
checks before source checkpoints. Root runs scripts/verify on the combined candidate.

At plan creation: AGENTS, product definition, Gate4 execution documents and ADR027/028
read; base clean; fetch completed. No implementation/tests/builds have run for this
slice. Prior branch checks are not evidence for it.

## Risks and next action

The transport/history change crosses several strict serializers. Preserve old variant
shapes and version-dispatch each acceptance path, including trusted internal table
commits after presentation initialization. Restore origin allowlists must be narrow,
with semantic action replay proving the actual change, not broad table-event permission.

A genuine old source-controlled fixture may not be constructible through the current
public table API. Root has been asked for an existing corpus; search authenticated
old/internal paths before claiming that evidence. Begin with the marker/control reducer
and explicit compatibility types, keeping old projectors unchanged until fixture
capture is settled. No compiler or frontend process until root releases the slot.
