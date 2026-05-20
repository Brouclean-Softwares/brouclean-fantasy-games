use crate::AppState;
use crate::app::templates::role_playing_games::campaigns::{CampaignPage, CampaignsPage};
use crate::data::role_playing_games::campaigns::Campaign;
use crate::data::role_playing_games::{campaigns, characters, games};
use crate::data::users::User;
use axum::extract::{OriginalUri, Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

pub mod arcs;
pub mod sessions;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(campaigns).post(add_new))
        .route("/delete", post(delete))
        .route("/campaign", get(campaign).post(update))
        .route("/new_arc", post(arcs::new))
        .route("/delete_arc", post(arcs::delete))
        .route("/arc", get(arcs::arc).post(arcs::update))
        .route("/new_session", post(sessions::new))
        .route("/delete_session", post(sessions::delete))
        .route("/session", get(sessions::session).post(sessions::update))
        .route(
            "/link_character_to_session",
            post(sessions::link_character_to_session),
        )
        .route(
            "/unlink_character_from_session",
            post(sessions::unlink_character_from_session),
        )
}

pub async fn campaigns(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<CampaignsPage, Redirect> {
    let campaigns = campaigns::select_all(&app_state)
        .await
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

    let games = games::select_all(&app_state)
        .await
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

    Ok(CampaignsPage::get(
        app_state, profile, &uri, campaigns, games,
    ))
}

#[derive(Deserialize)]
pub struct CampaignQueryParams {
    pub id: i32,
    pub tab_name: Option<String>,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn campaign(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<CampaignQueryParams>,
) -> Result<CampaignPage, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/campaigns");

    let campaign = campaigns::select_by_id(&app_state, params.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let is_owner = match (&campaign.game_master, &profile) {
        (Some(owner), Some(connected_user)) => owner.id.eq(&connected_user.id),
        (_, _) => false,
    };

    let games = games::select_all(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let arcs_with_sessions =
        campaigns::arcs::select_for_campaign_with_game_sessions(&app_state, campaign.id)
            .await
            .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let deletable = is_owner && arcs_with_sessions.is_empty();

    let characters = characters::select_for_campaign(&app_state, campaign.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    Ok(CampaignPage::get(
        app_state,
        profile.clone(),
        &uri,
        campaign,
        params.tab_name,
        deletable,
        is_owner,
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
        games,
        arcs_with_sessions,
        characters,
    ))
}

#[derive(Deserialize)]
pub struct NewCampaignForm {
    pub name: String,
    pub game_id: i32,
}

pub async fn add_new(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<NewCampaignForm>,
) -> Result<Redirect, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/campaigns");

    let profile = profile.ok_or(redirect_if_error.clone())?;

    let game = games::select_by_id(&app_state, form.game_id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let campaign = Campaign {
        id: 0,
        name: form.name,
        external_image_url: None,
        description: "".to_string(),
        notes: "".to_string(),
        game_id: game.id,
        game_name: game.name,
        game_external_logo_url: game.external_logo_url,
        game_master: None,
    };

    let new_campaign_id = campaigns::create(&app_state, &profile, &campaign)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}",
        new_campaign_id
    )))
}

#[derive(Deserialize)]
pub struct UpdateCampaignForm {
    pub id: i32,
    pub tab_name: String,
    pub name: Option<String>,
    pub game_id: Option<i32>,
    pub external_image_url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<UpdateCampaignForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error = Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}&tab_name={}",
        form.id, form.tab_name
    ));

    let updating_user = profile.ok_or(redirect_when_error.clone())?;

    let mut campaign = campaigns::select_by_id(&app_state, form.id)
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
        campaign.name = name;
    }

    if let Some(game_id) = form.game_id {
        campaign.game_id = game_id;
    }

    if let Some(external_image_url) = form.external_image_url {
        campaign.external_image_url = if external_image_url.len() > 0 {
            Some(external_image_url)
        } else {
            None
        };
    }

    if let Some(description) = form.description {
        campaign.description = description;
    }

    if let Some(notes) = form.notes {
        campaign.notes = notes;
    }

    campaigns::update(&app_state, &updating_user, &campaign)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}&tab_name={}",
        form.id, form.tab_name
    )))
}

#[derive(Deserialize)]
pub struct DeleteCampaignForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<DeleteCampaignForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error = Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}&tab_name=info",
        form.id
    ));

    if let Some(connected_user) = profile {
        if campaigns::delete(&app_state, &connected_user, form.id)
            .await
            .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?
        {
            return Ok(Redirect::to("/role_playing_games/campaigns"));
        }
    }

    Err(redirect_when_error)
}
