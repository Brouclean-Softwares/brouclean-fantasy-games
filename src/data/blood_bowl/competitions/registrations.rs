use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::teams;
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct TeamRegistrationRow {
    team_id: i32,
    validated: Option<bool>,
    team_number: Option<i32>,
}

impl TeamRegistrationRow {
    async fn into_team_registration(self, state: &AppState) -> Result<TeamRegistration, AppError> {
        Ok(TeamRegistration {
            team_summary: teams::select_summary_by_id(state, self.team_id).await?,
            validated: self.validated,
            team_number: self.team_number,
        })
    }

    async fn into_team_summary(self, state: &AppState) -> Result<TeamSummary, AppError> {
        Ok(teams::select_summary_by_id(state, self.team_id).await?)
    }
}

pub struct TeamRegistration {
    pub team_summary: TeamSummary,
    pub validated: Option<bool>,
    pub team_number: Option<i32>,
}

impl TeamRegistration {
    pub async fn select_for_competition(
        state: &AppState,
        competition: &Competition,
    ) -> Result<Vec<TeamRegistration>, AppError> {
        tracing::debug!("select_for_competition for id={}", competition.id);

        let registration_rows: Vec<TeamRegistrationRow> = sqlx::query_as(
            "SELECT bb_competitions_teams.team_id,
                    bb_competitions_teams.validated,
                    bb_competitions_teams.team_number
                FROM bb_competitions_teams
                INNER JOIN bb_teams
                ON bb_teams.id = bb_competitions_teams.team_id
                WHERE bb_competitions_teams.competition_id = $1
                ORDER BY bb_teams.coach_id, bb_teams.name",
        )
        .bind(competition.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut registrations: Vec<TeamRegistration> = Vec::with_capacity(registration_rows.len());

        for registration_row in registration_rows {
            registrations.push(registration_row.into_team_registration(state).await?);
        }

        Ok(registrations)
    }

    pub async fn insert(
        state: &AppState,
        connected_user: &User,
        competition: &Competition,
        team_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert for competition_id={} and team_id={}",
            competition.id,
            team_id
        );

        let team = teams::select_by_id_without_staff_nor_players(state, team_id).await?;

        if (connected_user.eq(&competition.director) || connected_user.eq(&team.coach))
            && !competition.started
        {
            sqlx::query(
                "INSERT INTO bb_competitions_teams (
                        competition_id,
                        team_id)
                    VALUES ($1, $2)
                    ON CONFLICT (competition_id, team_id) DO NOTHING",
            )
            .bind(competition.id.clone())
            .bind(team_id.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }

    pub async fn delete(
        state: &AppState,
        connected_user: &User,
        competition: &Competition,
        team_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "delete for competition_id={} and team_id={}",
            competition.id,
            team_id
        );

        let team = teams::select_by_id_without_staff_nor_players(state, team_id).await?;

        if (connected_user.eq(&competition.director) || connected_user.eq(&team.coach))
            && !competition.started
        {
            sqlx::query(
                "DELETE
                    FROM bb_competitions_teams
                    WHERE competition_id = $1
                    AND team_id = $2",
            )
            .bind(competition.id.clone())
            .bind(team_id.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }

    pub async fn update_validation(
        state: &AppState,
        connected_user: &User,
        competition: &Competition,
        team_id: i32,
        validation: bool,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "update_validation for competition_id={} and team_id={} with validation={}",
            competition.id,
            team_id,
            validation
        );

        if connected_user.eq(&competition.director) && !competition.started {
            sqlx::query(
                "UPDATE bb_competitions_teams
                    SET validated = $3
                    WHERE competition_id = $1
                    AND team_id = $2",
            )
            .bind(competition.id.clone())
            .bind(team_id.clone())
            .bind(validation.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }

    pub async fn select_validated_teams_for_competition(
        state: &AppState,
        competition: &Competition,
    ) -> Result<Vec<TeamSummary>, AppError> {
        tracing::debug!(
            "select_validated_teams_for_competition for id={}",
            competition.id
        );

        let registration_rows: Vec<TeamRegistrationRow> = sqlx::query_as(
            "SELECT team_id,
                    validated,
                    team_number
                FROM bb_competitions_teams
                WHERE competition_id = $1
                AND validated = TRUE
                ORDER BY team_number",
        )
        .bind(competition.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut teams: Vec<TeamSummary> = Vec::with_capacity(registration_rows.len());

        for registration_row in registration_rows {
            teams.push(registration_row.into_team_summary(state).await?);
        }

        Ok(teams)
    }
}
