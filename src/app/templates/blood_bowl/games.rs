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
use blood_bowl_rs::games::GameStatus;
use blood_bowl_rs::inducements::{Inducement, TreasuryAndPettyCash};
use blood_bowl_rs::players::{Player, PlayerStatistics};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::weather::Weather;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/games_page.html")]
pub struct GamesPage {
    navigation_bar: NavigationBar,
    games_playing: Vec<GameSummary>,
    games_played: Vec<GameSummary>,
    can_create: bool,
}

impl GamesPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        games_playing: Vec<GameSummary>,
        games_played: Vec<GameSummary>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            games_playing,
            games_played,
            can_create: profile.is_some(),
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/game_page.html")]
pub struct GamePage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    game: Game,
    editable: bool,
    edit_mode: bool,
    game_date_input: String,
    game_date: String,
    game_status: String,
    first_team_money: TreasuryAndPettyCash,
    second_team_money: TreasuryAndPettyCash,
    first_team_buyable_inducements: Vec<Inducement>,
    second_team_buyable_inducements: Vec<Inducement>,
    first_team_inducements: Vec<Inducement>,
    second_team_inducements: Vec<Inducement>,
    first_team_players_statistics: Vec<(i32, Player, PlayerStatistics)>,
    second_team_players_statistics: Vec<(i32, Player, PlayerStatistics)>,
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

        let game_status = game.status().name("fr");

        let game_date_input = game.game_at.format("%Y-%m-%dT%H:%M").to_string();

        let game_date = game.game_at.format("%d/%m/%Y à %H:%M").to_string();

        let (first_team_money, second_team_money) = game.teams_money_left()?;

        let (first_team_buyable_inducements, second_team_buyable_inducements) =
            game.inducements_buyable_by_teams()?;

        let (first_team_inducements, second_team_inducements) = game.teams_inducements();

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            game: game.clone(),
            editable,
            edit_mode,
            game_date_input,
            game_date,
            game_status,
            first_team_money,
            second_team_money,
            first_team_buyable_inducements,
            second_team_buyable_inducements,
            first_team_inducements,
            second_team_inducements,
            first_team_players_statistics: game.players_statistics_for_team(&game.first_team),
            second_team_players_statistics: game.players_statistics_for_team(&game.second_team),
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
