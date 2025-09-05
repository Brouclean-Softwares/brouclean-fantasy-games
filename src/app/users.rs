use crate::app::NavigationBar;
use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/user", get(user))
}

pub async fn user(
    profile: Option<UserProfile>,
    State(app_state): State<AppState>,
    Query(_params): Query<UserQueryParams>,
) -> UserPage {
    UserPage::from(app_state, profile)
}

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
