use crate::app::templates::blood_bowl::teams::{TeamCard, TeamSelector};
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::teams::Team;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_page.html")]
pub struct GamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    game: Game,
    team_a_card: TeamCard,
    team_b_card: TeamCard,
}

impl GamePage {
    pub fn get(app_state: AppState, profile: Option<User>, game: Game) -> Result<Self, AppError> {
        Self::get_with_message(app_state, profile, None, game)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        game: Game,
    ) -> Result<Self, AppError> {
        let team_a = game.first_team.clone();
        let team_b = game.second_team.clone();

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            game,
            team_a_card: TeamCard::get(team_a),
            team_b_card: TeamCard::get(team_b),
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/new_game_page.html")]
pub struct NewGamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    team_a_id: i32,
    team_b_id: i32,
    team_a_card: Option<TeamCard>,
    team_b_card: Option<TeamCard>,
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
        alert_message: Option<AlertMessage>,
        team_a: Option<Team>,
        team_b: Option<Team>,
    ) -> Self {
        let team_a_id = team_a.clone().and_then(|team| Some(team.id)).unwrap_or(-1);
        let team_b_id = team_b.clone().and_then(|team| Some(team.id)).unwrap_or(-1);
        let team_a_card = team_a.and_then(|team| Some(TeamCard::get_with_details(team, true)));
        let team_b_card = team_b.and_then(|team| Some(TeamCard::get_with_details(team, true)));

        Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile)),
            alert_message,
            team_a_id,
            team_b_id,
            team_a_card,
            team_b_card,
            team_a_selector: TeamSelector::get("team_a_id".to_string()),
            team_b_selector: TeamSelector::get("team_b_id".to_string()),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_card.html")]
pub struct GameCard {
    game: Game,
}
