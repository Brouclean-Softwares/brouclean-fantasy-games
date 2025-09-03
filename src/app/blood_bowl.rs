use crate::app::NavigationBar;
use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::routing::get;
use axum::Router;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/rosters", get(rosters))
}

pub async fn rosters(
    State(app_state): State<AppState>,
    profile: Option<UserProfile>,
) -> RostersPage {
    RostersPage::from(app_state, profile)
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters.html")]
pub struct RostersPage {
    navigation_bar: NavigationBar,
    rosters: Vec<Roster>,
}

impl RostersPage {
    pub fn from(app_state: AppState, profile: Option<UserProfile>) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            rosters: Roster::list(Version::V5),
        }
    }
}
