use crate::app::templates::blood_bowl::statistics::StatisticsPage;
use crate::app::templates::{blood_bowl, NavigationBar};
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamsTopStatistics;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(statistics))
}

pub async fn statistics(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<StatisticsPage, Redirect> {
    let error_redirect = |error| {
        tracing::debug!("Error : {}", error);
        Redirect::to("/blood_bowl")
    };

    let teams_top_statistics = TeamsTopStatistics::global(&app_state)
        .await
        .map_err(error_redirect)?;

    let players_top_statistics = PlayersTopStatistics::global(&app_state)
        .await
        .map_err(error_redirect)?;

    Ok(StatisticsPage {
        navigation_bar: NavigationBar::get(&app_state, &profile),
        breadcrumb: blood_bowl::breadcrumb(),
        teams_top_statistics: teams_top_statistics.into(),
        players_top_statistics: players_top_statistics.into(),
    })
}
