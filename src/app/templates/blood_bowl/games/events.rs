use crate::app::templates::blood_bowl::games::GameStatus;
use crate::app::templates::blood_bowl::games::Weather;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::errors::AppError;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::actions::Success;
use blood_bowl_rs::events::GameEvent;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::inducements::{Inducement, TreasuryAndPettyCash};
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::prayers::PrayerToNuffle;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/pre_game_sequence.html")]
pub struct PreGameSequence {
    game: Game,
    first_team_money: TreasuryAndPettyCash,
    second_team_money: TreasuryAndPettyCash,
    first_team_buyable_inducements: Vec<Inducement>,
    second_team_buyable_inducements: Vec<Inducement>,
}

impl PreGameSequence {
    pub fn try_from_game(game: &Game) -> Result<Self, AppError> {
        let (first_team_money, second_team_money) = game.teams_money_left()?;

        let (first_team_buyable_inducements, second_team_buyable_inducements) =
            game.inducements_buyable_by_teams()?;

        Ok(Self {
            game: game.clone(),
            first_team_money,
            second_team_money,
            first_team_buyable_inducements,
            second_team_buyable_inducements,
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
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/events_controller.html")]
pub struct EventsController {
    game: Game,
    pre_game_sequence: PreGameSequence,
    post_game_sequence: PostGameSequence,
    first_team_event_controller: TeamEventController,
    second_team_event_controller: TeamEventController,
}

impl EventsController {
    pub fn try_from_game(game: &Game) -> Result<Self, AppError> {
        let pre_game_sequence = PreGameSequence::try_from_game(game)?;
        let post_game_sequence = PostGameSequence::try_from_game(game)?;

        Ok(Self {
            game: game.clone(),
            pre_game_sequence,
            post_game_sequence,
            first_team_event_controller: TeamEventController::from_team_game(
                game.clone(),
                game.first_team.clone(),
            ),
            second_team_event_controller: TeamEventController::from_team_game(
                game.clone(),
                game.second_team.clone(),
            ),
        })
    }
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
}

impl PostGameSequence {
    pub fn try_from_game(game: &Game) -> Result<Self, AppError> {
        let (first_team_winnings, second_team_winnings) = game.winnings();

        Ok(Self {
            game: game.clone(),
            first_team_winnings,
            second_team_winnings,
        })
    }
}
