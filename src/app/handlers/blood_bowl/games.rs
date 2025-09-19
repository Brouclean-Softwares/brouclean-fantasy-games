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
    profile: User,
    Query(params): Query<GameQueryParams>,
) -> Result<GamePage, AppError> {
    let game = games::select_by_id(&app_state, params.id).await?;

    Ok(GamePage::get(app_state, profile, game))
}

#[derive(Deserialize)]
pub struct NewGameQueryParams {
    pub team_a_id: Option<i32>,
    pub team_b_id: Option<i32>,
}

pub async fn new_game(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<NewGameQueryParams>,
) -> Result<NewGamePage, AppError> {
    let mut team_a: Option<Team> = None;

    if let Some(id) = params.team_a_id {
        team_a = Some(teams::select_by_id(&app_state, id).await?);
    }

    let mut team_b: Option<Team> = None;

    if let Some(id) = params.team_b_id {
        team_b = Some(teams::select_by_id(&app_state, id).await?);
    }

    let new_game_page = NewGamePage::get(app_state, profile, team_a, team_b);

    Ok(new_game_page)
}

#[derive(Deserialize)]
pub struct NewTeamForm {
    pub team_a_id: Option<i32>,
    pub team_b_id: Option<i32>,
    pub played_at: Option<String>,
}

pub async fn create_game(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<NewTeamForm>,
) -> Result<Redirect, NewGamePage> {
    let fn_if_id_positive = |id| {
        if id < 0 {
            None
        } else {
            Some(id)
        }
    };

    let team_a_id = form.team_a_id.and_then(fn_if_id_positive);
    let team_b_id = form.team_b_id.and_then(fn_if_id_positive);

    match (team_a_id, team_b_id, form.played_at) {
        (Some(team_a_id), None, None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?team_a_id={}",
            team_a_id
        ))),

        (None, Some(team_b_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?team_b_id={}",
            team_b_id
        ))),

        (Some(team_a_id), Some(team_b_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?team_a_id={}&team_b_id={}",
            team_a_id, team_b_id
        ))),

        (Some(team_a_id), Some(team_b_id), Some(played_at)) => {
            let team_a = teams::select_by_id(&app_state, team_a_id)
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

            let team_b = teams::select_by_id(&app_state, team_b_id)
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

            let played_at =
                NaiveDateTime::parse_from_str(&*played_at, "%Y-%m-%dT%H:%M").map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "Veuillez remplir la date et l'heure du match.".to_string(),
                        }),
                        Some(team_a.clone()),
                        Some(team_b.clone()),
                    )
                })?;

            let game_id = games::create(&app_state, &profile, &team_a, &team_b, played_at)
                .await
                .map_err(|error| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: error.to_string(),
                        }),
                        Some(team_a.clone()),
                        Some(team_b.clone()),
                    )
                })?;

            Ok(Redirect::to(&format!("./game?id={}", game_id)))
        }

        _ => Ok(Redirect::to("/blood_bowl/games/new")),
    }
}
