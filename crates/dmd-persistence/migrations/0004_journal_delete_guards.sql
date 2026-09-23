CREATE TRIGGER tr_command_audit_no_delete
BEFORE DELETE ON command_audit
BEGIN
    SELECT RAISE(ABORT, 'command audit records are immutable');
END;

CREATE TRIGGER tr_event_journal_no_delete
BEFORE DELETE ON event_journal
BEGIN
    SELECT RAISE(ABORT, 'event journal records are immutable');
END;

CREATE TRIGGER tr_event_causes_no_delete
BEFORE DELETE ON event_causes
BEGIN
    SELECT RAISE(ABORT, 'event causal edges are immutable');
END;
