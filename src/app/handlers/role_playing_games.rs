use crate::app::templates::role_playing_games;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

pub mod campaigns;
pub mod characters;
pub mod games;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .nest("/campaigns", campaigns::init_router())
        .nest("/characters", characters::init_router())
        .nest("/games", games::init_router())
}

pub async fn home(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<role_playing_games::HomePage, Redirect> {
    let redirect_if_error = Redirect::to("/");

    if let Some(connected_user) = profile {
        let owned_characters =
            crate::data::role_playing_games::characters::select_owned(&app_state, &connected_user)
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let owned_campaigns =
            crate::data::role_playing_games::campaigns::select_owned(&app_state, &connected_user)
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let games = crate::data::role_playing_games::games::select_all(&app_state)
            .await
            .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let home_page = role_playing_games::HomePage::get(
            &app_state,
            &connected_user,
            owned_characters,
            owned_campaigns,
            games,
        )
        .await
        .or_else(|_| Err(redirect_if_error.clone()))?;

        Ok(home_page)
    } else {
        Err(redirect_if_error.clone())
    }
}
