use crate::app::templates::NavigationBar;
use crate::auth::UserProfile;
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
    profile: Option<UserProfile>,
}

impl UserPage {
    pub fn from(app_state: AppState, profile: Option<UserProfile>) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            profile,
        }
    }
}
