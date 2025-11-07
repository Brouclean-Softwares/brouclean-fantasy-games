use crate::app::templates::HomePage;
use crate::auth::SESSION_ID;
use crate::data::sessions::Session;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use chrono::{Duration, Local};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use shuttle_runtime::SecretStore;
use time::Duration as TimeDuration;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

pub async fn callback(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
) -> Result<impl IntoResponse, AppError> {
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

    let profile = profile.json::<User>().await?;

    let Some(secs) = token.expires_in() else {
        return Err(AppError::OptionError);
    };

    let secs: i64 = secs.as_secs().try_into()?;

    let max_age = Local::now().naive_local() + Duration::try_seconds(secs).unwrap();

    let cookie = Cookie::build((SESSION_ID, token.access_token().secret().to_owned()))
        .same_site(SameSite::Strict)
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(secs));

    let user_profile: User = profile.upsert(&app_state).await?;

    Session::upsert(
        &app_state,
        profile.email,
        token.access_token().secret().to_owned(),
        max_age,
    )
    .await?;

    Ok((
        jar.add(cookie),
        HomePage::get(app_state, Some(user_profile)).await,
    ))
}

pub fn build_oauth_client(secrets: &SecretStore) -> BasicClient {
    let client_id = secrets.get("GOOGLE_OAUTH_CLIENT_ID").unwrap();
    let client_secret = secrets.get("GOOGLE_OAUTH_CLIENT_SECRET").unwrap();
    let app_url = secrets.get("APP_URL").unwrap();

    let redirect_url = format!("{}/auth/google_callback", app_url);

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

pub fn connection_url(app_state: AppState) -> String {
    let oauth_client = app_state.google_oauth_client;

    format!(
        "{}?scope=openid%20profile%20email&client_id={}&response_type=code&redirect_uri={}",
        oauth_client.auth_url().to_string(),
        oauth_client.client_id().to_string(),
        oauth_client.redirect_url().unwrap().to_string()
    )
}
