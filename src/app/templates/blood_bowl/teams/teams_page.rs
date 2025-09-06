use crate::app::templates::NavigationBar;
use crate::data::blood_bowl::teams::Team;
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/teams_page.html")]
pub struct TeamsPage {
    navigation_bar: NavigationBar,
    teams: Vec<Team>,
}

impl TeamsPage {
    pub fn get(app_state: AppState, profile: Option<User>, teams: Vec<Team>) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            teams,
        }
    }
}
