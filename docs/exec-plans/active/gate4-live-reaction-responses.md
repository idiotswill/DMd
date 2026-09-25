# Gate4 — Live reaction responses and Ready release

Status: required active Gate4 work; runtime windows not implemented.
Writer: root. Branch: codex/gate4-live-reaction-responses, created from freshly
fetched main d5d1db76532be6f4f0b2f6fc13c78e8bd8d7cc38 after the protected PR38
foundation merge. Umbrella: gate4-reaction-ready-runtime.md.
Authority: product definition, Gate04, ADR026/028, pinned SRD5.2.1 source and the
complete audit gate4-reactions-and-ready.md. No requirement is transferred to Gate5/6/9.

## Objective and slices

1. Document an explicit new execution-version boundary so newly introduced pauses
   never reinterpret already accepted ReactionsV1 or Legacy history. Preserve old
   suspended continuations and exact retry; upgrade only at a settled boundary.
2. Attach actual source trigger windows to the existing frame stack. Preserve the
   original declaration, triggering command, work cause and suspended physical cursor.
   Avoid a second queue or overloaded single attack/movement slot losing its parent.
3. Implement hit/target Shield and cast-interrupt Counterspell, including Magic
   Missile before its first dart, natural20, current source grants/components,
   60-foot visibility, CON save, shared creature uses, slot exceptions, nesting,
   concentration replacement and revalidation before paying a selected response.
4. Implement witnessed physical Ready release/ignore/attack/move and paid held spells.
   Declaration pays casting resources and can be countered; holding requires actual
   concentration; release pays Reaction, binds current targets and never casts/pays
   again. Expiry/abandonment remove only the matching held source.
5. Real private desktop/table decisions, uniform public ordering stage, exact-trigger
   delegation and actor-known explicit total order without default or arrival priority.
   Preserve unrelated DTO/revision/transcript through zero/one/two hidden offers;
   child triggers require distinct decisions and cannot inherit parent delegation.
6. Genuine file-SQLite source scenarios: reopen at offers/order/nested save/attack/
   hold/release, exact accepted retry, independent semantic replay and hostile anchors.
   Review and verify canonical/UI/Linux/native exact heads; merge protected and check
   main parity/post-merge before claiming this execution objective complete.

## Concrete design constraints

Current attacks validate raw request and source facts against live state. Shield or
nested source effects need an authenticated pre-reaction fact image and deliberate
post-response hit evaluation; do not merely relax source validation to accept client
numbers. The original attack raw record's accepted_by remains the cause of its later
damage request, even if an ordering/reaction command resumes the queue. Actual nested
physical responses require typed parent ownership for attack/movement and their work.

Collect source-safe response intents privately, record the actual turn controller's
ordering instruction independently of eligible counts, then revalidate each selected
intent after prior responses. Stale accepted intent is not prepaid permission. Hidden
creature identity must not leak through ordering options, revision changes or errors.

## Validation status and next action

No live window/release implementation or passing runtime evidence exists yet.
Foundation tests are prerequisites only. Current source, product requirements and
ADR028 are read. First retain the genuine prior saves and their portable regression
coverage, then implement the explicitly versioned Shield vertical path while keeping
Counterspell and all Ready mechanisms mandatory here.
The full twelve-family Gate4 ledger and eighteen spell mechanisms remain binding.

Compatibility PR42 head7616cf7a4ccef34505bfdb117a4077bf858d783c has now passed all
four Linux checks in36177043229, including691 Rust tests. The three newly committed
genuine ReactionsV1 continuation tests pass together in58.11s on the default stack:
actual old AttackRoll completion, KnockoutChoice, and privately owned paid Ready
abandonment, with cold retry and independent semantic restore. This supersedes the
earlier uncompiled-baseline note below. Native36177043351 and local canonical
verification remain pending. The original fixture bytes and hashes are unchanged.

The own-turn unarmed slice preserves original Opportunity attack reconstruction.
At the new live-response execution boundary, apply current armor-training penalties
to fresh unarmed opportunity attacks while retaining old accepted interpretation.
Add actual source/ownership/replay cases; this remains Gate4 source fidelity work.

Compatibility review: the existing journaled UpgradeExecution variant must keep its
historical Legacy-to-ReactionsV1 result. Do not repoint that old accepted command to
a newer default. Introduce a distinct explicit upgrade request for live responses,
with a retained target version, and preserve both earlier continuation allowlists,
including Savage Attacker's two-set submission. Capture a genuine ReactionsV1 save
before changing semantics; a fixture regenerated with the new executor cannot prove
that earlier accepted attacks avoid newly introduced reaction pauses.

## Source and integration audit before implementation

Read-only review of fb83db7 confirms Magic Missile currently has a definition but no
table-admitted caster grant. Pinned SRD p311 Night Hag Spellcasting grants at-will
level4 Magic Missile with a material-only waiver. A source-derived SelectedFeatures
profile is the narrow existing CreatureFeature/SourceFeature path for actual
before-first-dart acceptance; retain printed statistics and explicitly enumerate
omitted attacks, traits and spells. Do not invent a Mage spell list. Night Hag's
Magic Resistance also matters if a scenario asks it to make a magical save: either
implement that source trait or do not claim such a scenario is source-complete.
The alternative Wand of Magic Missiles (p251) needs a new item grant/charge path,
dawn recharge and last-charge destruction, so it is not merely a definition entry.

Shield must interrupt after the original raw hit but before damage, retain the
authenticated pre-reaction facts and evaluate the resulting hit against current
source defenses. Preserve natural20, prior attack/ammo payment and the original
raw acceptance's causality. Nested reaction casting must use the same resolution;
ordinary casting::begin replaces it and cannot safely serve as that entry point.
Mage Protective Magic's shared three-use feature and Reaction are paid only after
current source/component/control revalidation. Magic Missile opens its targeted
window before any dart, once per targeted actor rather than once per dart.

ReactionsV1 can retain a paid physical Ready declaration even with no resolution or
raw request. The new upgrade must reject that state until legitimate abandonment/
expiry, or explicitly authenticate a documented adoption policy. Checking only an
empty resolution would silently make an older declaration executable. Audit every
exact-version predicate in Ready, work ancestry, live admission and table projection.

Required first vertical cases: real Mage hit/Shield/miss; natural20 and later attacks;
owner-Start expiry, shared uses/exhaustion and blocked V/S; opportunity-parent resume;
source Night Hag targeting; private uniform ordering and hidden-offer invariance;
cold reopen at each decision, exact retry and forged trigger/fact/cause rejection.
Counterspell and all Ready mechanisms remain required here after that first path.

The current production builder creates Human Fighter1/Soldier/Skilled PCs and only
autonomous source NPCs. It cannot yet create a player Shield respondent. Rules-level
Player creature control alone is insufficient: table metadata, encounter admission,
owned projections, tactical raw-roll visibility and desktop controls still assume a
PC CharacterId. Add genuine journaled host assignment of source-creature control,
initially before encounter setup, preserving source context and disallowing transfer
through pending work. Keep authenticated attending-player identity separate from the
owned responding actor; derive that actor from validated current ownership and an
audience-bound capability, never a client assertion or fabricated PC row. Extend
source-aware tactical projections/roll controls without broadening legacy PC kernel
queries or changing historical audience digests. Bind control provenance during
semantic replay and handle every new projection at an explicit compatible boundary.

The required player path is normal player/PC creation, genuine Mage creation, host
control assignment, session/encounter admission, another actor's hit, player-only
Shield offer and player acceptance, followed by cold exact retry. Wrong player,
host substitution, forged actor, absent attendance and reassignment during the pause
must fail. This remains Gate4 ownership work; injected profiles cannot prove it.

## Foundation merge and present verification boundary

PR38 exact100c7dabe07b07b7430bcb721b1dd7f48e4792cf passed all six checks:
Linux36172523960 (688 Rust tests), Windows36172524043 (690 Rust tests,67 UI,
zero static errors/warnings and native installer10881663666). Local canonical
source fb83db7 passed687 GNU Rust tests, full lint/check/guards and all43 table-loop
cases. Independent exact-head source/privacy/replay review is clear. The protected
squash main d5d1db7 and reviewed100 both have tree801db9204a1418303ffada37d46cabf073fef159.
Post-main Linux36176391117 and Windows36176391189 passed all six checks with688/690
Rust tests,67 UI tests and native installer10883109174. The bounded foundation plan
is archived. PR39 and PR40 post-main checks are all green. Gate4 remains unaccepted.

The genuine flow2 attack/event13, knockout/event22 and paid-Ready/event13 exports
have independently passed unchanged-source file restore, cold reopen, all ten exact
original transport retries, complete export equality and all29 unchanged SQLite
table counts. Root prepared a reviewed, uncompiled permanent regression module
outside the repository before this branch existed. Copy those exact exports and
their provenance now, preserve original bytes, and commit their baseline tests before any
new executor behavior. Do not claim the new test module has run until it does.

A separate disposable harness has now captured the original accepted
UpgradeExecution1-to2 through unchanged100 code after genuine legacy Savage completion.
Its first default-stack locked/offline run passed without source or harness fixes:
event16,16 audits,11 presentation records,2 bindings,1 unchanged original snapshot.
Both cold retries recovered exact responses without changing full normalized exports
or any of29 SQLite table counts; independent file restore/export equality passed.
The104740-byte export's SHA256 is
0aac1462c6e0d244127a5332e08e5ecb24a88e04c1425bc38a1d09c75a12b69d. It is copied as
reactions-v1-upgrade-100c7da.json with original bytes preserved and documented
provenance. Its new permanent regression is drafted but has not run yet; earlier
three-case Linux evidence does not validate this additional test.

## Execution-boundary decision before implementation

Reserve an explicit next semantic version for live responses; never change flow1
or flow2 replay. The original unit UpgradeExecution permanently means1-to2.
A distinct UpgradeExecutionTo with an explicit retained target authorizes a supported
forward transition only for a host/system at Active with no pending raw request,
resolution, or paid Ready declaration. Old accepted retries precede live admission.
The new upgrade changes only the executor plus normal accepted event bookkeeping;
it cannot refund budgets, erase effects, advance timing or reinterpret old commands.
Test its valid-target rejection while Ready exists and success after the owner's
legitimate abandonment; the old1-to2 command's rejection of every flow2 state is
not evidence for this safeguard. Keep both old continuation sets, including Savage
raw sets and owned physical Ready abandonment. Work ancestry remains mandatory for
flow2 and all supported newer versions. Further mandatory pause families require
their own deliberate retained version boundary when they would alter older history.

Shield's hit stage must retain source-derived rolled facts and authenticated original
roll/work identity before admitting a response. Keep the original AC/modifier/mode
and source damage; derive the later defense from real installed effects plus retained
cover, never from response numbers. Natural20 remains a hit. Phase-specific validators
must cover physical, intrinsic and spell attacks without broadly weakening source
reconstruction. Semantic replay must reject changed fact images, effect identity,
parent keys and causes. Damage dice use the original attack roll's accepted_by even
when a defender/host command advances the queue. A bounded typed suspended parent
record and resume work in the existing frame stack must preserve physical attack,
movement/casting attachment and prior payments through nested responses.

## Selected-response command ownership

The first implementation will separate a private nonpaying response intent from
the selected respondent's actual execution command. The current-turn controller
records the explicit total order. Once collection and ordering select a respondent,
that actor receives the current source action/target controls and may execute or
decline. Shield labels must distinguish offering a response from casting it. Ready
already requires this selected stage to choose its actual attack target or path.

This is an engineering decision within ADR028's consent/timing policy. The existing
source feature hook and spell planner require a command at the current event head
from the actual controller. An earlier private intent can precede another player's
ordering or nested response. Reusing it as the execution command would be stale;
changing its sequence, fabricating System authority, or attributing its cost to the
other player's advancement command would destroy provenance. A fresh owned command
avoids that mismatch without weakening source authorization. Preserve both accepted
commands and the exact trigger/selection relationship in replay.

Selection never prepays permission. Revalidate current grant, components, range,
perception and all budgets before accepting the execution command. A stale final
attempt changes nothing. A response invalidated by a completed child is skipped
without spending, retaining the actual causal command and keeping private reasons
out of unrelated projections. Every child trigger still needs its own ordering.

Before live execution lands, PR42 remains a bounded compatibility-corpus change.
Implement the actual response vertical on a fresh branch after its verified merge;
do not silently expand this compatibility PR into the whole reaction runtime.

## Parallel prerequisites and exact next action

PR41 merged the source Night Hag grant and honest menu wording as
d82d7b2c28be3b74e6e2b0b8c85ffe8c042b1278. Exact51210d1 passed canonical690 GNU
Rust tests,691 Linux/693 native Rust and67 UI tests, all six checks, strict lint,
guards and native packaging. Fetched main/source tree equality is verified at
48aae57d58e13f3f1fdf1fd5904015999184c185. Post-main36179980349/36179980413 is pending.
The reviewed source also has20 focused source/profile tests, the real six-dart file
case and genuine old-history replay passing. PR42 integrated this main without
production changes of its own.
The source-control branch has its own writer/plan for real Mage ownership, actor
selection, self-casting and raw dice with explicit table/transport/presentation
activation. It does not change tactical execution versions. Integrate these only
after coherent reviewed checkpoints and preserve both histories deliberately.

Source review found the inherited Magic Missile driver serializes darts and their
nested consequences, while SRD146 says they strike simultaneously. The current
Night Hag test uses a surviving target without concentration; it does not prove
simultaneous consequence behavior. Before claiming full Magic Missile/Shield
acceptance, establish the simultaneous strike/target milestone, correct current-turn
ordering and source amount/consequence grouping, with death, concentration, defense,
repeated/multiple targets and off-turn Ready interactions. This stays in Gate4.

Exact next action: verify the four-case corpus on this integrated main baseline,
including full canonical verification and all six final-head CI checks. Root owns
this branch and the heavy local slot; only one heavy verification process may run.
Independent source-control stack/recovery fixes and aftermath authoring may continue
without competing builds. Merge compatibility PR42 only after verified review, then
create a fresh branch for actual versioned responses. No gate pause or scope waiver.
