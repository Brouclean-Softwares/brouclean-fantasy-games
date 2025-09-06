use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;

pub mod blood_bowl;
pub mod users;

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    google_connection_url: String,
}

impl HomePage {
    pub fn from(app_state: AppState, profile: Option<User>) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            profile,
            google_connection_url: crate::auth::google::connection_url(app_state),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "navigation_bar.html")]
pub struct NavigationBar {
    profile: Option<User>,
    google_connection_url: String,
}

impl NavigationBar {
    pub fn from(app_state: &AppState, profile: &Option<User>) -> Self {
        Self {
            profile: profile.clone(),
            google_connection_url: crate::auth::google::connection_url(app_state.clone()),
        }
    }
}
