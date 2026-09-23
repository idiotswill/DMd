CREATE UNIQUE INDEX ux_play_session_participant_ordinal
ON play_session_participants(session_id, ordinal);

CREATE TRIGGER tr_play_sessions_campaign_immutable
BEFORE UPDATE OF campaign_id ON play_sessions
FOR EACH ROW
WHEN OLD.campaign_id <> NEW.campaign_id
BEGIN
    SELECT RAISE(ABORT, 'play session campaign_id is immutable');
END;
