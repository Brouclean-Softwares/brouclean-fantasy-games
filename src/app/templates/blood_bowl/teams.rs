use crate::app::templates::blood_bowl::games::GameCard;
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use crate::{data, AppState};
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::{Roster, RosterDefinition};
use blood_bowl_rs::teams::{Team, TeamSummary};
use blood_bowl_rs::translation::{TranslatedName, TypeName};
use blood_bowl_rs::versions::Version;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/teams_page.html")]
pub struct TeamsPage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    teams: Vec<TeamSummary>,
}

impl TeamsPage {
    pub fn get(app_state: AppState, profile: Option<User>, teams: Vec<TeamSummary>) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            profile,
            teams,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/new_team_page.html")]
pub struct NewTeamPage {
    navigation_bar: NavigationBar,
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
    alert_message: Option<AlertMessage>,
    team: Team,
    roster_definition: RosterDefinition,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    focus: Option<String>,
    positions_buyable: Vec<(Position, u32, bool)>,
}

impl TeamPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        team: Team,
        roster_definition: RosterDefinition,
        edit_mode: bool,
        focus: Option<String>,
        positions_buyable: Vec<(Position, u32, bool)>,
    ) -> Self {
        let editable = match profile.clone() {
            Some(user) => team.coach.eq(&user.into()),
            None => false,
        };

        let edit_mode = edit_mode && editable;

        let deletable = editable
            && team.games_played.len() == 0
            && team.games_scheduled.len() == 0
            && team.game_playing.is_none();

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            team,
            roster_definition,
            deletable,
            editable,
            edit_mode,
            focus,
            positions_buyable,
        }
    }
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
