use crate::app::templates::blood_bowl::games::{GamePage, GamesPage, NewGamePage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::{games, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use blood_bowl_rs::teams::Team;
use chrono::NaiveDateTime;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(games))
        .route("/game", get(game).post(update))
        .route("/new", get(new).post(create))
        .route("/delete", post(delete))
}

pub async fn games(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<GamesPage, AppError> {
    let games_playing = games::select_all_playing(&app_state).await?;
    let games_played = games::select_all_played(&app_state).await?;

    GamesPage::get(app_state, profile, games_playing, games_played)
}

#[derive(Deserialize)]
pub struct GameQueryParams {
    pub id: i32,
    pub edit_mode: Option<bool>,
    pub info_message: Option<String>,
    pub error_message: Option<String>,
}

pub async fn game(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<GameQueryParams>,
) -> Result<GamePage, AppError> {
    let game = games::select_by_id(&app_state, params.id).await?;

    let edit_mode = params.edit_mode.unwrap_or(false);

    let mut alert_message: Option<AlertMessage> = None;

    if let Some(message) = params.info_message {
        alert_message = Some(AlertMessage {
            alert_type: AlertType::Primary,
            message,
        });
    };

    if let Some(message) = params.error_message {
        alert_message = Some(AlertMessage {
            alert_type: AlertType::Danger,
            message,
        });
    };

    Ok(GamePage::get_with_message(
        app_state,
        profile,
        alert_message,
        game,
        edit_mode,
    )?)
}

#[derive(Deserialize)]
pub struct GameForm {
    pub game_id: i32,
    pub game_at: Option<String>,
    pub started_at: Option<String>,
}

fn redirect_when_update_ko(game_id: &i32, error: AppError) -> Redirect {
    Redirect::to(&format!(
        "/blood_bowl/games/game?id={}&error_message={}",
        game_id,
        error.to_string()
    ))
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<GameForm>,
) -> Result<Redirect, Redirect> {
    let redirect_ok = Redirect::to(&format!("/blood_bowl/games/game?id={}", form.game_id));

    let mut game = games::select_by_id(&app_state, form.game_id)
        .await
        .map_err(|err| redirect_when_update_ko(&form.game_id, err))?;

    if let Some(game_date) = form.game_at {
        game.game_at = NaiveDateTime::parse_from_str(&*game_date, "%Y-%m-%dT%H:%M")
            .map_err(|err| redirect_when_update_ko(&form.game_id, err.into()))?;

        games::update_schedule(&app_state, &profile, &game)
            .await
            .map_err(|err| redirect_when_update_ko(&form.game_id, err))?;

        return Ok(redirect_ok);
    }

    if let Some(game_date) = form.started_at {
        let game_start = NaiveDateTime::parse_from_str(&*game_date, "%Y-%m-%dT%H:%M")
            .map_err(|err| redirect_when_update_ko(&form.game_id, err.into()))?;

        game.game_at = game_start;
        game.start();

        games::update_start(&app_state, &profile, &game)
            .await
            .map_err(|err| redirect_when_update_ko(&form.game_id, err))?;

        return Ok(redirect_ok);
    }

    Ok(redirect_ok)
}

#[derive(Deserialize)]
pub struct NewGameQueryParams {
    pub first_team_id: Option<i32>,
    pub second_team_id: Option<i32>,
}

pub async fn new(
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

pub async fn create(
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

#[derive(Deserialize)]
pub struct DeleteGameForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<DeleteGameForm>,
) -> Result<Redirect, Redirect> {
    games::delete(&app_state, &profile, form.id.clone())
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./game?id={}&message={}",
                form.id, app_error
            )))
        })?;

    Ok(Redirect::to("/blood_bowl"))
}
