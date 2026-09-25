-- Presentation revisions and transport recovery are separate from game state/history.
-- Only the complete campaign-root purge may delete their immutable evidence.
CREATE TABLE table_projection_history (
    campaign_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK (ordinal > 0),
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, ordinal),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE
);

CREATE TABLE table_transport_bindings (
    command_id TEXT PRIMARY KEY NOT NULL,
    campaign_id TEXT NOT NULL,
    projection_ordinal INTEGER NOT NULL CHECK (projection_ordinal > 0),
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE
);

CREATE INDEX idx_table_transport_campaign ON table_transport_bindings(campaign_id);

CREATE TRIGGER tr_table_projection_no_update
BEFORE UPDATE ON table_projection_history
BEGIN SELECT RAISE(ABORT, 'table projection history is immutable'); END;

CREATE TRIGGER tr_table_projection_no_selective_delete
BEFORE DELETE ON table_projection_history
WHEN EXISTS (SELECT 1 FROM campaign_state_current WHERE campaign_id = OLD.campaign_id)
BEGIN SELECT RAISE(ABORT, 'table projection history is immutable'); END;

CREATE TRIGGER tr_table_transport_no_update
BEFORE UPDATE ON table_transport_bindings
BEGIN SELECT RAISE(ABORT, 'table transport bindings are immutable'); END;

CREATE TRIGGER tr_table_transport_no_selective_delete
BEFORE DELETE ON table_transport_bindings
WHEN EXISTS (SELECT 1 FROM campaign_state_current WHERE campaign_id = OLD.campaign_id)
BEGIN SELECT RAISE(ABORT, 'table transport bindings are immutable'); END;
