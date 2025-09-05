use crate::app::templates::NavigationBar;
use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RosterQueryParams {
    pub version: Option<Version>,
    pub roster: Option<Roster>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/roster_page.html")]
pub struct RosterPage {
    navigation_bar: NavigationBar,
    roster: Option<Roster>,
    version: Option<Version>,
}

impl RosterPage {
    pub fn from(
        app_state: AppState,
        profile: Option<UserProfile>,
        version: Option<Version>,
        roster: Option<Roster>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            roster,
            version,
        }
    }
}
