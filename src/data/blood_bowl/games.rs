use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::blood_bowl::{coaches, players, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::errors::AppError::BloodBowlAppError;
use crate::AppState;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::players::Player;
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

        let mut first_team_playing_players = vec![];
        let mut second_team_playing_players = vec![];

        if self.started_at.is_some() {
            first_team_playing_players =
                select_playing_players_in_game_for_team(state, self.id, first_team.id.clone())
                    .await?;

            second_team_playing_players =
                select_playing_players_in_game_for_team(state, self.id, second_team.id.clone())
                    .await?;
        }

        let game = Game {
            id: self.id,
            version: self.version,
            created_by,
            closed_at: self.closed_at,
            scheduled_at: self.scheduled_at,
            started_at: self.started_at,
            first_team,
            second_team,
            first_team_playing_players,
            second_team_playing_players,
            events: vec![],
        };

        Ok(game)
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

pub async fn can_create_or_start_for_team(
    state: &AppState,
    coach: &User,
    game_id: &Option<i32>,
    game_date: &NaiveDateTime,
    team: &Team,
) -> Result<bool, AppError> {
    tracing::debug!(
        "can_create_or_start_for_team by coach={:?} to play at {} for the following team: team_id={}",
        coach,
        game_date,
        team.id,
    );

    let game_played_after: Option<Id> = sqlx::query_as(
        "SELECT id
            FROM bb_games
            WHERE (started_at > $2 OR closed_at > $2)
            AND (first_team_id = $1 OR second_team_id = $1)
            AND id <> $3
            LIMIT 1",
    )
    .bind(team.id.clone())
    .bind(game_date.clone())
    .bind(game_id.unwrap_or(-1).clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(game_played_after.is_none())
}

pub async fn create(
    state: &AppState,
    coach: &User,
    first_team: &Team,
    second_team: &Team,
    scheduled_at: NaiveDateTime,
) -> Result<i32, AppError> {
    tracing::debug!(
        "create by coach={:?} to play at {} for the following teams: team_a_id={} and team_b_id={}",
        coach,
        scheduled_at,
        first_team.id,
        second_team.id,
    );

    if !can_create_or_start_for_team(state, coach, &None, &scheduled_at, &first_team).await?
        || !can_create_or_start_for_team(state, coach, &None, &scheduled_at, &second_team).await?
    {
        return Err(BloodBowlAppError(
            "Impossible de créer ou démarrer un match avant un autre déjà joué ou en cours !"
                .to_string(),
        ));
    }

    let game = Game::create(
        -1,
        Some(coach.clone().into()),
        first_team.version,
        scheduled_at,
        &first_team,
        &second_team,
    )?;

    let new_game_id: Id = sqlx::query_as(
        "INSERT INTO bb_games (
                version,
                created_by,
                scheduled_at,
                first_coach_id,
                first_team_id,
                second_coach_id,
                second_team_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id",
    )
    .bind(game.version.clone())
    .bind(coach.id.clone())
    .bind(game.scheduled_at.clone())
    .bind(first_team.coach.id.unwrap_or_default().clone())
    .bind(first_team.id.clone())
    .bind(second_team.coach.id.unwrap_or_default().clone())
    .bind(second_team.id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(new_game_id.id)
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
                    second_team_is_winner
            FROM bb_games
            WHERE closed_at IS NOT NULL
            AND (first_team_id = $1 OR second_team_id = $1)
            ORDER BY closed_at ASC",
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
                    second_team_is_winner
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
                    second_team_is_winner
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

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerIdWithNumber {
    player_id: i32,
    player_number: i32,
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
                    player_number
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
        let player_number = player_id_with_number.player_number;
        let player = players::select_by_id(state, player_id_with_number.player_id).await?;

        players.push((player_number, player));
    }

    Ok(players)
}

pub async fn update(state: &AppState, profile: &User, game: Game) -> Result<(), AppError> {
    tracing::debug!(
        "update by coach_id={:?} for game_id {}",
        profile.id,
        game.id
    );

    if profile.ne(&game.first_team.coach)
        && profile.ne(&game.second_team.coach)
        && profile.ne(&game.created_by)
    {
        return Err(BloodBowlAppError(
            "Seuls les coachs des équipes ou le créateur du match peuvent mettre à jour !"
                .to_string(),
        ));
    }

    let game_date = game.started_at.unwrap_or(game.scheduled_at);

    if !can_create_or_start_for_team(state, profile, &Some(game.id), &game_date, &game.first_team)
        .await?
        || !can_create_or_start_for_team(
            state,
            profile,
            &Some(game.id),
            &game_date,
            &game.second_team,
        )
        .await?
    {
        return Err(BloodBowlAppError("Impossible de créer, modifier ou démarrer un match précédant un autre déjà joué ou en cours !".to_string()));
    }

    let mut transaction = state.db.begin().await?;

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
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        "DELETE
            FROM bb_games_events
            USING bb_games
            WHERE bb_games.id = bb_games_events.game_id
            AND bb_games.id = $1
            AND (bb_games.created_by = $2 OR bb_games.first_coach_id = $2 OR bb_games.second_coach_id = $2)",
    )
        .bind(game.id.clone())
        .bind(profile.id.unwrap_or(-1).clone())
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

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
            FROM bb_games_events
            USING bb_games
            WHERE bb_games.id = bb_games_events.game_id
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
