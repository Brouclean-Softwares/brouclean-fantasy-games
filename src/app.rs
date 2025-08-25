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

#[axum::debug_handler]
pub async fn home_page(Extension(oauth_id): Extension<String>) -> HomePage {
    HomePage {
        google_connection_url: crate::auth::google::connection_url(oauth_id),
    }
}

pub async fn connected_user(profile: UserProfile) -> impl IntoResponse {
    (StatusCode::OK, profile.email)
}

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    google_connection_url: String,
}
