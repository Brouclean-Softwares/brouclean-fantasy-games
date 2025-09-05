use crate::AppState;
use axum::routing::get;
use axum::Router;

pub mod handlers;
pub mod templates;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .nest("/blood_bowl", handlers::blood_bowl::init_router())
        .nest("/users", handlers::users::init_router())
        .route("/", get(handlers::home_page))
}
