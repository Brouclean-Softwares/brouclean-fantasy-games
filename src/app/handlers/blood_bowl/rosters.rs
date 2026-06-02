use crate::AppState;
use crate::app::templates::blood_bowl::rosters::{RosterPage, RostersPage};
use crate::data::users::MayBeUser;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(rosters))
        .route("/roster", get(roster))
}

pub async fn rosters(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
) -> RostersPage {
    RostersPage::get(app_state, profile)
}

#[derive(Deserialize)]
pub struct RosterQueryParams {
    pub version: Option<Version>,
    pub roster: Option<Roster>,
}

pub async fn roster(
    State(app_state): State<AppState>,
    MayBeUser(profile): MayBeUser,
    Query(params): Query<RosterQueryParams>,
) -> RosterPage {
    let version = params.version.unwrap_or(Version::LAST_VERSION);

    let mut stars_available = blood_bowl_rs::stars::star_position_list(&version);

    let mut mega_stars_available = blood_bowl_rs::stars::mega_star_position_list(&version);

    if let Some(roster) = &params.roster {
        stars_available.retain(|star| {
            blood_bowl_rs::stars::star_maximum_for_roster(star, roster, &version) > 0
        });

        stars_available.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

        mega_stars_available.retain(|star| {
            blood_bowl_rs::stars::star_maximum_for_roster(star, roster, &version) > 0
        });

        mega_stars_available.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));
    }

    RosterPage::get(
        app_state,
        profile,
        version,
        params.roster,
        stars_available,
        mega_stars_available,
    )
}
