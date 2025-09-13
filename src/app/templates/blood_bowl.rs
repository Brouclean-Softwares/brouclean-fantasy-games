use crate::app::templates::blood_bowl::teams::OwnedTeamsBlock;
use crate::app::templates::NavigationBar;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

pub mod rosters;
pub mod teams;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct OwnedTeamListRow {
    pub id: i32,
    pub version: Version,
    pub name: String,
    pub roster: Roster,
    pub value: i32,
    pub current_value: i32,
    pub external_logo_url: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    owned_teams_block: OwnedTeamsBlock,
}

impl HomePage {
    pub async fn get(app_state: AppState, profile: User) -> Result<Self, AppError> {
        let owned_teams_block = OwnedTeamsBlock::get(&app_state, &profile.clone()).await?;

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile.clone())),
            owned_teams_block,
        })
    }
}
