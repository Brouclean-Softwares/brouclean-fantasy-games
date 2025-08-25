use askama::Template;
use askama_web::WebTemplate;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};

use crate::auth::UserProfile;
use crate::AppState;

pub fn init_router(oauth_id: String) -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .route("/connected_user", get(connected_user))
        .layer(Extension(oauth_id))
}

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    oauth_id: String,
}

#[axum::debug_handler]
pub async fn home_page(Extension(oauth_id): Extension<String>) -> HomePage {
    HomePage { oauth_id }
}

pub async fn connected_user(profile: UserProfile) -> impl IntoResponse {
    (StatusCode::OK, profile.email)
}
