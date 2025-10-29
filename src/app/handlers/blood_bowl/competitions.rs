use crate::app::templates::blood_bowl::competitions::{CompetitionPage, CompetitionsPage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(competitions))
        .route("/new", get(new).post(save))
        .route("/delete", post(delete))
        .route("/competition", get(competition).post(save))
        .route("/register_team", post(register_team))
}

pub async fn competitions(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<CompetitionsPage, Redirect> {
    let error_handler = |error| {
        tracing::debug!("competitions: Error: {}", error);
        Redirect::to("..")
    };

    let competitions_in_progress = Competition::select_all_in_progress(&app_state)
        .await
        .map_err(error_handler)?;

    let competitions_closed = Competition::select_all_closed(&app_state)
        .await
        .map_err(error_handler)?;

    Ok(CompetitionsPage::get(
        app_state,
        profile,
        competitions_in_progress,
        competitions_closed,
    ))
}

pub async fn new(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<CompetitionPage, Redirect> {
    if profile.is_some() {
        let competition = Competition::new(profile.clone());

        Ok(CompetitionPage::get(app_state, profile, None, competition, true).await)
    } else {
        Err(Redirect::to(".."))
    }
}

#[derive(Deserialize)]
pub struct CompetitionQueryParams {
    pub id: Option<i32>,
    pub edit: Option<bool>,
    pub alert_message: Option<String>,
}

pub async fn competition(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<CompetitionQueryParams>,
) -> Result<CompetitionPage, Redirect> {
    if let Some(competition_id) = params.id {
        let error_handler = |error| {
            tracing::debug!("competition: Error: {}", error);
            Redirect::to("../competitions")
        };

        let alert_message: Option<AlertMessage> = params.alert_message.and_then(|message| {
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message,
            })
        });

        let competition = Competition::select_by_id(&app_state, competition_id)
            .await
            .map_err(error_handler)?
            .ok_or(Redirect::to("../competitions"))?;

        Ok(CompetitionPage::get(
            app_state,
            profile,
            alert_message,
            competition,
            params.edit.unwrap_or(false),
        )
        .await)
    } else {
        Err(Redirect::to("../competitions"))
    }
}

#[derive(Deserialize)]
pub struct CompetitionForm {
    pub competition_name: Option<String>,
    pub competition_description: Option<String>,
}

pub async fn save(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<CompetitionQueryParams>,
    Form(form): Form<CompetitionForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error| {
        Redirect::to(&format!(
            "./competition?id={:?}&alert_message={}&edit={}",
            params.id,
            error,
            params.edit.unwrap_or(false),
        ))
    };

    let mut competition = if let Some(competition_id) = params.id {
        Competition::select_by_id(&app_state, competition_id)
            .await
            .map_err(error_handler)?
            .ok_or(Redirect::to("../competitions"))?
    } else {
        Competition::new(Some(profile.clone()))
    };

    // Name
    if let Some(competition_name) = form.competition_name {
        competition.name = competition_name;
    }

    // Description
    if let Some(competition_description) = form.competition_description {
        competition.description = competition_description;
    }

    competition
        .save(&app_state, &profile)
        .await
        .map_err(error_handler)?;

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        competition.id,
    )))
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<DeleteForm>,
) -> Result<Redirect, Redirect> {
    Competition::delete(&app_state, &profile, form.id)
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./competition?id={}&message={}",
                form.id, app_error
            )))
        })?;

    Ok(Redirect::to("/blood_bowl"))
}

#[derive(Deserialize)]
pub struct RegisterTeamForm {
    pub competition_id: i32,
    pub team_to_registered_id: i32,
}

pub async fn register_team(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<RegisterTeamForm>,
) -> Redirect {
    if profile.is_some() {
        if let Err(error) = Competition::insert_team_registration(
            &app_state,
            form.competition_id,
            form.team_to_registered_id,
        )
        .await
        {
            tracing::debug!("competition registering team: Error: {}", error);
        }
    }

    Redirect::to(&format!("./competition?id={}", form.competition_id))
}
