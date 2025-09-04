use crate::errors::ApiError;
use crate::AppState;
use axum::routing::get;
use axum::{
    extract::FromRequestParts,
    response::{IntoResponse, Redirect},
    Router,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::PrivateCookieJar;
use core::fmt::Debug;
use http::request::Parts;
use serde::Deserialize;

pub mod google;

const SESSION_ID: &str = "sid";

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/sign_out", get(sign_out))
        .route("/google_callback", get(google::callback))
}

pub async fn sign_out(jar: PrivateCookieJar) -> impl IntoResponse {
    (
        jar.clone().remove(Cookie::build(SESSION_ID).path("/")),
        Redirect::to("/"),
    )
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct UserProfile {
    pub email: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for UserProfile {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();

        let cookie_jar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts.clone(), &state).await?;

        let Some(cookie) = cookie_jar
            .get(SESSION_ID)
            .map(|cookie| cookie.value().to_owned())
        else {
            return Err(ApiError::Unauthorized);
        };

        let res = sqlx::query_as::<_, UserProfile>(
            "SELECT
        users.email,
        users.name,
        users.given_name,
        users.family_name,
        users.picture
        FROM sessions
        LEFT JOIN USERS ON sessions.user_id = users.id
        WHERE sessions.session_id = $1
        LIMIT 1",
        )
        .bind(cookie)
        .fetch_one(&state.db)
        .await?;

        Ok(Self {
            email: res.email,
            name: res.name,
            given_name: res.given_name,
            family_name: res.family_name,
            picture: res.picture,
        })
    }
}
