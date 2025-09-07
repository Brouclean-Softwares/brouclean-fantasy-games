use crate::app::templates::blood_bowl::teams::teams_page::TeamsPage;
use crate::data::blood_bowl::teams::BBTeam;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(teams))
        .route("/team", get(get_team))
}

pub async fn teams(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<TeamsPage, AppError> {
    let teams = BBTeam::select_all(&app_state).await?;

    Ok(TeamsPage::get(app_state, profile, teams))
}

#[derive(Deserialize)]
pub struct TeamQueryParams {
    pub id: Option<i32>,
}

pub async fn get_team() {}
