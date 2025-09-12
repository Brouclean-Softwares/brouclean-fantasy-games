use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerDetail {
    id: i32,
    version: Version,
    name: String,
    position: Position,
    number: i32,
}

pub async fn select_under_contract_for_team(
    state: &AppState,
    team_id: i32,
) -> Result<Vec<(i32, Player)>, AppError> {
    tracing::debug!("select_under_contract_for_team with team_id={}", team_id);

    let players_detail: Vec<PlayerDetail> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_players.version,
                    bb_players.name,
                    bb_players.position,
                    bb_teams_players.number
            FROM bb_players
            LEFT JOIN bb_teams_players ON bb_players.id = bb_teams_players.player_id
            WHERE bb_teams_players.team_id = $1
            AND bb_teams_players.contract_end IS NULL
            ORDER BY bb_teams_players.number ASC",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut players: Vec<(i32, Player)> = Vec::new();

    for player_detail in players_detail {
        players.push((
            player_detail.number,
            Player {
                id: Some(player_detail.id),
                version: player_detail.version,
                position: player_detail.position,
                name: player_detail.name,
            },
        ));
    }

    Ok(players)
}
