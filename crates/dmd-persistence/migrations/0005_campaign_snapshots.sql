CREATE TABLE campaign_snapshots (
    campaign_id TEXT NOT NULL,
    event_sequence INTEGER NOT NULL CHECK (event_sequence >= 0),
    state_schema_version INTEGER NOT NULL CHECK (state_schema_version > 0),
    state_json TEXT NOT NULL,
    created_at_utc TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (campaign_id, event_sequence),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE
);

-- Databases created before snapshot support cannot fabricate older historical states. Preserve the
-- authoritative state that actually exists at migration time as the first replay/recovery anchor.
INSERT INTO campaign_snapshots (campaign_id, event_sequence, state_schema_version, state_json)
SELECT campaign_id, applied_event_sequence, schema_version, state_json
FROM campaign_state_current;

CREATE TRIGGER tr_campaign_snapshots_no_update
BEFORE UPDATE ON campaign_snapshots
BEGIN
    SELECT RAISE(ABORT, 'campaign snapshots are immutable');
END;

CREATE TRIGGER tr_campaign_snapshots_no_delete
BEFORE DELETE ON campaign_snapshots
BEGIN
    SELECT RAISE(ABORT, 'campaign snapshots are immutable');
END;

-- New campaigns get a sequence-0 recovery anchor atomically with campaign-state initialization.
CREATE TRIGGER tr_campaign_snapshot_on_state_insert
AFTER INSERT ON campaign_state_current
BEGIN
    INSERT INTO campaign_snapshots (
        campaign_id, event_sequence, state_schema_version, state_json
    ) VALUES (
        NEW.campaign_id, NEW.applied_event_sequence, NEW.schema_version, NEW.state_json
    );
END;

-- Snapshot after at least 100 additional material events. The trigger runs inside the same SQLite
-- transaction as the authoritative state-head update, so a failed journal commit cannot leave a
-- snapshot that describes an uncommitted transition.
CREATE TRIGGER tr_campaign_snapshot_periodic
AFTER UPDATE OF applied_event_sequence, schema_version, state_json ON campaign_state_current
WHEN NEW.applied_event_sequence > OLD.applied_event_sequence
 AND NEW.applied_event_sequence - COALESCE((
        SELECT MAX(event_sequence)
        FROM campaign_snapshots
        WHERE campaign_id = NEW.campaign_id
    ), 0) >= 100
BEGIN
    INSERT INTO campaign_snapshots (
        campaign_id, event_sequence, state_schema_version, state_json
    ) VALUES (
        NEW.campaign_id, NEW.applied_event_sequence, NEW.schema_version, NEW.state_json
    );
END;
