use crate::app::templates::NavigationBar;
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserQueryParams {}

#[derive(Template, WebTemplate)]
#[template(path = "users/user_page.html")]
pub struct UserPage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
}

impl UserPage {
    pub fn from(app_state: AppState, profile: Option<User>) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            profile,
        }
    }
}
