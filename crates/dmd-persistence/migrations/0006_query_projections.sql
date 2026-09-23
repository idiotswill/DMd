-- Query projections are derivative current-state indexes. Authoritative truth remains
-- campaign_state_current + append-only journal/snapshots. Every row is campaign-scoped and the
-- projection head records the exact materialized sequence/schema it mirrors.
CREATE TABLE projection_heads (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    state_schema_version INTEGER NOT NULL CHECK (state_schema_version > 0),
    applied_event_sequence INTEGER NOT NULL CHECK (applied_event_sequence >= 0),
    campaigns_count INTEGER NOT NULL CHECK (campaigns_count = 1),
    players_count INTEGER NOT NULL CHECK (players_count >= 0),
    characters_count INTEGER NOT NULL CHECK (characters_count >= 0),
    entities_count INTEGER NOT NULL CHECK (entities_count >= 0),
    factions_count INTEGER NOT NULL CHECK (factions_count >= 0),
    locations_count INTEGER NOT NULL CHECK (locations_count >= 0),
    scenes_count INTEGER NOT NULL CHECK (scenes_count >= 0),
    scene_presences_count INTEGER NOT NULL CHECK (scene_presences_count >= 0),
    items_count INTEGER NOT NULL CHECK (items_count >= 0),
    facts_count INTEGER NOT NULL CHECK (facts_count >= 0),
    claims_count INTEGER NOT NULL CHECK (claims_count >= 0),
    beliefs_count INTEGER NOT NULL CHECK (beliefs_count >= 0),
    knowledge_count INTEGER NOT NULL CHECK (knowledge_count >= 0),
    directives_count INTEGER NOT NULL CHECK (directives_count >= 0),
    FOREIGN KEY (campaign_id) REFERENCES campaign_state_current(campaign_id) ON DELETE CASCADE
);

CREATE TABLE projection_campaigns (
    campaign_id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    world_now INTEGER NOT NULL,
    calendar_id TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);

CREATE TABLE projection_players (
    campaign_id TEXT NOT NULL,
    player_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, player_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_players_name ON projection_players(campaign_id, display_name);

CREATE TABLE projection_characters (
    campaign_id TEXT NOT NULL,
    character_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    controlling_player_id TEXT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, character_id),
    UNIQUE (campaign_id, entity_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_characters_player ON projection_characters(campaign_id, controlling_player_id);
CREATE INDEX ix_projection_characters_status ON projection_characters(campaign_id, status);

CREATE TABLE projection_entities (
    campaign_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    kind_repr TEXT NOT NULL,
    existence TEXT NOT NULL,
    location_id TEXT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, entity_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_entities_location ON projection_entities(campaign_id, location_id);
CREATE INDEX ix_projection_entities_existence ON projection_entities(campaign_id, existence);

CREATE TABLE projection_factions (
    campaign_id TEXT NOT NULL,
    faction_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, faction_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_factions_status ON projection_factions(campaign_id, status);

CREATE TABLE projection_locations (
    campaign_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    parent_location_id TEXT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, location_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_locations_parent ON projection_locations(campaign_id, parent_location_id);

CREATE TABLE projection_scenes (
    campaign_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    mode_repr TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, scene_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_scenes_location ON projection_scenes(campaign_id, location_id);
CREATE INDEX ix_projection_scenes_status ON projection_scenes(campaign_id, status);

CREATE TABLE projection_scene_presences (
    campaign_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    role TEXT NOT NULL,
    PRIMARY KEY (campaign_id, scene_id, entity_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE,
    FOREIGN KEY (campaign_id, scene_id) REFERENCES projection_scenes(campaign_id, scene_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_scene_presences_entity ON projection_scene_presences(campaign_id, entity_id);

CREATE TABLE projection_items (
    campaign_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    definition_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity >= 0),
    owner_repr TEXT NOT NULL,
    custody_repr TEXT NOT NULL,
    state_repr TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, item_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_items_definition ON projection_items(campaign_id, definition_id);

CREATE TABLE projection_facts (
    campaign_id TEXT NOT NULL,
    fact_id TEXT NOT NULL,
    predicate TEXT NOT NULL,
    valid_from INTEGER NOT NULL,
    valid_until INTEGER NULL,
    source_event_id TEXT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, fact_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_facts_predicate ON projection_facts(campaign_id, predicate);

CREATE TABLE projection_claims (
    campaign_id TEXT NOT NULL,
    claim_id TEXT NOT NULL,
    predicate TEXT NOT NULL,
    made_at INTEGER NOT NULL,
    source_event_id TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, claim_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_claims_predicate ON projection_claims(campaign_id, predicate);

CREATE TABLE projection_beliefs (
    campaign_id TEXT NOT NULL,
    belief_id TEXT NOT NULL,
    holder_repr TEXT NOT NULL,
    predicate TEXT NOT NULL,
    confidence TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, belief_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_beliefs_predicate ON projection_beliefs(campaign_id, predicate);

CREATE TABLE projection_knowledge (
    campaign_id TEXT NOT NULL,
    knowledge_id TEXT NOT NULL,
    holder_repr TEXT NOT NULL,
    target_repr TEXT NOT NULL,
    acquired_at INTEGER NOT NULL,
    source_event_id TEXT NOT NULL,
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, knowledge_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);

CREATE TABLE projection_directives (
    campaign_id TEXT NOT NULL,
    directive_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    active INTEGER NOT NULL CHECK (active IN (0, 1)),
    record_json TEXT NOT NULL CHECK (json_valid(record_json)),
    PRIMARY KEY (campaign_id, directive_id),
    FOREIGN KEY (campaign_id) REFERENCES projection_heads(campaign_id) ON DELETE CASCADE
);
CREATE INDEX ix_projection_directives_scene_active ON projection_directives(campaign_id, scene_id, active);

-- Rebuild the complete derivative image whenever valid authoritative materialized state is inserted
-- or advanced. These triggers execute inside the caller's SQLite transaction, so projection failure
-- aborts the same authoritative transition rather than allowing drift.
CREATE TRIGGER tr_projection_state_insert
AFTER INSERT ON campaign_state_current
WHEN json_valid(NEW.state_json)
BEGIN
    INSERT INTO projection_heads VALUES (
        NEW.campaign_id, NEW.schema_version, NEW.applied_event_sequence, 1,
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.players')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.characters')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.entities')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.factions')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.locations')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.scenes')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.scenes') s, json_each(s.value, '$.presences')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.items')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.facts')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.claims')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.beliefs')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.knowledge')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.directives'))
    );
    INSERT INTO projection_campaigns
    SELECT NEW.campaign_id, json_extract(NEW.state_json, '$.campaign.display_name'), json_extract(NEW.state_json, '$.campaign.status'),
           json_extract(NEW.state_json, '$.clock.now'), json_extract(NEW.state_json, '$.clock.calendar_id'),
           json_extract(NEW.state_json, '$.campaign');
    INSERT INTO projection_players SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), value FROM json_each(NEW.state_json,'$.players');
    INSERT INTO projection_characters SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.entity_id'), json_extract(value,'$.controlling_player_id'), json_extract(value,'$.display_name'), json_extract(value,'$.status'), value FROM json_each(NEW.state_json,'$.characters');
    INSERT INTO projection_entities SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.kind'), json_extract(value,'$.existence'), json_extract(value,'$.location_id'), value FROM json_each(NEW.state_json,'$.entities');
    INSERT INTO projection_factions SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.status'), value FROM json_each(NEW.state_json,'$.factions');
    INSERT INTO projection_locations SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.parent_location_id'), value FROM json_each(NEW.state_json,'$.locations');
    INSERT INTO projection_scenes SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.location_id'), json_extract(value,'$.mode'), json_extract(value,'$.status'), json_extract(value,'$.started_at'), value FROM json_each(NEW.state_json,'$.scenes');
    INSERT INTO projection_scene_presences SELECT NEW.campaign_id, json_extract(s.value,'$.id'), json_extract(p.value,'$.entity_id'), json_extract(p.value,'$.role') FROM json_each(NEW.state_json,'$.scenes') s, json_each(s.value,'$.presences') p;
    INSERT INTO projection_items SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.definition_id'), json_extract(value,'$.display_name'), json_extract(value,'$.quantity'), json_extract(value,'$.owner'), json_extract(value,'$.custody'), json_extract(value,'$.state'), value FROM json_each(NEW.state_json,'$.items');
    INSERT INTO projection_facts SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.valid_from'), json_extract(value,'$.valid_until'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.facts');
    INSERT INTO projection_claims SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.made_at'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.claims');
    INSERT INTO projection_beliefs SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.holder'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.confidence'), json_extract(value,'$.updated_at'), value FROM json_each(NEW.state_json,'$.beliefs');
    INSERT INTO projection_knowledge SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.holder'), json_extract(value,'$.target'), json_extract(value,'$.acquired_at'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.knowledge');
    INSERT INTO projection_directives SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.scene_id'), json_extract(value,'$.summary'), json_extract(value,'$.created_at'), CASE json_extract(value,'$.active') WHEN 1 THEN 1 ELSE 0 END, value FROM json_each(NEW.state_json,'$.directives');
END;

CREATE TRIGGER tr_projection_state_update
AFTER UPDATE OF schema_version, applied_event_sequence, state_json ON campaign_state_current
WHEN json_valid(NEW.state_json)
BEGIN
    DELETE FROM projection_heads WHERE campaign_id = NEW.campaign_id;
    INSERT INTO projection_heads VALUES (
        NEW.campaign_id, NEW.schema_version, NEW.applied_event_sequence, 1,
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.players')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.characters')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.entities')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.factions')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.locations')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.scenes')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.scenes') s, json_each(s.value, '$.presences')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.items')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.facts')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.claims')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.beliefs')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.knowledge')),
        (SELECT COUNT(*) FROM json_each(NEW.state_json, '$.directives'))
    );
    INSERT INTO projection_campaigns SELECT NEW.campaign_id, json_extract(NEW.state_json, '$.campaign.display_name'), json_extract(NEW.state_json, '$.campaign.status'), json_extract(NEW.state_json, '$.clock.now'), json_extract(NEW.state_json, '$.clock.calendar_id'), json_extract(NEW.state_json, '$.campaign');
    INSERT INTO projection_players SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), value FROM json_each(NEW.state_json,'$.players');
    INSERT INTO projection_characters SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.entity_id'), json_extract(value,'$.controlling_player_id'), json_extract(value,'$.display_name'), json_extract(value,'$.status'), value FROM json_each(NEW.state_json,'$.characters');
    INSERT INTO projection_entities SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.kind'), json_extract(value,'$.existence'), json_extract(value,'$.location_id'), value FROM json_each(NEW.state_json,'$.entities');
    INSERT INTO projection_factions SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.status'), value FROM json_each(NEW.state_json,'$.factions');
    INSERT INTO projection_locations SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.display_name'), json_extract(value,'$.parent_location_id'), value FROM json_each(NEW.state_json,'$.locations');
    INSERT INTO projection_scenes SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.location_id'), json_extract(value,'$.mode'), json_extract(value,'$.status'), json_extract(value,'$.started_at'), value FROM json_each(NEW.state_json,'$.scenes');
    INSERT INTO projection_scene_presences SELECT NEW.campaign_id, json_extract(s.value,'$.id'), json_extract(p.value,'$.entity_id'), json_extract(p.value,'$.role') FROM json_each(NEW.state_json,'$.scenes') s, json_each(s.value,'$.presences') p;
    INSERT INTO projection_items SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.definition_id'), json_extract(value,'$.display_name'), json_extract(value,'$.quantity'), json_extract(value,'$.owner'), json_extract(value,'$.custody'), json_extract(value,'$.state'), value FROM json_each(NEW.state_json,'$.items');
    INSERT INTO projection_facts SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.valid_from'), json_extract(value,'$.valid_until'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.facts');
    INSERT INTO projection_claims SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.made_at'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.claims');
    INSERT INTO projection_beliefs SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.holder'), json_extract(value,'$.proposition.predicate'), json_extract(value,'$.confidence'), json_extract(value,'$.updated_at'), value FROM json_each(NEW.state_json,'$.beliefs');
    INSERT INTO projection_knowledge SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.holder'), json_extract(value,'$.target'), json_extract(value,'$.acquired_at'), json_extract(value,'$.source_event_id'), value FROM json_each(NEW.state_json,'$.knowledge');
    INSERT INTO projection_directives SELECT NEW.campaign_id, json_extract(value,'$.id'), json_extract(value,'$.scene_id'), json_extract(value,'$.summary'), json_extract(value,'$.created_at'), CASE json_extract(value,'$.active') WHEN 1 THEN 1 ELSE 0 END, value FROM json_each(NEW.state_json,'$.directives');
END;

-- A malformed materialized recovery artifact must not prevent snapshot+journal replay from doing its
-- job. Invalidate the derivative image without attempting to parse bad JSON. A later replay-backed
-- repair writes valid materialized state and the valid-state trigger rebuilds projections atomically.
CREATE TRIGGER tr_projection_state_invalid
AFTER UPDATE OF schema_version, applied_event_sequence, state_json ON campaign_state_current
WHEN NOT json_valid(NEW.state_json)
BEGIN
    DELETE FROM projection_heads WHERE campaign_id = NEW.campaign_id;
END;

-- Backfill existing campaigns after installing the triggers. Valid accepted materialized state derives
-- a projection image; corrupt materialized rows merely remain projection-less and recoverable through
-- snapshot+journal replay. This update does not append, delete, or rewrite authoritative history.
UPDATE campaign_state_current SET state_json = state_json;
