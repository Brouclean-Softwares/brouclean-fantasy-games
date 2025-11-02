CREATE TABLE IF NOT EXISTS bb_competitions (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    edition_number INTEGER NOT NULL,
    director INTEGER REFERENCES users ON DELETE SET NULL,
    version VARCHAR NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    closed_at TIMESTAMP WITH TIME ZONE,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bb_competitions_teams (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    team_id INTEGER REFERENCES bb_teams ON DELETE RESTRICT,
    validated BOOLEAN,
    draw_number DOUBLE PRECISION NOT NULL DEFAULT random(),
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (competition_id, team_id)
);

CREATE TABLE IF NOT EXISTS bb_competitions_stages (
    id SERIAL PRIMARY KEY,
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    stage_type VARCHAR NOT NULL,
    stage_name VARCHAR NOT NULL,
    rules TEXT NOT NULL,
    standings TEXT,
    schedule TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bb_competitions_stages_games (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE CASCADE,
    stage_id INTEGER NOT NULL REFERENCES bb_competitions_stages ON DELETE CASCADE,
    game_id INTEGER NOT NULL REFERENCES bb_games ON DELETE CASCADE UNIQUE,
    step_reference VARCHAR NOT NULL,
    step_name VARCHAR NOT NULL
);