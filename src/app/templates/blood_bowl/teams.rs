use crate::app::templates::blood_bowl::OwnedTeamListRow;
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::users::User;
use crate::errors::AppError;
use crate::{data, AppState};
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::{Roster, RosterDefinition};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::{TranslatedName, TypeName};
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamListRow {
    pub id: i32,
    pub version: Version,
    pub name: String,
    pub roster: Roster,
    pub coach_id: Option<i32>,
    pub coach_name: Option<String>,
    pub value: i32,
    pub current_value: i32,
    pub external_logo_url: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/teams_page.html")]
pub struct TeamsPage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    teams: Vec<TeamListRow>,
}

impl TeamsPage {
    pub fn get(app_state: AppState, profile: Option<User>, teams: Vec<TeamListRow>) -> Self {
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
    editable: bool,
    edit_mode: bool,
    positions_buyable: Vec<(Position, u32, bool)>,
}

impl TeamPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        team: Team,
        roster_definition: RosterDefinition,
        editable: bool,
        edit_mode: bool,
        positions_buyable: Vec<(Position, u32, bool)>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            team,
            roster_definition,
            editable,
            edit_mode,
            positions_buyable,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/owned_teams_block.html")]
pub struct OwnedTeamsBlock {
    owned_teams: Vec<OwnedTeamListRow>,
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
    teams: Vec<TeamListRow>,
    input_id_to_change: String,
}

impl TeamFilteredList {
    pub fn get(teams: Vec<TeamListRow>, input_id_to_change: String) -> Self {
        Self {
            teams,
            input_id_to_change,
        }
    }
}
