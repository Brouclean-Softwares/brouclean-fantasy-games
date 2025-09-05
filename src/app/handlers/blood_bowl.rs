use crate::AppState;
use axum::Router;

pub mod rosters;
pub mod teams;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .nest("/rosters", rosters::init_router())
        .nest("/teams", teams::init_router())
}
