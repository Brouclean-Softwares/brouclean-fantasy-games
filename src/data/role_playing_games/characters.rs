use crate::data::users::User;
use crate::data::Id;
use crate::errors::AppError;
use crate::AppState;
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
                    users.name as user_name
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
                    users.name as user_name
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
                    users.name as user_name
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

pub async fn create(state: &AppState, user: &User, character: &Character) -> Result<i32, AppError> {
    tracing::debug!(
        "create for user={:?} the following character={:?}",
        user,
        character,
    );

    let new_character_id: Id = sqlx::query_as(
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

    Ok(new_character_id.id)
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
            and user_id = $2",
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
