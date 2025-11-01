use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams;
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::data::Id;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::versions::Version;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct CompetitionRow {
    id: i32,
    name: String,
    edition_number: i32,
    director: Option<i32>,
    version: Version,
    description: String,
    started: bool,
    closed: bool,
}

impl CompetitionRow {
    async fn into_competition(self, state: &AppState) -> Result<Competition, AppError> {
        let director = User::select_by_id(state, self.director).await?;

        Ok(Competition {
            id: self.id,
            name: self.name,
            edition_number: self.edition_number,
            director,
            version: self.version,
            description: self.description,
            started: self.started,
            closed: self.closed,
        })
    }
}

#[derive(Clone)]
pub struct Competition {
    pub id: i32,
    pub name: String,
    pub edition_number: i32,
    pub director: Option<User>,
    pub version: Version,
    pub description: String,
    pub started: bool,
    pub closed: bool,
}

impl Competition {
    pub fn name(&self) -> String {
        if self.edition_number > 1 {
            format!("{} - {}ème édition", self.name, self.edition_number)
        } else {
            self.name.clone()
        }
    }

    pub fn edition_number(&self) -> String {
        if self.edition_number == 1 {
            format!("{}ère", self.edition_number)
        } else if self.edition_number > 1 {
            format!("{}ème", self.edition_number)
        } else {
            self.edition_number.to_string()
        }
    }

    pub fn new(creator: Option<User>) -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            edition_number: 1,
            director: creator,
            version: Version::LAST_VERSION,
            description: "".to_string(),
            started: false,
            closed: false,
        }
    }

    pub async fn save(&mut self, state: &AppState, connected_user: &User) -> Result<(), AppError> {
        tracing::debug!("save by user={:?} with id={}", connected_user, self.id);

        let editions = self.select_editions(state).await?;

        if let Some(director) = &self.director {
            if self.id > 0 {
                if connected_user.eq(director) {
                    let edition_number = if let Some(position) = editions
                        .iter()
                        .position(|competition| competition.id.eq(&self.id))
                    {
                        position as i32 + 1
                    } else {
                        editions.len() as i32 + 1
                    };

                    sqlx::query(
                        "UPDATE bb_competitions
                            SET name = $1,
                                edition_number = $2,
                                director = $3,
                                version = $4,
                                description = $5,
                                last_updated = CURRENT_TIMESTAMP
                            WHERE id = $6",
                    )
                    .bind(self.name.clone())
                    .bind(edition_number.clone())
                    .bind(director.id.clone())
                    .bind(self.version.clone())
                    .bind(self.description.clone())
                    .bind(self.id.clone())
                    .execute(&state.db)
                    .await?;
                }
            } else {
                let edition_number = editions.len() as i32 + 1;

                let new_competition_id: Id = sqlx::query_as(
                    "INSERT INTO bb_competitions (
                            name,
                            edition_number,
                            director,
                            version,
                            description)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING id",
                )
                .bind(self.name.clone())
                .bind(edition_number.clone())
                .bind(director.id.clone())
                .bind(self.version.clone())
                .bind(self.description.clone())
                .fetch_one(&state.db)
                .await?;

                self.id = new_competition_id.id;
            }
        }

        Ok(())
    }

    pub async fn delete(state: &AppState, connected_user: &User, id: i32) -> Result<(), AppError> {
        tracing::debug!("delete by user={:?} with id={}", connected_user, id);

        sqlx::query(
            "DELETE
                FROM bb_competitions
                WHERE id = $1
                AND director = $2",
        )
        .bind(id.clone())
        .bind(connected_user.id.clone())
        .execute(&state.db)
        .await?;

        Ok(())
    }

    pub async fn select_editions(&self, state: &AppState) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_editions with name={}", self.name);

        let rows: Vec<CompetitionRow> = sqlx::query_as(
            "SELECT id,
                    name,
                    edition_number,
                    director,
                    version,
                    description,
                    started_at IS NOT NULL as started,
                    closed_at IS NOT NULL as closed
            FROM bb_competitions
            WHERE name = $1
            ORDER BY edition_number, created_at",
        )
        .bind(self.name.clone())
        .fetch_all(&state.db)
        .await?;

        let mut competitions: Vec<Competition> = Vec::with_capacity(rows.len());

        for row in rows {
            competitions.push(row.into_competition(state).await?);
        }

        Ok(competitions)
    }

    pub async fn select_by_id(state: &AppState, id: i32) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_by_id with id={}", id);

        let row: Option<CompetitionRow> = sqlx::query_as(
            "SELECT id,
                        name,
                        edition_number,
                        director,
                        version,
                        description,
                        started_at IS NOT NULL as started,
                        closed_at IS NOT NULL as closed
                FROM bb_competitions
                WHERE id = $1",
        )
        .bind(id.clone())
        .fetch_optional(&state.db)
        .await?;

        if let Some(competition_row) = row {
            Ok(Some(competition_row.into_competition(state).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn select_all_in_progress(state: &AppState) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_all_in_progress");

        let rows: Vec<CompetitionRow> = sqlx::query_as(
            "SELECT id,
                    name,
                    edition_number,
                    director,
                    version,
                    description,
                    started_at IS NOT NULL as started,
                    closed_at IS NOT NULL as closed
            FROM bb_competitions
            WHERE closed_at IS NULL
            ORDER BY last_updated DESC",
        )
        .fetch_all(&state.db)
        .await?;

        let mut competitions: Vec<Competition> = Vec::with_capacity(rows.len());

        for competition_row in rows {
            competitions.push(competition_row.into_competition(state).await?);
        }

        Ok(competitions)
    }

    pub async fn select_all_closed(state: &AppState) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_all_closed");

        let rows: Vec<CompetitionRow> = sqlx::query_as(
            "SELECT id,
                    name,
                    edition_number,
                    director,
                    version,
                    description,
                    started_at IS NOT NULL as started,
                    closed_at IS NOT NULL as closed
            FROM bb_competitions
            WHERE closed_at IS NOT NULL
            ORDER BY closed_at DESC",
        )
        .fetch_all(&state.db)
        .await?;

        let mut competitions: Vec<Competition> = Vec::with_capacity(rows.len());

        for row in rows {
            competitions.push(row.into_competition(state).await?);
        }

        Ok(competitions)
    }

    pub async fn select_stages(&self, state: &AppState) -> Result<Vec<CompetitionStage>, AppError> {
        tracing::debug!("select_stages for competition_id={}", self.id);

        let rows: Vec<CompetitionStageRow> = sqlx::query_as(
            "SELECT id,
                    stage_name,
                    stage_type,
                    stage_rules
            FROM bb_competitions_stages
            WHERE competition_id = $1
            ORDER BY created_at ASC",
        )
        .bind(self.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut competition_stages: Vec<CompetitionStage> = Vec::with_capacity(rows.len());

        for competition_stage_row in rows {
            competition_stages.push(competition_stage_row.into_competition_stage().await?);
        }

        Ok(competition_stages)
    }

    pub async fn insert_stage(
        &mut self,
        state: &AppState,
        connected_user: &User,
        stage_type_to_add: CompetitionStageType,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_stage for competition_id={} and stage_type_to_add={:?}",
            self.id,
            stage_type_to_add,
        );

        if connected_user.eq(&self.director) && !self.closed {
            let rules: Vec<CompetitionStageRule> = vec![];

            sqlx::query(
                "INSERT INTO bb_competitions_stages (
                            competition_id,
                            stage_type,
                            stage_name,
                            stage_rules)
                        VALUES ($1, $2, $3, $4)",
            )
            .bind(self.id.clone())
            .bind(serde_json::to_string(&stage_type_to_add)?.clone())
            .bind(stage_type_to_add.to_string().clone())
            .bind(serde_json::to_string(&rules)?.clone())
            .execute(&state.db)
            .await?;

            self.save(state, connected_user).await?;
        }

        Ok(())
    }

    pub async fn delete_stage(
        &mut self,
        state: &AppState,
        connected_user: &User,
        stage_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "delete_stage for competition_id={} and stage_id={}",
            self.id,
            stage_id,
        );

        if connected_user.eq(&self.director) && !self.started {
            sqlx::query(
                "DELETE
                    FROM bb_competitions_stages
                    WHERE id = $1",
            )
            .bind(stage_id.clone())
            .execute(&state.db)
            .await?;

            self.save(state, connected_user).await?;
        }

        Ok(())
    }

    pub async fn select_teams_registrations(
        &self,
        state: &AppState,
    ) -> Result<Vec<TeamRegistration>, AppError> {
        tracing::debug!("select_teams_registrations for id={}", self.id);

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
        .bind(self.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut registrations: Vec<TeamRegistration> = Vec::with_capacity(registration_rows.len());

        for registration_row in registration_rows {
            registrations.push(registration_row.into_team_registration(state).await?);
        }

        Ok(registrations)
    }

    pub async fn insert_team_registration(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_team_registration for competition_id={} and team_id={}",
            self.id,
            team_id
        );

        let team = teams::select_by_id_without_staff_nor_players(state, team_id).await?;

        if (connected_user.eq(&self.director) || connected_user.eq(&team.coach)) && !self.started {
            sqlx::query(
                "INSERT INTO bb_competitions_teams (
                        competition_id,
                        team_id)
                    VALUES ($1, $2)
                    ON CONFLICT (competition_id, team_id) DO NOTHING",
            )
            .bind(self.id.clone())
            .bind(team_id.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }

    pub async fn delete_team_registration(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "delete_team_registration for competition_id={} and team_id={}",
            self.id,
            team_id
        );

        let team = teams::select_by_id_without_staff_nor_players(state, team_id).await?;

        if (connected_user.eq(&self.director) || connected_user.eq(&team.coach)) && !self.started {
            sqlx::query(
                "DELETE
                    FROM bb_competitions_teams
                    WHERE competition_id = $1
                    AND team_id = $2",
            )
            .bind(self.id.clone())
            .bind(team_id.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }

    pub async fn update_team_validation(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
        validation: bool,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "update_team_validation for competition_id={} and team_id={} with validation={}",
            self.id,
            team_id,
            validation
        );

        if connected_user.eq(&self.director) && !self.started {
            sqlx::query(
                "UPDATE bb_competitions_teams
                    SET validated = $3
                    WHERE competition_id = $1
                    AND team_id = $2",
            )
            .bind(self.id.clone())
            .bind(team_id.clone())
            .bind(validation.clone())
            .execute(&state.db)
            .await?;
        }

        Ok(())
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct CompetitionStageRow {
    id: i32,
    stage_name: String,
    stage_type: String,
    stage_rules: String,
}

impl CompetitionStageRow {
    async fn into_competition_stage(self) -> Result<CompetitionStage, AppError> {
        Ok(CompetitionStage {
            id: self.id,
            stage_name: self.stage_name,
            stage_type: serde_json::from_str(&self.stage_type)?,
            stage_rules: serde_json::from_str(&self.stage_rules)?,
        })
    }
}

pub struct CompetitionStage {
    pub id: i32,
    pub stage_name: String,
    pub stage_type: CompetitionStageType,
    pub stage_rules: Vec<CompetitionStageRule>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CompetitionStageType {
    Championship,
    Cup,
}

impl Display for CompetitionStageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            CompetitionStageType::Championship => "Championnat",
            CompetitionStageType::Cup => "Coupe",
        };

        write!(f, "{}", text)
    }
}

impl CompetitionStageType {
    pub fn available_list() -> Vec<Self> {
        let mut list = vec![Self::Championship, Self::Cup];

        list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

        list
    }
}

#[derive(Deserialize, Serialize)]
pub enum CompetitionStageRule {}

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
}

pub struct TeamRegistration {
    pub team_summary: TeamSummary,
    pub validated: Option<bool>,
    pub team_number: Option<i32>,
}

pub struct CompetitionResults {
    pub game_summary: Option<GameSummary>,
    pub game_reference: String,
}
