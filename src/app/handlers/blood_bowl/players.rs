use crate::app::templates::blood_bowl::players::PlayerPage;
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::statistics::players::PlayerStatistics;
use crate::data::blood_bowl::{games, players, statistics, teams};
use crate::data::users::User;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Form, Router};
use blood_bowl_rs::players::PlayerType;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/player", get(player).post(update_player))
        .route("/added_player", get(added_player).post(update_added_player))
}

#[derive(Deserialize)]
pub struct PlayerQueryParams {
    pub player_id: i32,
    pub team_id: i32,
    pub alert_message: Option<String>,
    pub edit: Option<bool>,
}

pub async fn player(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<PlayerQueryParams>,
) -> Result<PlayerPage, Redirect> {
    let error_handler = |error| {
        tracing::debug!("get_player: Error: {}", error);
        Redirect::to("../teams")
    };

    let team = teams::select_by_id_without_staff_nor_players(&app_state, params.team_id)
        .await
        .map_err(error_handler)?;

    let mut is_playing_game = false;
    if let Some(game) = games::select_playing_by_team(&app_state, team.id)
        .await
        .map_err(error_handler)?
    {
        is_playing_game = game.started && !game.finished;
    }

    let alert_message: Option<AlertMessage> = params.alert_message.and_then(|message| {
        Some(AlertMessage {
            alert_type: AlertType::Danger,
            message,
        })
    });

    let editable = !is_playing_game
        && match profile.clone() {
            Some(user) => team.coach.eq(&user.into()),
            None => false,
        };

    let (number, player) =
        players::select_by_id_for_team(&app_state, params.player_id, params.team_id)
            .await
            .map_err(error_handler)?
            .ok_or(Redirect::to("../teams"))?;

    let player_advancements =
        players::select_advancements_with_choices(&app_state, params.player_id)
            .await
            .map_err(error_handler)?;

    let can_buyout = editable
        && players::is_under_contract_for_team(&app_state, params.player_id, params.team_id)
            .await
            .map_err(error_handler)?;

    let stats = statistics::players::select_statistics(&app_state, params.player_id)
        .await
        .map_err(error_handler)?;

    Ok(PlayerPage::get(
        app_state,
        profile,
        alert_message,
        format!("player?player_id={}&team_id={}", player.id, team.id),
        number,
        player,
        player_advancements,
        team,
        editable,
        params.edit.unwrap_or(false) && editable,
        false,
        can_buyout,
        stats,
    ))
}

#[derive(Deserialize)]
pub struct PlayerForm {
    pub player_number: Option<i32>,
    pub player_name: Option<String>,
    pub advancement_choice: Option<String>,
    pub advancement_to_add: Option<String>,
}

pub async fn update_player(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<PlayerQueryParams>,
    Form(form): Form<PlayerForm>,
) -> Result<Redirect, Redirect> {
    // Player number
    if let (Some(profile), Some(player_number)) = (profile.clone(), form.player_number) {
        players::update_number(
            &app_state,
            &profile,
            &params.team_id,
            &params.player_id,
            &player_number,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Player name
    if let (Some(profile), Some(player_name)) = (profile.clone(), form.player_name) {
        players::update_name(
            &app_state,
            &profile,
            &params.team_id,
            &params.player_id,
            &player_name,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Advancement choice
    if let (Some(profile), Some(advancement_choice)) = (profile.clone(), form.advancement_choice) {
        let advancement_choice =
            serde_json::from_str(&advancement_choice).or_else(|app_error| {
                Err(Redirect::to(&format!(
                    "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                    params.player_id,
                    params.team_id,
                    app_error,
                    params.edit.unwrap_or(false),
                )))
            })?;

        players::add_advancement_choice(
            &app_state,
            &profile,
            params.team_id,
            params.player_id,
            advancement_choice,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    // Advancement
    if let (Some(profile), Some(advancement_to_add)) = (profile.clone(), form.advancement_to_add) {
        let advancement_to_add =
            serde_json::from_str(&advancement_to_add).or_else(|app_error| {
                Err(Redirect::to(&format!(
                    "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                    params.player_id,
                    params.team_id,
                    app_error,
                    params.edit.unwrap_or(false),
                )))
            })?;

        players::add_advancement(
            &app_state,
            &profile,
            params.team_id,
            params.player_id,
            advancement_to_add,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;
    }

    Ok(Redirect::to(&format!(
        "./player?player_id={}&team_id={}&edit={}",
        params.player_id,
        params.team_id,
        params.edit.unwrap_or(false),
    )))
}

#[derive(Deserialize)]
pub struct AddedPlayerQueryParams {
    pub player_id_in_game: i32,
    pub team_id: i32,
    pub game_id: i32,
    pub alert_message: Option<String>,
    pub edit: Option<bool>,
}

pub async fn added_player(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<AddedPlayerQueryParams>,
) -> Result<PlayerPage, Redirect> {
    let error_handler = |error| {
        tracing::debug!("journeyman: Error: {}", error);
        Redirect::to(&format!("../games/game?id={}", params.game_id))
    };

    let game = games::select_by_id(&app_state, params.game_id)
        .await
        .map_err(|error| {
            tracing::debug!("journeyman: Error: {}", error);
            Redirect::to("../games")
        })?;

    let (number, player) = games::select_playing_team_player_for_game(
        &app_state,
        &game,
        params.team_id,
        params.player_id_in_game,
    )
    .await
    .map_err(error_handler)?
    .ok_or(Redirect::to(&format!(
        "../games/game?id={}",
        params.game_id
    )))?;

    if matches!(player.player_type, PlayerType::FromRoster) {
        return Err(Redirect::to(&format!(
            "../players/player?player_id={}&team_id={}",
            player.id, params.team_id
        )));
    }

    let alert_message: Option<AlertMessage> = params.alert_message.and_then(|message| {
        Some(AlertMessage {
            alert_type: AlertType::Danger,
            message,
        })
    });

    let team = teams::select_by_id_without_staff_nor_players(&app_state, params.team_id)
        .await
        .map_err(error_handler)?;

    let editable = matches!(player.player_type, PlayerType::Journeyman)
        && game.started
        && !game.game_finished()
        && match profile.clone() {
            Some(user) => team.coach.eq(&user.into()),
            None => false,
        };

    let is_last_game_for_team =
        games::is_last_for_team(&app_state, &params.game_id, &params.team_id)
            .await
            .map_err(error_handler)?;

    let can_buy = matches!(player.player_type, PlayerType::Journeyman)
        && game.game_finished()
        && is_last_game_for_team
        && team.can_buy_journeyman()
        && match profile.clone() {
            Some(user) => team.coach.eq(&user.into()),
            None => false,
        };

    let player_statistics = game.player_statistics(params.team_id, params.player_id_in_game);

    let stats = PlayerStatistics {
        games_number: 1,
        passing_completions: player_statistics.passing_completions as i64,
        throwing_completions: player_statistics.throwing_completions as i64,
        interceptions: player_statistics.interceptions as i64,
        casualties: player_statistics.casualties as i64,
        touchdowns: player_statistics.touchdowns as i64,
        most_valuable_player: player_statistics.most_valuable_player as i64,
        star_player_points: player_statistics.star_player_points as i64,
    };

    Ok(PlayerPage::get(
        app_state,
        profile,
        alert_message,
        format!(
            "added_player?player_id_in_game={}&team_id={}&game_id={}",
            player.id, team.id, game.id
        ),
        number,
        player,
        vec![],
        team,
        editable,
        params.edit.unwrap_or(false) && editable,
        can_buy,
        false,
        stats,
    ))
}

#[derive(Deserialize)]
pub struct AddedPlayerForm {
    pub player_number: Option<i32>,
    pub journeyman_to_recruit_id_in_game: Option<i32>,
}

pub async fn update_added_player(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<AddedPlayerQueryParams>,
    Form(form): Form<AddedPlayerForm>,
) -> Result<Redirect, Redirect> {
    // Player number
    if let (Some(profile), Some(player_number)) = (profile.clone(), form.player_number) {
        games::update_number_for_added_player_in_game(
            &app_state,
            &profile,
            params.team_id,
            params.player_id_in_game,
            params.game_id,
            player_number,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id_in_game,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;

        return Ok(Redirect::to(&format!(
            "../games/game?id={}",
            params.game_id
        )));
    }

    // Recruit journeyman
    if let (Some(profile), Some(journeyman_to_recruit_id_in_game)) =
        (profile.clone(), form.journeyman_to_recruit_id_in_game)
    {
        players::buy_journeyman_in_game_for_team(
            &app_state,
            &profile,
            params.team_id,
            journeyman_to_recruit_id_in_game,
            params.game_id,
        )
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./player?player_id={}&team_id={}&alert_message={}&edit={}",
                params.player_id_in_game,
                params.team_id,
                app_error,
                params.edit.unwrap_or(false),
            )))
        })?;

        return Ok(Redirect::to(&format!(
            "../teams/team?id={}",
            params.team_id
        )));
    }

    Ok(Redirect::to(&format!(
        "./player?player_id={}&team_id={}&edit={}",
        params.player_id_in_game,
        params.team_id,
        params.edit.unwrap_or(false),
    )))
}
