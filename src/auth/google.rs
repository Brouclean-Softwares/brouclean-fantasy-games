use crate::auth::{AuthRequest, UserProfile, SESSION_ID};
use crate::errors::ApiError;
use crate::{app, AppState};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use chrono::{Duration, Local};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use time::Duration as TimeDuration;

pub const REDIRECT_URL: &str = "http://localhost:8000/auth/google_callback";

pub async fn callback(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let token = app_state
        .google_oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await?;

    let profile = app_state
        .http_requester
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
    .fetch_one(&app_state.db)
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
    .execute(&app_state.db)
    .await?;

    Ok((
        jar.add(cookie),
        app::HomePage::from(app_state, Some(user_profile)),
    ))
}

pub fn build_oauth_client(client_id: String, client_secret: String) -> BasicClient {
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
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.to_string()).unwrap())
}

pub fn connection_url(app_state: AppState) -> String {
    let oauth_client = app_state.google_oauth_client;

    format!(
        "{}?scope=openid%20profile%20email&client_id={}&response_type=code&redirect_uri={}",
        oauth_client.auth_url().to_string(),
        oauth_client.client_id().to_string(),
        oauth_client.redirect_url().unwrap().to_string()
    )
}
