use crate::AppState;
use crate::app::templates::role_playing_games::campaigns::NarrativeArcPage;
use crate::data::role_playing_games::campaigns;
use crate::data::role_playing_games::campaigns::arcs;
use crate::data::users::User;
use axum::Form;
use axum::extract::{Query, State};
use axum::response::Redirect;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewNarrativeArcForm {
    pub name: String,
    pub campaign_id: i32,
}

pub async fn new(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<NewNarrativeArcForm>,
) -> Result<Redirect, Redirect> {
    let redirect = Redirect::to(&format!(
        "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
        form.campaign_id
    ));

    let profile = profile.ok_or(redirect.clone())?;

    let campaign = campaigns::select_by_id(&app_state, form.campaign_id)
        .await
        .map_err(|error| error.log_and_redirect(Redirect::to("/role_playing_games/campaigns")))?;

    let _ = campaigns::arcs::push_new_into_campaign(&app_state, &profile, &campaign, form.name)
        .await
        .map_err(|error| error.log_and_redirect(redirect.clone()))?;

    Ok(redirect)
}

#[derive(Deserialize)]
pub struct NarrativeArcQueryParams {
    pub id: i32,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn arc(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<NarrativeArcQueryParams>,
) -> Result<NarrativeArcPage, Redirect> {
    let redirect_if_error = Redirect::to("/role_playing_games/campaigns");

    let arc = arcs::select_by_id(&app_state, params.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let campaign = campaigns::select_by_id(&app_state, arc.campaign_id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let is_owner = match (&campaign.game_master, &profile) {
        (Some(owner), Some(connected_user)) => owner.id.eq(&connected_user.id),
        (_, _) => false,
    };

    Ok(NarrativeArcPage::get(
        app_state,
        profile.clone(),
        arc,
        is_owner,
        params.edit.unwrap_or(false) && profile.is_some(),
        params.field_edited,
    ))
}

#[derive(Deserialize)]
pub struct UpdateNarrativeArcForm {
    pub id: i32,
    pub name: Option<String>,
    pub external_image_url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<UpdateNarrativeArcForm>,
) -> Result<Redirect, Redirect> {
    let redirect_when_error =
        Redirect::to(&format!("/role_playing_games/campaigns/arc?id={}", form.id));

    let mut arc = arcs::select_by_id(&app_state, form.id)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    let updating_user = profile.ok_or(redirect_when_error.clone())?;

    let campaign = campaigns::select_by_id(&app_state, arc.campaign_id)
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
        arc.name = name;
    }

    if let Some(external_image_url) = form.external_image_url {
        arc.external_image_url = if external_image_url.len() > 0 {
            Some(external_image_url)
        } else {
            None
        };
    }

    if let Some(description) = form.description {
        arc.description = description;
    }

    if let Some(notes) = form.notes {
        arc.notes = notes;
    }

    arcs::update(&app_state, &updating_user, &arc)
        .await
        .map_err(|error| error.log_and_redirect(redirect_when_error.clone()))?;

    Ok(Redirect::to(&format!(
        "/role_playing_games/campaigns/arc?id={}",
        form.id
    )))
}
