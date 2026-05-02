use crate::app::templates::role_playing_games::characters::{CharacterPage, CharactersPage};
use crate::data::role_playing_games::characters::Character;
use crate::data::role_playing_games::{characters, games};
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(characters).post(add_new))
        .route("/character", get(character).post(update))
}

pub async fn characters(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<CharactersPage, Redirect> {
    let characters = characters::select_all(&app_state)
        .await
        .or_else(|_| Err(Redirect::to("/")))?;

    let games = games::select_all(&app_state)
        .await
        .or_else(|_| Err(Redirect::to("/")))?;

    Ok(CharactersPage::get(app_state, profile, characters, games))
}

#[derive(Deserialize)]
pub struct CharacterQueryParams {
    pub id: i32,
    pub tab_name: Option<String>,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn character(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<CharacterQueryParams>,
) -> Result<CharacterPage, Redirect> {
    let character = characters::select_by_id(&app_state, params.id)
        .await
        .map_err(|_| Redirect::to("/role_playing_games/characters"))?;

    Ok(CharacterPage::get(
        app_state,
        profile.clone(),
        character,
        params.tab_name,
        profile.is_some(),
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
    ))
}

#[derive(Deserialize)]
pub struct NewCharacterForm {
    pub name: String,
    pub game_id: i32,
}

pub async fn add_new(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<NewCharacterForm>,
) -> Result<Redirect, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/characters");

    let profile = profile.ok_or(redirect_if_error.clone())?;

    let game = games::select_by_id(&app_state, form.game_id)
        .await
        .map_err(|_| redirect_if_error.clone())?;

    let character = Character {
        id: 0,
        name: form.name,
        external_image_url: None,
        description: "".to_string(),
        notes: "".to_string(),
        game_id: game.id,
        game_name: game.name,
        game_external_logo_url: game.external_logo_url,
        user: None,
    };

    let new_character_id = characters::create(&app_state, &profile, &character)
        .await
        .map_err(|_| redirect_if_error.clone())?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/characters/character?id={}",
        new_character_id
    )))
}

#[derive(Deserialize)]
pub struct UpdateCharacterForm {
    pub id: i32,
    pub tab_name: String,
    pub name: Option<String>,
    pub external_image_url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<UpdateCharacterForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error = Redirect::to(&format!(
        "/role_playing_games/characters/character?id={}&tab_name={}",
        form.id, form.tab_name
    ));

    let updating_user = profile.ok_or(redirect_when_error.clone())?;

    let mut character = characters::select_by_id(&app_state, form.id)
        .await
        .map_err(|_| redirect_when_error.clone())?;

    let character_owner = character.user.clone().ok_or(redirect_when_error.clone())?;
    if updating_user.id.ne(&character_owner.id) {
        return Err(redirect_when_error);
    }

    if let Some(name) = form.name {
        character.name = name;
    }

    if let Some(external_image_url) = form.external_image_url {
        character.external_image_url = Some(external_image_url);
    }

    if let Some(description) = form.description {
        character.description = description;
    }

    if let Some(notes) = form.notes {
        character.notes = notes;
    }

    characters::update(&app_state, &updating_user, &character)
        .await
        .map_err(|_| redirect_when_error.clone())?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/characters/character?id={}",
        form.id
    )))
}
