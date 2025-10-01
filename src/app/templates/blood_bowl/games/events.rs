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
    editable: bool,
    first_team_money: TreasuryAndPettyCash,
    second_team_money: TreasuryAndPettyCash,
    first_team_buyable_inducements: Vec<Inducement>,
    second_team_buyable_inducements: Vec<Inducement>,
    first_team_inducements: Vec<Inducement>,
    second_team_inducements: Vec<Inducement>,
    first_team_prayers: Vec<PrayerToNuffle>,
    second_team_prayers: Vec<PrayerToNuffle>,
}

impl PreGameSequence {
    pub fn try_from_game(game: Game, editable: bool) -> Result<Option<Self>, AppError> {
        if editable {
            let (first_team_money, second_team_money) = game.teams_money_left()?;

            let (first_team_buyable_inducements, second_team_buyable_inducements) =
                game.inducements_buyable_by_teams()?;

            let (first_team_inducements, second_team_inducements) = game.teams_inducements();

            let (first_team_prayers, second_team_prayers) = game.teams_prayers();

            Ok(Some(Self {
                game: game.clone(),
                editable,
                first_team_money,
                second_team_money,
                first_team_buyable_inducements,
                second_team_buyable_inducements,
                first_team_inducements,
                second_team_inducements,
                first_team_prayers,
                second_team_prayers,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/game_sequence.html")]
pub struct GameSequence {
    game: Game,
    editable: bool,
    game_controller: GameEventController,
}

impl GameSequence {
    pub fn from_game(game: Game, editable: bool) -> Self {
        Self {
            game: game.clone(),
            editable,
            game_controller: GameEventController {
                game: game.clone(),
                first_team_event_controller: TeamEventController {
                    game: game.clone(),
                    team: game.first_team.clone(),
                },
                second_team_event_controller: TeamEventController {
                    game: game.clone(),
                    team: game.second_team.clone(),
                },
            },
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/game_event_controller.html")]
pub struct GameEventController {
    game: Game,
    first_team_event_controller: TeamEventController,
    second_team_event_controller: TeamEventController,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/team_event_controller.html")]
pub struct TeamEventController {
    game: Game,
    team: Team,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/games/events/post_game_sequence.html")]
pub struct PostGameSequence {
    game: Game,
}

impl PostGameSequence {
    pub fn try_from_game(game: Game, editable: bool) -> Result<Option<Self>, AppError> {
        if editable {
            Ok(Some(Self { game }))
        } else {
            Ok(None)
        }
    }
}
