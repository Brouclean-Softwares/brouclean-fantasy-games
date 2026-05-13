use crate::AppState;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink, blood_bowl};
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::characteristics::Characteristic;
use blood_bowl_rs::positions::{Position, PositionDefinition};
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use http::Uri;
use std::collections::HashMap;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Rosters", "/blood_bowl/rosters"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/rosters_page.html")]
pub struct RostersPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    versions: Vec<Version>,
    rosters_by_version: HashMap<Version, Vec<Roster>>,
}

impl RostersPage {
    pub fn get(app_state: AppState, profile: Option<User>, uri: &Uri) -> Self {
        let mut versions = Version::list();
        versions.reverse();

        let mut rosters_by_version = HashMap::with_capacity(versions.len());

        for version in versions.iter() {
            let mut ordered_rosters = Roster::list(version.clone());
            ordered_rosters.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

            rosters_by_version.insert(version.clone(), ordered_rosters);
        }

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: blood_bowl::breadcrumb(),
            versions,
            rosters_by_version,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/roster_page.html")]
pub struct RosterPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    roster: Option<Roster>,
    version: Version,
    stars_available: Vec<Position>,
    mega_stars_available: Vec<Position>,
}

impl RosterPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        version: Version,
        roster: Option<Roster>,
        stars_available: Vec<Position>,
        mega_stars_available: Vec<Position>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: breadcrumb(),
            profile,
            roster,
            version,
            stars_available,
            mega_stars_available,
        }
    }
}

fn characteristic_value_into_html(value: Option<u8>, str_after_value: &str) -> String {
    if let Some(value) = value {
        format!("{}{}", value, str_after_value)
    } else {
        "-".to_string()
    }
}

pub fn movement_allowance_html(position_definition: &PositionDefinition) -> String {
    characteristic_value_into_html(
        position_definition.characteristic_value(Characteristic::MovementAllowance),
        "",
    )
}

pub fn strength_html(position_definition: &PositionDefinition) -> String {
    characteristic_value_into_html(
        position_definition.characteristic_value(Characteristic::Strength),
        "",
    )
}

pub fn agility_html(position_definition: &PositionDefinition) -> String {
    characteristic_value_into_html(
        position_definition.characteristic_value(Characteristic::Agility),
        "+",
    )
}

pub fn passing_ability_html(position_definition: &PositionDefinition) -> String {
    characteristic_value_into_html(
        position_definition.characteristic_value(Characteristic::PassingAbility),
        "+",
    )
}

pub fn armour_value_html(position_definition: &PositionDefinition) -> String {
    characteristic_value_into_html(
        position_definition.characteristic_value(Characteristic::ArmourValue),
        "+",
    )
}
