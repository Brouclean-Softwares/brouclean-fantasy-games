use crate::app::templates::blood_bowl::teams::OwnedTeamsBlock;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;

pub mod competitions;
pub mod games;
pub mod players;
pub mod rosters;
pub mod teams;

pub fn breadcrumb() -> BreadCrumb {
    BreadCrumb::only_home().plus_link(UrlLink::from("Blood bowl", "/blood_bowl"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    owned_teams_block: OwnedTeamsBlock,
}

impl HomePage {
    pub async fn get(app_state: AppState, profile: User) -> Result<Self, AppError> {
        let owned_teams_block = OwnedTeamsBlock::get(&app_state, &profile.clone()).await?;

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile.clone())),
            breadcrumb: BreadCrumb::only_home(),
            owned_teams_block,
        })
    }
}
