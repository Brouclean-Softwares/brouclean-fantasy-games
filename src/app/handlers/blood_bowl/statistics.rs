use crate::AppState;
use crate::app::templates::blood_bowl::statistics::StatisticsPage;
use crate::app::templates::{NavigationBar, blood_bowl};
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamsTopStatistics;
use crate::data::users::User;
use crate::errors::AppError;
use axum::Router;
use axum::extract::{OriginalUri, State};
use axum::response::Redirect;
use axum::routing::get;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(statistics))
}

pub async fn statistics(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<StatisticsPage, Redirect> {
    let error_redirect = |error: AppError| error.log_and_redirect(Redirect::to("/blood_bowl"));

    let teams_top_statistics = TeamsTopStatistics::global(&app_state)
        .await
        .map_err(error_redirect)?;

    let players_top_statistics = PlayersTopStatistics::global(&app_state)
        .await
        .map_err(error_redirect)?;

    Ok(StatisticsPage {
        navigation_bar: NavigationBar::get(&app_state, &profile, &uri),
        breadcrumb: blood_bowl::breadcrumb(),
        teams_top_statistics: teams_top_statistics.into(),
        players_top_statistics: players_top_statistics.into(),
    })
}
