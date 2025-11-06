use crate::app::templates::blood_bowl::teams::TeamSelector;
use crate::app::templates::{blood_bowl, AlertMessage, BreadCrumb, NavigationBar, UrlLink};
use crate::data::blood_bowl::competitions::registrations::TeamRegistration;
use crate::data::blood_bowl::competitions::schedule::StageSchedule;
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::competitions::standings::StageStandings;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Compétitions", "/blood_bowl/competitions"))
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competitions_page.html")]
pub struct CompetitionsPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    competitions_in_progress: Vec<Competition>,
    competitions_closed: Vec<Competition>,
}

impl CompetitionsPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        competitions_in_progress: Vec<Competition>,
        competitions_closed: Vec<Competition>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: blood_bowl::breadcrumb(),
            profile,
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
    link_url: String,
    information: CompetitionInformation,
    standings: CompetitionStandings,
    schedule: CompetitionSchedule,
}

impl CompetitionPage {
    pub async fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        competition: Competition,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        let competition_id = competition.id;
        let editable = User::optional_user_eq_other(&profile, &competition.director);
        let competition_not_started = !competition.started;
        let deletable = editable && competition_not_started;
        let edit_mode = edit_mode && editable;
        let link_url = format!("competition?id={}", competition.id);

        let teams_registrations = competition.select_teams_registrations(&app_state).await?;

        let stages = competition.select_stages(&app_state).await?;

        let (schedule, standings) = competition.schedule_and_standings(&app_state).await?;

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            breadcrumb: breadcrumb(),
            competition: competition.clone(),
            editable,
            edit_mode,
            link_url: link_url.clone(),
            information: CompetitionInformation {
                competition,
                competition_stages: CompetitionStages {
                    stages: stages.clone(),
                    competition_id,
                    editable,
                    competition_not_started,
                },
                stage_types: CompetitionStageType::available_list(),
                teams_registrations,
                profile,
                deletable,
                editable,
                edit_mode,
                link_url,
                team_selector: TeamSelector::get("team_to_registered_id".to_string()),
            },
            schedule: CompetitionSchedule { schedule },
            standings: CompetitionStandings { standings },
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_information.html")]
pub struct CompetitionInformation {
    competition: Competition,
    competition_stages: CompetitionStages,
    stage_types: Vec<CompetitionStageType>,
    teams_registrations: Vec<TeamRegistration>,
    profile: Option<User>,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    link_url: String,
    team_selector: TeamSelector,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_stages.html")]
pub struct CompetitionStages {
    stages: Vec<CompetitionStage>,
    competition_id: i32,
    editable: bool,
    competition_not_started: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_standings.html")]
pub struct CompetitionStandings {
    standings: Vec<StageStandings>,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_schedule.html")]
pub struct CompetitionSchedule {
    schedule: Vec<StageSchedule>,
}
