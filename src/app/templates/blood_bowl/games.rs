use crate::app::templates::blood_bowl::teams::{TeamCard, TeamSelector};
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::{TranslatedName, TypeName};

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_page.html")]
pub struct GamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    game: Game,
    editable: bool,
    edit_mode: bool,
}

impl GamePage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        game: Game,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        Self::get_with_message(app_state, profile, None, game, edit_mode)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        game: Game,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        let mut editable = false;

        if let Some(connected_user) = profile.clone() {
            editable = connected_user.is_option_coach(&game.created_by)
                || connected_user.is_coach(&game.first_team.coach)
                || connected_user.is_coach(&game.second_team.coach);
        }

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            game,
            editable,
            edit_mode,
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/new_game_page.html")]
pub struct NewGamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    first_team_id: i32,
    second_team_id: i32,
    first_team_card: Option<TeamCard>,
    second_team_card: Option<TeamCard>,
    first_team_selector: TeamSelector,
    second_team_selector: TeamSelector,
}

impl NewGamePage {
    pub fn get(
        app_state: AppState,
        profile: User,
        first_team: Option<Team>,
        second_team: Option<Team>,
    ) -> Self {
        Self::get_with_message(app_state, profile, None, first_team, second_team)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: User,
        alert_message: Option<AlertMessage>,
        first_team: Option<Team>,
        second_team: Option<Team>,
    ) -> Self {
        let first_team_id = first_team
            .clone()
            .and_then(|team| Some(team.id))
            .unwrap_or(-1);
        let second_team_id = second_team
            .clone()
            .and_then(|team| Some(team.id))
            .unwrap_or(-1);
        let first_team_card =
            first_team.and_then(|team| Some(TeamCard::get_with_details(team, true)));
        let second_team_card =
            second_team.and_then(|team| Some(TeamCard::get_with_details(team, true)));

        Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile)),
            alert_message,
            first_team_id,
            second_team_id,
            first_team_card,
            second_team_card,
            first_team_selector: TeamSelector::get("first_team_id".to_string()),
            second_team_selector: TeamSelector::get("second_team_id".to_string()),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_card.html")]
pub struct GameCard {
    game: GameSummary,
}

impl GameCard {
    pub fn get(game: GameSummary) -> Self {
        Self { game }
    }
}
