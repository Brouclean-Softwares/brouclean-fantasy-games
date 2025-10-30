use crate::app::templates::blood_bowl::teams::TeamSelector;
use crate::app::templates::{AlertMessage, NavigationBar};
use crate::data::blood_bowl::competitions::{Competition, CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competitions_page.html")]
pub struct CompetitionsPage {
    navigation_bar: NavigationBar,
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
    competition: Competition,
    editable: bool,
    edit_mode: bool,
    link_url: String,
    information: CompetitionInformation,
}

impl CompetitionPage {
    pub async fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        competition: Competition,
        edit_mode: bool,
    ) -> Result<Self, AppError> {
        let editable = if let Some(connected_user) = profile.clone() {
            connected_user.eq(&competition.director)
        } else {
            false
        };

        let deletable = editable && !competition.started;

        let edit_mode = edit_mode && editable;

        let link_url = format!("competition?id={}", competition.id);

        let registered_teams: Vec<TeamSummary> =
            competition.select_registered_teams(&app_state).await?;

        let stages = competition.select_stages(&app_state).await?;

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            competition: competition.clone(),
            editable,
            edit_mode,
            link_url: link_url.clone(),
            information: CompetitionInformation {
                competition,
                stages,
                stage_types: CompetitionStageType::available_list(),
                registered_teams,
                profile,
                deletable,
                editable,
                edit_mode,
                link_url,
                team_selector: TeamSelector::get("team_to_registered_id".to_string()),
            },
        })
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/competition_information.html")]
pub struct CompetitionInformation {
    competition: Competition,
    stages: Vec<CompetitionStage>,
    stage_types: Vec<CompetitionStageType>,
    registered_teams: Vec<TeamSummary>,
    profile: Option<User>,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    link_url: String,
    team_selector: TeamSelector,
}
