use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct BBTeam {
    pub id: Option<i32>,
    pub version: Version,
    pub name: String,
    pub roster: Roster,
    pub coach_id: Option<i32>,
    pub treasury: i32,
    pub value: i32,
    pub current_value: i32,
    pub external_logo_url: Option<String>,
}

impl BBTeam {
    pub async fn select_all(state: &AppState) -> Result<Vec<Self>, AppError> {
        let teams: Vec<BBTeam> = sqlx::query_as(
            "SELECT id, version, name, roster, coach_id, treasury, external_logo_url, value, current_value
                FROM bb_teams
                ORDER BY value DESC",
        )
        .fetch_all(&state.db)
        .await?;

        Ok(teams)
    }

    pub async fn select_owned(state: &AppState, coach: User) -> Result<Vec<Self>, AppError> {
        let teams: Vec<BBTeam> = sqlx::query_as(
            "SELECT id, version, name, roster, coach_id, treasury, external_logo_url, value, current_value
                FROM bb_teams
                WHERE coach_id = $1
                ORDER BY value DESC",
        )
        .bind(coach.id.clone())
        .fetch_all(&state.db)
        .await?;

        Ok(teams)
    }

    pub async fn create() {}
}
