use crate::data::users::User;
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
            INNER JOIN bb_teams_players ON bb_players.id = bb_teams_players.player_id
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

pub async fn update_name(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
    player_id: &i32,
    name: &String,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_name by user={:?} for team_id={} and player_id={} with name={}",
        connected_user,
        team_id,
        player_id,
        name
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_players
                SET name = $1
                FROM bb_teams_players, bb_teams
                WHERE bb_players.id = bb_teams_players.player_id
                AND bb_teams.id = bb_teams_players.team_id
                AND bb_players.id = $2
                AND bb_teams.id = $3
                AND bb_teams.coach_id = $4",
        )
        .bind(name.clone())
        .bind(player_id.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

pub async fn update_number(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
    player_id: &i32,
    number: &i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_number by user={:?} for team_id={} and player_id={} with number={}",
        connected_user,
        team_id,
        player_id,
        number
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_teams_players
                SET number = $1
                FROM bb_teams
                WHERE bb_teams.id = bb_teams_players.team_id
                AND bb_teams_players.player_id = $2
                AND bb_teams.id = $3
                AND bb_teams.coach_id = $4",
        )
        .bind(number.clone())
        .bind(player_id.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}
