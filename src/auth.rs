use crate::AppState;
use crate::app::templates::users::UserPage;
use crate::data::users::MayBeUser;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{
    Form, Router,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Deserialize;

pub mod google;

pub const SESSION_ID: &str = "sid";
pub const REDIRECT_URI_AFTER_AUTH: &str = "redir_auth";

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(profile))
        .route("/sign_in", post(sign_in))
        .route("/sign_out", get(sign_out))
        .route("/google_callback", get(google::callback))
}

pub async fn profile(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
) -> impl IntoResponse {
    UserPage::from(app_state, profile).into_response()
}

#[derive(Deserialize)]
pub struct SignInForm {
    pub redirection_uri: String,
}

pub async fn sign_in(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    jar: PrivateCookieJar,
    Form(form): Form<SignInForm>,
) -> impl IntoResponse {
    let redirection_uri_when_connected = form.redirection_uri;

    if profile.is_some() {
        Redirect::to(&redirection_uri_when_connected).into_response()
    } else {
        let url = google::connection_url(&app_state);

        let jar = jar.add(Cookie::new(
            REDIRECT_URI_AFTER_AUTH,
            redirection_uri_when_connected,
        ));

        (jar, Redirect::to(&url)).into_response()
    }
}

pub async fn sign_out(jar: PrivateCookieJar) -> impl IntoResponse {
    (
        jar.clone().remove(Cookie::build(SESSION_ID).path("/")),
        Redirect::to("/"),
    )
}
