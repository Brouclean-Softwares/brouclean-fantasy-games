use crate::AppState;
use crate::app::templates::blood_bowl::games::GamesScheduleTable;
use crate::app::templates::blood_bowl::statistics::{
    PlayersTopStatisticsLists, TeamsTopStatisticsLists,
};
use crate::app::templates::blood_bowl::teams::TeamSelector;
use crate::app::templates::{AlertMessage, BreadCrumb, NavigationBar, UrlLink, blood_bowl};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::competitions::registrations::TeamRegistration;
use crate::data::blood_bowl::competitions::schedule::CompetitionSchedule;
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::competitions::standings::CompetitionStandings;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use http::Uri;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Compétitions", "/blood_bowl/competitions"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competitions_page.html")]
pub struct CompetitionsPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    competitions_preparing: Vec<Competition>,
    competitions_in_progress: Vec<Competition>,
    competitions_closed: Vec<Competition>,
}

impl CompetitionsPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        competitions_preparing: Vec<Competition>,
        competitions_in_progress: Vec<Competition>,
        competitions_closed: Vec<Competition>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: blood_bowl::breadcrumb(),
            profile,
            competitions_preparing,
            competitions_in_progress,
            competitions_closed,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_page.html")]
pub struct CompetitionPage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    breadcrumb: BreadCrumb,
    competition: Competition,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    tab: Option<String>,
    link_url: String,
    information: CompetitionInformationTab,
    standings: CompetitionStandingsTab,
    schedule: CompetitionScheduleTab,
    statistics: CompetitionStatisticsTab,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/owned_competitions_block.html")]
pub struct OwnedCompetitionsBlock {
    pub owned_competitions: Vec<Competition>,
}

impl CompetitionPage {
    pub async fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        alert_message: Option<AlertMessage>,
        competition: Competition,
        teams_top_statistics: TeamsTopStatisticsLists,
        players_top_statistics: PlayersTopStatisticsLists,
        edit_mode: bool,
        field_edited: Option<String>,
        tab: Option<String>,
    ) -> Result<Self, AppError> {
        let competition_id = competition.id;
        let editable = User::optional_user_eq_other(&profile, &competition.director);
        let competition_not_started = !competition.started;
        let edit_mode = edit_mode && editable;
        let link_url = format!("competition?id={}", competition.id);

        let teams_registrations = competition.select_teams_registrations(&app_state).await?;

        let stages = competition.select_stages(&app_state).await?;

        let (schedule, standings) = competition.schedule_and_standings(&app_state).await?;

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            alert_message,
            breadcrumb: breadcrumb(),
            competition: competition.clone(),
            editable,
            edit_mode,
            field_edited: field_edited.clone().unwrap_or_default(),
            tab,
            link_url: link_url.clone(),
            information: CompetitionInformationTab {
                competition,
                competition_stages: CompetitionStagesBloc {
                    stages: stages.clone(),
                    competition_id,
                    editable,
                    competition_not_started,
                },
                stage_types: CompetitionStageType::available_list(),
                teams_registrations,
                profile,
                editable,
                edit_mode,
                field_edited: field_edited.unwrap_or_default(),
                link_url,
                team_selector: TeamSelector::get("team_to_registered_id".to_string()),
            },
            schedule: CompetitionScheduleTab {
                schedule,
                competition_id,
                editable,
            },
            standings: CompetitionStandingsTab { standings },
            statistics: CompetitionStatisticsTab {
                teams_top_statistics,
                players_top_statistics,
            },
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_information.html")]
pub struct CompetitionInformationTab {
    competition: Competition,
    competition_stages: CompetitionStagesBloc,
    stage_types: Vec<CompetitionStageType>,
    teams_registrations: Vec<TeamRegistration>,
    profile: Option<User>,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    link_url: String,
    team_selector: TeamSelector,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_stages.html")]
pub struct CompetitionStagesBloc {
    stages: Vec<CompetitionStage>,
    competition_id: i32,
    editable: bool,
    competition_not_started: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_standings.html")]
pub struct CompetitionStandingsTab {
    standings: CompetitionStandings,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_schedule.html")]
pub struct CompetitionScheduleTab {
    schedule: CompetitionSchedule,
    competition_id: i32,
    editable: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_statistics.html")]
pub struct CompetitionStatisticsTab {
    pub teams_top_statistics: TeamsTopStatisticsLists,
    pub players_top_statistics: PlayersTopStatisticsLists,
}
