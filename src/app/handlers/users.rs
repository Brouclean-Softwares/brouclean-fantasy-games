use crate::AppState;
use crate::app::templates::users::UserPage;
use crate::data::users::MayBeUser;
use axum::Router;
use axum::extract::State;
use axum::routing::get;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/user", get(user))
}

pub async fn user(State(app_state): State<AppState>, MayBeUser(profile): MayBeUser) -> UserPage {
    UserPage::from(app_state, profile)
}
