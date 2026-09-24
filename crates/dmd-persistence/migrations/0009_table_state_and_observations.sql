-- Schema 3 adds only an absent table runtime to existing current images. SQLx applies the
-- state upgrade and observation ledger atomically; immutable anchors and journals stay exact.
CREATE TEMP TABLE dmd_table_schema_upgrade_preflight (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    valid INTEGER NOT NULL CHECK (valid = 1)
);

INSERT INTO dmd_table_schema_upgrade_preflight (campaign_id, valid)
SELECT state.campaign_id,
    CASE WHEN json_valid(state.state_json) THEN
        CASE WHEN json_type(state.state_json) = 'object'
          AND json_type(state.state_json, '$.schema_version') = 'integer'
          AND json_extract(state.state_json, '$.schema_version') = 2
          AND json_extract(state.state_json, '$.campaign.id') = state.campaign_id
          AND json_type(state.state_json, '$.applied_event_sequence') = 'integer'
          AND json_extract(state.state_json, '$.applied_event_sequence') = state.applied_event_sequence
          AND NOT EXISTS (
              SELECT 1 FROM json_each(state.state_json) fields
              GROUP BY fields.key HAVING COUNT(*) > 1
          )
          AND (json_type(state.state_json, '$.table') IS NULL
               OR json_type(state.state_json, '$.table') = 'null')
          AND EXISTS (
              SELECT 1 FROM campaign_lifecycle lifecycle
              WHERE lifecycle.campaign_id = state.campaign_id
                AND lifecycle.state_schema_version = 2
          )
        THEN 1 ELSE 0 END
    ELSE 0 END
FROM campaign_state_current state
WHERE state.schema_version = 2;

UPDATE campaign_state_current
SET schema_version = 3,
    state_json = json_set(state_json, '$.schema_version', 3, '$.table', NULL)
WHERE campaign_id IN (SELECT campaign_id FROM dmd_table_schema_upgrade_preflight);

DROP TABLE dmd_table_schema_upgrade_preflight;

CREATE TABLE session_observations (
    id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK (ordinal > 0),
    session_id TEXT NULL,
    observed_event_sequence INTEGER NOT NULL CHECK (observed_event_sequence >= 0),
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    UNIQUE (campaign_id, ordinal),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES play_sessions(id) ON DELETE RESTRICT
);

CREATE TRIGGER tr_session_observations_no_update
BEFORE UPDATE ON session_observations
BEGIN
    SELECT RAISE(ABORT, 'session observations are immutable');
END;

CREATE TRIGGER tr_session_observations_no_selective_delete
BEFORE DELETE ON session_observations
WHEN EXISTS (SELECT 1 FROM campaign_state_current WHERE campaign_id = OLD.campaign_id)
BEGIN
    SELECT RAISE(ABORT, 'session observations are immutable');
END;
