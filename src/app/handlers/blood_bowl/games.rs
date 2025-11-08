use crate::app::templates::blood_bowl::games::{GamePage, GamesPage, NewGamePage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::{games, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use blood_bowl_rs::actions::Success;
use blood_bowl_rs::events::GameEvent;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::prayers::PrayerToNuffle;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::weather::Weather;
use chrono::NaiveDateTime;
use serde::Deserialize;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .route("/", get(games))
        .route("/game", get(game).post(update))
        .route("/new", get(new).post(create))
        .route("/delete", post(delete))
}

pub async fn games(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<GamesPage, AppError> {
    let games_playing = games::select_all_playing(&app_state).await?;
    let games_scheduled = games::select_all_scheduled(&app_state).await?;
    let games_played = games::select_all_played(&app_state).await?;

    GamesPage::get(
        app_state,
        profile,
        games_playing,
        games_scheduled,
        games_played,
    )
}

#[derive(Deserialize)]
pub struct GameQueryParams {
    pub id: i32,
    pub edit_mode: Option<bool>,
}

pub async fn game(
    State(app_state): State<AppState>,
    profile: Option<User>,
    Query(params): Query<GameQueryParams>,
) -> Result<GamePage, AppError> {
    let game = games::select_by_id(&app_state, params.id).await?;

    let competition = Competition::select_for_game_id(&app_state, game.id).await?;

    let edit_mode = params.edit_mode.unwrap_or(false);

    Ok(GamePage::get(
        app_state,
        profile,
        game,
        competition,
        edit_mode,
    )?)
}

#[derive(Deserialize)]
pub struct GameForm {
    pub game_id: i32,
    pub game_at: Option<String>,
    pub started_at: Option<String>,
    pub cancel_start: Option<bool>,
    pub cancel_last_event: Option<bool>,
    pub auto: Option<bool>,
    pub first_team_fan_factor: Option<String>,
    pub second_team_fan_factor: Option<String>,
    pub weather: Option<Weather>,
    pub generate_journeymen: Option<bool>,
    pub first_team_inducement: Option<String>,
    pub second_team_inducement: Option<String>,
    pub first_team_prayer: Option<PrayerToNuffle>,
    pub second_team_prayer: Option<PrayerToNuffle>,
    pub toss_winner: Option<i32>,
    pub kicking_team: Option<i32>,
    pub team_id: Option<i32>,
    pub player_id: Option<i32>,
    pub injury: Option<Injury>,
    pub success: Option<Success>,
    pub end_game: Option<String>,
    pub first_team_winnings: Option<String>,
    pub second_team_winnings: Option<String>,
    pub first_team_dedicated_fans_delta: Option<String>,
    pub second_team_dedicated_fans_delta: Option<String>,
    pub first_team_mvp: Option<i32>,
    pub second_team_mvp: Option<i32>,
    pub first_team_expensive_mistakes: Option<String>,
    pub second_team_expensive_mistakes: Option<String>,
    pub close_game: Option<String>,
}

fn redirect_when_update_ko(
    app_state: &AppState,
    profile: &User,
    game: Option<&Game>,
    competition: &Option<Competition>,
    error_message: String,
) -> Response {
    if let Some(game) = game {
        GamePage::get_with_message(
            app_state.clone(),
            Some(profile.clone()),
            Some(AlertMessage {
                alert_type: AlertType::Danger,
                message: error_message,
            }),
            game.clone(),
            competition.clone(),
            false,
        )
        .into_response()
    } else {
        tracing::debug!(error_message);
        Redirect::to("/blood_bowl/games").into_response()
    }
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<GameForm>,
) -> Result<Redirect, Response> {
    let redirect_ok = Redirect::to(&format!("/blood_bowl/games/game?id={}", form.game_id));

    let mut game = games::select_by_id(&app_state, form.game_id)
        .await
        .map_err(|err| {
            redirect_when_update_ko(&app_state, &profile, None, &None, err.to_string())
        })?;

    let competition = Competition::select_for_game_id(&app_state, game.id)
        .await
        .map_err(|err| {
            redirect_when_update_ko(&app_state, &profile, None, &None, err.to_string())
        })?;

    let mut event: Option<GameEvent> = None;

    // Scheduling
    if let Some(game_date) = form.game_at {
        game.game_at =
            NaiveDateTime::parse_from_str(&*game_date, "%Y-%m-%dT%H:%M").map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        games::update_schedule(&app_state, &profile, &game)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    // Game start
    if let Some(game_date) = form.started_at {
        let game_start =
            NaiveDateTime::parse_from_str(&*game_date, "%Y-%m-%dT%H:%M").map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        game.game_at = game_start;
        game.start();

        games::update_start(&app_state, &profile, &game)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    // Cancel start
    if form.cancel_start.is_some() {
        games::cancel_start(&app_state, &profile, &game)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    // Cancel event
    if form.cancel_last_event.is_some() {
        event = game.cancel_last_event().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;
    }

    // Fans
    if let (Some(first_fan_factor), Some(second_fan_factor)) =
        (form.first_team_fan_factor, form.second_team_fan_factor)
    {
        if form.auto.is_some() {
            game.generate_fans().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        } else {
            let first_fan_factor: u8 = first_fan_factor.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state, &profile, Some(&game), &competition,
                    "Veuillez remplir la valeur de fan factor (D3 + fans dévoués) ou bien générer en automatique".to_string(),
                )
            })?;

            let second_fan_factor: u8 = second_fan_factor.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state, &profile, Some(&game), &competition,
                    "Veuillez remplir la valeur de fan factor (D3 + fans dévoués) ou bien générer en automatique".to_string(),
                )
            })?;

            game.set_team_fan_factor(game.first_team.clone(), first_fan_factor)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        err.to_string(),
                    )
                })?;

            game.set_team_fan_factor(game.second_team.clone(), second_fan_factor)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        err.to_string(),
                    )
                })?;
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Weather
    if let Some(weather) = form.weather {
        if form.auto.is_some() {
            game.generate_weather().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        } else {
            game.push_weather(weather).map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Journey men
    if form.generate_journeymen.is_some() && !game.journeymen_ok() {
        let _ = game.generate_journeymen().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Inducements
    if let Some(inducement) = form.first_team_inducement {
        let inducement = serde_json::from_str(&inducement).map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        game.team_buy_inducement(game.first_team.id.clone(), inducement)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(inducement) = form.second_team_inducement {
        let inducement = serde_json::from_str(&inducement).map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        game.team_buy_inducement(game.second_team.id.clone(), inducement)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Prayers
    if let Some(mut prayer) = form.first_team_prayer {
        if form.auto.is_some() {
            prayer = PrayerToNuffle::roll();
        }

        game.push_prayer(game.first_team.id.clone(), prayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(mut prayer) = form.second_team_prayer {
        if form.auto.is_some() {
            prayer = PrayerToNuffle::roll();
        }

        game.push_prayer(game.second_team.id.clone(), prayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Toss
    if let Some(toss_winner) = form.toss_winner {
        if form.auto.is_some() {
            game.generate_toss_winner().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        } else {
            game.push_toss_winner(toss_winner).map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(kicking_team) = form.kicking_team {
        game.push_kicking_team(kicking_team).map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Injuries
    if let (Some(team_id), Some(player_id), Some(injury)) =
        (form.team_id, form.player_id, form.injury)
    {
        game.push_injury(team_id, player_id, injury)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Successes
    if let (Some(team_id), Some(player_id), Some(success)) =
        (form.team_id, form.player_id, form.success)
    {
        game.push_success(team_id, player_id, success)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Game end
    if form.end_game.is_some() {
        game.end_game().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Winnings
    if form.first_team_winnings.is_some() || form.second_team_winnings.is_some() {
        if form.auto.is_some() {
            game.generate_winnings().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        } else {
            if let Some(winnings) = form.first_team_winnings {
                let winnings: u32 = winnings.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state, &profile, Some(&game), &competition,
                        "Veuillez remplir la valeur des gains (10000 * TD + Fans / 2) ou bien générer en automatique".to_string(),
                    )
                })?;

                game.push_winnings(game.first_team.id, winnings)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game),
                            &competition,
                            err.to_string(),
                        )
                    })?;
            }

            if let Some(winnings) = form.second_team_winnings {
                let winnings: u32 = winnings.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state, &profile, Some(&game), &competition,
                        "Veuillez remplir la valeur des gains (10000 * TD + Fans / 2) ou bien générer en automatique".to_string(),
                    )
                })?;

                game.push_winnings(game.second_team.id, winnings)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game),
                            &competition,
                            err.to_string(),
                        )
                    })?;
            }
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Dedicated fans
    if form.first_team_dedicated_fans_delta.is_some()
        || form.second_team_dedicated_fans_delta.is_some()
    {
        if form.auto.is_some() {
            game.generate_dedicated_fans_updates().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
        } else {
            if let Some(delta) = form.first_team_dedicated_fans_delta {
                let delta: i8 = delta.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        "Veuillez remplir le delta en terme de fans dévoués (0, +1 ou -1)"
                            .to_string(),
                    )
                })?;

                game.push_dedicated_fans_update(game.first_team.id, delta)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game),
                            &competition,
                            err.to_string(),
                        )
                    })?;
            }

            if let Some(delta) = form.second_team_dedicated_fans_delta {
                let delta: i8 = delta.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        "Veuillez remplir le delta en terme de fans dévoués (0, +1 ou -1)"
                            .to_string(),
                    )
                })?;

                game.push_dedicated_fans_update(game.second_team.id, delta)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game),
                            &competition,
                            err.to_string(),
                        )
                    })?;
            }
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // MVPs
    if let Some(mvp_id) = form.first_team_mvp {
        game.push_success(game.first_team.id, mvp_id, Success::MostValuablePlayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(mvp_id) = form.second_team_mvp {
        game.push_success(game.second_team.id, mvp_id, Success::MostValuablePlayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Expensive mistakes
    if form.first_team_expensive_mistakes.is_some() || form.second_team_expensive_mistakes.is_some()
    {
        if let Some(loss) = form.first_team_expensive_mistakes {
            let loss: i32 = loss.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    "Veuillez remplir la valeur des pertes liées aux erreurs coûteuses".to_string(),
                )
            })?;

            game.push_expensive_mistakes(game.first_team.id, loss)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        err.to_string(),
                    )
                })?;
        }

        if let Some(loss) = form.second_team_expensive_mistakes {
            let loss: i32 = loss.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    "Veuillez remplir la valeur des pertes liées aux erreurs coûteuses".to_string(),
                )
            })?;

            game.push_expensive_mistakes(game.second_team.id, loss)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game),
                        &competition,
                        err.to_string(),
                    )
                })?;
        }

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Game closure
    if form.close_game.is_some() {
        game.close_game().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game),
                &competition,
                err.to_string(),
            )
        })?;

        if let Some(last_event) = game.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Update after event if some
    if let Some(event) = event {
        games::update_after_event(&app_state, &profile, &game, &event)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    Ok(redirect_ok)
}

#[derive(Deserialize)]
pub struct NewGameQueryParams {
    pub first_team_id: Option<i32>,
    pub second_team_id: Option<i32>,
}

pub async fn new(
    State(app_state): State<AppState>,
    profile: User,
    Query(params): Query<NewGameQueryParams>,
) -> Result<NewGamePage, AppError> {
    let mut first_team: Option<Team> = None;

    if let Some(id) = params.first_team_id {
        first_team = Some(teams::select_by_id_with_staff_and_players(&app_state, id).await?);
    }

    let mut second_team: Option<Team> = None;

    if let Some(id) = params.second_team_id {
        second_team = Some(teams::select_by_id_with_staff_and_players(&app_state, id).await?);
    }

    let new_game_page = NewGamePage::get(app_state, profile, first_team, second_team);

    Ok(new_game_page)
}

#[derive(Deserialize)]
pub struct NewGameForm {
    pub first_team_id: Option<i32>,
    pub second_team_id: Option<i32>,
    pub scheduled_at: Option<String>,
}

pub async fn create(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<NewGameForm>,
) -> Result<Redirect, NewGamePage> {
    let fn_if_id_positive = |id| {
        if id < 0 {
            None
        } else {
            Some(id)
        }
    };

    let first_team_id = form.first_team_id.and_then(fn_if_id_positive);
    let second_team_id = form.second_team_id.and_then(fn_if_id_positive);

    match (first_team_id, second_team_id, form.scheduled_at) {
        (Some(first_team_id), None, None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?first_team_id={}",
            first_team_id
        ))),

        (None, Some(second_team_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?second_team_id={}",
            second_team_id
        ))),

        (Some(first_team_id), Some(second_team_id), None) => Ok(Redirect::to(&format!(
            "/blood_bowl/games/new?first_team_id={}&second_team_id={}",
            first_team_id, second_team_id
        ))),

        (Some(first_team_id), Some(second_team_id), Some(scheduled_at)) => {
            let first_team = teams::select_by_id_with_staff_and_players(&app_state, first_team_id)
                .await
                .map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "la première équipe est introuvable.".to_string(),
                        }),
                        None,
                        None,
                    )
                })?;

            let second_team =
                teams::select_by_id_with_staff_and_players(&app_state, second_team_id)
                    .await
                    .map_err(|_| {
                        NewGamePage::get_with_message(
                            app_state.clone(),
                            profile.clone(),
                            Some(AlertMessage {
                                alert_type: AlertType::Danger,
                                message: "la deuxième équipe est introuvable.".to_string(),
                            }),
                            None,
                            None,
                        )
                    })?;

            let scheduled_at = NaiveDateTime::parse_from_str(&*scheduled_at, "%Y-%m-%dT%H:%M")
                .map_err(|_| {
                    NewGamePage::get_with_message(
                        app_state.clone(),
                        profile.clone(),
                        Some(AlertMessage {
                            alert_type: AlertType::Danger,
                            message: "Veuillez remplir la date et l'heure du match.".to_string(),
                        }),
                        Some(first_team.clone()),
                        Some(second_team.clone()),
                    )
                })?;

            let game_id = games::create_friendly(
                &app_state,
                &profile,
                &first_team,
                &second_team,
                scheduled_at,
            )
            .await
            .map_err(|error| {
                NewGamePage::get_with_message(
                    app_state.clone(),
                    profile.clone(),
                    Some(AlertMessage {
                        alert_type: AlertType::Danger,
                        message: error.to_string(),
                    }),
                    Some(first_team.clone()),
                    Some(second_team.clone()),
                )
            })?;

            Ok(Redirect::to(&format!("./game?id={}", game_id)))
        }

        _ => Ok(Redirect::to("/blood_bowl/games/new")),
    }
}

#[derive(Deserialize)]
pub struct DeleteGameForm {
    pub id: i32,
}

pub async fn delete(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<DeleteGameForm>,
) -> Result<Redirect, Redirect> {
    let competition = Competition::select_for_game_id(&app_state, form.id.clone())
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./game?id={}&message={}",
                form.id, app_error
            )))
        })?;

    games::delete(&app_state, &profile, form.id.clone())
        .await
        .or_else(|app_error| {
            Err(Redirect::to(&format!(
                "./game?id={}&message={}",
                form.id, app_error
            )))
        })?;

    if let Some(competition) = competition {
        Ok(Redirect::to(&format!(
            "/blood_bowl/competitions/competition?id={}&tab=schedule",
            competition.id
        )))
    } else {
        Ok(Redirect::to("/blood_bowl"))
    }
}
