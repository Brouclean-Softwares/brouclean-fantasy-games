use crate::AppState;
use crate::app::templates::blood_bowl::competitions::{CompetitionPage, CompetitionsPage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::competitions::stages::{
    CompetitionStage, CompetitionStageRule, CompetitionStageType,
};
use crate::data::blood_bowl::games;
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamsTopStatistics;
use crate::data::users::User;
use crate::errors::AppError;
use axum::extract::{OriginalUri, Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use blood_bowl_rs::versions::Version;
use chrono::NaiveDateTime;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(competitions).post(new))
        .route("/delete", post(delete))
        .route("/competition", get(competition).post(save))
        .route("/add_stage", post(add_stage))
        .route("/delete_stage", post(delete_stage))
        .route("/add_rule", post(add_rule))
        .route("/delete_rule", post(delete_rule))
        .route("/register_team", post(register_team))
        .route("/unregister_team", post(unregister_team))
        .route("/update_team_validation", post(update_team_validation))
        .route("/insert_games", post(insert_games))
}

pub async fn competitions(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<CompetitionsPage, Redirect> {
    let redirect_if_error = Redirect::to("..");

    let competitions_preparing = Competition::select_all_preparing(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let competitions_in_progress = Competition::select_all_in_progress(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    let competitions_closed = Competition::select_all_closed(&app_state)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    Ok(CompetitionsPage::get(
        app_state,
        profile,
        &uri,
        competitions_preparing,
        competitions_in_progress,
        competitions_closed,
    ))
}

#[derive(Deserialize)]
pub struct CompetitionQueryParams {
    pub id: Option<i32>,
    pub tab_name: Option<String>,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
    pub alert_message: Option<String>,
}

pub async fn competition(
    OriginalUri(uri): OriginalUri,
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<CompetitionQueryParams>,
) -> Result<CompetitionPage, Redirect> {
    if let Some(competition_id) = params.id {
        let redirect_if_error = Redirect::to("../competitions");

        let alert_message: Option<AlertMessage> = params.alert_message.and_then(|message| {
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message,
            })
        });

        let competition = Competition::select_by_id(&app_state, competition_id)
            .await
            .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?
            .ok_or(redirect_if_error.clone())?;

        let teams_top_statistics =
            TeamsTopStatistics::for_competition_id(&app_state, competition.id)
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        let players_top_statistics =
            PlayersTopStatistics::for_competition_id(&app_state, competition.id)
                .await
                .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

        Ok(CompetitionPage::get(
            app_state,
            profile,
            &uri,
            alert_message,
            competition,
            params.tab_name,
            teams_top_statistics.into(),
            players_top_statistics.into(),
            params.edit.unwrap_or(false),
            params.field_edited,
        )
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?)
    } else {
        Err(Redirect::to("../competitions"))
    }
}

#[derive(Deserialize)]
pub struct CompetitionForm {
    pub competition_name: Option<String>,
    pub competition_description: Option<String>,
    pub competition_version: Option<Version>,
}

pub async fn new(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<CompetitionForm>,
) -> Result<Redirect, Redirect> {
    let redirect_if_error = Redirect::to("../competitions");

    let mut competition =
        Competition::new(profile.clone(), form.competition_name.unwrap_or_default());

    competition
        .save(&app_state, &profile)
        .await
        .map_err(|error| error.log_and_redirect(redirect_if_error.clone()))?;

    if competition.is_new() {
        Err(redirect_if_error)
    } else {
        Ok(Redirect::to(&format!(
            "./competitions/competition?id={}",
            competition.id,
        )))
    }
}

pub async fn save(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<CompetitionQueryParams>,
    Form(form): Form<CompetitionForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={:?}&alert_message={}&edit={}",
            params.id,
            error,
            params.edit.unwrap_or(false),
        )))
    };

    let mut competition = if let Some(competition_id) = params.id {
        Competition::select_by_id(&app_state, competition_id)
            .await
            .map_err(error_handler)?
            .ok_or(Redirect::to("../competitions"))?
    } else {
        Competition::new(profile.clone(), "".to_string())
    };

    // Name
    if let Some(competition_name) = form.competition_name {
        competition.name = competition_name;
    }

    // Description
    if let Some(competition_description) = form.competition_description {
        competition.description = competition_description;
    }

    // Version
    if let Some(competition_version) = form.competition_version {
        competition.version = competition_version;
    }

    competition
        .save(&app_state, &profile)
        .await
        .map_err(error_handler)?;

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        competition.id,
    )))
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<DeleteForm>,
) -> Result<Redirect, Redirect> {
    Competition::delete(&app_state, &profile, form.id)
        .await
        .or_else(|app_error| {
            Err(app_error.log_and_redirect(Redirect::to(&format!(
                "./competition?id={}&message={}",
                form.id, app_error
            ))))
        })?;

    Ok(Redirect::to("/blood_bowl/competitions"))
}

#[derive(Deserialize)]
pub struct AddStageForm {
    pub competition_id: i32,
    pub stage_type_to_add: String,
}

pub async fn add_stage(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<AddStageForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(mut competition), Some(connected_user)) = (competition, profile) {
        let stage_type_to_add: CompetitionStageType = serde_json::from_str(&form.stage_type_to_add)
            .map_err(|error| {
                tracing::error!("{}", error);

                Redirect::to(&format!(
                    "./competition?id={}&alert_message={}",
                    form.competition_id,
                    error.to_string()
                ))
            })?;

        competition
            .insert_stage(&app_state, &connected_user, stage_type_to_add)
            .await
            .map_err(error_handler)?;
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct DeleteStageForm {
    pub competition_id: i32,
    pub stage_id: i32,
}

pub async fn delete_stage(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<DeleteStageForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(mut competition), Some(connected_user)) = (competition, profile) {
        competition
            .delete_stage(&app_state, &connected_user, form.stage_id)
            .await
            .map_err(error_handler)?;
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct AddRuleForm {
    pub competition_id: i32,
    pub stage_id: i32,
    pub rule_to_add: String,
}

pub async fn add_rule(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<AddRuleForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let stage = CompetitionStage::select_by_id(&app_state, form.stage_id)
        .await
        .map_err(error_handler)?;

    if let Some(mut stage) = stage {
        let competition = Competition::select_by_id(&app_state, form.competition_id)
            .await
            .map_err(error_handler)?;

        if let (Some(mut competition), Some(connected_user)) = (competition, profile) {
            let rule_to_add: CompetitionStageRule = serde_json::from_str(&form.rule_to_add)
                .map_err(|error| {
                    tracing::error!("{}", error);

                    Redirect::to(&format!(
                        "./competition?id={}&alert_message={}",
                        form.competition_id,
                        error.to_string()
                    ))
                })?;

            stage.rules.push(rule_to_add);

            stage
                .update_for_competition(&app_state, &connected_user, &mut competition)
                .await
                .map_err(error_handler)?;
        }
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct DeleteRuleForm {
    pub competition_id: i32,
    pub stage_id: i32,
    pub rule_to_delete: String,
}

pub async fn delete_rule(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<DeleteRuleForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let stage = CompetitionStage::select_by_id(&app_state, form.stage_id)
        .await
        .map_err(error_handler)?;

    if let Some(mut stage) = stage {
        let competition = Competition::select_by_id(&app_state, form.competition_id)
            .await
            .map_err(error_handler)?;

        if let (Some(mut competition), Some(connected_user)) = (competition, profile) {
            let rule_to_delete: CompetitionStageRule = serde_json::from_str(&form.rule_to_delete)
                .map_err(|error| {
                tracing::error!("{}", error);

                Redirect::to(&format!(
                    "./competition?id={}&alert_message={}",
                    form.competition_id,
                    error.to_string()
                ))
            })?;

            if let Some(position) = stage.rules.iter().position(|rule| rule.eq(&rule_to_delete)) {
                stage.rules.remove(position);

                stage
                    .update_for_competition(&app_state, &connected_user, &mut competition)
                    .await
                    .map_err(error_handler)?;
            }
        }
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct RegisterTeamForm {
    pub competition_id: i32,
    pub team_to_registered_id: i32,
}

pub async fn register_team(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<RegisterTeamForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(competition), Some(connected_user)) = (competition, profile) {
        competition
            .insert_team_registration(&app_state, &connected_user, form.team_to_registered_id)
            .await
            .map_err(error_handler)?;
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct UnregisterTeamForm {
    pub competition_id: i32,
    pub team_to_unregistered_id: i32,
}

pub async fn unregister_team(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<UnregisterTeamForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(competition), Some(connected_user)) = (competition, profile) {
        competition
            .delete_team_registration(&app_state, &connected_user, form.team_to_unregistered_id)
            .await
            .map_err(error_handler)?;
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct TeamValidationForm {
    pub competition_id: i32,
    pub team_id: i32,
    pub validation: bool,
}

pub async fn update_team_validation(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<TeamValidationForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(competition), Some(connected_user)) = (competition, profile) {
        competition
            .update_team_validation(&app_state, &connected_user, form.team_id, form.validation)
            .await
            .map_err(error_handler)?;
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}",
        form.competition_id
    )))
}

#[derive(Deserialize)]
pub struct InsertGamesForm {
    pub competition_id: i32,
    pub stage_id: i32,
    pub round_index: usize,
    pub scheduled_at: String,
}

pub async fn insert_games(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Form(form): Form<InsertGamesForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| {
        error.log_and_redirect(Redirect::to(&format!(
            "./competition?id={}&tab_name=schedule&alert_message={}",
            form.competition_id,
            error.to_string()
        )))
    };

    let scheduled_at = NaiveDateTime::parse_from_str(&*form.scheduled_at, "%Y-%m-%dT%H:%M")
        .map_err(|error| {
            tracing::error!("{}", error);
            Redirect::to(&format!(
                "./competition?id={}&tab_name=schedule&alert_message=Veuillez remplir la date et l'heure des matchs.",
                form.competition_id
            ))
        })?;

    let competition = Competition::select_by_id(&app_state, form.competition_id)
        .await
        .map_err(error_handler)?;

    if let (Some(competition), Some(connected_user)) = (competition, profile) {
        let (schedule, _) = competition
            .schedule_and_standings(&app_state)
            .await
            .map_err(error_handler)?;

        if let Some(round_schedule) = schedule.get_stage_round(form.stage_id, form.round_index) {
            games::create_for_competition_stage_round(
                &app_state,
                &connected_user,
                competition.id,
                form.stage_id,
                round_schedule,
                scheduled_at,
            )
            .await
            .map_err(error_handler)?;
        }
    }

    Ok(Redirect::to(&format!(
        "./competition?id={}&tab_name=schedule",
        form.competition_id
    )))
}
