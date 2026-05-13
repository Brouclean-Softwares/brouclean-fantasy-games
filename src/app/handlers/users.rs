use crate::AppState;
use crate::app::templates::users::UserPage;
use crate::data::users::User;
use axum::Router;
use axum::extract::{OriginalUri, State};
use axum::routing::get;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/user", get(user))
}

pub async fn user(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> UserPage {
    UserPage::from(app_state, profile, &uri)
}
