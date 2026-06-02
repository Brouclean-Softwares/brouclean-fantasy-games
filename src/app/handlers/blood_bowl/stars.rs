use crate::AppState;
use crate::app::templates::blood_bowl::stars::{StarPage, StarsPage};
use crate::data::users::MayBeUser;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(stars))
        .route("/star", get(star))
}

pub async fn stars(State(app_state): State<AppState>, MayBeUser(profile): MayBeUser) -> StarsPage {
    StarsPage::get(app_state, profile)
}

#[derive(Deserialize)]
pub struct StarQueryParams {
    pub version: Option<Version>,
    pub star: Option<Position>,
}

pub async fn star(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Query(params): Query<StarQueryParams>,
) -> StarPage {
    let version = params.version.unwrap_or(Version::LAST_VERSION);

    let mut rosters_available = Roster::list(version.clone());

    if let Some(star_position) = &params.star {
        rosters_available.retain(|roster| {
            blood_bowl_rs::stars::star_maximum_for_roster(star_position, roster, &version) > 0
        });

        rosters_available.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));
    }

    StarPage::get(app_state, profile, version, params.star, rosters_available)
}
