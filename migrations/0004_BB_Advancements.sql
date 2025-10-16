CREATE TABLE IF NOT EXISTS bb_players_advancements (
    player_id INTEGER NOT NULL REFERENCES bb_players ON DELETE CASCADE,
    advancement VARCHAR NOT NULL,
    star_player_points INTEGER NOT NULL,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);