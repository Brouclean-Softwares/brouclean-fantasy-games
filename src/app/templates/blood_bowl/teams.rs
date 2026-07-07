use crate::AppState;
use crate::app::templates::blood_bowl::games::GameCard;
use crate::app::templates::blood_bowl::statistics::PlayersTopStatisticsLists;
use crate::app::templates::{AlertMessage, BreadCrumb, NavigationBar, UrlLink, blood_bowl};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamStatistics;
use crate::data::blood_bowl::teams::{TeamLogo, TeamSummary, TeamSummaryWithResults};
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::{Roster, RosterDefinition, SpecialRule};
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
    tab_name: String,
    editable: bool,
    upgradable: bool,
    edit_mode: bool,
    field_edited: String,
    sheet: TeamSheetTab,
    results: TeamResultsTab,
    statistics: TeamStatisticsTab,
    former_players: FormerPlayersTab,
}

impl TeamPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        team: Team,
        tab_name: Option<String>,
        games_scheduled: Vec<GameSummary>,
        game_playing: Option<GameSummary>,
        games_played: Vec<GameSummary>,
        roster_definition: RosterDefinition,
        edit_mode: bool,
        field_edited: Option<String>,
        positions_buyable: Vec<(Position, u32, bool)>,
        victories: usize,
        draws: usize,
        losses: usize,
        team_statistics: TeamStatistics,
        players_top_statistics: PlayersTopStatistics,
        former_players: Vec<(i32, Player)>,
        competitions: Vec<Competition>,
    ) -> Self {
        let mut is_playing_game = false;
        if let Some(game) = game_playing.clone() {
            is_playing_game = game.started && !game.finished;
        }

        let editable = !is_playing_game
            && match profile.clone() {
                Some(user) => user.has_optional_id(&team.coach.id),
                None => false,
            };

        let edit_mode = edit_mode && editable;

        let deletable = editable && games_played.len() == 0 && games_scheduled.len() == 0;

        let upgradable = if let Some(next_version) = team.version.next() {
            editable
                && team.roster_definition_for_next_version().is_some()
                && games_scheduled
                    .iter()
                    .filter(|&game| game.version.ne(&next_version))
                    .count()
                    == 0
        } else {
            false
        };

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            breadcrumb: breadcrumb(),
            team: team.clone(),
            tab_name: tab_name.unwrap_or("sheet".to_owned()),
            editable,
            upgradable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            sheet: TeamSheetTab {
                team: team.clone(),
                roster_definition,
                deletable,
                editable,
                edit_mode,
                positions_buyable,
            },
            results: TeamResultsTab {
                team: team.clone(),
                editable,
                games_scheduled,
                game_playing,
                games_played,
            },
            statistics: TeamStatisticsTab {
                victories,
                draws,
                losses,
                team_statistics,
                players_top_statistics: players_top_statistics.into(),
                competitions,
            },
            former_players: FormerPlayersTab {
                team,
                former_players,
            },
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_sheet.html")]
struct TeamSheetTab {
    team: Team,
    roster_definition: RosterDefinition,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    positions_buyable: Vec<(Position, u32, bool)>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_results.html")]
struct TeamResultsTab {
    team: Team,
    editable: bool,
    games_scheduled: Vec<GameSummary>,
    game_playing: Option<GameSummary>,
    games_played: Vec<GameSummary>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/team_statistics.html")]
struct TeamStatisticsTab {
    victories: usize,
    draws: usize,
    losses: usize,
    team_statistics: TeamStatistics,
    players_top_statistics: PlayersTopStatisticsLists,
    competitions: Vec<Competition>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/former_players.html")]
struct FormerPlayersTab {
    team: Team,
    former_players: Vec<(i32, Player)>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/teams/owned_teams_block.html")]
pub struct OwnedTeamsBlock {
    pub owned_teams: Vec<TeamSummaryWithResults>,
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
