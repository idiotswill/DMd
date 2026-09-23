CREATE TABLE play_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'closed')),
    started_at_world INTEGER NOT NULL,
    ended_at_world INTEGER NULL
);

CREATE UNIQUE INDEX ux_play_sessions_one_active_per_campaign
ON play_sessions(campaign_id)
WHERE status = 'active';

CREATE TABLE play_session_participants (
    session_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL,
    player_id TEXT NOT NULL,
    character_id TEXT NULL,
    attendance TEXT NOT NULL CHECK (attendance IN ('present', 'absent')),
    PRIMARY KEY (session_id, player_id),
    FOREIGN KEY (session_id) REFERENCES play_sessions(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ux_play_session_character_assignment
ON play_session_participants(session_id, character_id)
WHERE character_id IS NOT NULL;
