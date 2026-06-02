use crate::AppState;
use crate::app::templates::blood_bowl;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::users::MayBeUser;
use axum::Router;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;

pub mod competitions;
pub mod games;
pub mod players;
pub mod rosters;
pub mod stars;
pub mod statistics;
pub mod teams;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .nest("/competitions", competitions::init_router())
        .nest("/games", games::init_router())
        .nest("/players", players::init_router())
        .nest("/rosters", rosters::init_router())
        .nest("/stars", stars::init_router())
        .nest("/statistics", statistics::init_router())
        .nest("/teams", teams::init_router())
}

pub async fn home(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
) -> Result<blood_bowl::HomePage, Redirect> {
    let redirect_if_error = Redirect::to("/");

    let playing_games = crate::data::blood_bowl::games::select_all_playing(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    if let Some(connected_user) = profile {
        let scheduled_games = if let Some(user_id) = connected_user.id {
            crate::data::blood_bowl::games::select_scheduled_for_coach(&app_state, &user_id)
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?
        } else {
            Vec::new()
        };

        let owned_competitions = Competition::select_owned(&app_state, connected_user.clone())
            .await
            .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let owned_teams =
            crate::data::blood_bowl::teams::select_owned(&app_state, connected_user.clone())
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let home_page = blood_bowl::HomePage::get(
            &app_state,
            &connected_user,
            playing_games,
            scheduled_games,
            owned_competitions,
            owned_teams,
        )
        .await
        .or_else(|error| Err(error.log_and_redirect(redirect_if_error)))?;

        Ok(home_page)
    } else {
        Err(redirect_if_error)
    }
}
