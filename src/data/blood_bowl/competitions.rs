use crate::AppState;
use crate::data::blood_bowl::competitions::registrations::TeamRegistration;
use crate::data::blood_bowl::competitions::schedule::{CompetitionSchedule, StageSchedule};
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::competitions::standings::{CompetitionStandings, StageStandings};
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub mod registrations;
pub mod schedule;
pub mod stages;
pub mod standings;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct CompetitionProgressRow {
    stage_name: String,
    round_name: String,
}

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
        let mut playing_stage_name = None;
        let mut playing_round_name = None;

        if let Some(progress) = Competition::select_progress_for_id(state, self.id).await? {
            playing_stage_name = Some(progress.stage_name);
            playing_round_name = Some(progress.round_name);
        }

        Ok(Competition {
            id: self.id,
            name: self.name,
            edition_number: self.edition_number,
            director,
            version: self.version,
            description: self.description,
            started: self.started,
            closed: self.closed,
            playing_stage_name,
            playing_round_name,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Competition {
    pub id: i32,
    pub name: String,
    pub edition_number: i32,
    pub director: Option<User>,
    pub version: Version,
    pub description: String,
    pub started: bool,
    pub closed: bool,
    pub playing_stage_name: Option<String>,
    pub playing_round_name: Option<String>,
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

    pub fn result(&self) -> String {
        if self.closed {
            "🏆 <Vainqueur...>".to_string()
        } else {
            self.progress_status()
        }
    }

    pub fn progress_status(&self) -> String {
        if let (Some(playing_stage_name), Some(playing_round_name)) =
            (&self.playing_stage_name, &self.playing_round_name)
        {
            format!("{} - {}", playing_stage_name, playing_round_name)
        } else {
            "Pas encore démarré".to_string()
        }
    }

    pub fn new(creator: User, name: String) -> Self {
        Self {
            id: 0,
            name,
            edition_number: 1,
            director: Some(creator),
            version: Version::LAST_VERSION,
            description: "".to_string(),
            started: false,
            closed: false,
            playing_stage_name: None,
            playing_round_name: None,
        }
    }

    pub fn is_new(&self) -> bool {
        self.id.eq(&0)
    }

    pub async fn save(&mut self, state: &AppState, connected_user: &User) -> Result<(), AppError> {
        tracing::debug!("save by user={:?} with id={}", connected_user, self.id);

        let editions = self.select_editions(state).await?;

        if let Some(director) = &self.director {
            if !self.is_new() {
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
                                last_updated = CURRENT_TIMESTAMP,
                                started_at = (
                                    SELECT MIN(bb_games.game_at)
                                    FROM bb_competitions_stages_schedule
                                    INNER JOIN bb_games
                                    ON bb_games.id = bb_competitions_stages_schedule.game_id
                                    WHERE bb_competitions_stages_schedule.competition_id = bb_competitions.id
                                )
                            WHERE bb_competitions.id = $6",
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

                let new_competition_id: i32 = sqlx::query_scalar(
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

                self.id = new_competition_id;
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

    pub async fn select_for_game_id(
        state: &AppState,
        game_id: i32,
    ) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_for_game_id with game_id={}", game_id);

        let row: Option<CompetitionRow> = sqlx::query_as(
            "SELECT bb_competitions.id,
                        bb_competitions.name,
                        bb_competitions.edition_number,
                        bb_competitions.director,
                        bb_competitions.version,
                        bb_competitions.description,
                        bb_competitions.started_at IS NOT NULL as started,
                        bb_competitions.closed_at IS NOT NULL as closed
                FROM bb_competitions
                INNER JOIN bb_competitions_stages_schedule
                ON bb_competitions_stages_schedule.competition_id = bb_competitions.id
                WHERE bb_competitions_stages_schedule.game_id = $1",
        )
        .bind(game_id.clone())
        .fetch_optional(&state.db)
        .await?;

        if let Some(competition_row) = row {
            Ok(Some(competition_row.into_competition(state).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn select_all_preparing(state: &AppState) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_all_preparing");

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
            WHERE started_at IS NULL
            AND closed_at IS NULL
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
            WHERE started_at IS NOT NULL
            AND closed_at IS NULL
            ORDER BY started_at DESC",
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

    pub async fn select_owned(state: &AppState, director: User) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_owned for director={:?}", director);

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
            WHERE director = $1
            ORDER BY last_updated DESC",
        )
        .bind(director.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut competitions: Vec<Competition> = Vec::with_capacity(rows.len());

        for row in rows {
            competitions.push(row.into_competition(state).await?);
        }

        Ok(competitions)
    }

    pub async fn select_for_team(state: &AppState, team_id: i32) -> Result<Vec<Self>, AppError> {
        tracing::debug!("select_for_team for team_id={}", team_id);

        let rows: Vec<CompetitionRow> = sqlx::query_as(
            "SELECT bb_competitions.id,
                    bb_competitions.name,
                    bb_competitions.edition_number,
                    bb_competitions.director,
                    bb_competitions.version,
                    bb_competitions.description,
                    bb_competitions.started_at IS NOT NULL as started,
                    bb_competitions.closed_at IS NOT NULL as closed
            FROM bb_competitions
            INNER JOIN bb_competitions_teams
            ON bb_competitions_teams.competition_id = bb_competitions.id
            WHERE bb_competitions_teams.team_id = $1
            AND bb_competitions_teams.validated = TRUE
            ORDER BY last_updated DESC",
        )
        .bind(team_id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut competitions: Vec<Competition> = Vec::with_capacity(rows.len());

        for row in rows {
            competitions.push(row.into_competition(state).await?);
        }

        Ok(competitions)
    }

    async fn select_progress_for_id(
        state: &AppState,
        competition_id: i32,
    ) -> Result<Option<CompetitionProgressRow>, AppError> {
        tracing::debug!(
            "select_progress_for_id for competition_id={}",
            competition_id
        );

        let progress: Option<CompetitionProgressRow> = sqlx::query_as(
            "SELECT bb_competitions_stages.stage_name,
                        bb_competitions_stages_schedule.round_name
                FROM bb_competitions_stages_schedule
                INNER JOIN bb_games
                ON bb_games.id = bb_competitions_stages_schedule.game_id
                INNER JOIN bb_competitions_stages
                ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
                WHERE bb_competitions_stages_schedule.competition_id = $1
                ORDER BY bb_games.game_at DESC
                LIMIT 1",
        )
        .bind(competition_id.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(progress)
    }

    pub async fn select_stages(&self, state: &AppState) -> Result<Vec<CompetitionStage>, AppError> {
        CompetitionStage::select_for_competition(state, &self).await
    }

    pub async fn insert_stage(
        &mut self,
        state: &AppState,
        connected_user: &User,
        stage_type_to_add: CompetitionStageType,
    ) -> Result<(), AppError> {
        CompetitionStage::insert_for_competition(state, connected_user, self, stage_type_to_add)
            .await
    }

    pub async fn delete_stage(
        &mut self,
        state: &AppState,
        connected_user: &User,
        stage_id: i32,
    ) -> Result<(), AppError> {
        CompetitionStage::delete_for_competition(state, connected_user, self, stage_id).await
    }

    pub async fn select_teams_registrations(
        &self,
        state: &AppState,
    ) -> Result<Vec<TeamRegistration>, AppError> {
        TeamRegistration::select_for_competition(state, self).await
    }

    pub async fn insert_team_registration(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
    ) -> Result<(), AppError> {
        TeamRegistration::insert(state, connected_user, self, team_id).await
    }

    pub async fn delete_team_registration(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
    ) -> Result<(), AppError> {
        TeamRegistration::delete(state, connected_user, self, team_id).await
    }

    pub async fn update_team_validation(
        &self,
        state: &AppState,
        connected_user: &User,
        team_id: i32,
        validation: bool,
    ) -> Result<(), AppError> {
        TeamRegistration::update_validation(state, connected_user, self, team_id, validation).await
    }

    pub async fn select_playing_teams(
        &self,
        state: &AppState,
    ) -> Result<Vec<TeamSummary>, AppError> {
        TeamRegistration::select_validated_teams_for_competition(state, self).await
    }

    pub async fn schedule_and_standings(
        &self,
        state: &AppState,
    ) -> Result<(CompetitionSchedule, CompetitionStandings), AppError> {
        let mut teams_entering_next_stage =
            TeamSummary::list_into_list_with_option(&self.select_playing_teams(state).await?);

        let stages = self.select_stages(state).await?;

        let mut stages_schedules: Vec<StageSchedule> = Vec::with_capacity(stages.len());
        let mut stages_standings: Vec<StageStandings> = Vec::with_capacity(stages.len());

        for stage in stages.iter() {
            let (stage_schedule, stage_standings) = stage
                .schedule_and_standings(state, &teams_entering_next_stage)
                .await?;

            let stage_is_finished = stage_schedule.finished;

            stages_schedules.push(stage_schedule);
            stages_standings.push(stage_standings.clone());

            if stage_is_finished {
                teams_entering_next_stage = stage_standings.into();
            } else {
                teams_entering_next_stage = vec![None; teams_entering_next_stage.len()];
            }
        }

        Ok((stages_schedules.into(), stages_standings.into()))
    }
}
