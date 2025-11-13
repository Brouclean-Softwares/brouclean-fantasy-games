use crate::app::templates::{blood_bowl, BreadCrumb, NavigationBar, UrlLink};
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::characteristics::Characteristic;
use blood_bowl_rs::positions::PositionDefinition;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Rosters", "/blood_bowl/rosters"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/rosters/rosters_page.html")]
pub struct RostersPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    rosters: Vec<Roster>,
    version: Version,
}

impl RostersPage {
    pub fn get(app_state: AppState, profile: Option<User>, version: Version) -> Self {
        let mut ordered_rosters = Roster::list(version);
        ordered_rosters.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: blood_bowl::breadcrumb(),
            profile,
            rosters: ordered_rosters,
            version,
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
}

impl RosterPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        version: Version,
        roster: Option<Roster>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            profile,
            roster,
            version,
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
