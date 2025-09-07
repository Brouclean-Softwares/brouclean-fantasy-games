use crate::app::templates::blood_bowl::rosters::roster_page::RosterPage;
use crate::app::templates::blood_bowl::rosters::rosters_page::RostersPage;
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(rosters))
        .route("/roster", get(roster))
}

#[derive(Deserialize)]
pub struct RostersQueryParams {
    pub version: Option<Version>,
}

pub async fn rosters(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<RostersQueryParams>,
) -> RostersPage {
    RostersPage::get(app_state, profile, params.version)
}

#[derive(Deserialize)]
pub struct RosterQueryParams {
    pub version: Option<Version>,
    pub roster: Option<Roster>,
}

pub async fn roster(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<RosterQueryParams>,
) -> RosterPage {
    RosterPage::get(app_state, profile, params.version, params.roster)
}
