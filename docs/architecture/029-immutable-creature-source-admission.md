# ADR 029: Immutable creature source admission

Status: accepted for the bounded Gate4 Air Elemental prerequisite, 2026-09-28.

## Context

Creature profiles fingerprint the whole serialized source definition. Spell pins
fingerprint a spell/creature/feature tuple. Editing the current definition behind
an old ID invalidates history even when the edited trait was never used. The
Counterspell preflight accepted full-pin immutable lookup and explicit admission;
this first additive implementation introduces no corrected Hag or Magic Resistance.

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

## Consequences and remaining obligations

This is a fixed, small registry, not a general content framework or upgrade API.
Future revisions must retain all immutable entries, explicitly choose admission,
and audit every tuple/resolver boundary. Gate4 still requires Air Form/Whirlwind,
Shove and its genuine immune no-effect continuation, Counterspell and genuine
Magic Resistance evidence. Authored tests and source data alone do not satisfy
production-path acceptance. See the active Air Elemental plan for exact evidence.
