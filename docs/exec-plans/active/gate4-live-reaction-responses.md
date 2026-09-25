# Gate4 — Live reaction responses and Ready release

Status: required active Gate4 work; runtime windows not implemented.
Writer: root. Next branch: codex/gate4-live-reaction-responses, to be created from
fresh main after PR38 foundation merge. Umbrella: gate4-reaction-ready-runtime.md.
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
Foundation tests are prerequisites only. After PR38, read current source and ADR028,
write the execution-boundary decision, then implement Shield as the first complete
vertical path while keeping Counterspell and all Ready mechanisms mandatory here.
The full twelve-family Gate4 ledger and eighteen spell mechanisms remain binding.

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
