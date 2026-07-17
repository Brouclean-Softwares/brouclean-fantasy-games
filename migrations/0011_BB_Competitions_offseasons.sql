CREATE TABLE IF NOT EXISTS bb_redrafts_in_offseasons (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE RESTRICT,
    team_id INTEGER NOT NULL REFERENCES bb_teams ON DELETE RESTRICT,
    raised_funds INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    closed_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (competition_id, team_id)
);

CREATE TABLE IF NOT EXISTS bb_redrafting_players (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE RESTRICT,
    team_id INTEGER NOT NULL REFERENCES bb_teams ON DELETE RESTRICT,
    player_id INTEGER NOT NULL REFERENCES bb_players ON DELETE RESTRICT,
    has_experience BOOLEAN NOT NULL,
    drafted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (competition_id, team_id, player_id)
);

CREATE TABLE IF NOT EXISTS bb_redrafting_staff (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE RESTRICT,
    team_id INTEGER NOT NULL REFERENCES bb_teams ON DELETE RESTRICT,
    staff VARCHAR NOT NULL,
    number INTEGER NOT NULL,
    drafted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (competition_id, team_id, staff)
);

CREATE TABLE IF NOT EXISTS bb_redrafting_positions (
    competition_id INTEGER NOT NULL REFERENCES bb_competitions ON DELETE RESTRICT,
    team_id INTEGER NOT NULL REFERENCES bb_teams ON DELETE RESTRICT,
    position VARCHAR NOT NULL,
    number INTEGER NOT NULL,
    drafted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (competition_id, team_id, position)
);