use crate::app::NavigationBar;
use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
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

pub async fn rosters(
    profile: Option<UserProfile>,
    State(app_state): State<AppState>,
    Query(params): Query<RostersQueryParams>,
) -> RostersPage {
    RostersPage::from(app_state, profile, params.version)
}

pub async fn roster(
    profile: Option<UserProfile>,
    State(app_state): State<AppState>,
    Query(params): Query<RosterQueryParams>,
) -> RosterPage {
    RosterPage::from(app_state, profile, params.version, params.roster)
}

#[derive(Deserialize)]
pub struct RostersQueryParams {
    version: Option<Version>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters.html")]
pub struct RostersPage {
    navigation_bar: NavigationBar,
    rosters: Vec<Roster>,
    version: Option<Version>,
}

impl RostersPage {
    pub fn from(
        app_state: AppState,
        profile: Option<UserProfile>,
        version: Option<Version>,
    ) -> Self {
        let mut ordered_rosters = Roster::list(version);
        ordered_rosters.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            rosters: ordered_rosters,
            version,
        }
    }
}

#[derive(Deserialize)]
pub struct RosterQueryParams {
    version: Option<Version>,
    roster: Roster,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/roster.html")]
pub struct RosterPage {
    navigation_bar: NavigationBar,
    roster: Roster,
    version: Option<Version>,
}

impl RosterPage {
    pub fn from(
        app_state: AppState,
        profile: Option<UserProfile>,
        version: Option<Version>,
        roster: Roster,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            roster,
            version,
        }
    }
}
