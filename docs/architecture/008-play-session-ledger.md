# ADR 008 — Play-session ledger and world-state boundary

Status: **Accepted for Gate 0 foundation**

## Context

A tabletop play session is durable campaign history, but it is not itself a world fact. Attendance, the wall-clock game night, transcript provenance, and the fact that a command was issued during a particular session belong to execution/audit history.

`CampaignState` is the materialized world snapshot. If every historical play session is embedded in it, snapshots grow forever and world-state validation becomes coupled to operational history.

## Decision

Play sessions are persistent domain records stored in the campaign SQLite database **beside** the materialized world snapshot, not inside `CampaignState`.

`PlaySessionId` is stable and may be referenced by commands and events:

```text
PlaySession ledger
      │
      ├── GameCommand.session_id
      └── EventMeta.session_id

CampaignState
      └── world/materialized state only
```

A `None` session ID is valid for between-session simulation, imports, migrations, and maintenance/admin work that did not occur during a tabletop session.

## Attendance and character lifecycle

Attendance is session-scoped:

- `Present`
- `Absent`

It is not a permanent `CharacterStatus`.

Character lifecycle remains about the character:

- active
- retired
- dead

A retired character may remain alive in the world. A player may miss one session without changing the character's lifecycle.

## Session participant invariants

A play session may contain any number of participants. Shape validation rejects:

- duplicate player assignment;
- duplicate character assignment;
- an active session with an end time;
- a closed session without an end time;
- an end time earlier than the start time.

Reference validation checks the session's campaign, players, and optional character IDs against the loaded `CampaignState` before session start/update.

A participant may have no character during character creation, after a death, or when attending without controlling a PC.

## Persistence invariants

SQLite is authoritative for session-ledger persistence.

The initial schema uses normalized `play_sessions` and `play_session_participants` tables. Database constraints enforce:

- one active play session per campaign;
- one participant row per player within a session;
- one assignment of a character within a session;
- participant rows cannot outlive their session record.

The participant ordinal is persisted so session records round-trip without silently reordering the table.

## World time vs wall-clock time

`started_at_world` and `ended_at_world` describe in-world campaign time.

Real-world timestamps, microphone timing, latency, pauses, interruptions, and other play-quality telemetry belong to the telemetry/session-observation layer and must not be confused with world time.

## Consequences

- World snapshots stay bounded by current materialized world state rather than accumulating game-night history.
- A command/event can still answer "which tabletop session caused this?"
- Between-session simulation remains first-class instead of being forced into a fake session.
- Attendance and replacement-character workflows no longer contaminate character lifecycle state.
- Session persistence can evolve independently from world snapshot schema migrations.

## Non-goals for Gate 0

This ADR does not yet define transcript storage, audio retention, speaker-attribution telemetry, recap generation, break tracking, or the session director's pacing metrics. Those systems may reference `PlaySessionId` but must not become authoritative world state.
