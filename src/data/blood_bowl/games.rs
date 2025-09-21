use crate::auth::profile;
use crate::data::blood_bowl::{coaches, players, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::games::{Game, GameSummary};
use blood_bowl_rs::players::Player;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::{Team, TeamSummary};
use blood_bowl_rs::versions::Version;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

pub async fn create(
    state: &AppState,
    coach: &User,
    first_team: &Team,
    second_team: &Team,
    played_at: NaiveDateTime,
) -> Result<i32, AppError> {
    tracing::debug!(
        "create by coach={:?} to play at {} for the following teams: team_a_id={} and team_b_id={}",
        coach,
        played_at,
        first_team.id,
        second_team.id,
    );

    let game = Game::create(
        -1,
        Some(coach.clone().into()),
        first_team.version,
        played_at,
        &first_team,
        &second_team,
    )?;

    let first_team_json_summary = serde_json::to_string(&TeamSummary::from(first_team))?;
    let second_team_json_summary = serde_json::to_string(&TeamSummary::from(first_team))?;

    let new_game_id: Id = sqlx::query_as(
        "INSERT INTO bb_games (
                version,
                played_at,
                created_by,
                first_coach_id,
                first_team_id,
                first_team_json_summary,
                second_coach_id,
                second_team_id,
                second_team_json_summary)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id",
    )
    .bind(game.version.clone())
    .bind(game.played_at.clone())
    .bind(coach.id.clone())
    .bind(first_team.coach.id.unwrap_or_default().clone())
    .bind(first_team.id.clone())
    .bind(first_team_json_summary.clone())
    .bind(second_team.coach.id.unwrap_or_default().clone())
    .bind(second_team.id.clone())
    .bind(second_team_json_summary.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(new_game_id.id)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameRow {
    id: i32,
    version: Version,
    played_at: NaiveDateTime,
    created_by: Option<i32>,
    closed_at: Option<NaiveDateTime>,
    first_coach_id: Option<i32>,
    first_team_id: Option<i32>,
    first_team_json_summary: String,
    first_team_score: i32,
    first_team_casualties: i32,
    first_team_is_winner: bool,
    second_coach_id: Option<i32>,
    second_team_id: Option<i32>,
    second_team_json_summary: String,
    second_team_score: i32,
    second_team_casualties: i32,
    second_team_is_winner: bool,
}

impl GameRow {
    pub async fn into_game(self, state: &AppState) -> Result<Game, AppError> {
        let mut created_by = None;

        if let Some(coach_id) = self.created_by {
            created_by = coaches::select_by_id(state, Some(coach_id)).await?;
        }

        let first_team_summary: TeamSummary = serde_json::from_str(&*self.first_team_json_summary)?;
        let mut first_team = Team::from(&first_team_summary);
        if let Some(team_id) = self.first_team_id {
            if let Ok(team) = teams::select_by_id(state, team_id).await {
                first_team = team;
            }
        }

        let second_team_summary: TeamSummary =
            serde_json::from_str(&*self.second_team_json_summary)?;
        let mut second_team = Team::from(&second_team_summary);
        if let Some(team_id) = self.second_team_id {
            if let Ok(team) = teams::select_by_id(state, team_id).await {
                second_team = team;
            }
        }

        let game = Game {
            id: self.id,
            version: self.version,
            created_by,
            played_at: self.played_at,
            closed_at: self.closed_at,
            first_team,
            second_team,
            first_team_playing_players: vec![],
            second_team_playing_players: vec![],
            events: vec![],
        };

        Ok(game)
    }

    pub async fn into_game_summary(self, state: &AppState) -> Result<GameSummary, AppError> {
        let mut created_by = None;

        if let Some(coach_id) = self.created_by {
            created_by = coaches::select_by_id(state, Some(coach_id)).await?;
        }

        let first_team_summary = serde_json::from_str(&*self.first_team_json_summary)?;
        let second_team_summary = serde_json::from_str(&*self.second_team_json_summary)?;

        let game_summary = GameSummary {
            id: self.id,
            version: self.version,
            created_by,
            played_at: self.played_at,
            closed_at: self.closed_at,
            first_team: first_team_summary,
            second_team: second_team_summary,
            first_team_score: self.first_team_score as usize,
            second_team_score: self.second_team_score as usize,
            first_team_casualties: self.first_team_casualties as usize,
            second_team_casualties: self.second_team_casualties as usize,
        };

        Ok(game_summary)
    }
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let game_row: GameRow = sqlx::query_as(
        "SELECT id,
                    version,
                    played_at,
                    created_by,
                    closed_at,
                    first_coach_id,
                    first_team_id,
                    first_team_json_summary,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_coach_id,
                    second_team_id,
                    second_team_json_summary,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner
            FROM bb_games
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let game = game_row.into_game(state).await?;

    Ok(game)
}

pub async fn select_played_by_team(
    state: &AppState,
    team: &Team,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team.id);

    let mut game_list: Vec<GameSummary> = Vec::new();

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT id,
                    version,
                    played_at,
                    created_by,
                    closed_at,
                    first_coach_id,
                    first_team_id,
                    first_team_json_summary,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_coach_id,
                    second_team_id,
                    second_team_json_summary,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner
            FROM bb_games
            WHERE closed_at IS NOT NULL
            AND (first_team_id = $1 OR second_team_id = $1)
            ORDER BY played_at ASC",
    )
    .bind(team.id.clone())
    .fetch_all(&state.db)
    .await?;

    for game_row in game_rows {
        game_list.push(game_row.into_game_summary(state).await?)
    }

    Ok(game_list)
}

pub async fn select_scheduled_for_team(
    state: &AppState,
    team: &Team,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team.id);

    let mut game_list: Vec<GameSummary> = Vec::new();

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.created_by,
                    bb_games.closed_at,
                    bb_games.first_coach_id,
                    bb_games.first_team_id,
                    bb_games.first_team_json_summary,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_coach_id,
                    bb_games.second_team_id,
                    bb_games.second_team_json_summary,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner
            FROM bb_games
            LEFT JOIN bb_games_events
            ON bb_games_events.game_id = bb_games.id
            WHERE bb_games.closed_at IS NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
            AND bb_games_events.event IS NULL
            ORDER BY played_at ASC",
    )
    .bind(team.id.clone())
    .fetch_all(&state.db)
    .await?;

    for game_row in game_rows {
        game_list.push(game_row.into_game_summary(state).await?)
    }

    Ok(game_list)
}

pub async fn select_playing_by_team(
    state: &AppState,
    team: &Team,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_playing_by_team for team_id={:?}", team.id);

    let game_row: Option<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.created_by,
                    bb_games.closed_at,
                    bb_games.first_coach_id,
                    bb_games.first_team_id,
                    bb_games.first_team_json_summary,
                    bb_games.first_team_score,
                    bb_games.first_team_casualties,
                    bb_games.first_team_is_winner,
                    bb_games.second_coach_id,
                    bb_games.second_team_id,
                    bb_games.second_team_json_summary,
                    bb_games.second_team_score,
                    bb_games.second_team_casualties,
                    bb_games.second_team_is_winner
            FROM bb_games
            INNER JOIN bb_games_events
            ON bb_games_events.game_id = bb_games.id
            WHERE bb_games.closed_at IS NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
            LIMIT 1",
    )
    .bind(team.id.clone())
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
struct PlayerIdWithNumber {
    player_id: i32,
    number: i32,
}

pub async fn select_playing_players_in_game_for_team(
    state: &AppState,
    game_id: i32,
    team_id: i32,
) -> Result<Vec<(i32, Player)>, AppError> {
    tracing::debug!(
        "select_playing_players_in_game_for_team with game_id={} and team_id={}",
        game_id,
        team_id
    );

    let players_id_with_number: Vec<PlayerIdWithNumber> = sqlx::query_as(
        "SELECT player_id,
                    number
            FROM bb_games_teams_players
            WHERE game_id = $1
            AND team_id = $2",
    )
    .bind(game_id.clone())
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut players: Vec<(i32, Player)> = Vec::with_capacity(players_id_with_number.len());

    for player_id_with_number in players_id_with_number {
        let number = player_id_with_number.number;
        let player = players::select_by_id(state, player_id_with_number.player_id).await?;

        players.push((number, player));
    }

    Ok(players)
}
