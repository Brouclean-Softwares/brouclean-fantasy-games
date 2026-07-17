use crate::AppState;
use crate::app::templates::blood_bowl::games::GamesScheduleTable;
use crate::app::templates::blood_bowl::statistics::{
    PlayersTopStatisticsLists, TeamsTopStatisticsLists,
};
use crate::app::templates::blood_bowl::teams::TeamSelector;
use crate::app::templates::shared::ModalButton;
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

pub fn breadcrumb() -> BreadCrumb {
    blood_bowl::breadcrumb().plus_link(UrlLink::from("Compétitions", "/blood_bowl/competitions"))
}

pub fn rank_text(position: &usize, is_final_ranking: bool, is_meta: bool) -> String {
    let winner = match is_meta {
        false => "Vainqueur 🏆".to_string(),
        true => "🏆".to_string(),
    };

    let second = match is_meta {
        false => "2ème 🥈".to_string(),
        true => "🥈".to_string(),
    };

    let third = match is_meta {
        false => "3ème 🥉".to_string(),
        true => "🥉".to_string(),
    };

    let other_position = |pos: &usize| match (pos, is_meta) {
        (1, false) => "1er".to_string(),
        (_, false) => format!("{}ème", pos),
        (_, true) => format!("<span class=\"uk-text-meta\">#{}</span>", pos),
    };

    if is_final_ranking {
        if position.eq(&1) {
            winner
        } else if position.eq(&2) {
            second
        } else if position.eq(&3) {
            third
        } else {
            other_position(position)
        }
    } else {
        other_position(position)
    }
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
        competitions_preparing: Vec<Competition>,
        competitions_in_progress: Vec<Competition>,
        competitions_closed: Vec<Competition>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: blood_bowl::breadcrumb(),
            profile,
            competitions_preparing,
            competitions_in_progress,
            competitions_closed,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/owned_competitions_block.html")]
pub struct OwnedCompetitionsBlock {
    pub owned_competitions: Vec<Competition>,
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
    tab_name: String,
    link_url: String,
    information: CompetitionInformationTab,
    standings: CompetitionStandingsTab,
    schedule: CompetitionScheduleTab,
    statistics: CompetitionStatisticsTab,
}

impl CompetitionPage {
    pub fn tab_name_for_competition(competition: &Competition) -> String {
        if competition.closed {
            "standings".to_owned()
        } else if competition.started {
            "schedule".to_owned()
        } else {
            "info".to_owned()
        }
    }

    pub async fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        competition: Competition,
        tab_name: Option<String>,
        teams_top_statistics: TeamsTopStatisticsLists,
        players_top_statistics: PlayersTopStatisticsLists,
        edit_mode: bool,
        field_edited: Option<String>,
    ) -> Result<Self, AppError> {
        let competition_id = competition.id;
        let editable = User::optional_user_eq_other(&profile, &competition.director);
        let competition_not_started = !competition.started;
        let edit_mode = edit_mode && editable;
        let link_url = format!("competition?id={}", competition.id);

        let teams_registrations = competition.select_teams_registrations(&app_state).await?;

        let stages = competition.select_stages(&app_state).await?;

        let (schedule, standings) = competition.schedule_and_standings(&app_state).await?;

        let competition_can_be_closed = !competition.closed && schedule.is_finished() && editable;

        let competition_should_have_offseason = schedule.should_imply_offseason();

        let close_modal_button = ModalButton::from(
            "primary",
            "Clôturer la compétition",
            "close_modal",
            "Clôture de la compétition",
            CloseCompetitionModalButton {
                competition_id,
                competition_should_have_offseason,
            }
            .render()
            .unwrap(),
            "Clôturer",
            "./close",
        );

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            breadcrumb: breadcrumb(),
            competition: competition.clone(),
            editable,
            edit_mode,
            field_edited: field_edited.clone().unwrap_or_default(),
            tab_name: tab_name.unwrap_or("info".to_owned()),
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
            standings: CompetitionStandingsTab {
                competition_standings: standings,
                competition_can_be_closed,
                close_modal_button,
            },
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
    competition_standings: CompetitionStandings,
    competition_can_be_closed: bool,
    close_modal_button: ModalButton,
}

impl CompetitionStandingsTab {
    pub fn position_text_for_team_standings_in_stage(
        &self,
        stage_index: &usize,
        position: &usize,
    ) -> String {
        let is_last_stage =
            stage_index.eq(&(self.competition_standings.stages_standings.len() - 1));

        rank_text(position, is_last_stage, true)
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/competitions/close_competition_modal.html")]
struct CloseCompetitionModalButton {
    competition_id: i32,
    competition_should_have_offseason: bool,
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
