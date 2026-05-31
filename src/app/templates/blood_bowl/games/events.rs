use crate::app::templates::blood_bowl::games::Weather;
use crate::app::templates::blood_bowl::games::{GamePage, GameStatus};
use crate::data::blood_bowl::teams::TeamLogo;
use crate::errors::AppError;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::actions::Success;
use blood_bowl_rs::events::GameEvent;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::inducements::{Inducement, TreasuryAndPettyCash};
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::players::PlayerType;
use blood_bowl_rs::positions::Keyword;
use blood_bowl_rs::prayers::PrayerToNuffle;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/pre_game_sequence.html")]
pub struct PreGameSequence {
    game: Game,
    weather_controller: WeatherController,
    first_team_money: TreasuryAndPettyCash,
    second_team_money: TreasuryAndPettyCash,
    first_team_buyable_inducements: Vec<Inducement>,
    second_team_buyable_inducements: Vec<Inducement>,
    first_team_recalculated_value: u32,
    second_team_recalculated_value: u32,
}

impl PreGameSequence {
    pub fn try_from_game(game: &Game) -> Result<Self, AppError> {
        let weather_controller = WeatherController { game_id: game.id };

        let (first_team_money, second_team_money) = game.teams_money_left()?;

        let (first_team_buyable_inducements, second_team_buyable_inducements) =
            game.inducements_buyable_by_teams()?;

        let (first_team_recalculated_value, second_team_recalculated_value) =
            game.recalculated_current_team_values()?;

        Ok(Self {
            game: game.clone(),
            weather_controller,
            first_team_money,
            second_team_money,
            first_team_buyable_inducements,
            second_team_buyable_inducements,
            first_team_recalculated_value,
            second_team_recalculated_value,
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/game_events.html")]
pub struct GameEvents {
    game: Game,
}

impl GameEvents {
    pub fn from_game(game: &Game) -> Self {
        Self { game: game.clone() }
    }

    pub fn player_in_game_url(&self, team_id: &i32, player_id: &i32) -> String {
        GamePage::player_in_game_url(&self.game.id, team_id, player_id)
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/events_controller.html")]
pub struct EventsController {
    game: Game,
    pre_game_sequence: PreGameSequence,
    post_game_sequence: PostGameSequence,
    weather_controller: WeatherController,
    first_team_event_controller: TeamEventController,
    second_team_event_controller: TeamEventController,
}

impl EventsController {
    pub fn try_from_game(game: &Game) -> Result<Option<Self>, AppError> {
        if !game.closed {
            let pre_game_sequence = PreGameSequence::try_from_game(game)?;
            let post_game_sequence = PostGameSequence::try_from_game(game)?;
            let weather_controller = WeatherController { game_id: game.id };

            Ok(Some(Self {
                game: game.clone(),
                pre_game_sequence,
                post_game_sequence,
                weather_controller,
                first_team_event_controller: TeamEventController::from_team_game(
                    game.clone(),
                    game.first_team.clone(),
                ),
                second_team_event_controller: TeamEventController::from_team_game(
                    game.clone(),
                    game.second_team.clone(),
                ),
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/weather_controller.html")]
pub struct WeatherController {
    game_id: i32,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/team_event_controller.html")]
pub struct TeamEventController {
    game: Game,
    team: Team,
}

impl TeamEventController {
    pub fn from_team_game(game: Game, team: Team) -> Self {
        Self { game, team }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/post_game_sequence.html")]
pub struct PostGameSequence {
    game: Game,
    first_team_winnings: Option<u32>,
    second_team_winnings: Option<u32>,
    first_team_dedicated_fans_delta: Option<i8>,
    second_team_dedicated_fans_delta: Option<i8>,
    most_valuable_players_should_be_nominated: bool,
    first_team_expensive_mistakes: Option<u32>,
    second_team_expensive_mistakes: Option<u32>,
}

impl PostGameSequence {
    pub fn try_from_game(game: &Game) -> Result<Self, AppError> {
        let (first_team_winnings, second_team_winnings) = game.winnings();

        let (first_team_dedicated_fans_delta, second_team_dedicated_fans_delta) =
            game.dedicated_fans_updates();

        let is_a_competition_game = game.title.is_some();

        let (first_team_mvps, second_team_mvps) = game.most_valuable_players();

        let most_valuable_players_should_be_nominated =
            (first_team_mvps.len() + second_team_mvps.len()) < 2 && is_a_competition_game;

        let (first_team_expensive_mistakes, second_team_expensive_mistakes) =
            game.expensive_mistakes();

        Ok(Self {
            game: game.clone(),
            first_team_winnings,
            second_team_winnings,
            first_team_dedicated_fans_delta,
            second_team_dedicated_fans_delta,
            most_valuable_players_should_be_nominated,
            first_team_expensive_mistakes,
            second_team_expensive_mistakes,
        })
    }
}
