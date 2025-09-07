use crate::app::templates::NavigationBar;
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/roster_page.html")]
pub struct RosterPage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    roster: Option<Roster>,
    version: Option<Version>,
}

impl RosterPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        version: Option<Version>,
        roster: Option<Roster>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            profile,
            roster,
            version,
        }
    }
}
