CREATE TABLE IF NOT EXISTS bb_games (
    id SERIAL PRIMARY KEY,
    version VARCHAR NOT NULL,
    created_by INTEGER REFERENCES users ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    game_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE,
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
    second_team_is_winner BOOLEAN NOT NULL DEFAULT FALSE,
    events TEXT NOT NULL,
    playing_players TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS bb_games_teams_players (
    game_id INTEGER REFERENCES bb_games ON DELETE CASCADE,
    team_id INTEGER REFERENCES bb_teams ON DELETE RESTRICT,
    player_id INTEGER REFERENCES bb_players ON DELETE RESTRICT,
    player_id_in_game INTEGER NOT NULL,
    player_number INTEGER NOT NULL,
    player_position VARCHAR NOT NULL,
    passing_completions INTEGER NOT NULL DEFAULT 0,
    throwing_completions INTEGER NOT NULL DEFAULT 0,
    deflections INTEGER NOT NULL DEFAULT 0,
    interceptions INTEGER NOT NULL DEFAULT 0,
    casualties INTEGER NOT NULL DEFAULT 0,
    touchdowns INTEGER NOT NULL DEFAULT 0,
    most_valuable_player INTEGER NOT NULL DEFAULT 0,
    star_player_points INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS bb_players_injuries (
    player_id INTEGER NOT NULL REFERENCES bb_players ON DELETE CASCADE,
    game_id INTEGER NOT NULL REFERENCES bb_games ON DELETE RESTRICT,
    injury VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    recovered_at TIMESTAMP WITH TIME ZONE
);