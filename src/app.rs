use crate::AppState;
use axum::Router;
use axum::routing::get;

pub mod handlers;
pub mod templates;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .nest("/blood_bowl", handlers::blood_bowl::init_router())
        .nest(
            "/role_playing_games",
            handlers::role_playing_games::init_router(),
        )
        .nest("/users", handlers::users::init_router())
        .route("/", get(handlers::home_page))
}
