use crate::AppState;
use crate::app::templates::blood_bowl::statistics::StatisticsPage;
use crate::app::templates::{NavigationBar, blood_bowl};
use crate::data::blood_bowl::coaches;
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamsTopStatistics;
use crate::data::users::MayBeUser;
use crate::errors::AppError;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(statistics))
}

#[derive(Deserialize)]
pub struct StatisticsQueryParams {
    pub tab_name: Option<String>,
}

pub async fn statistics(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Query(params): Query<StatisticsQueryParams>,
) -> Result<StatisticsPage, Redirect> {
    let error_redirect = |error: AppError| error.log_and_redirect(Redirect::to("/blood_bowl"));

    let tab_name = params.tab_name.unwrap_or("teams".to_string());

    let teams_top_statistics = if tab_name.eq("teams") {
        Some(
            TeamsTopStatistics::global(&app_state)
                .await
                .map_err(error_redirect)?
                .into(),
        )
    } else {
        None
    };

    let players_top_statistics = if tab_name.eq("players") {
        Some(
            PlayersTopStatistics::global(&app_state)
                .await
                .map_err(error_redirect)?
                .into(),
        )
    } else {
        None
    };

    let coaches_elo_ranking = if tab_name.eq("coaches") {
        Some(
            coaches::select_elo_ranking(&app_state)
                .await
                .map_err(error_redirect)?
                .into(),
        )
    } else {
        None
    };

    Ok(StatisticsPage {
        navigation_bar: NavigationBar::get(&app_state, &profile),
        breadcrumb: blood_bowl::breadcrumb(),
        tab_name,
        teams_top_statistics,
        players_top_statistics,
        coaches_elo_ranking,
    })
}
