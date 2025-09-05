use crate::app::templates::HomePage;
use crate::auth::UserProfile;
use crate::AppState;
use axum::extract::State;

pub mod blood_bowl;
pub mod users;

pub async fn home_page(
    State(app_state): State<AppState>,
    profile: Option<UserProfile>,
) -> HomePage {
    HomePage::from(app_state, profile)
}
