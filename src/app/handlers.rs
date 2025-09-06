use crate::app::templates::HomePage;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;

pub mod blood_bowl;
pub mod users;

pub async fn home_page(State(app_state): State<AppState>, profile: Option<User>) -> HomePage {
    HomePage::get(app_state, profile)
}
