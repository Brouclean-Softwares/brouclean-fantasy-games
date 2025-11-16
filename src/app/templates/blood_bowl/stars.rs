use crate::app::templates::{blood_bowl, BreadCrumb, NavigationBar, UrlLink};
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use std::collections::HashMap;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Stars", "/blood_bowl/stars"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/stars/stars_page.html")]
pub struct StarsPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    versions: Vec<Version>,
    stars_by_version: HashMap<Version, Vec<Position>>,
}

impl StarsPage {
    pub fn get(app_state: AppState, profile: Option<User>) -> Self {
        let mut versions = Version::list();
        versions.reverse();

        let mut stars_by_version = HashMap::with_capacity(versions.len());

        for version in versions.iter() {
            let mut ordered_stars = blood_bowl_rs::stars::star_position_list(version);
            ordered_stars.sort_by(|a, b| a.name("fr").cmp(&b.name("fr")));

            stars_by_version.insert(version.clone(), ordered_stars);
        }

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: blood_bowl::breadcrumb(),
            versions,
            stars_by_version,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/stars/star_page.html")]
pub struct StarPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    star: Option<Position>,
    version: Version,
    rosters_available: Vec<Roster>,
}

impl StarPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        version: Version,
        star: Option<Position>,
        rosters_available: Vec<Roster>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            star,
            version,
            rosters_available,
        }
    }
}
