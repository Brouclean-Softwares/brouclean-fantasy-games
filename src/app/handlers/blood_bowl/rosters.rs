use crate::app::templates::blood_bowl::rosters::roster_page::{RosterPage, RosterQueryParams};
use crate::app::templates::blood_bowl::rosters::rosters_page::{RostersPage, RostersQueryParams};
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(rosters))
        .route("/roster", get(roster))
}

pub async fn rosters(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<RostersQueryParams>,
) -> RostersPage {
    RostersPage::from(app_state, profile, params.version)
}

pub async fn roster(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<RosterQueryParams>,
) -> RosterPage {
    RosterPage::from(app_state, profile, params.version, params.roster)
}
