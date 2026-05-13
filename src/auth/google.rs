use crate::AppState;
use crate::auth::{REDIRECT_URI_AFTER_AUTH, SESSION_ID};
use crate::data::sessions::Session;
use crate::data::users::User;
use crate::errors::AppError;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
    basic::BasicClient, reqwest::async_http_client,
};
use serde::Deserialize;
use std::env;

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

    let access_token = token.access_token().secret();
    let _refresh_token = token.refresh_token().and_then(|token| Some(token.secret()));

    let profile = app_state
        .http_requester
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token.to_owned())
        .send()
        .await?;

    let profile = profile.json::<User>().await?;

    let cookie = Cookie::build((SESSION_ID, access_token.to_owned()))
        .same_site(SameSite::Lax)
        .path("/")
        .secure(true)
        .http_only(true);

    profile.upsert(&app_state).await?;

    Session::upsert(
        &app_state,
        profile.email,
        token.access_token().secret().to_owned(),
    )
    .await?;

    let redirect_uri = jar
        .get(REDIRECT_URI_AFTER_AUTH)
        .map(|c| c.value().to_string())
        .unwrap_or("/".to_string());

    let redirect_uri = if redirect_uri.starts_with('/') {
        redirect_uri
    } else {
        "/".into()
    };

    let jar = jar.remove(Cookie::from(REDIRECT_URI_AFTER_AUTH));

    Ok((jar.add(cookie), Redirect::to(&redirect_uri)))
}

pub fn build_oauth_client() -> BasicClient {
    let client_id = env::var("GOOGLE_OAUTH_CLIENT_ID").expect("GOOGLE_OAUTH_CLIENT_ID must be set");

    let client_secret =
        env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("GOOGLE_OAUTH_CLIENT_SECRET must be set");

    let app_url = env::var("APP_URL").expect("APP_URL must be set");

    let redirect_url = format!("{}/auth/google_callback", app_url);

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
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
        "{}?scope=openid%20profile%20email&client_id={}&response_type=code&access_type=offline&redirect_uri={}",
        oauth_client.auth_url().to_string(),
        oauth_client.client_id().to_string(),
        oauth_client.redirect_url().unwrap().to_string()
    )
}
