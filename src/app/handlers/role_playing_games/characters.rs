use crate::AppState;
use crate::app::templates::role_playing_games::characters::{CharacterPage, CharactersPage};
use crate::data::role_playing_games::characters::Character;
use crate::data::role_playing_games::{characters, games};
use crate::data::users::User;
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
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

    let games = games::select_all(&app_state)
        .await
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

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
    let redirect_if_error = Redirect::to("/role_playing_games/characters");

    let character = characters::select_by_id(&app_state, params.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let is_owner = match (&character.user, &profile) {
        (Some(owner), Some(connected_user)) => owner.id.eq(&connected_user.id),
        (_, _) => false,
    };

    let games = games::select_all(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    Ok(CharacterPage::get(
        app_state,
        profile.clone(),
        character,
        params.tab_name,
        is_owner,
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
        games,
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
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let character = Character {
        id: 0,
        name: form.name,
        external_image_url: None,
        description: "".to_string(),
        profile: "".to_string(),
        private_note: "".to_string(),
        public_note: "".to_string(),
        game_id: game.id,
        game_name: game.name,
        game_external_logo_url: game.external_logo_url,
        user: None,
    };

    let new_character_id = characters::create(&app_state, &profile, &character)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

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
    pub game_id: Option<i32>,
    pub external_image_url: Option<String>,
    pub description: Option<String>,
    pub profile: Option<String>,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
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
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    let character_owner = character.user.clone().ok_or(redirect_when_error.clone())?;
    if updating_user.id.ne(&character_owner.id) {
        return Err(redirect_when_error);
    }

    if let Some(name) = form.name {
        character.name = name;
    }

    if let Some(game_id) = form.game_id {
        character.game_id = game_id;
    }

    if let Some(external_image_url) = form.external_image_url {
        character.external_image_url = if external_image_url.len() > 0 {
            Some(external_image_url)
        } else {
            None
        };
    }

    if let Some(description) = form.description {
        character.description = description;
    }

    if let Some(profile) = form.profile {
        character.profile = profile;
    }

    if let Some(private_note) = form.private_note {
        character.private_note = private_note;
    }

    if let Some(public_note) = form.public_note {
        character.public_note = public_note;
    }

    characters::update(&app_state, &updating_user, &character)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/characters/character?id={}&tab_name={}",
        form.id, form.tab_name
    )))
}
