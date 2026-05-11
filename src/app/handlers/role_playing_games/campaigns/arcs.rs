use crate::data::role_playing_games::campaigns;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::Form;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewArcForm {
    pub name: String,
    pub campaign_id: i32,
}

pub async fn new_arc(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<NewArcForm>,
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
