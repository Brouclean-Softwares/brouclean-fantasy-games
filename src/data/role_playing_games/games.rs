use crate::AppState;
use crate::data::users::User;
use crate::errors::AppError;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct Game {
    pub id: i32,
    pub name: String,
    pub external_logo_url: Option<String>,
    pub description: String,
}

pub async fn select_all(state: &AppState) -> Result<Vec<Game>, AppError> {
    tracing::debug!("select_all");

    let games: Vec<Game> = sqlx::query_as(
        "SELECT id, 
                    name,
                    external_logo_url,
                    description
            FROM rpg_games
            ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(games)
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let game: Game = sqlx::query_as(
        "SELECT id, 
                    name,
                    external_logo_url,
                    description
            FROM rpg_games
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(game)
}

pub async fn create(state: &AppState, user: &User, game: &Game) -> Result<i32, AppError> {
    tracing::debug!("create game={:?} by user={:?}", game, user,);

    let new_game_id: i32 = sqlx::query_scalar(
        "INSERT INTO rpg_games (name)
            VALUES ($1)
            RETURNING id",
    )
    .bind(game.name.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(new_game_id)
}

pub async fn update(state: &AppState, user: &User, game: &Game) -> Result<(), AppError> {
    tracing::debug!("update game with id={} by user={:?}", game.id, user);

    sqlx::query(
        "UPDATE rpg_games
            SET name = $2,
                external_logo_url = $3,
                description = $4,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $1",
    )
    .bind(game.id.clone())
    .bind(game.name.clone())
    .bind(game.external_logo_url.clone())
    .bind(game.description.clone())
    .execute(&state.db)
    .await?;

    Ok(())
}
