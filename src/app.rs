use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .route("/connected_user", get(connected_user))
}

#[axum::debug_handler]
pub async fn home_page(State(app_state): State<AppState>) -> HomePage {
    HomePage {
        google_connection_url: crate::auth::google::connection_url(app_state),
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
