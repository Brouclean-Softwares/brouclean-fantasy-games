CREATE TABLE IF NOT EXISTS bb_games (
    id SERIAL PRIMARY KEY,
    version VARCHAR NOT NULL,
    created_by INTEGER REFERENCES users ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    scheduled_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    closed_at TIMESTAMP WITH TIME ZONE,
    first_coach_id INTEGER REFERENCES users ON DELETE SET NULL,
    first_team_id INTEGER REFERENCES bb_teams ON DELETE RESTRICT,
    first_team_score INTEGER NOT NULL DEFAULT 0,
    first_team_casualties INTEGER NOT NULL DEFAULT 0,
    first_team_is_winner BOOLEAN NOT NULL DEFAULT FALSE,
    second_coach_id INTEGER REFERENCES users ON DELETE SET NULL,
    second_team_id INTEGER REFERENCES bb_teams ON DELETE RESTRICT,
    second_team_score INTEGER NOT NULL DEFAULT 0,
    second_team_casualties INTEGER NOT NULL DEFAULT 0,
    second_team_is_winner BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS bb_games_teams_players (
    game_id INTEGER REFERENCES bb_games ON DELETE CASCADE,
    team_id INTEGER REFERENCES bb_teams ON DELETE SET NULL,
    player_id INTEGER REFERENCES bb_players ON DELETE SET NULL,
    player_number INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS bb_games_events (
    game_id INTEGER REFERENCES bb_games ON DELETE CASCADE,
    event JSONB NOT NULL,
    event_order INTEGER NOT NULL
);