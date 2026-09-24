-- Schema 4 adds optional typed encounter authority. Existing immutable history stays exact.
-- SQLx applies this preflight and all current-image updates as one transaction.
CREATE TEMP TABLE dmd_encounter_schema_upgrade_preflight (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    valid INTEGER NOT NULL CHECK (valid = 1)
);

INSERT INTO dmd_encounter_schema_upgrade_preflight (campaign_id, valid)
SELECT state.campaign_id,
    CASE WHEN json_valid(state.state_json) THEN
        CASE WHEN json_type(state.state_json) = 'object'
          AND json_type(state.state_json, '$.schema_version') = 'integer'
          AND json_extract(state.state_json, '$.schema_version') = 3
          AND json_extract(state.state_json, '$.campaign.id') = state.campaign_id
          AND json_type(state.state_json, '$.applied_event_sequence') = 'integer'
          AND json_extract(state.state_json, '$.applied_event_sequence') = state.applied_event_sequence
          AND NOT EXISTS (
              SELECT 1 FROM json_each(state.state_json) fields
              GROUP BY fields.key HAVING COUNT(*) > 1
          )
          AND (json_type(state.state_json, '$.encounter') IS NULL
               OR json_type(state.state_json, '$.encounter') = 'null')
          AND EXISTS (
              SELECT 1 FROM campaign_lifecycle lifecycle
              WHERE lifecycle.campaign_id = state.campaign_id
                AND lifecycle.state_schema_version = 3
          )
        THEN 1 ELSE 0 END
    ELSE 0 END
FROM campaign_state_current state
WHERE state.schema_version = 3;

-- Existing lifecycle/projection triggers follow this update; the sequence does not advance,
-- so the immutable periodic-snapshot trigger does not synthesize a replacement anchor.
UPDATE campaign_state_current
SET schema_version = 4,
    state_json = json_set(state_json, '$.schema_version', 4, '$.encounter', NULL)
WHERE campaign_id IN (SELECT campaign_id FROM dmd_encounter_schema_upgrade_preflight);

DROP TABLE dmd_encounter_schema_upgrade_preflight;
