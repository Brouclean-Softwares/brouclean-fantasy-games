use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::rosters::{Roster, RosterDefinition};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
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
    pub treasury: i32,
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
    edit_mode: bool,
}

impl TeamPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        team: Team,
        roster_definition: RosterDefinition,
        edit_mode: bool,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            team,
            roster_definition,
            edit_mode,
        }
    }
}
