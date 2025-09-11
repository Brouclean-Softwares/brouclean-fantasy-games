use crate::app::templates::blood_bowl::teams::{NewTeamPage, TeamPage, TeamsPage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::teams;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::{Roster, Staff};
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;
use std::collections::HashMap;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(teams))
        .route("/new", get(new_team).post(create_team))
        .route("/team", get(get_team))
}

pub async fn teams(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<TeamsPage, AppError> {
    let teams = teams::select_all(&app_state).await?;

    Ok(TeamsPage::get(app_state, profile, teams))
}

#[derive(Deserialize)]
pub struct NewTeamQueryParams {
    pub version: Version,
    pub roster: Roster,
}

pub async fn new_team(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<NewTeamQueryParams>,
) -> Result<NewTeamPage, AppError> {
    Ok(NewTeamPage::get(
        app_state,
        profile,
        params.version,
        params.roster,
    ))
}

#[derive(Deserialize)]
pub struct NewTeamForm {
    pub version: Version,
    pub roster: Roster,
    pub players: String,
    pub treasury: i32,
    pub dedicated_fans: u8,
    pub re_roll: u8,
    pub cheerleader: u8,
    pub assistant_coach: u8,
    pub apothecary: Option<u8>,
}

impl NewTeamForm {
    pub fn extract_positions_quantities(&self) -> HashMap<Position, u8> {
        let mut positions_quantities: HashMap<Position, u8> = HashMap::new();

        let position_quantities: HashMap<String, u8> =
            serde_json::from_str(&*self.players).unwrap();

        for position in self
            .roster
            .definition(Some(self.version))
            .unwrap()
            .positions
        {
            if let Some(position_quantity) = position_quantities.get(&position.type_name()) {
                positions_quantities.insert(position, *position_quantity);
            }
        }

        positions_quantities
    }
}

pub async fn create_team(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<NewTeamForm>,
) -> Result<Redirect, NewTeamPage> {
    let position_quantities = form.extract_positions_quantities();

    let mut staff_quantities: HashMap<Staff, u8> = HashMap::new();
    staff_quantities.insert(Staff::ReRoll, form.re_roll);
    staff_quantities.insert(Staff::Cheerleader, form.cheerleader);
    staff_quantities.insert(Staff::AssistantCoach, form.assistant_coach);
    if let Some(apothecary) = form.apothecary {
        staff_quantities.insert(Staff::Apothecary, apothecary);
    }

    let team = Team::create_new(
        form.version,
        form.roster,
        form.treasury,
        staff_quantities,
        position_quantities,
        form.dedicated_fans,
    )
    .map_err(|error| {
        NewTeamPage::get_with_message(
            app_state.clone(),
            profile.clone(),
            form.version,
            form.roster,
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message: error.translate_to("fr"),
            }),
        )
    })?;

    let created_team_id = teams::create(&app_state, &profile, &team).await;

    match created_team_id {
        Ok(team_id) => Ok(Redirect::to(&format!("team?id={}", team_id))),

        Err(error) => Err(NewTeamPage::get_with_message(
            app_state,
            profile,
            form.version,
            form.roster,
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message: error.to_string(),
            }),
        )),
    }
}

#[derive(Deserialize)]
pub struct TeamQueryParams {
    pub id: i32,
}

pub async fn get_team(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<TeamQueryParams>,
) -> Result<TeamPage, Redirect> {
    let team = teams::select_from_id(&app_state, params.id)
        .await
        .map_err(|error| {
            tracing::debug!("get_team: Error: {}", error);

            Redirect::to("../teams")
        })?;

    let roster_definition = team
        .roster
        .definition(Some(team.version))
        .ok_or(Redirect::to("../teams"))?;

    Ok(TeamPage::get(app_state, profile, team, roster_definition))
}
