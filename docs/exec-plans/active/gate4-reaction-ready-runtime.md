# Gate 4 — Actual reaction and Ready execution

Status: active Gate4 umbrella; live Shield/Counterspell/Ready responses remain incomplete.
Root owns compatibility PR42 on `codex/gate4-live-reaction-responses`.
Completed PR38 evidence is in `../completed/gate4-reaction-foundation.md`;
current bounded status and required successor acceptance are in
`gate4-live-reaction-responses.md`.

Verified sourcefb83db7 passes687 GNU Rust tests,43 actual table-loop cases, strict lint
and both guards, plus67 UI tests/static/build. First aid PR39 and unarmed PR40 are
merged; source/main reconciliations preserve their reviewed trees. PR38 merged as
d5d1db7 with all six source and post-main checks passing. PR41 Night Hag merged as
d82d7b2 after all six source checks; its post-main checks are still pending. Genuine
ReactionsV1 attack, knockout, paid-Ready and original1-to2 upgrade saves are retained
in PR42 before any new response semantics.
Source casting, player-owned creature control, privacy, Counterspell and all Ready
mechanisms remain mandatory Gate4 work. No family or gate closure is claimed.

## Earlier planning and implementation record

The following preserves earlier decisions and test history. Its old running-job,
branch-ownership and pending-verification notes are superseded by the current bounded
foundation plan above; they are not instructions to resume abandoned source branches.

Status: **Active; a61e95d foundation passes canonical verification and all six CI checks; actual reaction responses remain incomplete.**
Foundation PR38 branch: `codex/gate4-reaction-ready-runtime`; base `1b39980ec77f909b81004ab7327060156c297945`.
Writer: root, taking over after rules_architecture reached its account usage limit.
The authored/reviewed checkpoint is dd5123d; root owns its first executable batch,
integration, main-based PRs and gate acceptance. No root source edits preceded that handoff.

## Objective and authority

Complete source-grounded Shield/Counterspell reaction windows and Ready attack/move/spell
execution through the real table, with durable interruption, actual source grants,
physical dice and exact restart/replay. Follow the implementation audit in
`gate4-reactions-and-ready.md`, ADR026, Gate4 and the product's combat timing,
player agency, information boundary and authoritative save/resume requirements.

SRD5.2.1: reactions pp.10/186, OA p.15, casting/components pp.105–106, Counterspell
p.120, Shield pp.161–162, Attack/armor training p.177, concentration p.179, Ready
pp.186–187 and simultaneous effects p.187. Source descriptions control specific
exceptions; a string trigger or client-provided permission is never state authority.

## Ownership and dependencies

- Bootstrap released exclusive shared tactical scheduler/domain ownership after its
  reviewed area source. Preserve `areas`, consent/host-order scope and every existing
  source/attack/movement/falling continuation. Merge its coherent verified application
  checkpoint before compiling this composition. Verified area `a45fe57` is merged as
  `f3daf22`; its prior 65-app-test/strict-lint evidence remains inherited only.
- Root's physical/casting extraction does not change the integrated production tree
  except separately reviewed corrections. Coordinate any such changes explicitly.
- The durable audience/retry protocol is merged through PR35. Supporting agents are
  quota-blocked; root is the sole active writer. Reaction views/actions must adopt
  the protocol's actual actor/audience contracts before integrated acceptance.
- Bootstrap's genuine OA/concentration test is reviewed separately. It now owns source
  Mage/Counterspell/Mage Armor definitions/profile/equipment plus a private reaction
  feature hook; dependencies `5f15783` and `01e857c` are integrated here. Those changes
  remain uncompiled. This writer owns effect/program/retained/shared composition.
- Local Rust/frontend execution is globally serialized. Medicine's combined canonical
  job currently owns the shared target; this work may edit/review without compiling.
- Root owns encounter finish separately. It must reject nonempty Ready records and
  unresolved reaction/roll/work until this slice provides explicit abandonment.
  Abandonment retains the actual controller's command, refunds nothing, and ends only
  the held spell's matching concentration group; it never fabricates a next-own-Start
  or releases a spell without its real trigger. Other timed effects/concentration and
  source counters survive finish through root's explicit lifecycle handoff.
- Bootstrap owns new Fighter source leaves without shared queue edits. Counterspell
  reserves raw-role tag15; tactical Second Wind reserves tag16. Savage Attacker stays
  attached to the exact AttackDamage work/request and must reroll only source weapon
  dice (including their critical dice), preserving added-damage dice and raw evidence.

## Reviewable PR boundaries (2026-09-25)

This remains the active umbrella plan for all live reactions and Ready execution.
PR38 already spans88 files and over5,000 added lines before actual response windows.
Keep it as the bounded source/owned-work/legacy-recovery foundation described in
`gate4-reaction-foundation.md`. Its acceptance does not complete this plan or any
Gate4 reaction family. The actual Shield/Counterspell/Ready releases continue in
`gate4-live-reaction-responses.md` on a fresh branch after verified integration.
This is an implementation split inside the approved gate, not a deferred requirement.

Every new pause changes historical interpretation. If the foundation merges first,
actual live windows need a new explicit execution version and a quiescent journaled
upgrade. ReactionsV1 events already accepted under the foundation must continue to
replay without newly invented pauses. Record the precise next version and migration
in ADR028 before implementing the new runtime. Do not silently change old semantics.

## Implementation slices

1. Durable flow-lived Ready declarations and bounded reaction windows in new domain
   and rules modules. Preserve a single resolution frame stack and typed suspended
   parent ownership. Retain original declaration/cast identity independently from
   triggering and reacting commands; keep current-turn controller separate from actor.
   Existing state wire and source replay remain compatible; new authority cannot be
   silently ignored, forged in an earliest anchor or attached without source events.
2. Complete Shield hit/target windows and Counterspell casting/CON-save windows,
   including nested cancellation, reaction/slot reservations, effective AC/protection,
   concentration replacement and all existing save/LR/Inspiration behavior. Add exact
   Counterspell content/source pinning and a genuine table-creatable NPC spell grant;
   no synthetic PC class, level or slot grant. Source Mage is a candidate, with one
   shared Protective Magic 3/day pool and explicit included-Mage-Armor interpretation.
3. Ready declaration/ignore/expiry and actual attack, movement and held-spell release.
   Reuse physical weapon/geometry/OA/fall/source spell reducers. A readied Attack's
   action identity differs from its Reaction payment; readied movement has a bounded
   response allowance separate from another actor's turn budget. Bind release targets
   using current perception/geometry without recasting or charging components again.
4. Real table controls and actor-private offers under the verified audience protocol,
   complete origin collection/semantic restore and genuine file-SQLite scenarios.
   Reopen at offers, nested saves/attacks, held spell and release; replay identical
   accepted commands; verify uncertain retry, stale/foreign refusal and hostile export.
5. Independent complete exact-head source/queue/app review and focused tests/lints,
   then root's combined/canonical and packaged acceptance. Intermediate source helpers
   or new schema are milestones, not completion of this execution slice or Gate4.

## Acceptance boundaries and risks

Use the concrete source and recovery matrix in the audit plan. Shield does not erase
natural20; Counterspell uses a CON save and its exact slot exception; Ready triggers
after completion, OA before departure. Actual costs, raw faces, source identities,
item custody, partial movement and unrelated parent work survive nested reactions.
No auto-selected PC response, hidden reaction count, fabricated source effect install,
free off-turn equip permission, duplicate damage/slot/reaction or second queue.
Same-trigger responses are collected privately, then competing accepted responses
require current-turn ordering (or explicit occurrence-scoped host delegation). Network
arrival is never fictional priority. Child-created triggers have separate nested
windows, and remaining accepted intents are revalidated after each selected response.
ADR028 records this application interpretation of SRD pp.10/120/186–187.
The chosen privacy policy presents a uniform decision stage at every public trigger
for zero, one or many private competitors; optional exact-trigger delegation is
independent of eligibility. DTO/revision tests must distinguish private collection
from visible results without revealing hidden response counts. Actual Ready response
targets/path are chosen after ordering selects that response, using current geometry.

Current PC catalog expansion stays Gate6. Player-controlled source-creature assignment
is a separate still-required Gate4 interface; host-created source NPCs are the genuine
bounded entry here. Other Ready actions, unimplemented spell families and generic
natural circumstances remain explicit active work until complete typed adapters or
retained host adjudications exist. Unsupported requests fail before costs.

## Earlier implementation history

Inherited training correction:43 focused spell tests and strict domain/rules all-target
Clippy passed at `1b39980`, with independent source/fixture signoff. Those checks do
not verify this new execution work. Rustfmt and whitespace checks pass on the working
tree; no compiler/test/lint execution has occurred on this branch.

Authored checkpoint:

- Explicit `ReactionsV1` live Begin and quiescent privileged upgrade, while omitted
  historical execution remains Legacy and internal table/tactical replay preserves
  original semantics. New requests cannot choose Legacy. Protocol emission must be
  explicit; historical accepted envelope interpretation must never use today's default.
- Durable paid physical Ready declarations and own-Start expiry. Actual witnessed
  response/ignore/attack/move/held-spell release are not implemented at this checkpoint.
- Current-execution causal work ancestry confines area ordering delegation to its own
  authenticated descendants; independent actions cannot inherit permission merely
  because an area record exists elsewhere. Legacy work images remain omitted/unchanged.
- Source-derived defense clauses live in the existing tactical effect lifecycle.
  Effective AC queries/attacks include them without mutating equipment. Same-spell
  precedence remains the existing overlap rule. Mage Armor uses actual self casting,
  real source material, 13+Dex and eight-hour expiry; honest source Mage starts at 12.
  Other willing targets/competing armor formulas still require explicit admission
  choices, and the new armor-don observation is an internal lifecycle hook, not a claim
  that a public don-armor action already exists. Shield's defense leaves exist, but
  public ordinary CastSpell still rejects reaction activation without a live window.
- Added declaration/controller/forgery/legacy upgrade, work-ancestry corruption,
  defense timing/overlap/armor-ending/wire and genuine source Mage casting regressions.
  All remain unrun. A temporary, local dead-code annotation on the private source
  reaction hook documents the next real caller; remove it when window admission lands.
- Independent defense review found and corrected a held-spell edge: nonconcentration
  Mage Armor must not retain Ready's already-ended hold-only group. The retained
  self-only formula boundary is also rechecked against foreign-target rewrites.
  A genuine source plan Held→Released leaf regression now covers both; it is not
  presented as public Ready response or table acceptance.
- Review also found a legacy non-area liquid-choice projection trying to read current
  ancestry. It now retains actor ordering without a trace; a replayed legacy movement
  pause regression covers projection and actual continued choice. This fix is unrun.

Root's first one-job domain/rules all-target check passed on dd5123d (1m39s).
The domain/rules test batch passed 206 tests, then failed one old rules-unit
expectation: Shield was still listed among unsupported programs even though this
checkpoint adds its defense program. Later integration suites and strict Clippy
did not run after that failure. Logs are outside the worktree under
tooling/logs/gate4-ready-foundation-{check,tests}.log.

The correction separates program support from activation: a new leaf assertion
requires Shield's Defense program and Reaction cost, while the public source Mage
scenario rejects ordinary Shield casting before spending its Action on Mage Armor.
That rejection now proves the trigger boundary with an otherwise unspent actor,
rather than after an Action was already spent. The subsequent actual Mage Armor
scenario and every other unsupported-program case remain. Only comments/tests
changed; the correction has not yet been rerun. Format and diff checks pass.

The corrected25f99ab checkpoint passed all483 domain/rules tests and strict
domain/rules all-target Clippy with warnings denied (1m34s), serialized at one job.
Logs: tooling/logs/gate4-ready-foundation-tests-r2.log and
tooling/logs/gate4-ready-foundation-clippy-r2.log. The earlier all-target check passed
on dd5123d; the intervening correction changed only tests/comments. These are
foundation checks, not full application, CI or completed reaction/Ready evidence.

Next: add complete nested reaction windows and actual Ready response integration,
then verify the current main/protocol integration and explicit live Begin emission. Domain types,
declaration-only behavior and internal leaves do not satisfy the slice or Gate4.

Root resolved the open private-ordering design seam in ADR028: every uniform public
trigger asks for an explicit total-order instruction over potential respondents.
The current-turn controller can rank known participants and choose forward/reverse
established initiative for the unlisted set, placed before or after the named list,
or explicitly delegate this occurrence to the host. Controls do not depend on actual
private eligibility/acceptance. No automatic fallback or arrival priority is allowed.
Apply the selected instruction only after private collection, revalidating each
intent and giving each nested trigger its own decision. This is a documented design
decision; no new runtime or privacy test result is claimed yet.

Integrationf0b9fe8 imports the exact fetched Fighterf669389, including area main172a15a
and protocol source. Resolve conflicts by retaining both typed actions and the modern
opaque transport test expectations. Second Wind now initializes the current work
ancestry, and the new OA/concentration fixture explicitly starts ReactionsV1.
No global compiler ran while Second Wind's canonical job owned the shared target.

The subsequent integration review found that declared Second Wind replay bypassed
the historical tactical policy. Both direct and declared tactical table paths now
require the exact retained child metadata/action before historical replay; new
requests still use current live admission. The desktop Begin form explicitly emits
ReactionsV1 and its real component regression checks that field. No old accepted
transport envelope is rewritten. These integration changes still need application
regressions, including a genuine legacy declared-Second-Wind export, before acceptance.

Draft PR38 now runs full CI against the integrated foundation. The genuine f669389
event22 pause is retained as an89KB portable fixture: original direct/declared uses,
one round and a pending raw d10. Its authored regression requires byte-preserving
restore, original retry, completion under legacy semantics, refusal of a fresh legacy
action, explicit settled upgrade and independent restored continuation.

The desktop now offers the host an explicit "Continue saved encounter" action once
old rolls/decisions are settled. Ordinary new actions wait; old continuation controls
remain available. The optional execution projection field is omitted for legacy
states, preserving their prior serialized DTO/digest, and populated only by an
explicit ReactionsV1 flow. The fixture and component cases cover omission, upgrade,
host ownership and pending choices. Modern UI fixtures explicitly carry the new
execution version. The Second Wind forgery case changes both pending work and its
new redundant causal trace so it continues to test semantic journal rejection after
structural validation, rather than merely detecting mismatched copies. These latest
changes are authored/format-checked and await fresh executable evidence.

Initial integrated6c37630 CI36149846850 passed fast verification, strict Clippy,
MSRV and both guards. Its actual Linux log confirms the genuine legacy fixture
regression passed (21.68s), together with the ten app-unit,22 rules-runtime and
nine runnable-campaign tests. The later full suite was cancelled by the next push;
there is no complete integrated-suite claim. This confirms the original fixture
and historical replay correction before the added projection/UI assertions.

The next authored leaf encodes the ADR028 ordering instruction without a default.
It validates ranked identities against existing known initiative participants,
then filters an explicit forward/reverse before/after order to the privately accepted
set. Tests vary arrival order, hidden-set size and controller instruction, and reject
unknown ranks, duplicate respondents and omitted choices. This pure function is
not yet attached to an authenticated runtime window and grants no action authority;
actual windows, response payment and DTO/revision privacy tests remain required.

CI36150438429 on00ddb34 rejected the strengthened forgery helper at compilation:
Rust evaluates the assignment value before its target, so moving the cloned work
also moved the occurrence captured by the target-search closure (E0382). Retain
the copyable occurrence before that assignment; keep both forged work images and
all semantic rejection assertions. Re-run full CI on the corrected combined head.

## Current verified checkpoint

Correcteda61e95d passes canonical verification locally:664 GNU Rust tests, strict
all-target Clippy/check, formatting and both guards. Desktop checking reports zero
errors/warnings,57 tests pass and the134-module production build passes. All six
exact-head checks pass (Linux36150846605, Windows36150846528); actual native logs
confirm667 Rust tests,57 UI tests and a fresh offline installer. These verify the
foundation, legacy replay/upgrade and pure ordering function, not live reactions.

Savage Attacker sourced901058 is merged into this branch as efcabf4. Fetched main
cd8d4a4 has a byte-identical full tree to d901058; only after importing that source
and proving ancestry did the branch reconcile the main squash. Adjacent typed-action
conflicts retain both source variants. This new composition is not yet verified.

Next: connect the retained trigger/window and explicit ordering instruction to the
single scheduler, source response admission and audience-safe table choices. Shield
hit windows are the first concrete vertical slice; Counterspell and physical/spell
Ready releases remain mandatory acceptance here. Import Medicine after its verified
merge and initialize its current-execution work ancestry. No Gate4 criterion moves.

The next bounded implementation also supplies explicit abandonment of an owned
physical Ready declaration. This is required before encounter finish can safely
require quiescence. It is allowed only between resolutions, refunds no Action or
Reaction, and does not advance a turn or end unrelated concentration. A host cannot
make this meaningful choice for a player-controlled actor. The private Ready view
omits empty lists to preserve legacy projection bytes; unrelated viewers must keep
the same DTO/revision when only an owned declaration is abandoned. Held spells are
still inadmissible at this checkpoint; their matching concentration cleanup remains
part of the held-casting implementation. Verify actual table retry/reopen and UI.

AbandonReady now has controller-only admission, off-turn idle handling and a private
desktop control. Authored tests cover unchanged timing/effects, multiple declarations,
host/foreign rejection and pending dice. A genuine file-SQLite case starts from normal
creation/initiative, reopens the paid declaration, compares an independently restored
cancellation, retries the accepted request after another reopen and rejects a coherent
invented replacement declaration. It requires the unrelated audience's entire modern
view, revision and transcript to remain identical. These changes are format-checked
but have not run; Medicine still owns the serialized local compiler slot.

CI on c39365e failed one new pending-dice reducer fixture on both Linux and native
Windows: setting HP to zero without Prone violates the existing vitality invariant.
The fixture now includes the required Prone state; production validation and all
assertions remain unchanged. All six local Ready reducer tests pass after correction.
The prior native log also confirms the actual Ready SQLite case and all40 table-loop
tests passed, plus61 UI tests/static/build. Full corrected-head CI is still required.

## Current composition and exact next action (2026-09-25)

PR38 is now bounded by gate4-reaction-foundation.md; this umbrella remains active
through actual response execution. Normal merge643cdf3 imports Medicine a0b4b57.
Its four adjacent code/UI conflicts retain both features, initialize Medicine's
current work_trace and explicitly version its new Begin/UI fixtures. Source Medicine
has prior canonical654 GNU/655 Linux/657 native evidence and its strengthened four
cases pass; combined execution is new and unverified. Run canonical/UI and exact-head
CI on the composition, reconcile PR39's actual squash only after tree/ancestry proof,
then finish the foundation review/merge and start the fresh live-response branch.
Do not treat cancelled pre-integration CI as a full pass or mark this umbrella done.
