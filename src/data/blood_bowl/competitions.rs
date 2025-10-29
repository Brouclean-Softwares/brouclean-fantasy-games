use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct CompetitionRow {
    id: i32,
    name: String,
    edition_number: i32,
    director: Option<i32>,
    version: Version,
    description: String,
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
                                description = $5
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

    pub async fn select_by_id(state: &AppState, id: i32) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_by_id with id={}", id);

        let row: Option<CompetitionRow> = sqlx::query_as(
            "SELECT id,
                        name,
                        edition_number,
                        director,
                        version,
                        description
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
                    description
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
                    description
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

    pub async fn select_registered_teams(
        &self,
        state: &AppState,
    ) -> Result<Vec<TeamSummary>, AppError> {
        tracing::debug!("select_registered_teams for id={}", self.id);

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
            FROM bb_competitions_teams
            INNER JOIN bb_teams
            ON bb_teams.id = bb_competitions_teams.team_id
            LEFT JOIN users
            ON bb_teams.coach_id = users.id
            WHERE bb_competitions_teams.competition_id = $1
            ORDER BY users.name, bb_teams.name ASC",
        )
        .bind(self.id.clone())
        .fetch_all(&state.db)
        .await?;

        Ok(teams)
    }

    pub async fn insert_team_registration(
        state: &AppState,
        competition_id: i32,
        team_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_team_registration for competition_id={} and team_id={}",
            competition_id,
            team_id
        );

        sqlx::query(
            "INSERT INTO bb_competitions_teams (
                    competition_id,
                    team_id)
                VALUES ($1, $2)
                ON CONFLICT (competition_id, team_id) DO NOTHING",
        )
        .bind(competition_id.clone())
        .bind(team_id.clone())
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
                    description
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
}

pub struct CompetitionStage {
    pub position: i32,
    pub name: String,
    pub stage_type: CompetitionStageType,
    pub rules: Vec<CompetitionStageRule>,
}

pub enum CompetitionStageType {
    Championship,
    Groups,
    Tournament,
}

pub enum CompetitionStageRule {
    TeamMaximum(usize),
    GameNumberPerOpponent(usize),
    WinsExpected(usize),
}

pub struct CompetitionGame {
    pub game_id: Option<i32>,
}
