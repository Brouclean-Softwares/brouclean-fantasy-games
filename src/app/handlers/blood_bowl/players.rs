use crate::app::templates::blood_bowl::players::PlayerPage;
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::{games, players, teams};
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/player", get(player).post(update))
}

#[derive(Deserialize)]
pub struct PlayerQueryParams {
    pub player_id: i32,
    pub team_id: i32,
    pub alert_message: Option<String>,
    pub edit: Option<bool>,
}

pub async fn player(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<PlayerQueryParams>,
) -> Result<PlayerPage, Redirect> {
    let error_handler = |error| {
        tracing::debug!("get_player: Error: {}", error);
        Redirect::to("../teams")
    };

    let team = teams::select_by_id_without_staff_nor_players(&app_state, params.team_id)
        .await
        .map_err(error_handler)?;

    let mut is_playing_game = false;
    if let Some(game) = games::select_playing_by_team(&app_state, &team.id)
        .await
        .map_err(error_handler)?
    {
        is_playing_game = game.started && !game.finished;
    }

    let alert_message: Option<AlertMessage> = params.alert_message.and_then(|message| {
        Some(AlertMessage {
            alert_type: AlertType::Danger,
            message,
        })
    });

    let editable = !is_playing_game
        && match profile.clone() {
            Some(user) => team.coach.eq(&user.into()),
            None => false,
        };

    let (number, player) =
        players::select_by_id_for_team(&app_state, params.player_id, params.team_id)
            .await
            .map_err(error_handler)?;

    let player_advancements =
        players::select_advancements_with_choices(&app_state, params.player_id)
            .await
            .map_err(error_handler)?;

    Ok(PlayerPage::get(
        app_state,
        profile,
        alert_message,
        number,
        player,
        player_advancements,
        team,
        editable,
        params.edit.unwrap_or(false) && editable,
    ))
}

#[derive(Deserialize)]
pub struct PlayerForm {
    pub player_number: Option<i32>,
    pub player_name: Option<String>,
    pub advancement_choice: Option<String>,
    pub advancement_to_add: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<PlayerQueryParams>,
    Form(form): Form<PlayerForm>,
) -> Result<Redirect, Redirect> {
    // Player number
    if let (Some(profile), Some(player_number)) = (profile.clone(), form.player_number) {
        players::update_number(
            &app_state,
            &profile,
            &params.team_id,
            &params.player_id,
            &player_number,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Player name
    if let (Some(profile), Some(player_name)) = (profile.clone(), form.player_name) {
        players::update_name(
            &app_state,
            &profile,
            &params.team_id,
            &params.player_id,
            &player_name,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Advancement choice
    if let (Some(profile), Some(advancement_choice)) = (profile.clone(), form.advancement_choice) {
        let advancement_choice =
            serde_json::from_str(&advancement_choice).or_else(|app_error| {
                Err(Redirect::to(&format!(
                    "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                    params.player_id,
                    params.team_id,
                    app_error,
                    params.edit.unwrap_or(false),
                )))
            })?;

        players::add_advancement_choice(
            &app_state,
            &profile,
            params.team_id,
            params.player_id,
            advancement_choice,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Advancement
    if let (Some(profile), Some(advancement_to_add)) = (profile.clone(), form.advancement_to_add) {
        let advancement_to_add =
            serde_json::from_str(&advancement_to_add).or_else(|app_error| {
                Err(Redirect::to(&format!(
                    "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                    params.player_id,
                    params.team_id,
                    app_error,
                    params.edit.unwrap_or(false),
                )))
            })?;

        players::add_advancement(
            &app_state,
            &profile,
            params.team_id,
            params.player_id,
            advancement_to_add,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    Ok(Redirect::to(&format!(
        "./player?player_id={}&team_id={}&edit={}",
        params.player_id,
        params.team_id,
        params.edit.unwrap_or(false),
    )))
}
