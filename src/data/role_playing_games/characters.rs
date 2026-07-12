use crate::AppState;
use crate::data::users::User;
use crate::errors::AppError;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct CharacterRow {
    pub id: i32,
    pub name: String,
    pub external_image_url: Option<String>,
    pub description: String,
    pub profile: String,
    pub private_note: String,
    pub public_note: String,
    pub game_id: i32,
    pub game_name: String,
    pub game_external_logo_url: Option<String>,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub sessions_number: i64,
}

impl CharacterRow {
    pub async fn into_character(self, state: &AppState) -> Result<Character, AppError> {
        let user = User::select_by_id(state, self.user_id).await?;

        Ok(Character {
            id: self.id,
            name: self.name,
            external_image_url: self.external_image_url,
            description: self.description,
            profile: self.profile,
            private_note: self.private_note,
            public_note: self.public_note,
            game_id: self.game_id,
            game_name: self.game_name,
            game_external_logo_url: self.game_external_logo_url,
            user,
        })
    }
}

#[derive(Debug)]
pub struct Character {
    pub id: i32,
    pub name: String,
    pub external_image_url: Option<String>,
    pub description: String,
    pub profile: String,
    pub private_note: String,
    pub public_note: String,
    pub game_id: i32,
    pub game_name: String,
    pub game_external_logo_url: Option<String>,
    pub user: Option<User>,
}

pub async fn select_all(state: &AppState) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_all");

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            ORDER BY rpg_characters.name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_owned(state: &AppState, user: &User) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_owned for user={:?}", user);

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.user_id = $1
            ORDER BY rpg_characters.name ASC",
    )
    .bind(user.id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Character, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let character_row: CharacterRow = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let character = character_row.into_character(state).await?;

    Ok(character)
}

pub async fn exists_for_game(state: &AppState, game_id: i32) -> Result<bool, AppError> {
    tracing::debug!("exists_for_game with game_id={}", game_id);

    let character_id: Option<i32> = sqlx::query_scalar(
        "SELECT id
            FROM rpg_characters
            WHERE game_id = $1
            LIMIT 1",
    )
    .bind(game_id.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(character_id.is_some())
}

pub async fn select_for_game(
    state: &AppState,
    game_id: i32,
) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_for_game with game_id={}", game_id);

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.game_id = $1",
    )
    .bind(game_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_filtered_for_game(
    state: &AppState,
    filter: String,
    game_id: i32,
) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!(
        "select_filtered_for_game with filter={} and game_id={}",
        filter,
        game_id
    );

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.game_id = $1
            AND (
                LOWER(rpg_characters.name) LIKE $2
                OR LOWER(users.name) LIKE $2
            )
            ORDER BY rpg_characters.name ASC",
    )
    .bind(game_id.clone())
    .bind(format!("%{}%", filter.to_lowercase()))
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_for_campaign(
    state: &AppState,
    campaign_id: i32,
) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_for_campaign with campaign_id={}", campaign_id);

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.id in (
                SELECT rpg_sessions_characters.character_id
                FROM rpg_sessions_characters
                INNER JOIN rpg_sessions
                ON rpg_sessions.id = rpg_sessions_characters.session_id
                INNER JOIN rpg_arcs
                ON rpg_arcs.id = rpg_sessions.arc_id
                WHERE rpg_sessions_characters.character_id = rpg_characters.id
                AND rpg_arcs.campaign_id = $1
            )
            ORDER BY rpg_characters.name ASC",
    )
    .bind(campaign_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_for_arc(state: &AppState, arc_id: i32) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_for_arc with arc_id={}", arc_id);

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_characters.id in (
                SELECT rpg_sessions_characters.character_id
                FROM rpg_sessions_characters
                INNER JOIN rpg_sessions
                ON rpg_sessions.id = rpg_sessions_characters.session_id
                WHERE rpg_sessions_characters.character_id = rpg_characters.id
                AND rpg_sessions.arc_id = $1
            )
            ORDER BY rpg_characters.name ASC",
    )
    .bind(arc_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn select_for_session(
    state: &AppState,
    session_id: i32,
) -> Result<Vec<CharacterRow>, AppError> {
    tracing::debug!("select_for_session with session_id={}", session_id);

    let character_rows: Vec<CharacterRow> = sqlx::query_as(
        "SELECT rpg_characters.id,
                    rpg_characters.name,
                    rpg_characters.external_image_url,
                    rpg_characters.description,
                    rpg_characters.profile,
                    rpg_characters.private_note,
                    rpg_characters.public_note,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as user_id,
                    users.name as user_name,
                    (SELECT COUNT(session_id) from rpg_sessions_characters WHERE character_id = rpg_characters.id) as sessions_number
            FROM rpg_characters
            INNER JOIN rpg_sessions_characters
            ON rpg_sessions_characters.character_id = rpg_characters.id
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_characters.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_characters.user_id
            WHERE rpg_sessions_characters.session_id = $1
            ORDER BY rpg_characters.name ASC",
    )
    .bind(session_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(character_rows)
}

pub async fn is_user_game_master_of_character(
    state: &AppState,
    user: &User,
    character_id: i32,
) -> Result<bool, AppError> {
    let game_master_id: Option<i32> = sqlx::query_scalar(
        "SELECT rpg_campaigns.game_master_id
            FROM rpg_sessions_characters
            INNER JOIN rpg_sessions
            ON rpg_sessions_characters.session_id = rpg_sessions.id
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            INNER JOIN rpg_campaigns
            ON rpg_arcs.campaign_id = rpg_campaigns.id
            WHERE rpg_sessions_characters.character_id = $2
            AND rpg_campaigns.game_master_id = $1
            LIMIT 1",
    )
    .bind(user.id.clone())
    .bind(character_id.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(game_master_id.is_some())
}

pub async fn create(state: &AppState, user: &User, character: &Character) -> Result<i32, AppError> {
    tracing::debug!(
        "create for user={:?} the following character={:?}",
        user,
        character,
    );

    let new_character_id: i32 = sqlx::query_scalar(
        "INSERT INTO rpg_characters (
                game_id,
                name,
                user_id)
            VALUES ($1, $2, $3)
            RETURNING id",
    )
    .bind(character.game_id.clone())
    .bind(character.name.clone())
    .bind(user.id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(new_character_id)
}

pub async fn update(
    state: &AppState,
    connected_user: &User,
    character: &Character,
) -> Result<(), AppError> {
    tracing::debug!(
        "update character with id={} by user={:?}",
        character.id,
        connected_user
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE rpg_characters
            SET name = $3,
                external_image_url = $4,
                description = $5,
                profile = $6,
                private_note = $7,
                public_note = $8,
                game_id = $9,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $1
            AND user_id = $2",
        )
        .bind(character.id.clone())
        .bind(connected_user_id.clone())
        .bind(character.name.clone())
        .bind(character.external_image_url.clone())
        .bind(character.description.clone())
        .bind(character.profile.clone())
        .bind(character.private_note.clone())
        .bind(character.public_note.clone())
        .bind(character.game_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}
