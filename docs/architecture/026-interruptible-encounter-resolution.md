# ADR 026 — Interruptible, replayable encounter resolution

Status: **Proposed — Gate 4 implementation pending.**

## Problem

The first table supports one player's check and one follow-up boundary. Complete encounters
need movement interrupted by another actor's reaction, attacks followed by damage and a
different actor's concentration save, shared area damage, ordered triggers and choices that
can outlive an application process. Keeping that sequence in UI callbacks would lose truth
on restart and could spend resources twice.

## Decision

The pure rules layer owns typed encounter actions and a bounded serializable continuation.
An accepted command validates its issuer and entire immediate operation, works on a clone,
then returns state plus a semantic event for the existing atomic application commit. Each
pause exposes exactly the next required raw roll or material decision to its controller.
The remaining plan and its originating cause remain durable. Invalid input cannot advance
the plan, spend resources, move a token or partially apply an effect.

RulesState remains the single HP/condition/resource/dice/timing authority. Encounter state
adds spatial truth, source-defined combat capabilities, movement/feature accounting and
continuations that existing primitives do not represent. New budgets must not duplicate
an existing budget. The actor can spend at most one spell slot per turn; off-turn casters
must be accounted independently, without treating the active actor's old slot flag as a
global restriction on every creature.

Dash persists the selected Speed/special-speed and originating command (SRD p.180). The
default cross-mode interpretation increases the selected speed's movement allowance only;
all modes still subtract shared movement already spent (p.188). Thus Speed 30/Fly 60 with
Dash using Speed has allowances 60/60; Dash using Fly has allowances 30/120. Current speed
modifiers change the selected grant accordingly. The source requires a speed choice but
does not explicitly settle transferring its extra movement to a different mode; the rule
above preserves that choice and treats extra movement separately from a change to Speed.
This boundary interpretation must be visible in rules explanations. Any alternative needs
an explicit persisted typed campaign adjudication; prose alone cannot change accounting.

Opportunity attacks interrupt immediately before leaving visible reach. The declared
remaining path resumes only after the reaction and its follow-ups resolve, and legality is
rechecked against resulting conditions/positions. Forced motion and teleport are distinct
from voluntary movement. Ready reacts after its perceptible trigger completes; ignoring a
trigger does not silently spend the reaction. Readied magic pays casting resources at the
time of casting and is held by concentration until release or expiry.

Simultaneous effects present ordering to the controller whose turn it is (SRD p.187).
Player-only initiative ties require player decisions; mixed and monster ties use the host
(p.13). A save can be voluntarily failed by its legitimate controller (p.187). No scheduler
heuristic may make a PC's material choice solely to keep processing.

An ongoing spell is one source instance with dependent targets/effects. Adding another
target to the same instance must not replace concentration. Shared damage is rolled once
for simultaneous saving-throw targets (p.16). Trigger identity plus turn/entry occurrence
prevents duplicate application on restart or retries. Concentration loss removes all
dependent effects without deleting historical receipts or unrelated conditions.

## Historical semantics and information boundaries

Keep legacy event versions deterministic. New tactical interpretation must not change how
an old unresolved text declaration replays. New event kinds/versions dispatch explicitly;
unknown versions fail closed. NPC policy is outside authoritative replay: the accepted
typed proposal and raw random faces are recorded and revalidated, not regenerated.

Accepted events retain trusted original issuer/actor/cause. An internal derived check may
use a rule-selected DC or modifier; it must not manufacture an Admin command from player
text. Public APIs cannot supply a spatial permission to bypass current encounter truth.

Store audience-aware table presentation at resolution time, since reconstructing history
from current visibility can either leak a past secret or erase an already witnessed event.
Views, map markers, roll reasons, errors, transcript and recap all use the same explicit
knowledge boundaries. The host may inspect authority; player channels never receive it.

## Verification obligations

Test every interrupt stage through serialization and later through SQLite restart and
semantic export/restore. Include reaction chains, automatic and physical saves, shared
damage, concurrent/foreign command rejection, changed movement legality, lost concentration,
simultaneous ordering and hidden-view serialization. Source/architecture tests alone do not
complete Gate 4; the packaged multi-round desktop scenario remains required.
