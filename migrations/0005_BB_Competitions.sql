CREATE TABLE IF NOT EXISTS bb_competitions (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    edition_number INTEGER NOT NULL,
    director INTEGER REFERENCES users ON DELETE SET NULL,
    version VARCHAR NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    closed_at TIMESTAMP WITH TIME ZONE,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bb_competitions_teams (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    team_id INTEGER REFERENCES bb_teams ON DELETE RESTRICT,
    validated BOOLEAN,
    team_number INTEGER,
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bb_competitions_stages (
    id SERIAL PRIMARY KEY,
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    stage_position INTEGER NOT NULL,
    stage_type VARCHAR NOT NULL,
    stage_name VARCHAR NOT NULL,
    stage_rules VARCHAR
);

CREATE TABLE IF NOT EXISTS bb_competitions_games (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    stage_id INTEGER NOT NULL REFERENCES bb_competitions_stages ON DELETE CASCADE,
    game_id INTEGER NOT NULL REFERENCES bb_games ON DELETE CASCADE,
    game_reference VARCHAR NOT NULL,
    game_name VARCHAR NOT NULL
);