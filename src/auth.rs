use crate::AppState;
use crate::app::templates::users::UserPage;
use crate::data::users::User;
use axum::extract::State;
use axum::routing::get;
use axum::{
    Router,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;

pub mod google;

pub const SESSION_ID: &str = "sid";

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(profile))
        .route("/sign_out", get(sign_out))
        .route("/google_callback", get(google::callback))
}

pub async fn profile(profile: Option<User>, State(app_state): State<AppState>) -> UserPage {
    UserPage::from(app_state, profile)
}

pub async fn sign_out(jar: PrivateCookieJar) -> impl IntoResponse {
    (
        jar.clone().remove(Cookie::build(SESSION_ID).path("/")),
        Redirect::to("/"),
    )
}
