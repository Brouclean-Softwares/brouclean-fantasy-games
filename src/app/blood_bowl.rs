use crate::AppState;
use axum::Router;

pub mod rosters;

pub fn init_router() -> Router<AppState> {
    Router::new().nest("/rosters", rosters::init_router())
}
