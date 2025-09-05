use crate::app::templates::users::user_page::{UserPage, UserQueryParams};
use crate::auth::UserProfile;
use crate::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;

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
