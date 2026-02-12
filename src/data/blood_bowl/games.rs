use crate::data::blood_bowl::competitions::schedule::RoundSchedule;
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::blood_bowl::{coaches, players, teams};
use crate::data::users::User;
use crate::data::Id;
use crate::errors::AppError;
use crate::errors::AppError::BloodBowlAppError;
use crate::AppState;
use blood_bowl_rs::events::GameEvent;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::players::{Player, PlayerType};
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::versions::Version;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct GameSummary {
    pub id: i32,
    pub version: Version,
    pub title: Option<String>,
    pub game_at: NaiveDateTime,
    pub started: bool,
    pub finished: bool,
    pub closed: bool,
    pub first_team: TeamSummary,
    pub first_team_score: i32,
    pub first_team_casualties: i32,
    pub first_team_is_winner: bool,
    pub second_team: TeamSummary,
    pub second_team_score: i32,
    pub second_team_casualties: i32,
    pub second_team_is_winner: bool,
}

impl GameSummary {
    pub fn winner(&self) -> Option<TeamSummary> {
        if self.first_team_is_winner {
            Some(self.first_team.clone())
        } else if self.second_team_is_winner {
            Some(self.second_team.clone())
        } else {
            None
        }
    }

    pub fn loser(&self) -> Option<TeamSummary> {
        if self.first_team_is_winner {
            Some(self.second_team.clone())
        } else if self.second_team_is_winner {
            Some(self.first_team.clone())
        } else {
            None
        }
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameRow {
    id: i32,
    competition_name: Option<String>,
    stage_name: Option<String>,
    round_name: Option<String>,
    version: Version,
    created_by: Option<i32>,
    game_at: NaiveDateTime,
    started: bool,
    closed: bool,
    first_team_id: i32,
    first_team_score: i32,
    first_team_casualties: i32,
    first_team_is_winner: bool,
    second_team_id: i32,
    second_team_score: i32,
    second_team_casualties: i32,
    second_team_is_winner: bool,
    events: String,
    playing_players: Option<String>,
}

impl GameRow {
    fn game_title(&self) -> Option<String> {
        match (&self.competition_name, &self.stage_name, &self.round_name) {
            (Some(competition_name), _, _) => Some(competition_name.clone()),
            (_, _, _) => None,
        }
    }

    async fn into_game_summary(self, state: &AppState) -> Result<GameSummary, AppError> {
        let first_team = teams::select_summary_by_id(state, self.first_team_id).await?;
        let second_team = teams::select_summary_by_id(state, self.second_team_id).await?;
        let events: Vec<GameEvent> = serde_json::from_str(&self.events)?;
        let finished = events.contains(&GameEvent::GameEnd);

        let game_summary = GameSummary {
            id: self.id,
            version: self.version,
            title: self.game_title(),
            game_at: self.game_at,
            started: self.started,
            finished,
            closed: self.closed,
            first_team,
            first_team_score: self.first_team_score,
            first_team_casualties: self.first_team_casualties,
            first_team_is_winner: self.first_team_is_winner,
            second_team,
            second_team_score: self.second_team_score,
            second_team_casualties: self.second_team_casualties,
            second_team_is_winner: self.second_team_is_winner,
        };

        Ok(game_summary)
    }

    async fn into_game(self, state: &AppState) -> Result<Game, AppError> {
        let mut created_by = None;

        if let Some(coach_id) = self.created_by {
            created_by = coaches::select_by_id(state, Some(coach_id)).await?;
        }

        let first_team =
            teams::select_by_id_with_staff_and_players(state, self.first_team_id).await?;

        let second_team =
            teams::select_by_id_with_staff_and_players(state, self.second_team_id).await?;

        let mut game = Game {
            id: self.id,
            title: self.game_title(),
            version: self.version,
            created_by,
            game_at: self.game_at,
            started: self.started,
            closed: self.closed,
            first_team,
            second_team,
            events: serde_json::from_str(&self.events)?,
        };

        if let Some(players_str) = self.playing_players {
            let (first_team_players, second_team_players): (
                Vec<(i32, Player)>,
                Vec<(i32, Player)>,
            ) = serde_json::from_str(&players_str)?;

            game.first_team.players = first_team_players;
            game.second_team.players = second_team_players;
        } else if game.closed {
            let (first_team_players, second_team_players) =
                select_playing_players_for_game(state, &game).await?;

            game.first_team.players = first_team_players;
            game.second_team.players = second_team_players;
        }

        Ok(game)
    }
}

pub async fn select_all_scheduled(state: &AppState) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_all_scheduled");

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NULL
            ORDER BY bb_games.game_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_all_played(state: &AppState) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_all_played");

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NOT NULL
            ORDER BY bb_games.game_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_all_playing(state: &AppState) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_all_playing");

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NOT NULL
            ORDER BY bb_games.game_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_all_for_competition_stage(
    state: &AppState,
    stage_id: i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_all_for_competition_stage");

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_competitions_stages.id = $1
            ORDER BY bb_games.game_at ASC",
    )
    .bind(stage_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_played_by_team(
    state: &AppState,
    team_id: i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team_id);

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NOT NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
            ORDER BY bb_games.game_at DESC",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_scheduled_for_team(
    state: &AppState,
    team_id: i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_scheduled_for_team for team_id={:?}", team_id);

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
            ORDER BY bb_games.game_at ASC",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_playing_by_team(
    state: &AppState,
    team_id: i32,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_playing_by_team for team_id={:?}", team_id);

    let game_row: Option<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NOT NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
            ORDER BY bb_games.game_at
            LIMIT 1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(game_summary) = game_row {
        let game = game_summary.into_game_summary(state).await?;
        Ok(Some(game))
    } else {
        Ok(None)
    }
}

pub async fn select_scheduled_for_coach(
    state: &AppState,
    coach_id: &i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_scheduled_for_coach for coach_id={:?}", coach_id);

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NULL
            AND (bb_games.first_coach_id = $1 OR bb_games.second_coach_id = $1)
            ORDER BY bb_games.game_at ASC",
    )
    .bind(coach_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut games = Vec::with_capacity(game_rows.len());

    for game in game_rows {
        games.push(game.into_game_summary(state).await?);
    }

    Ok(games)
}

pub async fn select_playing_by_coach(
    state: &AppState,
    coach_id: i32,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_playing_by_coach for coach_id={:?}", coach_id);

    let game_row: Option<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.closed_at IS NULL
            AND bb_games.started_at IS NOT NULL
            AND (bb_games.first_coach_id = $1 OR bb_games.second_coach_id = $1)
            ORDER BY bb_games.game_at
            LIMIT 1",
    )
    .bind(coach_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(game_summary) = game_row {
        let game = game_summary.into_game_summary(state).await?;
        Ok(Some(game))
    } else {
        Ok(None)
    }
}

pub async fn is_last_for_team(
    state: &AppState,
    game_id: &i32,
    team_id: &i32,
) -> Result<bool, AppError> {
    tracing::debug!(
        "is_last_for_team for team_id={} with game_id={}",
        team_id,
        game_id
    );

    let last_game_id: Option<Id> = sqlx::query_as(
        "SELECT id
            FROM bb_games
            WHERE (first_team_id = $1 OR second_team_id = $1)
            AND started_at IS NOT NULL
            ORDER BY started_at DESC
            LIMIT 1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(last_game_id) = last_game_id {
        Ok(last_game_id.id.eq(game_id))
    } else {
        Ok(false)
    }
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let game_row: GameRow = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.id = $1
            LIMIT 1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let game = game_row.into_game(state).await?;

    Ok(game)
}

pub async fn select_summary_by_id(
    state: &AppState,
    id: i32,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_summary_by_id with id={}", id);

    let game_row: Option<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_competitions.name as competition_name,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages_schedule.round_name,
                    bb_games.version,
                    bb_games.created_by,
                    bb_games.game_at,
                    bb_games.started_at IS NOT NULL AS started,
                    bb_games.closed_at IS NOT NULL AS closed,
                    bb_games.first_team_id,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_team_id,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner,
                    bb_games.events,
                    bb_games.playing_players
            FROM bb_games
            LEFT JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            LEFT JOIN bb_competitions_stages
            ON bb_competitions_stages.id = bb_competitions_stages_schedule.stage_id
            LEFT JOIN bb_competitions
            ON bb_competitions.id = bb_competitions_stages_schedule.competition_id
            WHERE bb_games.id = $1
            LIMIT 1",
    )
    .bind(id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(game_row) = game_row {
        let game_summary = game_row.into_game_summary(state).await?;

        Ok(Some(game_summary))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameTeamPlayer {
    team_id: i32,
    player_id: Option<i32>,
    player_id_in_game: i32,
    player_number: i32,
    player_position: Position,
    player_roster: Roster,
    name: Option<String>,
}

impl GameTeamPlayer {
    fn into_player(self, game: &Game) -> (i32, Player) {
        (
            self.player_number,
            Player {
                id: self.player_id_in_game,
                version: game.version.clone(),
                position: self.player_position,
                roster: self.player_roster,
                name: self.name.unwrap_or("".to_string()),
                star_player_points: 0,
                player_type: self.player_position.player_type(&game.version),
                miss_next_game: false,
                advancements: Vec::new(),
                injuries: Vec::new(),
                hatred: Vec::new(),
                is_captain: false,
            },
        )
    }
}

pub async fn select_playing_players_for_game(
    state: &AppState,
    game: &Game,
) -> Result<(Vec<(i32, Player)>, Vec<(i32, Player)>), AppError> {
    tracing::debug!("select_playing_players_for_game with game_id={}", game.id);

    let game_teams_players: Vec<GameTeamPlayer> = sqlx::query_as(
        "SELECT bb_games_teams_players.team_id,
                    bb_games_teams_players.player_id,
                    bb_games_teams_players.player_id_in_game,
                    bb_games_teams_players.player_number,
                    bb_games_teams_players.player_position,
                    bb_teams.roster as player_roster,
                    bb_players.name
            FROM bb_games_teams_players
            INNER JOIN bb_teams
            ON bb_teams.id = bb_games_teams_players.team_id
            LEFT JOIN bb_players
            ON bb_players.id = bb_games_teams_players.player_id
            WHERE bb_games_teams_players.game_id = $1
            ORDER BY bb_games_teams_players.player_number",
    )
    .bind(game.id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut first_team_players: Vec<(i32, Player)> = vec![];
    let mut second_team_players: Vec<(i32, Player)> = vec![];

    for game_team_player in game_teams_players {
        if game_team_player.team_id.eq(&game.first_team.id) {
            first_team_players.push(game_team_player.into_player(&game));
        } else {
            second_team_players.push(game_team_player.into_player(&game));
        }
    }

    Ok((first_team_players, second_team_players))
}

pub async fn select_playing_team_player_for_game(
    state: &AppState,
    game: &Game,
    team_id: i32,
    player_id_in_game: i32,
) -> Result<Option<(i32, Player)>, AppError> {
    tracing::debug!(
        "select_playing_player_for_game with game_id={}, team_id={} and player_id_in_game={}",
        game.id,
        team_id,
        player_id_in_game,
    );

    let game_team_player: Option<GameTeamPlayer> = sqlx::query_as(
        "SELECT bb_games_teams_players.team_id,
                    bb_games_teams_players.player_id,
                    bb_games_teams_players.player_id_in_game,
                    bb_games_teams_players.player_number,
                    bb_games_teams_players.player_position,
                    bb_teams.roster as player_roster,
                    '' as name
            FROM bb_games_teams_players
            INNER JOIN bb_teams
            ON bb_teams.id = bb_games_teams_players.team_id
            WHERE bb_games_teams_players.game_id = $1
            AND bb_games_teams_players.team_id = $2
            AND bb_games_teams_players.player_id_in_game = $3
            LIMIT 1",
    )
    .bind(game.id.clone())
    .bind(team_id.clone())
    .bind(player_id_in_game.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(game_team_player) = game_team_player {
        if let Some(player_id) = game_team_player.player_id {
            players::select_by_id_for_team(&state, player_id, team_id).await
        } else {
            let (number, mut player) = game_team_player.into_player(&game);

            player.injuries = game.suffered_injuries(team_id, player_id_in_game);

            Ok(Some((number, player)))
        }
    } else {
        Ok(None)
    }
}

async fn can_be_saved(state: &AppState, profile: &User, game: &Game) -> Result<bool, AppError> {
    tracing::debug!(
        "can_be_updated by coach={:?} for game id={}",
        profile,
        game.id,
    );

    if game.first_team.coach.eq(&game.second_team.coach) {
        return Err(BloodBowlAppError(
            "Les deux équipes ont le même coach !".to_string(),
        ));
    }

    if profile.ne(&game.first_team.coach)
        && profile.ne(&game.second_team.coach)
        && profile.ne(&game.created_by)
    {
        return Err(BloodBowlAppError(
            "Vous n'êtes ni le créateur du match ni l'un des coachs !".to_string(),
        ));
    }

    if game.started {
        let other_playing_game: Option<Id> = sqlx::query_as(
            "SELECT id
            FROM bb_games
            WHERE started_at IS NOT NULL
            AND closed_at IS NULL
            AND (first_team_id = $2 OR second_team_id = $2 OR first_team_id = $3 OR second_team_id = $3)
            AND id <> $1
            LIMIT 1",
        )
            .bind(game.id.clone())
            .bind(game.first_team.id.clone())
            .bind(game.second_team.id.clone())
            .fetch_optional(&state.db)
            .await?;

        if other_playing_game.is_some() {
            return Err(BloodBowlAppError(
                "L'une des équipes est déjà en train de jouer un match !".to_string(),
            ));
        }
    }

    let game_played_after: Option<Id> = sqlx::query_as(
        "SELECT id
            FROM bb_games
            WHERE game_at > $2
            AND started_at IS NOT NULL
            AND (first_team_id = $3 OR second_team_id = $3 OR first_team_id = $4 OR second_team_id = $4)
            AND id <> $1
            LIMIT 1",
    )
        .bind(game.id.clone())
        .bind(game.game_at.clone())
        .bind(game.first_team.id.clone())
        .bind(game.second_team.id.clone())
        .fetch_optional(&state.db)
        .await?;

    if game_played_after.is_some() {
        return Err(BloodBowlAppError(
            "Ce match précède un autre déjà joué par l'une des équipes !".to_string(),
        ));
    }

    Ok(true)
}

pub async fn create_friendly(
    state: &AppState,
    profile: &User,
    first_team: &Team,
    second_team: &Team,
    game_at: NaiveDateTime,
) -> Result<i32, AppError> {
    create(
        state,
        profile,
        first_team,
        second_team,
        game_at,
        None,
        None,
        None,
    )
    .await
}

pub async fn create_for_competition_stage_round(
    state: &AppState,
    profile: &User,
    competition_id: i32,
    stage_id: i32,
    round_schedule: RoundSchedule,
    scheduled_at: NaiveDateTime,
) -> Result<(), AppError> {
    let games_to_create = round_schedule.games_that_can_be_created();

    for game_schedule in games_to_create {
        if let (Some(home_team), Some(away_team)) =
            (game_schedule.home_team, game_schedule.away_team)
        {
            let first_team =
                teams::select_by_id_without_staff_nor_players(state, home_team.id).await?;

            let second_team =
                teams::select_by_id_without_staff_nor_players(state, away_team.id).await?;

            create(
                state,
                profile,
                &first_team,
                &second_team,
                scheduled_at,
                Some(competition_id),
                Some(stage_id),
                Some(round_schedule.name.clone()),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn create(
    state: &AppState,
    profile: &User,
    first_team: &Team,
    second_team: &Team,
    game_at: NaiveDateTime,
    competition_id: Option<i32>,
    stage_id: Option<i32>,
    round_name: Option<String>,
) -> Result<i32, AppError> {
    tracing::debug!(
        "create by coach={:?} to play at {} for the following teams: team_a_id={} and team_b_id={} on behalf of competition_id={:?} and stage_id={:?} and round_name={:?}",
        profile,
        game_at,
        first_team.id,
        second_team.id,
        competition_id,
        stage_id,
        round_name,
    );

    let game = Game::create(
        -1,
        Some(profile.clone().into()),
        first_team.version,
        game_at,
        &first_team,
        &second_team,
    )?;

    let _ = can_be_saved(state, profile, &game).await?;

    let mut transaction = state.db.begin().await?;

    let mut new_game_id: Id = sqlx::query_as(
        "INSERT INTO bb_games (
                version,
                created_by,
                game_at,
                first_coach_id,
                first_team_id,
                second_coach_id,
                second_team_id,
                events)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id",
    )
    .bind(game.version.clone())
    .bind(profile.id.clone())
    .bind(game.game_at.clone())
    .bind(first_team.coach.id.unwrap_or_default().clone())
    .bind(first_team.id.clone())
    .bind(second_team.coach.id.unwrap_or_default().clone())
    .bind(second_team.id.clone())
    .bind(serde_json::to_string(&game.events)?)
    .fetch_one(&mut *transaction)
    .await?;

    if let (Some(competition_id), Some(stage_id), Some(round_name)) =
        (competition_id, stage_id, round_name)
    {
        new_game_id = sqlx::query_as(
            "INSERT INTO bb_competitions_stages_schedule (
                    competition_id,
                    stage_id,
                    game_id,
                    round_name)
                VALUES ($1, $2, $3, $4)
                RETURNING game_id as id",
        )
        .bind(competition_id.clone())
        .bind(stage_id.clone())
        .bind(new_game_id.id.clone())
        .bind(round_name.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE bb_competitions
                SET last_updated = CURRENT_TIMESTAMP,
                    started_at = (
                        SELECT MIN(bb_games.game_at)
                        FROM bb_competitions_stages_schedule
                        INNER JOIN bb_games
                        ON bb_games.id = bb_competitions_stages_schedule.game_id
                        WHERE bb_competitions_stages_schedule.competition_id = bb_competitions.id
                    )
                WHERE bb_competitions.id = $1",
        )
        .bind(competition_id.clone())
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(new_game_id.id)
}

pub async fn update_schedule(
    state: &AppState,
    profile: &User,
    game: &Game,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_schedule by coach_id={:?} for game_id {}",
        profile.id,
        game.id
    );

    let _ = can_be_saved(state, profile, &game).await?;

    sqlx::query(
        "UPDATE bb_games
            SET game_at = $3
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .bind(game.game_at.clone())
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn update_start(state: &AppState, profile: &User, game: &Game) -> Result<(), AppError> {
    tracing::debug!(
        "update_start by coach_id={:?} for game_id {}",
        profile.id,
        game.id
    );

    let _ = can_be_saved(state, profile, &game).await?;

    sqlx::query(
        "UPDATE bb_games
            SET game_at = $3,
                started_at = CURRENT_TIMESTAMP,
                playing_players = $4
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .bind(game.game_at.clone())
    .bind(serde_json::to_string(&game.playing_players())?)
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn cancel_start(state: &AppState, profile: &User, game: &Game) -> Result<(), AppError> {
    tracing::debug!(
        "cancel_start by coach_id={:?} for game_id {}",
        profile.id,
        game.id
    );

    let _ = can_be_saved(state, profile, &game).await?;

    sqlx::query(
        "UPDATE bb_games
            SET started_at = NULL,
                playing_players = NULL
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn update_after_event(
    state: &AppState,
    profile: &User,
    game: &Game,
    event: &GameEvent,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_after_event by coach_id={:?} for game_id {}",
        profile.id,
        game.id
    );

    if matches!(event, GameEvent::GameEnd | GameEvent::GameClosure) {
        if let Some(competition_stage) =
            CompetitionStage::select_for_game_id(state, game.id).await?
        {
            if matches!(competition_stage.stage_type, CompetitionStageType::Cup)
                && game.winning_team().is_none()
            {
                return Err(BloodBowlAppError(String::from(
                    "Le match doit avoir un gagnant",
                )));
            }
        }
    }

    let _ = can_be_saved(state, profile, &game).await?;

    let score = game.score();
    let casualties = game.casualties();
    let winner = game.winner();

    let mut transaction = state.db.begin().await?;

    sqlx::query(
        "UPDATE bb_games
            SET events = $3,
                playing_players = $4,
                first_team_score = $5,
                first_team_casualties = $6,
                first_team_is_winner = $7,
                second_team_score = $8,
                second_team_casualties = $9,
                second_team_is_winner = $10
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .bind(serde_json::to_string(&game.events)?)
    .bind(serde_json::to_string(&game.playing_players())?)
    .bind(score.0.clone() as i32)
    .bind(casualties.0.clone() as i32)
    .bind(winner.0.clone())
    .bind(score.1.clone() as i32)
    .bind(casualties.1.clone() as i32)
    .bind(winner.1.clone())
    .execute(&mut *transaction)
    .await?;

    let need_teams_update = match event {
        GameEvent::FanFactor { .. } => false,
        GameEvent::Weather(_) => false,
        GameEvent::Journeyman { .. } => false,
        GameEvent::BuyInducement { .. } => true,
        GameEvent::PrayerToNuffle { .. } => false,
        GameEvent::TossWinner { .. } => false,
        GameEvent::KickingTeam { .. } => false,
        GameEvent::TurnStart { .. } => false,
        GameEvent::Success { .. } => false,
        GameEvent::Injury { .. } => false,
        GameEvent::Hatred { .. } => false,
        GameEvent::GameEnd => false,
        GameEvent::Winnings { .. } => true,
        GameEvent::DedicatedFansUpdate { .. } => true,
        GameEvent::ExpensiveMistakes { .. } => true,
        GameEvent::GameClosure { .. } => false,
    };

    if need_teams_update {
        sqlx::query(
            "UPDATE bb_teams
                SET treasury = $2,
                    dedicated_fans = $3
                WHERE id = $1",
        )
        .bind(game.first_team.id.clone())
        .bind(game.first_team.treasury.clone())
        .bind(game.first_team.dedicated_fans.clone() as i32)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE bb_teams
                SET treasury = $2,
                    dedicated_fans = $3
                WHERE id = $1",
        )
        .bind(game.second_team.id.clone())
        .bind(game.second_team.treasury.clone())
        .bind(game.second_team.dedicated_fans.clone() as i32)
        .execute(&mut *transaction)
        .await?;
    }

    let need_players_update = match event {
        GameEvent::FanFactor { .. } => false,
        GameEvent::Weather(_) => false,
        GameEvent::Journeyman { .. } => true,
        GameEvent::BuyInducement { .. } => true,
        GameEvent::PrayerToNuffle { .. } => false,
        GameEvent::TossWinner { .. } => false,
        GameEvent::KickingTeam { .. } => false,
        GameEvent::TurnStart { .. } => false,
        GameEvent::Success { .. } => true,
        GameEvent::Injury { .. } => true,
        GameEvent::Hatred { .. } => true,
        GameEvent::GameEnd => false,
        GameEvent::Winnings { .. } => false,
        GameEvent::DedicatedFansUpdate { .. } => false,
        GameEvent::ExpensiveMistakes { .. } => false,
        GameEvent::GameClosure { .. } => false,
    };

    if need_players_update {
        sqlx::query(
            "DELETE
                FROM bb_players_injuries
                USING bb_games
                WHERE bb_games.id = bb_players_injuries.game_id
                AND bb_games.id = $1
                AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
        )
            .bind(game.id.clone())
            .bind(profile.id.unwrap_or(-1).clone())
            .execute(&mut *transaction)
            .await?;

        for event in game.events.iter() {
            if let GameEvent::Injury {
                player_id, injury, ..
            } = event
            {
                if injury.remains_after_game() {
                    sqlx::query(
                        "INSERT INTO bb_players_injuries (
                                player_id,
                                game_id,
                                injury)
                            VALUES ($1, $2, $3)",
                    )
                    .bind(player_id.clone())
                    .bind(game.id.clone())
                    .bind(injury.clone())
                    .execute(&mut *transaction)
                    .await?;
                }
            }
        }
        sqlx::query(
            "DELETE
                FROM bb_players_hatred
                USING bb_games
                WHERE bb_games.id = bb_players_hatred.game_id
                AND bb_games.id = $1
                AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
        )
            .bind(game.id.clone())
            .bind(profile.id.unwrap_or(-1).clone())
            .execute(&mut *transaction)
            .await?;

        for event in game.events.iter() {
            if let GameEvent::Hatred {
                player_id, keyword, ..
            } = event
            {
                sqlx::query(
                    "INSERT INTO bb_players_hatred (
                            player_id,
                            game_id,
                            keyword)
                        VALUES ($1, $2, $3)",
                )
                .bind(player_id.clone())
                .bind(game.id.clone())
                .bind(keyword.clone())
                .execute(&mut *transaction)
                .await?;
            }
        }

        sqlx::query(
            "DELETE
                FROM bb_games_teams_players
                USING bb_games
                WHERE bb_games.id = bb_games_teams_players.game_id
                AND bb_games.id = $1
                AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
        )
            .bind(game.id.clone())
            .bind(profile.id.unwrap_or(-1).clone())
            .execute(&mut *transaction)
            .await?;

        for (team_id, team_players) in vec![
            (game.first_team.id, game.playing_players().0),
            (game.second_team.id, game.playing_players().1),
        ] {
            for (number, player) in team_players {
                let statistics = game.player_statistics(team_id.clone(), player.id.clone());

                let player_id = if matches!(player.player_type, PlayerType::FromRoster) {
                    Some(player.id.clone())
                } else {
                    None
                };

                sqlx::query(
                    "INSERT INTO bb_games_teams_players (
                            game_id,
                            team_id,
                            player_id,
                            player_id_in_game,
                            player_number,
                            player_position,
                            passing_completions,
                            throwing_completions,
                            deflections,
                            interceptions,
                            casualties,
                            touchdowns,
                            most_valuable_player,
                            star_player_points)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
                )
                .bind(game.id.clone())
                .bind(team_id.clone())
                .bind(player_id.clone())
                .bind(player.id.clone())
                .bind(number.clone())
                .bind(player.position.clone())
                .bind(statistics.passing_completions.clone() as i32)
                .bind(statistics.throwing_completions.clone() as i32)
                .bind(statistics.deflections.clone() as i32)
                .bind(statistics.interceptions.clone() as i32)
                .bind(statistics.casualties.clone() as i32)
                .bind(statistics.touchdowns.clone() as i32)
                .bind(statistics.most_valuable_player.clone() as i32)
                .bind(statistics.star_player_points.clone() as i32)
                .execute(&mut *transaction)
                .await?;
            }
        }
    }

    if matches!(event, GameEvent::GameClosure) {
        sqlx::query(
            "UPDATE bb_games
            SET closed_at = CURRENT_TIMESTAMP,
                playing_players = NULL
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
        )
        .bind(game.id.clone())
        .bind(profile.id.unwrap_or(-1).clone())
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    teams::update_values(state, profile, game.first_team.id).await?;
    teams::update_values(state, profile, game.second_team.id).await?;

    Ok(())
}

pub async fn update_number_for_added_player_in_game(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    player_id_in_game: i32,
    game_id: i32,
    number: i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_number_for_added_player_in_game by user={:?} for team_id={}, player_id_in_game={} and game_id={} with number={}",
        connected_user,
        team_id,
        player_id_in_game,
        game_id,
        number,
    );

    if let Some(connected_user_id) = connected_user.id {
        let mut game = select_by_id(state, game_id).await?;

        if team_id.eq(&game.first_team.id) {
            game.first_team
                .update_player_number(player_id_in_game, number);
        }

        if team_id.eq(&game.second_team.id) {
            game.second_team
                .update_player_number(player_id_in_game, number);
        }

        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "UPDATE bb_games
            SET playing_players = $3
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
        )
        .bind(game.id.clone())
        .bind(connected_user_id.clone())
        .bind(serde_json::to_string(&game.playing_players())?)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE bb_games_teams_players
                SET player_number = $1
                FROM bb_games
                WHERE bb_games.id = bb_games_teams_players.game_id
                AND (bb_games.created_by = $4 OR bb_games.first_coach_id = $4 OR bb_games.second_coach_id = $4)
                AND bb_games.id = $5
                AND bb_games_teams_players.player_id_in_game = $2
                AND bb_games_teams_players.team_id = $3",
        )
            .bind(number.clone())
            .bind(player_id_in_game.clone())
            .bind(team_id.clone())
            .bind(connected_user_id.clone())
            .bind(game_id.clone())
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn delete(state: &AppState, profile: &User, game_id: i32) -> Result<(), AppError> {
    tracing::debug!(
        "delete by coach_id={:?} for game id {}",
        profile.id,
        game_id
    );

    let mut game = select_by_id(state, game_id).await?;

    if profile.ne(&game.created_by) {
        return Err(BloodBowlAppError(
            "Seul le créateur du match peut supprimer !".to_string(),
        ));
    }

    if game.closed {
        return Err(BloodBowlAppError(
            "Impossible de supprimer un match déjà clôturé !".to_string(),
        ));
    }

    let mut transaction = state.db.begin().await?;

    for _ in 0..game.events.len() {
        let cancelled_event = game.cancel_last_event()?;

        if let Some(event) = cancelled_event {
            update_after_event(state, profile, &game, &event).await?;
        }
    }

    sqlx::query(
        "DELETE
            FROM bb_players_injuries
            USING bb_games
            WHERE bb_games.id = bb_players_injuries.game_id
            AND bb_games.id = $1
            AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
    )
        .bind(game.id.clone())
        .bind(profile.id.unwrap_or(-1).clone())
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "DELETE
            FROM bb_games_teams_players
            USING bb_games
            WHERE bb_games.id = bb_games_teams_players.game_id
            AND bb_games.id = $1
            AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
    )
        .bind(game.id.clone())
        .bind(profile.id.unwrap_or(-1).clone())
        .execute(&mut *transaction)
        .await?;

    let competition_id: Option<Id> = sqlx::query_as(
        "DELETE
            FROM bb_competitions_stages_schedule
            WHERE game_id = $1
            RETURNING competition_id as id",
    )
    .bind(game.id.clone())
    .fetch_optional(&mut *transaction)
    .await?;

    if let Some(competition_id) = competition_id {
        sqlx::query(
            "UPDATE bb_competitions
                    SET last_updated = CURRENT_TIMESTAMP,
                        started_at = (
                            SELECT MIN(bb_games.game_at)
                            FROM bb_competitions_stages_schedule
                            INNER JOIN bb_games
                            ON bb_games.id = bb_competitions_stages_schedule.game_id
                            WHERE bb_competitions_stages_schedule.competition_id = bb_competitions.id
                        )
                    WHERE bb_competitions.id = $1",
        )
            .bind(competition_id.id.clone())
            .execute(&mut *transaction)
            .await?;
    }

    sqlx::query(
        "DELETE
            FROM bb_games
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
