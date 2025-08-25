use axum::routing::get;
use axum::{
    extract::{FromRequest, FromRequestParts, Query, Request, State},
    response::{IntoResponse, Redirect},
    Extension, Router,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use chrono::{Duration, Local};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use time::Duration as TimeDuration;

use crate::errors::ApiError;
use crate::{app, AppState};

const SESSION_ID: &str = "sid";

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/sign_out", get(sign_out))
        .route("/google_callback", get(google_callback))
}

pub async fn sign_out(jar: PrivateCookieJar) -> impl IntoResponse {
    (
        jar.clone().remove(Cookie::build(SESSION_ID).path("/")),
        Redirect::to("/"),
    )
}

pub async fn google_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
    Extension(oauth_client): Extension<BasicClient>,
) -> Result<impl IntoResponse, ApiError> {
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await?;

    let profile = state
        .ctx
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(token.access_token().secret().to_owned())
        .send()
        .await?;

    let profile = profile.json::<UserProfile>().await?;

    let Some(secs) = token.expires_in() else {
        return Err(ApiError::OptionError);
    };

    let secs: i64 = secs.as_secs().try_into()?;

    let max_age = Local::now().naive_local() + Duration::try_seconds(secs).unwrap();

    let cookie = Cookie::build((SESSION_ID, token.access_token().secret().to_owned()))
        .same_site(SameSite::Strict)
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(secs));

    let user_profile: UserProfile = sqlx::query_as(
        "INSERT INTO users (email, name, given_name, family_name, picture)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (email) DO UPDATE SET
        name = excluded.name,
        given_name = excluded.given_name,
        family_name = excluded.family_name,
        picture = excluded.picture,
        last_updated = CURRENT_TIMESTAMP
        RETURNING users.email, users.name, given_name, family_name, users.picture",
    )
    .bind(profile.email.clone())
    .bind(profile.name.clone())
    .bind(profile.given_name.clone())
    .bind(profile.family_name.clone())
    .bind(profile.picture.clone())
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO sessions (user_id, session_id, expires_at) VALUES (
        (SELECT ID FROM USERS WHERE email = $1 LIMIT 1),
         $2, $3)
        ON CONFLICT (user_id) DO UPDATE SET
        session_id = excluded.session_id,
        expires_at = excluded.expires_at",
    )
    .bind(profile.email)
    .bind(token.access_token().secret().to_owned())
    .bind(max_age)
    .execute(&state.db)
    .await?;

    Ok((jar.add(cookie), app::connected_user(user_profile).await))
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
impl FromRequest<AppState> for UserProfile {
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let (mut parts, _body) = req.into_parts();
        let cookiejar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts, &state).await?;

        let Some(cookie) = cookiejar
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

pub fn build_oauth_client(client_id: String, client_secret: String) -> BasicClient {
    let redirect_url = "http://localhost:8000/auth/google_callback".to_string();

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}
