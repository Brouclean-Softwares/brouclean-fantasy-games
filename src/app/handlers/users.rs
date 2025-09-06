use crate::app::templates::users::user_page::UserPage;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/user", get(user))
}

pub async fn user(State(app_state): State<AppState>, profile: Option<User>) -> UserPage {
    UserPage::from(app_state, profile)
}
