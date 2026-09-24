-- Schema 2 makes optional typed mechanical state explicit. Old binaries must reject the new
-- version instead of ignoring and later erasing rules state. No historical row is rewritten.
-- SQLx runs this complete migration in one transaction.
CREATE TEMP TABLE dmd_state_schema_upgrade_preflight (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    valid INTEGER NOT NULL CHECK (valid = 1)
);

-- Do not turn corrupt/mismatched state into a seemingly compatible save or discard unexpected
-- mechanical data. Missing/null rules is the only valid schema-1 starting point. CASE prevents
-- evaluating JSON expressions against malformed JSON.
INSERT INTO dmd_state_schema_upgrade_preflight (campaign_id, valid)
SELECT state.campaign_id,
    CASE WHEN json_valid(state.state_json) THEN
        CASE WHEN json_type(state.state_json) = 'object'
          AND json_type(state.state_json, '$.schema_version') = 'integer'
          AND json_extract(state.state_json, '$.schema_version') = 1
          AND json_extract(state.state_json, '$.campaign.id') = state.campaign_id
          AND json_type(state.state_json, '$.applied_event_sequence') = 'integer'
          AND json_extract(state.state_json, '$.applied_event_sequence') = state.applied_event_sequence
          AND NOT EXISTS (
              SELECT 1 FROM json_each(state.state_json) fields
              GROUP BY fields.key HAVING COUNT(*) > 1
          )
          AND (json_type(state.state_json, '$.rules') IS NULL
               OR json_type(state.state_json, '$.rules') = 'null')
          AND EXISTS (
              SELECT 1 FROM campaign_lifecycle lifecycle
              WHERE lifecycle.campaign_id = state.campaign_id
                AND lifecycle.state_schema_version = 1
          )
        THEN 1 ELSE 0 END
    ELSE 0 END
FROM campaign_state_current state
WHERE state.schema_version = 1;

-- Existing lifecycle and whole-image projection triggers update their metadata atomically.
-- The snapshot trigger requires a sequence advance, so these unchanged heads create no new anchor.
UPDATE campaign_state_current
SET schema_version = 2,
    state_json = json_set(state_json, '$.schema_version', 2, '$.rules', NULL)
WHERE campaign_id IN (SELECT campaign_id FROM dmd_state_schema_upgrade_preflight);

DROP TABLE dmd_state_schema_upgrade_preflight;
