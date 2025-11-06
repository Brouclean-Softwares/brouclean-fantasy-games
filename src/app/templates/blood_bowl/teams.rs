use crate::app::templates::blood_bowl::games::GameCard;
use crate::app::templates::{blood_bowl, AlertMessage, BreadCrumb, NavigationBar, UrlLink};
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::{TeamLogo, TeamSummary};
use crate::data::users::User;
use crate::errors::AppError;
use crate::{data, AppState};
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::{Roster, RosterDefinition};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::{TranslatedName, TypeName};
use blood_bowl_rs::versions::Version;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Équipes", "/blood_bowl/teams"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/teams_page.html")]
pub struct TeamsPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    teams: Vec<TeamSummary>,
}

impl TeamsPage {
    pub fn get(app_state: AppState, profile: Option<User>, teams: Vec<TeamSummary>) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: blood_bowl::breadcrumb(),
            profile,
            teams,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/new_team_page.html")]
pub struct NewTeamPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    alert_message: Option<AlertMessage>,
    version: Version,
    roster: Roster,
    initial_treasury: i32,
}

impl NewTeamPage {
    pub fn get(app_state: AppState, profile: User, version: Version, roster: Roster) -> Self {
        Self::get_with_message(app_state, profile, version, roster, None)
    }

    pub fn get_with_message(
        app_state: AppState,
        profile: User,
        version: Version,
        roster: Roster,
        message: Option<AlertMessage>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile)),
            alert_message: message,
            breadcrumb: breadcrumb(),
            version,
            roster,
            initial_treasury: Team::initial_treasury(&version),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_page.html")]
pub struct TeamPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    alert_message: Option<AlertMessage>,
    team: Team,
    editable: bool,
    edit_mode: bool,
    focus: Option<String>,
    sheet: TeamSheet,
    results: TeamResults,
    former_players: FormerPlayers,
}

impl TeamPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        team: Team,
        games_scheduled: Vec<GameSummary>,
        game_playing: Option<GameSummary>,
        games_played: Vec<GameSummary>,
        roster_definition: RosterDefinition,
        edit_mode: bool,
        focus: Option<String>,
        positions_buyable: Vec<(Position, u32, bool)>,
        former_players: Vec<(i32, Player)>,
    ) -> Self {
        let mut is_playing_game = false;
        if let Some(game) = game_playing.clone() {
            is_playing_game = game.started && !game.finished;
        }

        let editable = !is_playing_game
            && match profile.clone() {
                Some(user) => team.coach.eq(&user.into()),
                None => false,
            };

        let edit_mode = edit_mode && editable;

        let deletable = editable && games_played.len() == 0 && games_scheduled.len() == 0;

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            breadcrumb: breadcrumb(),
            team: team.clone(),
            editable,
            edit_mode,
            focus: focus.clone(),
            sheet: TeamSheet {
                team: team.clone(),
                roster_definition,
                deletable,
                editable,
                edit_mode,
                positions_buyable,
            },
            results: TeamResults {
                team: team.clone(),
                editable,
                games_scheduled,
                game_playing,
                games_played,
            },
            former_players: FormerPlayers {
                team,
                former_players,
            },
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_sheet.html")]
struct TeamSheet {
    team: Team,
    roster_definition: RosterDefinition,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    positions_buyable: Vec<(Position, u32, bool)>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_results.html")]
struct TeamResults {
    team: Team,
    editable: bool,
    games_scheduled: Vec<GameSummary>,
    game_playing: Option<GameSummary>,
    games_played: Vec<GameSummary>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/former_players.html")]
struct FormerPlayers {
    team: Team,
    former_players: Vec<(i32, Player)>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/owned_teams_block.html")]
pub struct OwnedTeamsBlock {
    owned_teams: Vec<TeamSummary>,
}

impl OwnedTeamsBlock {
    pub async fn get(app_state: &AppState, profile: &User) -> Result<Self, AppError> {
        let owned_teams =
            data::blood_bowl::teams::select_owned(&app_state, profile.clone()).await?;

        Ok(Self { owned_teams })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_card.html")]
pub struct TeamCard {
    team: Team,
    with_info: bool,
}

impl TeamCard {
    pub fn get(team: Team) -> Self {
        Self::get_with_details(team, false)
    }

    pub fn get_with_details(team: Team, with_info: bool) -> Self {
        TeamCard { team, with_info }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_selector.html")]
pub struct TeamSelector {
    team_filtered_list: TeamFilteredList,
    input_id_to_change: String,
}

impl TeamSelector {
    pub fn get(input_id_to_change: String) -> Self {
        Self {
            team_filtered_list: TeamFilteredList::get(vec![], input_id_to_change.clone()),
            input_id_to_change,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_filtered_list.html")]
pub struct TeamFilteredList {
    teams: Vec<TeamSummary>,
    input_id_to_change: String,
}

impl TeamFilteredList {
    pub fn get(teams: Vec<TeamSummary>, input_id_to_change: String) -> Self {
        Self {
            teams,
            input_id_to_change,
        }
    }
}
