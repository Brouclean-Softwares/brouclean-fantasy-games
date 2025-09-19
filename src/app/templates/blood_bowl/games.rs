use crate::app::templates::blood_bowl::teams::{TeamCard, TeamSelector};
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::teams::Team;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_page.html")]
pub struct GamePage {
    navigation_bar: NavigationBar,
    game: Game,
}

impl GamePage {
    pub fn get(app_state: AppState, profile: User, game: Game) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile)),
            game,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/new_game_page.html")]
pub struct NewGamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    team_a_id: i32,
    team_b_id: i32,
    team_a: Option<TeamCard>,
    team_b: Option<TeamCard>,
    team_a_selector: TeamSelector,
    team_b_selector: TeamSelector,
}

impl NewGamePage {
    pub fn get(
        app_state: AppState,
        profile: User,
        team_a: Option<Team>,
        team_b: Option<Team>,
    ) -> Self {
        Self::get_with_message(app_state, profile, None, team_a, team_b)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: User,
        message: Option<AlertMessage>,
        team_a: Option<Team>,
        team_b: Option<Team>,
    ) -> Self {
        let team_a_id = team_a.clone().and_then(|team| team.id).unwrap_or(-1);
        let team_b_id = team_b.clone().and_then(|team| team.id).unwrap_or(-1);

        Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile)),
            alert_message: message,
            team_a_id,
            team_b_id,
            team_a: TeamCard::get_with_details(team_a, true),
            team_b: TeamCard::get_with_details(team_b, true),
            team_a_selector: TeamSelector::get("team_a_id".to_string()),
            team_b_selector: TeamSelector::get("team_b_id".to_string()),
        }
    }
}
