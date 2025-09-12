use crate::app::templates::blood_bowl::teams::TeamListRow;
use crate::data::blood_bowl::{players, staff};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub async fn select_all(state: &AppState) -> Result<Vec<TeamListRow>, AppError> {
    tracing::debug!("select_all");

    let teams: Vec<TeamListRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.treasury,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            ORDER BY value DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_owned(state: &AppState, coach: User) -> Result<Vec<TeamListRow>, AppError> {
    tracing::debug!("select_owned for coach={:?}", coach);

    let teams: Vec<TeamListRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    users.picture as coach_picture,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE coach_id = $1
            ORDER BY value DESC",
    )
    .bind(coach.id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct TeamDetail {
    id: i32,
    version: Version,
    name: String,
    roster: Roster,
    coach_id: Option<i32>,
    coach_name: Option<String>,
    external_logo_url: Option<String>,
    treasury: i32,
    dedicated_fans: i32,
}

pub async fn select_from_id(state: &AppState, id: i32) -> Result<Team, AppError> {
    tracing::debug!("select_from_id with id={}", id);

    let team: TeamDetail = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    users.picture as coach_picture,
                    bb_teams.external_logo_url,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE bb_teams.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let players = players::select_under_contract_for_team(state, id).await?;

    let staff = staff::select_for_team(state, id).await?;

    Ok(Team {
        id: Some(team.id),
        version: team.version,
        roster: team.roster,
        name: team.name,
        coach_id: team.coach_id,
        coach_name: team.coach_name.unwrap_or_default(),
        treasury: team.treasury,
        external_logo_url: team.external_logo_url,
        staff,
        players,
        dedicated_fans: team.dedicated_fans as u8,
    })
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

pub async fn create(state: &AppState, coach: &User, bb_team: &Team) -> Result<i32, AppError> {
    tracing::debug!(
        "create for coach={:?} the following team={:?}",
        coach,
        bb_team
    );

    let mut transaction = state.db.begin().await?;

    let new_team_id: Id = sqlx::query_as(
        "INSERT INTO bb_teams (
                version,
                name,
                roster,
                coach_id,
                treasury,
                dedicated_fans,
                value,
                current_value)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id",
    )
    .bind(bb_team.version.clone())
    .bind(bb_team.name.clone())
    .bind(bb_team.roster.clone())
    .bind(coach.id.clone())
    .bind(bb_team.treasury.clone())
    .bind(bb_team.dedicated_fans.clone() as i32)
    .bind(bb_team.value()? as i32)
    .bind(bb_team.current_value()? as i32)
    .fetch_one(&mut *transaction)
    .await?;

    for (staff, quantity) in bb_team.staff.clone() {
        sqlx::query(
            "INSERT INTO bb_teams_staff (
                staff,
                number,
                team_id)
            VALUES ($1, $2, $3)",
        )
        .bind(staff.clone())
        .bind(quantity.clone() as i32)
        .bind(new_team_id.id.clone())
        .execute(&mut *transaction)
        .await?;
    }

    for (number, player) in bb_team.players.clone() {
        let new_player_id: Id = sqlx::query_as(
            "INSERT INTO bb_players (
                version,
                name,
                position)
            VALUES ($1, $2, $3)
            RETURNING id",
        )
        .bind(player.version.clone())
        .bind(player.name.clone())
        .bind(player.position.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO bb_teams_players (
                number,
                team_id,
                player_id)
            VALUES ($1, $2, $3)",
        )
        .bind(number.clone())
        .bind(new_team_id.id.clone())
        .bind(new_player_id.id.clone())
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(new_team_id.id)
}
