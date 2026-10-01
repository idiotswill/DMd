# ADR 029: Immutable creature source admission

Status: accepted for the bounded Gate4 Air Elemental prerequisite, 2026-09-28.
Extended by the approved bounded Night Hag/Magic Resistance design, 2026-09-30;
that implementation remains an unverified development checkpoint.

## Context

Creature profiles fingerprint the whole serialized source definition. Spell pins
fingerprint a spell/creature/feature tuple. Editing the current definition behind
an old ID invalidates history even when the edited trait was never used. The
Counterspell preflight accepted full-pin immutable lookup and explicit admission;
the first additive implementation introduced Air. The next bounded revision adds
Night Hag's printed Magic Resistance without changing its historical definition.

## Decision

Freeze the entire V1 tactical catalog and the separate V1 table picker. Keep the
old ID-only helper tied to V1, including profileless historical fallback. Store the
complete Air Elemental as a separate immutable entry. Profiles resolve by ruleset
ID/version, definition ID and definition fingerprint. Spell reconstruction selects
the exact original tuple, and actor bindings require that same creature revision.
Never derive an old definition by subtracting fields from a new one.

Current source options carry exact pins. New live creation requires a current
admission; unchanged V1 entries remain admitted. An omitted creation pin remains
V1 forever and is allowed during historical replay only. Normalization never fills
it. Accepted old retries are looked up before new-command admission. No profile,
request, receipt, presentation or export is migrated. This adds no blanket tactical
execution/version change: historical producers retain their accepted semantics.

The current binary requires declared, checksum-verified, byte-exact installed Air
content, as it already does for creation and tactical content. Historical campaign
and export replay uses the current verified installed bundle with unchanged V1
definitions. An older installation without the new file is unsupported/incomplete
until normal package update, with a recoverable content error and no database
mutation. This is package compatibility, not a save/profile migration.

Source completeness does not imply executor completeness. Air Form and Whirlwind
are typed faithfully. Unsupported shared-space/narrow-passage operations return a
distinct generic spatial error; unsupported Whirlwind/Multiattack fail before
payment. Current Host options disclose limits; frozen presentation stays intact.
No source is selectable with silent ordinary-body fallback for those exceptions.

### Same-ID correction and private saving causes

Store `night-hag-v2.json` as a complete immutable definition of the selected
adaptation, with the same logical ID and a distinct full fingerprint. Add only
Magic Resistance and remove only its omitted-feature entry. Other printed clauses
remain explicitly omitted. Current admission contains the corrected Hag alone;
both revisions remain resolvable and old profiles remain old. Do not use ID-only
lookup for a retained actor or a spell/creature/feature tuple. Require the declared,
length/checksum-verified and byte-exact installed revision independently of any
compiled definition cache, using the same recoverable package-error boundary as Air.

Magic Resistance is a typed source trait. Its saving circumstance is computed
privately from the target's full source pin and authoritative cause; there is no
serialized magical flag or new public roll reason. Existing spell SaveCondition
requests derive Spell from the exact validated program and actor-bound source
tuple. A retained repeated save derives Spell only when its selected pending
ticket and complete effect clause match the frozen canonical spell: condition,
trigger, frequency, ability, end behavior, overlap, expiry/group shape and original
deterministic effect/view/group identities. Preserve its retained DC and deadline;
do not require a discarded cast record or today's casting eligibility. Original
semantic application replay proves installation, original source tuple, target
type/selection and expenditure. Local clause matching cannot authenticate a
self-consistent invented history. The existing one-frozen-version-per-spell-ID
assumption must be revisited before any same-ID spell revision is admitted.

The exact frozen Chimera and red-dragon Fire Breath clauses are nonmagical under
SRD185. Concentration is separately nonmagical for this trait even after spell
damage. Unknown/new causes needing the trait fail explicitly; source names, damage
types and reason strings are never substitutes. Actors whose exact source lacks
the trait retain the existing request path, avoiding new category restrictions on
legacy pure mechanisms. The resulting circumstance enters ordinary save condition
composition, including cancellation and automatic outcomes. Counterspell, future
magical item/monster effects and physical save producers require their own source
admission/consumer proof. None is silently classified through a general fallback.

## Consequences and remaining obligations

This is a fixed, small registry, not a general content framework or upgrade API.
Future revisions must retain all immutable entries, explicitly choose admission,
and audit every tuple/resolver boundary. Gate4 still requires Air Form/Whirlwind,
Shove and its genuine immune no-effect continuation, Counterspell and genuine
Magic Resistance evidence. Authored tests and source data alone do not satisfy
production-path acceptance. See the active Air Elemental plan for exact evidence.
The [Magic Resistance prerequisite plan](../exec-plans/active/gate4-magic-resistance-source.md)
records qualified source/tests and remaining proof. Actual Cultist Hold Person
against the Fiend Hag remains paid no-effect with no save. Synthetic positive
category tests cannot replace the later genuine Counterspell and repeated-save
Magic Resistance gameplay obligations.
