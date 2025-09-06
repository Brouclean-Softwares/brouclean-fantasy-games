use crate::app::templates::blood_bowl::teams::teams_page::TeamsPage;
use crate::data::blood_bowl::teams::Team;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(teams))
    //.route("/roster", get(team))
}

pub async fn teams(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<TeamsPage, AppError> {
    let teams = Team::select_all(&app_state).await?;

    Ok(TeamsPage::get(app_state, profile, teams))
}
