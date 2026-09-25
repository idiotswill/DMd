# Gate 4 physical encounters and source casting consequences

Writer: root. Branch `codex/gate4-physical-encounters`, based on fetched merged main
`12ed29a72eb3d66ad8565caf358ae573a91aa0dc` after PR32. Reference integration is
`0fd7ddc348ab49f576a0427f5899984c91d32de4`; this document is committed on the fresh
branch before extraction. Gate4 remains active.

## Objective and scope

Connect real source weapons, paid shield changes, movement, opportunity attacks,
damage, knockout and falls through the existing table, physical dice and durable
turn scheduler. Advance the product's tactical rules, player agency, perception and
exact suspension requirements, Gate04 and ADR024/026. Preserve the current rules
coverage ledger; this slice is neither complete combat nor Gate4 acceptance.

Attacks and movement share interrupt state: crossing reach must suspend the original
segment, the selected owned reaction must resolve before movement resumes, damage
can break concentration/drop held gear, and lost flight/jump support can cause a fall.
These consequences must travel through the same source-authenticated queue. Treat
them as one coherent physical encounter objective rather than splitting their state
transitions into temporary implementations.

Include the already reviewed Immediate-only source casting/program execution.
The intended real damaging-opportunity-attack/concentration case needs a genuinely
installed ongoing spell; main plus PR32 has no legitimate casting entry and must
not gain an arbitrary effect-install escape hatch. Source Hold Person provides the
real prerequisite, while spell/physical attacks already share retained proof,
queue and vitality validation. Independent architecture review supports this
coherent expansion over stripping those interdependent paths into temporary stubs.

Area breath activation, new Ready/reaction casting and the separate privacy
protocol remain followups. Keep the exact supported source/program subset explicit
and reject unopened actions before costs. Do not claim general spell completeness.

## Planned extraction

1. Port current source attack, physical creature weapon, intrinsic melee/unarmed
   damage and opportunity adapters; source inventory/loadout/ammunition remain real
   identities. Include currently implemented Light/Nick/Graze choices, owned knockout
   choices, paid Don/Doff Shield and source/training AC. Do not claim unimplemented
   mastery/Savage Attacker/grapple/shove/Ready mechanics by association.
2. Port reviewed path/segment and falling reducers, perception fixes, jump clearance,
   supported movement modes and default-stack real landing scenarios. Re-enable
   airborne/liquid admission only with its actual automatic consequence resolver.
   Hidden truth is never a client-side target or route authority.
3. Activate shared queue dispatch/validation/raw history for these mechanisms and
   the reviewed source casting subset. Keep fixed raw-role numbers and original
   provenance; preserve actual cast grants, components, resources, target binding,
   player saves and separate concentration children. Ready and reaction casts stay
   explicitly unavailable before costs until their full source path is integrated.
4. Port production table projections/forms and actual SQLite tests. Preserve PR32's
   visible-purpose roll routing and cold-round tests, adapting only assertions that
   intentionally change when physical capabilities become available. Retain actual
   unsupported casting-mode and forged original-anchor rejection regressions.

## Acceptance and verification

- Demonstrate a real physical attack with raw hit/damage, item/hand/ammunition costs,
  source NPC weapon choices and damage/knockout durability through TableAction.
- Demonstrate actual damaging opportunity interruption, concentration child and
  resumed/rejected movement under source consequences. A miss/decline alone is not
  this evidence. Controller choice and Reaction use remain explicit.
- Demonstrate lost-support falling and each liquid landing option, retained request
  identity, default Windows stack, actual disk reopen, mirror restore continuation,
  same-command retry and no double spending. Invalid or forged input writes nothing.
- Player view excludes private map truth, target statistics, hidden NPC identities
  and raw source authority. The separately active per-audience transport privacy
  work remains required before Gate4 acceptance; do not hide known counter/ordering
  leaks or silently defer them to Gate5.
- Obtain independent full extraction review, run focused source/app/UI checks then
  canonical verification, all six exact-head CI checks, protected merge, fetched
  full-tree parity and post-merge checks. Prior integration source results do not
  count as verification of this new extraction.

The expanded diff is roughly25k lines, substantially test fixtures, from individually
reviewed source slices. Fresh review of the actual combined tree must separately
cover source/casting admission, shared attack/movement/fall order, application/
restore/origin/privacy boundaries and real SQLite/UI recovery. Earlier leaf green
heads never substitute for this combined verification.

## Current evidence and next action

PR33 is open at https://github.com/idiotswill/DMd/pull/33. Initial extraction
`b7f5dfb` passed all six checks: Linux36129213345 (590 Rust tests) and
Windows36129213349 (MSRV/stable including whole-workspace tests and installer).
Three independent bounded reviews cover combined source/admission, movement/attack/
fall recovery and application/projection/restore paths. The dead-target finding
below is corrected; the final combined head still needs its own full verification.

The admission correction is `a58d4dc`; all 71 affected attack tests pass locally
after correcting the two new fixture assumptions documented below. The genuine OA
test is integrated as `5e3f89a`, with its focused source result and independent
review clear. The real lethal/dead-target SQLite test is integrated as `9910ce7`
after independent review and passes on the default Windows stack after a two-line
test-helper mutability correction (one scenario, 9.66 seconds). All lethal/retry/
next-turn/zero-write/replay/restore assertions ran. Full canonical verification, UI checks,
fresh final CI, protected merge and post-merge checks remain required.

PR32's post-merge checks are now all green on main `12ed29a`: Linux36129020814 and
Windows36129020852. No base/source reconciliation is pending. The twelve Gate4
ledger families now correctly say `implementing`; scopes and evidence arrays stay
unchanged and no complete family or gate acceptance is claimed.

### Extraction provenance (earlier evidence and pending state at extraction)

Initial extraction is present; combined verification and acceptance remain pending. Reviewed source heads
and individual verification are recorded in the central Gate4 and weapon/movement/
falling/shield plans. PR32 source70b8333 has425 passing Linux Rust tests and424
passing local Windows GNU Rust tests under canonical verification, strict lint,
MSRV and guards. Its final Windows workflow now tests all workspace crates. That
expanded check caught only a non-persisted export timestamp comparison; reviewed
correction25be7f6 retains equality of all durable data. All four corrected Linux
jobs pass; the expanded native Windows test/installer run remains pending.
PR32 is now merged with expected-head protection after independent review and all
six checks on25be7f6: Linux36127754177 (425 tests), Windows36127754144 (427 tests,
MSRV/stable and offline installer). Fetched main12ed29a has exact full-tree parity.
Post-merge CI remains to be checked. The concrete source-program dependency is recorded above;
port the reviewed Immediate-only implementation and preserve the one scheduler.
Full Ready/reactions, remaining spells, NPC morale/knowledge,
improvisation, encounter finish and packaged integrated acceptance remain Gate4 work.

Combined source `49fac8c` passes all four Linux jobs in36131323977, including596
Rust tests and both new real recovery scenarios. Native Windows36131323987 passed
frontend checks and MSRV, but its stable workspace test run failed after every OA
scenario assertion, at final temporary-file removal with Windows sharing error32.
The other24 table-loop scenarios passed. A reviewed test-only correction closes
the pools, explicitly drops the fixtures/runtimes and retries only Windows sharing/
lock errors32/33 for at most1.9 seconds; persistent or other failures still fail
cleanup. No gameplay assertion or production source changes. Fresh canonical/UI/
exact-head CI remain required on the correction.

Exact cleanup head54520a9 then passed canonical verification locally:595 Windows GNU
Rust tests, workspace all-target Clippy, formatting/check and both guards. Its
native Windows36132430076 confirmed the corrected OA scenario passes, but caught
the same final-file sharing error in the older casting recovery fixture. The same
reviewed close/drop/bounded-sharing cleanup is therefore applied across this table
suite's disk fixtures. Recursive fixture cleanup additionally verifies the resolved
directory is a DMd-named direct child of the system temporary directory. No source
or scenario assertions change. Reverify this final test tree before merging.

Reviewed source training correction1b39980 is integrated in456272e. It follows SRD177:
untrained Shield use removes its AC bonus but does not itself forbid casting;
untrained worn armor and occupied component hands still prohibit the affected cast.
All43 source spell tests and strict domain/rules all-target Clippy pass, with
independent exact-head review. The initial genuine Cultist fixture lacked Hold
Person's material; the corrected fixture supplies that real component rather than
weakening admission. The correction changes no app/UI or reaction execution. Include
it in this combined source slice and rerun canonical and real app verification.

The new OA recovery scenario has a separate owned test branch planned from456272e:
PC0 equips a real Dagger through an ordinary attack, a source Cultist genuinely casts
Hold Person, then moves away and provokes PC0. Raw hit/damage must lead to the NPC's
concentration save before the original movement can resume. Test every accepted pause
on reopened disk and independently restored runtime, exact retry and forged restore.
It must preserve the reactor's own Reaction payment and the active mover's budget.

Before extraction, the reference integration preserves PR32's reviewed actual disk
round/retry/restore regression (including its export request timestamp correction).
Only intentional capability assertions change: the active owner's attack/movement
options now exist; another player's remain absent. Forged-budget restore rejection
expands from other_slot_casters to all seven former unopened budget examples, each
with current-image and backfilled-anchor corruption and zero-write checks. The two
former closed-action/work tests now retain rejection of source-less actions and
work missing its retained continuation. A blanket ban on every elevated position
is replaced by the separately implemented and verified real falling/landing tests.
These reference-test adaptations are formatted only; their runtime verification is
pending the combined main-based PR and serialized compiler. Production is unchanged.

The fresh branch's pre-code plan is5d250ba. Initial extraction copies the exact
reviewed reference source0fd7ddc for crates/apps/content/manifest/lock while preserving
main's harmless domain violation-enum order and expanded native Windows workflow.
The existing turn-core source and cold-round regression remain present. Historical
source plans accompany the integrated code as provenance, with this plan and actual
combined verification authoritative for this PR. The new real damaging OA recovery
scenario is still being authored separately and is required before acceptance.

Fresh combined review found that ordinary and intrinsic attacks could
admit an already-dead located target, spend resources and later fail damage
resolution. The printed source-weapon path already rejects this case. Before final
verification, share an admission-only target check across these four paths, with
actor knowledge checked before vitality. Keep retained source reconstruction legal
after an attack itself kills its target, and keep living zero-HP targets legal.
Do not filter player contacts using an undisclosed death flag. Add resource/no-write,
knowledge-first and lethal-completion regressions before accepting this correction.

The first new tests exposed two fixture assumptions. A lethal melee hit pauses for
the owner's knockout choice; the corrected lethal OA test explicitly chooses normal
damage. Mutating an accepted mover into a dead body cannot represent a valid pending
crossing: existing movement validation rejects its changed capability before attack
admission. That artificial test is removed, and the actual lethal OA completion test
retains full replay and movement-stop assertions. The new shared OA admission check
is defensive consistency, not a demonstrated bypass of the existing movement guard.
