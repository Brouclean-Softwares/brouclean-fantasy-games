use crate::app::templates::blood_bowl::games::events::{EventsController, GameEvents};
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
use blood_bowl_rs::players::{Player, PlayerStatistics};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::weather::Weather;

pub mod events;

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
    tab_displayed: String,
    game: Game,
    editable: bool,
    edit_mode: bool,
    score: (usize, usize),
    casualties: (usize, usize),
    game_date_input: String,
    game_date: String,
    game_status: String,
    events_controller: EventsController,
    game_events: GameEvents,
    first_team_statistics: TeamStatistics,
    second_team_statistics: TeamStatistics,
}

impl GamePage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        game: &Game,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        Self::get_with_message(app_state, profile, None, game, edit_mode)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        game: &Game,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        let mut editable = false;

        if let Some(connected_user) = profile.clone() {
            editable = connected_user.is_option_coach(&game.created_by)
                || connected_user.is_coach(&game.first_team.coach)
                || connected_user.is_coach(&game.second_team.coach);
        }

        let tab_displayed: String = "game".to_string();

        let game_status = game.status().name("fr");

        let score = game.score();

        let casualties = game.casualties();

        let game_date_input = game.game_at.format("%Y-%m-%dT%H:%M").to_string();

        let game_date = game.game_at.format("%d/%m/%Y à %H:%M").to_string();

        let game_events = GameEvents::from_game(game);

        let first_team_statistics = TeamStatistics {
            game: game.clone(),
            team: game.first_team.clone(),
            players_statistics: game.players_statistics_for_team(&game.first_team),
        };

        let second_team_statistics = TeamStatistics {
            game: game.clone(),
            team: game.second_team.clone(),
            players_statistics: game.players_statistics_for_team(&game.second_team),
        };

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            tab_displayed,
            game: game.clone(),
            editable,
            edit_mode,
            score,
            casualties,
            game_date_input,
            game_date,
            game_status,
            events_controller: EventsController::try_from_game(game)?,
            game_events,
            first_team_statistics,
            second_team_statistics,
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/team_statistics.html")]
struct TeamStatistics {
    game: Game,
    team: Team,
    players_statistics: Vec<(i32, Player, PlayerStatistics)>,
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
