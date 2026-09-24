# Mandatory human vertical-slice playtest

Status: **Required after Gate 6 and before Gate 7**

## Purpose

Automated tests can prove correctness properties. They cannot by themselves prove that DMd is playable, understandable, enjoyable or feels like tabletop D&D.

The first complete tabletop vertical slice is a product checkpoint, not an implementation demo.

## Entry conditions

Gates 2–5 accepted; Gate 6 engineering scope verified and merged. This playtest supplies the remaining human acceptance for Gate 6 before Gate 7 may begin.

A small legally distributable starter adventure exists in the production content/world system.

The desktop application can run the production rules/runtime path.

## Required playtest

The acceptance test requires four human players. Fewer may be used for early rehearsals but do not replace the intended table test.

The production build must support:

- campaign setup;
- Session Zero;
- character creation/selection;
- session start;
- social scene;
- exploration;
- investigation/evidence;
- tactical combat;
- physical dice;
- inventory/loot;
- shopping or resource acquisition;
- rest/resource recovery;
- save/exit;
- restart/resume;
- session close;
- second-session resume.

At least one player must attempt something the adventure author did not specifically anticipate.

No developer may repair state, edit SQLite/JSON/source/prompts or manually script the next outcome during play.

## Observe specifically

Record:

- where players needed developer knowledge;
- where UI obstructed ordinary play;
- unnecessary clarification questions;
- impossible/unsupported improvised actions;
- confusing rules feedback;
- moments where the game required formal command syntax;
- lost state after restart;
- places where the DM/system leaked hidden information;
- excessive bookkeeping;
- places where table conversation felt constrained by the software;
- places where players stopped treating the system as a DM and began treating it as an engineering tool;
- fun/emergent moments worth preserving.

## Pass criteria

The playtest does not need polished visuals or voice yet.

It passes only if a real group can complete the slice through the production application without developer state surgery and the discovered blocking defects are either fixed or explicitly resolved before Gate 7.

A list of severe "we'll fix this after autonomy/voice" usability blockers is not a pass.

## Output

Check in a concise playtest report containing:

- build/head;
- participants and setup (non-sensitive);
- scenarios exercised;
- defects;
- unexpected player behaviors;
- fixes made;
- remaining debt;
- human acceptance decision.

Do not embed copyrighted actual-play transcripts from third parties.
