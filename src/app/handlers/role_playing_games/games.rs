use crate::AppState;
use crate::app::templates::role_playing_games::games::{GamePage, GamesPage};
use crate::data::role_playing_games::games::Game;
use crate::data::role_playing_games::{campaigns, characters, games};
use crate::data::users::MayBeUser;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(games).post(add_new))
        .route("/delete", post(delete))
        .route("/game", get(game).post(update))
}

pub async fn games(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
) -> Result<GamesPage, Redirect> {
    let characters = games::select_all(&app_state)
        .await
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

    Ok(GamesPage::get(app_state, profile, characters))
}

#[derive(Deserialize)]
pub struct GameQueryParams {
    pub id: i32,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn game(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Query(params): Query<GameQueryParams>,
) -> Result<GamePage, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/games");

    let game = games::select_by_id(&app_state, params.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let has_campaigns = campaigns::exists_for_game(&app_state, game.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let has_characters = characters::exists_for_game(&app_state, game.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let deletable = profile.is_some() && !has_campaigns && !has_characters;

    Ok(GamePage::get(
        app_state,
        profile.clone(),
        game,
        deletable,
        profile.is_some(),
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
    ))
}

#[derive(Deserialize)]
pub struct NewGameForm {
    pub name: String,
}

pub async fn add_new(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Form(form): Form<NewGameForm>,
) -> Result<Redirect, Redirect> {
    if let Some(profile) = profile {
        let game = Game {
            id: 0,
            name: form.name,
            external_logo_url: None,
            description: "".to_string(),
        };

        let new_game_id = games::create(&app_state, &profile, &game)
            .await
            .map_err(|error| error.log_and_redirect(Redirect::to("/role_playing_games/games")))?;

        Ok(Redirect::to(&format!(
            "/role_playing_games/games/game?id={}",
            new_game_id
        )))
    } else {
        Ok(Redirect::to("/role_playing_games/games"))
    }
}

#[derive(Deserialize)]
pub struct UpdateGameForm {
    pub id: i32,
    pub name: Option<String>,
    pub external_logo_url: Option<String>,
    pub description: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Form(form): Form<UpdateGameForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error =
        Redirect::to(&format!("/role_playing_games/games/game?id={}", form.id));

    let updating_user = profile.ok_or(redirect_when_error.clone())?;

    let mut game = games::select_by_id(&app_state, form.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    if let Some(name) = form.name {
        game.name = name;
    }

    if let Some(external_logo_url) = form.external_logo_url {
        game.external_logo_url = Some(external_logo_url);
    }

    if let Some(description) = form.description {
        game.description = description;
    }

    games::update(&app_state, &updating_user, &game)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/games/game?id={}",
        form.id
    )))
}

#[derive(Deserialize)]
pub struct DeleteGameForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Form(form): Form<DeleteGameForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error =
        Redirect::to(&format!("/role_playing_games/games/game?id={}", form.id));

    if let Some(connected_user) = profile {
        if games::delete(&app_state, &connected_user, form.id)
            .await
            .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?
        {
            return Ok(Redirect::to("/role_playing_games/games"));
        }
    }

    Err(redirect_when_error)
}
