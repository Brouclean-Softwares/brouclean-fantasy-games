use crate::AppState;
use crate::app::templates::role_playing_games::campaigns::GameSessionPage;
use crate::data::role_playing_games::campaigns;
use crate::data::role_playing_games::campaigns::{arcs, sessions};
use crate::data::users::User;
use axum::Form;
use axum::extract::{Query, State};
use axum::response::Redirect;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewGameSessionForm {
    pub name: String,
    pub campaign_id: i32,
    pub arc_id: i32,
}

pub async fn new_session(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<NewGameSessionForm>,
) -> Result<Redirect, Redirect> {
    let redirect = Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
        form.campaign_id
    ));

    let profile = profile.ok_or(redirect.clone())?;

    let arc = arcs::select_by_id(&app_state, form.arc_id)
        .await
        .map_err(|error| error.log_and_redirect(Redirect::to("/role_playing_games/campaigns")))?;

    let _ = campaigns::sessions::push_new_into_arc(&app_state, &profile, &arc, form.name)
        .await
        .map_err(|error| error.log_and_redirect(redirect.clone()))?;

    Ok(redirect)
}

#[derive(Deserialize)]
pub struct GameSessionQueryParams {
    pub id: i32,
    pub tab_name: Option<String>,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn session(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<GameSessionQueryParams>,
) -> Result<GameSessionPage, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/campaigns");

    let session = sessions::select_by_id(&app_state, params.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let campaign = campaigns::select_by_id(&app_state, session.campaign_id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let is_owner = match (&campaign.game_master, &profile) {
        (Some(owner), Some(connected_user)) => owner.id.eq(&connected_user.id),
        (_, _) => false,
    };

    Ok(GameSessionPage::get(
        app_state,
        profile.clone(),
        session,
        params.tab_name,
        is_owner,
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
    ))
}

#[derive(Deserialize)]
pub struct UpdateGameSessionForm {
    pub id: i32,
    pub tab_name: String,
    pub name: Option<String>,
    pub session_date_input: Option<String>,
    pub external_image_url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<UpdateGameSessionForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error = Redirect::to(&format!(
        "/role_playing_games/campaigns/session?id={}&tab_name={}",
        form.id, form.tab_name
    ));

    let mut session = sessions::select_by_id(&app_state, form.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    let updating_user = profile.ok_or(redirect_when_error.clone())?;

    let campaign = campaigns::select_by_id(&app_state, session.campaign_id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    let game_master = campaign
        .game_master
        .clone()
        .ok_or(redirect_when_error.clone())?;
    if updating_user.id.ne(&game_master.id) {
        return Err(redirect_when_error);
    }

    if let Some(name) = form.name {
        session.name = name;
    }

    if let Some(session_date_input) = form.session_date_input {
        session.playing_at = Some(
            NaiveDateTime::parse_from_str(&*session_date_input, "%Y-%m-%dT%H:%M").map_err(
                |error| {
                    tracing::error!("{}", error);
                    redirect_when_error.clone()
                },
            )?,
        );
    }

    if let Some(external_image_url) = form.external_image_url {
        session.external_image_url = if external_image_url.len() > 0 {
            Some(external_image_url)
        } else {
            None
        };
    }

    if let Some(description) = form.description {
        session.description = description;
    }

    if let Some(notes) = form.notes {
        session.notes = notes;
    }

    sessions::update(&app_state, &updating_user, &session)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/campaigns/session?id={}&tab_name={}",
        form.id, form.tab_name
    )))
}
