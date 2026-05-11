use crate::AppState;
use crate::app::templates::blood_bowl::teams::{
    NewTeamPage, TeamFilteredList, TeamPage, TeamsPage,
};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::statistics;
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::{games, players, staff, teams};
use crate::data::users::User;
use crate::errors::AppError;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::staffs::Staff;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::{TranslatedName, TypeName};
use blood_bowl_rs::versions::Version;
use serde::Deserialize;
use std::collections::HashMap;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(teams))
        .route("/filtered_list", post(filtered_list))
        .route("/new", get(new).post(create))
        .route("/team", get(team).post(update))
        .route("/delete", post(delete))
        .route("/upgrade", post(upgrade))
}

pub async fn teams(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<TeamsPage, Redirect> {
    let teams = teams::select_all(&app_state)
        .await
        .or_else(|error| Err(error.log_and_redirect(Redirect::to("/"))))?;

    Ok(TeamsPage::get(app_state, profile, teams))
}

#[derive(Deserialize)]
pub struct TeamFilteredListForm {
    pub filter: Option<String>,
    pub input_id_to_change: String,
}

pub async fn filtered_list(
    State(app_state): State<AppState>,
    Form(form): Form<TeamFilteredListForm>,
) -> TeamFilteredList {
    if let Some(filter) = form.filter {
        TeamFilteredList::get(
            teams::select_all_filtered(&app_state, filter)
                .await
                .unwrap_or_default(),
            form.input_id_to_change,
        )
    } else {
        TeamFilteredList::get(
            teams::select_all(&app_state).await.unwrap_or_default(),
            form.input_id_to_change,
        )
    }
}

#[derive(Deserialize)]
pub struct NewTeamQueryParams {
    pub version: Version,
    pub roster: Roster,
}

pub async fn new(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<NewTeamQueryParams>,
) -> NewTeamPage {
    NewTeamPage::get(app_state, profile, params.version, params.roster)
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
    pub captain_position: Option<Position>,
}

impl NewTeamForm {
    pub fn extract_positions_quantities(&self) -> HashMap<Position, u8> {
        let mut positions_quantities: HashMap<Position, u8> = HashMap::new();

        let position_quantities: HashMap<String, u8> =
            serde_json::from_str(&*self.players).unwrap();

        for position in self.roster.definition(self.version).unwrap().positions {
            if let Some(position_quantity) = position_quantities.get(&position.type_name()) {
                positions_quantities.insert(position, *position_quantity);
            }
        }

        positions_quantities
    }
}

pub async fn create(
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
        profile.clone().into(),
        form.version,
        form.roster,
        form.treasury,
        staff_quantities,
        position_quantities,
        form.dedicated_fans,
        form.captain_position,
    )
    .map_err(|error| {
        NewTeamPage::get_with_message(
            app_state.clone(),
            profile.clone(),
            form.version,
            form.roster,
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message: error.name("fr"),
            }),
        )
    })?;

    let created_team_id = teams::create(&app_state, &profile, &team).await;

    match created_team_id {
        Ok(team_id) => Ok(Redirect::to(&format!("team?id={}&edit=true", team_id))),

        Err(error) => {
            tracing::error!("{}", error);

            Err(NewTeamPage::get_with_message(
                app_state,
                profile,
                form.version,
                form.roster,
                Some(AlertMessage {
                    alert_type: AlertType::Danger,
                    message: error.to_string(),
                }),
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct TeamQueryParams {
    pub id: i32,
    pub alert_message: Option<String>,
    pub edit: Option<bool>,
    pub field_edited: Option<String>,
}

pub async fn team(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<TeamQueryParams>,
) -> Result<TeamPage, Redirect> {
    let error_handler = |error: AppError| error.log_and_redirect(Redirect::to("../teams"));

    let team = teams::select_by_id_with_staff_and_players(&app_state, params.id)
        .await
        .map_err(error_handler)?;

    let mut alert_message: Option<AlertMessage> = None;

    if let Err(team_not_compliant_error) = team.check_if_rules_compliant() {
        alert_message = Some(AlertMessage {
            alert_type: AlertType::Warning,
            message: team_not_compliant_error.name("fr"),
        })
    };

    if let Some(message) = params.alert_message {
        alert_message = Some(AlertMessage {
            alert_type: AlertType::Danger,
            message,
        })
    };

    let roster_definition = team.roster_definition().ok_or(Redirect::to("../teams"))?;

    let edit_mode = params.edit.unwrap_or(false);

    let positions_buyable = team.positions_buyable();

    let games_scheduled = games::select_scheduled_for_team(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    let game_playing = games::select_playing_by_team(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    let games_played = games::select_played_by_team(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    let mut victories = 0;
    let mut draws = 0;
    let mut losses = 0;

    for game in games_played.iter() {
        if let (Some(winner), Some(_)) = (&game.winner(), &game.loser()) {
            if winner.id.eq(&team.id) {
                victories += 1;
            } else {
                losses += 1;
            }
        } else {
            draws += 1;
        }
    }

    let team_statistics = statistics::teams::select_statistics(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    let players_top_statistics = PlayersTopStatistics::for_team_id(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    let former_players = players::select_former_for_team(&app_state, team.id)
        .await
        .map_err(error_handler)?;

    Ok(TeamPage::get(
        app_state,
        profile,
        alert_message,
        team,
        games_scheduled,
        game_playing,
        games_played,
        roster_definition,
        edit_mode,
        params.field_edited,
        positions_buyable,
        victories,
        draws,
        losses,
        team_statistics,
        players_top_statistics,
        former_players,
    ))
}

#[derive(Deserialize)]
pub struct TeamForm {
    pub team_name: Option<String>,
    pub staff_to_buy: Option<Staff>,
    pub position_to_buy: Option<Position>,
    pub player_id_to_buyout: Option<i32>,
    pub player_id_to_name_captain: Option<i32>,
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<TeamQueryParams>,
    Form(form): Form<TeamForm>,
) -> Result<Redirect, Redirect> {
    // Team name
    if let (Some(profile), Some(team_name)) = (profile.clone(), form.team_name) {
        teams::update_name(&app_state, &profile, params.id, &team_name)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}&edit={}&focus=team_name",
                    params.id,
                    app_error,
                    params.edit.unwrap_or(false),
                ))))
            })?;

        return Ok(Redirect::to(&format!("./team?id={}", params.id,)));
    }

    // Buy Staff
    if let (Some(profile), Some(staff_to_buy)) = (profile.clone(), form.staff_to_buy) {
        staff::buy_staff_for_team(&app_state, &profile, params.id, staff_to_buy)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}",
                    params.id, app_error,
                ))))
            })?;

        return Ok(Redirect::to(&format!("./team?id={}", params.id,)));
    }

    // Buy Player
    if let (Some(profile), Some(position_to_buy)) = (profile.clone(), form.position_to_buy) {
        players::buy_position_for_team(&app_state, &profile, params.id, position_to_buy)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}",
                    params.id, app_error,
                ))))
            })?;

        return Ok(Redirect::to(&format!("./team?id={}", params.id,)));
    }

    // Buyout Player
    if let (Some(profile), Some(player_id_to_buyout)) = (profile.clone(), form.player_id_to_buyout)
    {
        players::buyout_for_team(&app_state, &profile, params.id, player_id_to_buyout)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}",
                    params.id, app_error,
                ))))
            })?;

        return Ok(Redirect::to(&format!("./team?id={}", params.id,)));
    }

    // Name Player captain
    if let (Some(profile), Some(player_id_to_name_captain)) =
        (profile.clone(), form.player_id_to_name_captain)
    {
        players::name_captain_for_team(&app_state, &profile, params.id, player_id_to_name_captain)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}",
                    params.id, app_error,
                ))))
            })?;

        return Ok(Redirect::to(&format!("./team?id={}", params.id,)));
    }

    Ok(Redirect::to(&format!("./team?id={}", params.id,)))
}

#[derive(Deserialize)]
pub struct DeleteTeamForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<DeleteTeamForm>,
) -> Result<Redirect, Redirect> {
    teams::delete(&app_state, &profile, form.id)
        .await
        .or_else(|app_error| {
            Err(app_error.log_and_redirect(Redirect::to(&format!(
                "./team?id={}&message={}",
                form.id, app_error
            ))))
        })?;

    Ok(Redirect::to("/blood_bowl"))
}

#[derive(Deserialize)]
pub struct UpgradeTeamForm {
    pub id: i32,
}

pub async fn upgrade(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<UpgradeTeamForm>,
) -> Result<Redirect, Redirect> {
    let error_handler = |error: AppError| error.log_and_redirect(Redirect::to("../teams"));

    let team = teams::select_by_id_with_staff_and_players(&app_state, form.id)
        .await
        .map_err(error_handler)?;

    if team.coach.eq(&profile.clone().into()) {
        teams::upgrade(&app_state, &profile, team)
            .await
            .or_else(|app_error| {
                Err(app_error.log_and_redirect(Redirect::to(&format!(
                    "./team?id={}&message={}",
                    form.id, app_error
                ))))
            })?;
    }

    Ok(Redirect::to(&format!("./team?id={}", form.id)))
}
