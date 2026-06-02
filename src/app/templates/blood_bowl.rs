use crate::AppState;
use crate::app::templates::blood_bowl::competitions::OwnedCompetitionsBlock;
use crate::app::templates::blood_bowl::games::GameCard;
use crate::app::templates::blood_bowl::games::GamesScheduleTable;
use crate::app::templates::blood_bowl::teams::OwnedTeamsBlock;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
use askama::Template;
use askama_web::WebTemplate;

pub mod competitions;
pub mod games;
pub mod players;
pub mod rosters;
pub mod stars;
pub mod statistics;
pub mod teams;

pub fn breadcrumb() -> BreadCrumb {
    BreadCrumb::only_home().plus_link(UrlLink::from("Blood bowl", "/blood_bowl"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    playing_games: Vec<GameSummary>,
    scheduled_games: Vec<GameSummary>,
    owned_competitions_block: Option<OwnedCompetitionsBlock>,
    owned_teams_block: OwnedTeamsBlock,
}

impl HomePage {
    pub async fn get(
        app_state: &AppState,
        profile: &User,
        playing_games: Vec<GameSummary>,
        scheduled_games: Vec<GameSummary>,
        owned_competitions: Vec<Competition>,
        owned_teams: Vec<TeamSummary>,
    ) -> Result<Self, AppError> {
        let owned_competitions_block = if owned_competitions.is_empty() {
            None
        } else {
            Some(OwnedCompetitionsBlock { owned_competitions })
        };

        let owned_teams_block = OwnedTeamsBlock { owned_teams };

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile.clone())),
            breadcrumb: BreadCrumb::only_home(),
            playing_games,
            scheduled_games,
            owned_competitions_block,
            owned_teams_block,
        })
    }
}
