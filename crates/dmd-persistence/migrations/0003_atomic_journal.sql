CREATE UNIQUE INDEX ux_play_sessions_campaign_and_id
ON play_sessions(campaign_id, id);

CREATE TABLE campaign_state_current (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    applied_event_sequence INTEGER NOT NULL CHECK (applied_event_sequence >= 0),
    state_json TEXT NOT NULL
);

CREATE TABLE command_audit (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL,
    session_id TEXT NULL,
    issuer_kind TEXT NOT NULL CHECK (issuer_kind IN ('player', 'system', 'admin', 'import')),
    issuer_player_id TEXT NULL,
    actor_kind TEXT NULL CHECK (actor_kind IS NULL OR actor_kind IN ('entity', 'faction')),
    actor_id TEXT NULL,
    expected_event_sequence INTEGER NOT NULL CHECK (expected_event_sequence >= 0),
    command_kind TEXT NOT NULL CHECK (length(command_kind) > 0),
    command_schema_version INTEGER NOT NULL CHECK (command_schema_version > 0),
    payload_json TEXT NOT NULL,
    accepted INTEGER NOT NULL CHECK (accepted IN (0, 1)),
    resolution_explanation TEXT NOT NULL,
    resulting_event_sequence INTEGER NOT NULL CHECK (resulting_event_sequence >= 0),
    CHECK (
        (issuer_kind = 'player' AND issuer_player_id IS NOT NULL) OR
        (issuer_kind <> 'player' AND issuer_player_id IS NULL)
    ),
    CHECK (
        (actor_kind IS NULL AND actor_id IS NULL) OR
        (actor_kind IS NOT NULL AND actor_id IS NOT NULL)
    ),
    UNIQUE (campaign_id, id),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, session_id) REFERENCES play_sessions(campaign_id, id) ON DELETE RESTRICT
);

CREATE TABLE event_journal (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    session_id TEXT NULL,
    occurred_at_world INTEGER NOT NULL,
    source TEXT NOT NULL CHECK (source IN (
        'player_action',
        'rule_resolution',
        'world_simulation',
        'procedural_generation',
        'admin_correction',
        'import'
    )),
    actor_kind TEXT NULL CHECK (actor_kind IS NULL OR actor_kind IN ('entity', 'faction')),
    actor_id TEXT NULL,
    command_id TEXT NOT NULL,
    event_kind TEXT NOT NULL CHECK (length(event_kind) > 0),
    event_schema_version INTEGER NOT NULL CHECK (event_schema_version > 0),
    payload_json TEXT NOT NULL,
    CHECK (
        (actor_kind IS NULL AND actor_id IS NULL) OR
        (actor_kind IS NOT NULL AND actor_id IS NOT NULL)
    ),
    UNIQUE (campaign_id, sequence),
    UNIQUE (campaign_id, id),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, session_id) REFERENCES play_sessions(campaign_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (campaign_id, command_id) REFERENCES command_audit(campaign_id, id) ON DELETE CASCADE
);

CREATE INDEX ix_event_journal_command
ON event_journal(campaign_id, command_id, sequence);

CREATE TABLE event_causes (
    campaign_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    cause_event_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    PRIMARY KEY (campaign_id, event_id, cause_event_id),
    UNIQUE (campaign_id, event_id, ordinal),
    CHECK (event_id <> cause_event_id),
    FOREIGN KEY (campaign_id, event_id) REFERENCES event_journal(campaign_id, id) ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, cause_event_id) REFERENCES event_journal(campaign_id, id) ON DELETE CASCADE
);

CREATE INDEX ix_event_causes_parent
ON event_causes(campaign_id, cause_event_id);

CREATE TRIGGER tr_command_audit_no_update
BEFORE UPDATE ON command_audit
BEGIN
    SELECT RAISE(ABORT, 'command audit records are immutable');
END;

CREATE TRIGGER tr_event_journal_no_update
BEFORE UPDATE ON event_journal
BEGIN
    SELECT RAISE(ABORT, 'event journal records are immutable');
END;

CREATE TRIGGER tr_event_causes_no_update
BEFORE UPDATE ON event_causes
BEGIN
    SELECT RAISE(ABORT, 'event causal edges are immutable');
END;

CREATE TRIGGER tr_event_cause_precedes
BEFORE INSERT ON event_causes
BEGIN
    SELECT CASE WHEN NOT EXISTS (
        SELECT 1
        FROM event_journal child
        JOIN event_journal parent
          ON parent.campaign_id = child.campaign_id
        WHERE child.campaign_id = NEW.campaign_id
          AND child.id = NEW.event_id
          AND parent.id = NEW.cause_event_id
          AND parent.sequence < child.sequence
    ) THEN RAISE(ABORT, 'causal parent must exist in the same campaign and precede the child event') END;
END;
