use crate::auth::UserProfile;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Extension;

#[axum::debug_handler]
pub async fn homepage(Extension(oauth_id): Extension<String>) -> Html<String> {
    Html(format!(
        r#"
        <p>Welcome!</p>
        <a href="https://accounts.google.com/o/oauth2/v2/auth?scope=openid%20profile%20email&client_id={oauth_id}&response_type=code&redirect_uri=http://localhost:8000/auth/google_callback">
            Click to sign into Google!
        </a>
    "#
    ))
}

pub async fn protected(profile: UserProfile) -> impl IntoResponse {
    (StatusCode::OK, profile.email)
}
