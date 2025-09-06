use crate::app::templates::NavigationBar;
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RostersQueryParams {
    pub version: Option<Version>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/rosters_page.html")]
pub struct RostersPage {
    pub navigation_bar: NavigationBar,
    pub rosters: Vec<Roster>,
    pub version: Option<Version>,
}

impl RostersPage {
    pub fn from(app_state: AppState, profile: Option<User>, version: Option<Version>) -> Self {
        let mut ordered_rosters = Roster::list(version);
        ordered_rosters.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            rosters: ordered_rosters,
            version,
        }
    }
}
