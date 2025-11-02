use crate::data::blood_bowl::{games, players, staff, teams};
use crate::data::users::User;
use crate::data::Id;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;
use std::collections::HashMap;

pub struct TeamLogo {
    pub url: String,
}

impl From<&Team> for TeamLogo {
    fn from(team: &Team) -> Self {
        if let Some(url) = team.external_logo_url.clone() {
            Self { url }
        } else {
            Self {
                url: format!(
                    "/assets/images/blood_bowl/{}Logo.webp",
                    team.roster.type_name()
                ),
            }
        }
    }
}

impl From<TeamSummary> for TeamLogo {
    fn from(team: TeamSummary) -> Self {
        Self::from(&team)
    }
}

impl From<&TeamSummary> for TeamLogo {
    fn from(team: &TeamSummary) -> Self {
        if let Some(url) = team.external_logo_url.clone() {
            Self { url }
        } else {
            Self {
                url: format!(
                    "/assets/images/blood_bowl/{}Logo.webp",
                    team.roster.type_name()
                ),
            }
        }
    }
}

impl From<Team> for TeamLogo {
    fn from(team: Team) -> Self {
        Self::from(&team)
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamSummary {
    pub id: i32,
    pub version: Version,
    pub name: String,
    pub roster: Roster,
    pub coach_id: Option<i32>,
    pub coach_name: String,
    pub external_logo_url: Option<String>,
    pub value: i32,
    pub current_value: i32,
    pub treasury: i32,
    pub dedicated_fans: i32,
    pub under_creation: bool,
}

impl PartialEq<Self> for TeamSummary {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl TeamSummary {
    pub fn list_into_list_with_option(team_list: &Vec<Self>) -> Vec<Option<Self>> {
        team_list.iter().map(|team| Some(team.clone())).collect()
    }
}

pub async fn select_all(state: &AppState) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_all");

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users
            ON bb_teams.coach_id = users.id
            ORDER BY bb_teams.name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_all_filtered(
    state: &AppState,
    filter: String,
) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_all_filtered with filter={}", filter);

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE LOWER(bb_teams.name) LIKE $1
            OR LOWER(users.name) LIKE $1
            ORDER BY bb_teams.name ASC",
    )
    .bind(format!("%{}%", filter.to_lowercase()))
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_owned(state: &AppState, coach: User) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_owned for coach={:?}", coach);

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
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

pub async fn select_summary_by_id(state: &AppState, id: i32) -> Result<TeamSummary, AppError> {
    tracing::debug!("select_summary_by_id with id={}", id);

    let team: TeamSummary = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE bb_teams.id = $1
            LIMIT 1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(team)
}

pub async fn select_by_id_with_staff_and_players(
    state: &AppState,
    id: i32,
) -> Result<Team, AppError> {
    select_by_id(state, id, true, true).await
}

pub async fn select_by_id_without_players(state: &AppState, id: i32) -> Result<Team, AppError> {
    select_by_id(state, id, true, false).await
}

pub async fn select_by_id_without_staff_nor_players(
    state: &AppState,
    id: i32,
) -> Result<Team, AppError> {
    select_by_id(state, id, false, false).await
}

async fn select_by_id(
    state: &AppState,
    id: i32,
    staff_needed: bool,
    players_needed: bool,
) -> Result<Team, AppError> {
    tracing::debug!("select_from_id with id={}", id);

    let team: TeamSummary = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE bb_teams.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let staff = if staff_needed {
        staff::select_for_team(state, id).await?
    } else {
        HashMap::new()
    };

    let players = if players_needed {
        players::select_under_contract_for_team(state, id).await?
    } else {
        vec![]
    };

    let team = Team {
        id: team.id,
        version: team.version,
        roster: team.roster,
        name: team.name,
        coach: Coach {
            id: team.coach_id,
            name: team.coach_name,
        },
        treasury: team.treasury,
        external_logo_url: team.external_logo_url,
        staff,
        players,
        dedicated_fans: team.dedicated_fans as u8,
        under_creation: team.under_creation,
    };

    Ok(team)
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
                current_value,
                under_creation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
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

pub async fn update_values(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_values by user={:?} for team_id={}",
        connected_user,
        team_id,
    );

    let team = teams::select_by_id_with_staff_and_players(state, team_id).await?;
    let team_value = team.value()?;
    let team_current_value = team.current_value()?;

    sqlx::query(
        "UPDATE bb_teams
        SET value = $1,
            current_value = $2,
            last_updated = CURRENT_TIMESTAMP
        WHERE id = $3",
    )
    .bind(team_value.clone() as i32)
    .bind(team_current_value.clone() as i32)
    .bind(team_id.clone())
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn update_name(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
    name: &String,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_name by user={:?} for team_id={} with name={}",
        connected_user,
        team_id,
        name
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_teams
            SET name = $1,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $2
            AND coach_id = $3",
        )
        .bind(name.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

pub async fn delete(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
) -> Result<bool, AppError> {
    tracing::debug!(
        "delete by user={:?} for team_id={}",
        connected_user,
        team_id,
    );

    let games_played = games::select_played_by_team(state, team_id).await?;
    let games_scheduled = games::select_scheduled_for_team(state, team_id).await?;
    let game_playing = games::select_playing_by_team(state, team_id).await?;

    if games_played.len() > 0 || games_scheduled.len() > 0 || game_playing.is_some() {
        return Ok(false);
    }

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "DELETE
                FROM bb_players
                USING bb_teams_players, bb_teams
                WHERE bb_players.id = bb_teams_players.player_id
                AND bb_teams.id = bb_teams_players.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams_players
                USING bb_teams
                WHERE bb_teams.id = bb_teams_players.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams_staff
                USING bb_teams
                WHERE bb_teams.id = bb_teams_staff.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams
                WHERE id = $1
                AND coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(true)
}
