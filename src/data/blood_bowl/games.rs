use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::blood_bowl::{coaches, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::errors::AppError::BloodBowlAppError;
use crate::AppState;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::versions::Version;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct GameSummary {
    pub id: i32,
    pub scheduled_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub closed_at: Option<NaiveDateTime>,
    pub first_team: TeamSummary,
    pub first_team_score: i32,
    pub first_team_casualties: i32,
    pub first_team_is_winner: bool,
    pub second_team: TeamSummary,
    pub second_team_score: i32,
    pub second_team_casualties: i32,
    pub second_team_is_winner: bool,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameRow {
    id: i32,
    version: Version,
    created_by: Option<i32>,
    scheduled_at: NaiveDateTime,
    started_at: Option<NaiveDateTime>,
    closed_at: Option<NaiveDateTime>,
    first_team_id: i32,
    first_team_score: i32,
    first_team_casualties: i32,
    first_team_is_winner: bool,
    second_team_id: i32,
    second_team_score: i32,
    second_team_casualties: i32,
    second_team_is_winner: bool,
    events: String,
}

impl GameRow {
    async fn into_game_summary(self, state: &AppState) -> Result<GameSummary, AppError> {
        let first_team = teams::select_summary_by_id(state, self.first_team_id).await?;
        let second_team = teams::select_summary_by_id(state, self.second_team_id).await?;

        let game_summary = GameSummary {
            id: self.id,
            scheduled_at: self.scheduled_at,
            started_at: self.started_at,
            closed_at: self.closed_at,
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

        let first_team = teams::select_by_id(state, self.first_team_id).await?;
        let second_team = teams::select_by_id(state, self.second_team_id).await?;

        let game = Game {
            id: self.id,
            version: self.version,
            created_by,
            closed_at: self.closed_at,
            scheduled_at: self.scheduled_at,
            started_at: self.started_at,
            first_team,
            second_team,
            events: serde_json::from_str(&self.events)?,
        };

        Ok(game)
    }
}

pub async fn select_all_played(state: &AppState) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_all_played");

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE closed_at IS NOT NULL
            ORDER BY started_at DESC",
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
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE closed_at IS NULL
            AND started_at IS NOT NULL
            ORDER BY started_at ASC",
    )
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
    team_id: &i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team_id);

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE closed_at IS NOT NULL
            AND (first_team_id = $1 OR second_team_id = $1)
            ORDER BY closed_at DESC",
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
    team_id: &i32,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_scheduled_for_team for team_id={:?}", team_id);

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE closed_at IS NULL
            AND started_at IS NULL
            AND (first_team_id = $1 OR second_team_id = $1)
            ORDER BY scheduled_at ASC",
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
    team_id: &i32,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_playing_by_team for team_id={:?}", team_id);

    let game_row: Option<GameRow> = sqlx::query_as(
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE closed_at IS NULL
            AND started_at IS NOT NULL
            AND (first_team_id = $1 OR second_team_id = $1)
            ORDER BY started_at
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

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let game_row: GameRow = sqlx::query_as(
        "SELECT id,
                    version,
                    created_by,
                    scheduled_at,
                    started_at,
                    closed_at,
                    first_team_id,
                    first_team_score,
                    first_team_casualties,
                    first_team_is_winner,
                    second_team_id,
                    second_team_score,
                    second_team_casualties,
                    second_team_is_winner,
                    events
            FROM bb_games
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let game = game_row.into_game(state).await?;

    Ok(game)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
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

    if game.started_at.is_some() {
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
            WHERE started_at > $2
            AND (first_team_id = $3 OR second_team_id = $3 OR first_team_id = $4 OR second_team_id = $4)
            AND id <> $1
            LIMIT 1",
    )
        .bind(game.id.clone())
        .bind(game.started_at.unwrap_or(game.scheduled_at).clone())
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

pub async fn create(
    state: &AppState,
    profile: &User,
    first_team: &Team,
    second_team: &Team,
    scheduled_at: NaiveDateTime,
) -> Result<i32, AppError> {
    tracing::debug!(
        "create by coach={:?} to play at {} for the following teams: team_a_id={} and team_b_id={}",
        profile,
        scheduled_at,
        first_team.id,
        second_team.id,
    );

    let game = Game::create(
        -1,
        Some(profile.clone().into()),
        first_team.version,
        scheduled_at,
        &first_team,
        &second_team,
    )?;

    let _ = can_be_saved(state, profile, &game).await?;

    let new_game_id: Id = sqlx::query_as(
        "INSERT INTO bb_games (
                version,
                created_by,
                scheduled_at,
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
    .bind(game.scheduled_at.clone())
    .bind(first_team.coach.id.unwrap_or_default().clone())
    .bind(first_team.id.clone())
    .bind(second_team.coach.id.unwrap_or_default().clone())
    .bind(second_team.id.clone())
    .bind(serde_json::to_string(&game.events)?)
    .fetch_one(&state.db)
    .await?;

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
            SET scheduled_at = $3
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .bind(game.scheduled_at.clone())
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

    sqlx::query(
        "UPDATE bb_games
            SET scheduled_at = $3,
                started_at = $4
            WHERE id = $1
            AND (created_by = $2 OR first_coach_id = $2 OR second_coach_id = $2)",
    )
    .bind(game.id.clone())
    .bind(profile.id.unwrap_or(-1).clone())
    .bind(game.scheduled_at.clone())
    .bind(game.started_at.clone())
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn delete(state: &AppState, profile: &User, game_id: i32) -> Result<(), AppError> {
    tracing::debug!(
        "delete by coach_id={:?} for game id {}",
        profile.id,
        game_id
    );

    let game = select_by_id(state, game_id).await?;

    if profile.ne(&game.first_team.coach)
        && profile.ne(&game.second_team.coach)
        && profile.ne(&game.created_by)
    {
        return Err(BloodBowlAppError(
            "Seuls les coachs des équipes ou le créateur du match peuvent supprimer !".to_string(),
        ));
    }

    if game.closed_at.is_some() {
        return Err(BloodBowlAppError(
            "Impossible de supprimer un match déjà clôturé !".to_string(),
        ));
    }

    let mut transaction = state.db.begin().await?;

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
