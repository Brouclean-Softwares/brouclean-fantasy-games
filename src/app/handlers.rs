use crate::AppState;
use crate::app::templates::HomePage;
use crate::data::users::MayBeUser;
use axum::extract::State;

pub mod blood_bowl;
pub mod role_playing_games;
pub mod users;

pub async fn home_page(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
) -> HomePage {
    HomePage::get(app_state, profile).await
}
