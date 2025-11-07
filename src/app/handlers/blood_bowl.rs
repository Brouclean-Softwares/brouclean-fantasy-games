use crate::app::templates::blood_bowl;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

pub mod competitions;
pub mod games;
pub mod players;
pub mod rosters;
pub mod teams;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .nest("/competitions", competitions::init_router())
        .nest("/games", games::init_router())
        .nest("/players", players::init_router())
        .nest("/rosters", rosters::init_router())
        .nest("/teams", teams::init_router())
}

pub async fn home(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<blood_bowl::HomePage, Redirect> {
    if let Some(connected_user) = profile {
        let home_page = blood_bowl::HomePage::get(&app_state, &connected_user)
            .await
            .or_else(|_| Err(Redirect::to("/")))?;

        Ok(home_page)
    } else {
        Err(Redirect::to("/"))
    }
}
