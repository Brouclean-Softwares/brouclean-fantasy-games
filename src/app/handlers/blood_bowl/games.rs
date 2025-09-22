use crate::app::templates::blood_bowl::games::{GamePage, NewGamePage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::{games, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use blood_bowl_rs::teams::Team;
use chrono::NaiveDateTime;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/game", get(game))
        .route("/new", get(new_game).post(create_game))
}

#[derive(Deserialize)]
pub struct GameQueryParams {
    pub id: i32,
}

pub async fn game(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<GameQueryParams>,
) -> Result<GamePage, AppError> {
    let game = games::select_by_id(&app_state, params.id).await?;

    Ok(GamePage::get(app_state, profile, game)?)
}

#[derive(Deserialize)]
pub struct NewGameQueryParams {
    pub first_team_id: Option<i32>,
    pub second_team_id: Option<i32>,
}

pub async fn new_game(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<NewGameQueryParams>,
) -> Result<NewGamePage, AppError> {
    let mut first_team: Option<Team> = None;

    if let Some(id) = params.first_team_id {
        first_team = Some(teams::select_by_id(&app_state, id).await?);
    }

    let mut second_team: Option<Team> = None;

    if let Some(id) = params.second_team_id {
        second_team = Some(teams::select_by_id(&app_state, id).await?);
    }

    let new_game_page = NewGamePage::get(app_state, profile, first_team, second_team);

    Ok(new_game_page)
}

#[derive(Deserialize)]
pub struct NewGameForm {
    pub first_team_id: Option<i32>,
    pub second_team_id: Option<i32>,
    pub scheduled_at: Option<String>,
}

pub async fn create_game(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<NewGameForm>,
) -> Result<Redirect, NewGamePage> {
    let fn_if_id_positive = |id| {
        if id < 0 {
            None
        } else {
            Some(id)
        }
    };

    let first_team_id = form.first_team_id.and_then(fn_if_id_positive);
    let second_team_id = form.second_team_id.and_then(fn_if_id_positive);

    match (first_team_id, second_team_id, form.scheduled_at) {
        (Some(first_team_id), None, None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?first_team_id={}",
            first_team_id
        ))),

        (None, Some(second_team_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?second_team_id={}",
            second_team_id
        ))),

        (Some(first_team_id), Some(second_team_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?first_team_id={}&second_team_id={}",
            first_team_id, second_team_id
        ))),

        (Some(first_team_id), Some(second_team_id), Some(scheduled_at)) => {
            let first_team = teams::select_by_id(&app_state, first_team_id)
                .await
                .map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "la première équipe est introuvable.".to_string(),
                        }),
                        None,
                        None,
                    )
                })?;

            let second_team = teams::select_by_id(&app_state, second_team_id)
                .await
                .map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "la deuxième équipe est introuvable.".to_string(),
                        }),
                        None,
                        None,
                    )
                })?;

            let scheduled_at = NaiveDateTime::parse_from_str(&*scheduled_at, "%Y-%m-%dT%H:%M")
                .map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "Veuillez remplir la date et l'heure du match.".to_string(),
                        }),
                        Some(first_team.clone()),
                        Some(second_team.clone()),
                    )
                })?;

            let game_id = games::create(
                &app_state,
                &profile,
                &first_team,
                &second_team,
                scheduled_at,
            )
            .await
            .map_err(|error| {
                NewGamePage::get_with_message(
                    app_state.clone(),
                    profile.clone(),
                    Some(AlertMessage {
                        alert_type: AlertType::Danger,
                        message: error.to_string(),
                    }),
                    Some(first_team.clone()),
                    Some(second_team.clone()),
                )
            })?;

            Ok(Redirect::to(&format!("./game?id={}", game_id)))
        }

        _ => Ok(Redirect::to("/blood_bowl/games/new")),
    }
}
