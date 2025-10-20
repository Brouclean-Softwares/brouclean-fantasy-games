CREATE TABLE IF NOT EXISTS bb_players_advancements (
    player_id INTEGER NOT NULL REFERENCES bb_players ON DELETE CASCADE,
    advancement VARCHAR,
    options_to_choose VARCHAR,
    star_player_points INTEGER NOT NULL,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);