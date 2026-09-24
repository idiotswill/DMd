CREATE TABLE campaign_lifecycle (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    storage_status TEXT NOT NULL CHECK (storage_status IN ('active', 'archived')),
    state_schema_version INTEGER NOT NULL CHECK (state_schema_version > 0),
    created_at_utc TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at_utc TEXT NULL,
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE
);

INSERT INTO campaign_lifecycle (
    campaign_id, display_name, storage_status, state_schema_version
)
SELECT
    campaign_id,
    CASE
        WHEN json_valid(state_json)
        THEN COALESCE(NULLIF(trim(json_extract(state_json, '$.campaign.display_name')), ''), campaign_id)
        ELSE campaign_id
    END,
    'active',
    schema_version
FROM campaign_state_current;

CREATE TRIGGER tr_campaign_lifecycle_on_state_insert
AFTER INSERT ON campaign_state_current
BEGIN
    INSERT INTO campaign_lifecycle (
        campaign_id, display_name, storage_status, state_schema_version
    ) VALUES (
        NEW.campaign_id,
        CASE
            WHEN json_valid(NEW.state_json)
            THEN COALESCE(NULLIF(trim(json_extract(NEW.state_json, '$.campaign.display_name')), ''), NEW.campaign_id)
            ELSE NEW.campaign_id
        END,
        'active',
        NEW.schema_version
    );
END;

CREATE TRIGGER tr_campaign_lifecycle_on_state_update
AFTER UPDATE OF schema_version, state_json ON campaign_state_current
BEGIN
    UPDATE campaign_lifecycle
    SET display_name = CASE
            WHEN json_valid(NEW.state_json)
            THEN COALESCE(NULLIF(trim(json_extract(NEW.state_json, '$.campaign.display_name')), ''), NEW.campaign_id)
            ELSE display_name
        END,
        state_schema_version = NEW.schema_version
    WHERE campaign_id = NEW.campaign_id;
END;

-- Rows in these tables exist only inside explicit lifecycle transactions. The purge row authorizes
-- deletion of campaign_state_current itself; history guards are independently tied to the root cascade.
-- Restore authorization suppresses only sequence-0 snapshot bootstrap while an exact export is imported.
CREATE TABLE campaign_purge_authorizations (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    authorized_by TEXT NOT NULL CHECK (authorized_by = 'admin'),
    reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    created_at_utc TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE campaign_restore_authorizations (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    authorized_by TEXT NOT NULL CHECK (authorized_by = 'import'),
    created_at_utc TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Authorization applies only to deleting the campaign root. Historical child rows do not
-- consult this table, so creating an authorization row can never unlock selective surgery.
CREATE TRIGGER tr_campaign_state_current_purge_guard
BEFORE DELETE ON campaign_state_current
WHEN NOT EXISTS (
    SELECT 1 FROM campaign_purge_authorizations authorization
    WHERE authorization.campaign_id = OLD.campaign_id
)
BEGIN
    SELECT RAISE(ABORT, 'campaign purge requires explicit authorization');
END;


-- Completing deletion of the authorized aggregate root mechanically removes every current
-- campaign-owned row that does not already cascade from campaign_state_current. History,
-- snapshots, lifecycle metadata, and query projections are removed by FK cascade first;
-- sessions then become deletable despite their journal RESTRICT references.
CREATE TRIGGER tr_campaign_state_current_purge_cleanup
AFTER DELETE ON campaign_state_current
BEGIN
    DELETE FROM play_sessions WHERE campaign_id = OLD.campaign_id;
    DELETE FROM campaign_purge_authorizations WHERE campaign_id = OLD.campaign_id;
    DELETE FROM campaign_restore_authorizations WHERE campaign_id = OLD.campaign_id;
END;

DROP TRIGGER tr_command_audit_no_delete;
DROP TRIGGER tr_event_journal_no_delete;
DROP TRIGGER tr_event_causes_no_delete;
DROP TRIGGER tr_campaign_snapshots_no_delete;

CREATE TRIGGER tr_command_audit_no_delete
BEFORE DELETE ON command_audit
WHEN EXISTS (
    SELECT 1 FROM campaign_state_current state
    WHERE state.campaign_id = OLD.campaign_id
)
BEGIN
    SELECT RAISE(ABORT, 'command audit records are immutable');
END;

CREATE TRIGGER tr_event_journal_no_delete
BEFORE DELETE ON event_journal
WHEN EXISTS (
    SELECT 1 FROM campaign_state_current state
    WHERE state.campaign_id = OLD.campaign_id
)
BEGIN
    SELECT RAISE(ABORT, 'event journal records are immutable');
END;

CREATE TRIGGER tr_event_causes_no_delete
BEFORE DELETE ON event_causes
WHEN EXISTS (
    SELECT 1 FROM campaign_state_current state
    WHERE state.campaign_id = OLD.campaign_id
)
BEGIN
    SELECT RAISE(ABORT, 'event causal edges are immutable');
END;

CREATE TRIGGER tr_campaign_snapshots_no_delete
BEFORE DELETE ON campaign_snapshots
WHEN EXISTS (
    SELECT 1 FROM campaign_state_current state
    WHERE state.campaign_id = OLD.campaign_id
)
BEGIN
    SELECT RAISE(ABORT, 'campaign snapshots are immutable');
END;

-- New sequence-0 campaigns still receive their recovery anchor automatically. An explicit restore
-- transaction suppresses this one trigger so it can restore the exact exported immutable snapshots,
-- including their original creation metadata, rather than synthesizing a new sequence-0 row.
DROP TRIGGER tr_campaign_snapshot_on_state_insert;

CREATE TRIGGER tr_campaign_snapshot_on_state_insert
AFTER INSERT ON campaign_state_current
WHEN NEW.applied_event_sequence = 0
 AND NOT EXISTS (
    SELECT 1 FROM campaign_restore_authorizations authorization
    WHERE authorization.campaign_id = NEW.campaign_id
 )
BEGIN
    INSERT INTO campaign_snapshots (
        campaign_id, event_sequence, state_schema_version, state_json
    ) VALUES (
        NEW.campaign_id, NEW.applied_event_sequence, NEW.schema_version, NEW.state_json
    );
END;
