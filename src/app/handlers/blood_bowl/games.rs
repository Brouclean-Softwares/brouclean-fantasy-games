use crate::AppState;
use crate::app::templates::blood_bowl::games::{GamePage, GamesPage, NewGamePage};
use crate::app::templates::{AlertMessage, AlertType};
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::{games, teams};
use crate::data::users::{MayBeUser, User};
use crate::errors::AppError;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use blood_bowl_rs::actions::Success;
use blood_bowl_rs::events::GameEvent;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::positions::{Keyword, Position};
use blood_bowl_rs::prayers::PrayerToNuffle;
use blood_bowl_rs::skills::Skill;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
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
    MayBeUser(profile): MayBeUser,
) -> Result<GamesPage, AppError> {
    let games_playing = games::select_all_playing(&app_state).await?;
    let games_scheduled = games::select_all_scheduled(&app_state).await?;
    let mut games_played = games::select_all_played(&app_state).await?;
    games_played.reverse();

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
    MayBeUser(profile): MayBeUser,
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
    pub resurrection: Option<bool>,
    pub position: Option<Position>,
    pub injury: Option<Injury>,
    pub hatred: Option<Keyword>,
    pub other_player_event: Option<String>,
    pub success: Option<Success>,
    pub half_time: Option<String>,
    pub extra_time: Option<String>,
    pub first_team_penalties_score: Option<usize>,
    pub second_team_penalties_score: Option<usize>,
    pub end_game: Option<String>,
    pub first_team_winnings: Option<String>,
    pub first_team_stalled: Option<bool>,
    pub second_team_winnings: Option<String>,
    pub second_team_stalled: Option<bool>,
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
        tracing::error!(error_message);
        Redirect::to("/blood_bowl/games").into_response()
    }
}

pub async fn update(
    State(app_state): State<AppState>,
    profile: User,
    Form(form): Form<GameForm>,
) -> Result<Redirect, Response> {
    let redirect_ok = Redirect::to(&format!("/blood_bowl/games/game?id={}", form.game_id));

    let game_before_update = games::select_by_id(&app_state, form.game_id)
        .await
        .map_err(|err| {
            redirect_when_update_ko(&app_state, &profile, None, &None, err.to_string())
        })?;

    let mut game_to_update = game_before_update.clone();

    let competition = Competition::select_for_game_id(&app_state, game_to_update.id)
        .await
        .map_err(|err| {
            redirect_when_update_ko(&app_state, &profile, None, &None, err.to_string())
        })?;

    let mut event: Option<GameEvent> = None;

    // Scheduling
    if let Some(game_date) = form.game_at {
        game_to_update.game_at = NaiveDateTime::parse_from_str(&*game_date, "%Y-%m-%dT%H:%M")
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.to_string(),
                )
            })?;

        games::update_schedule(&app_state, &profile, &game_to_update)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
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
                    Some(&game_before_update),
                    &competition,
                    err.to_string(),
                )
            })?;

        game_to_update.game_at = game_start;
        game_to_update.start();

        games::update_start(&app_state, &profile, &game_to_update)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    // Cancel start
    if form.cancel_start.is_some() {
        games::cancel_start(&app_state, &profile, &game_to_update)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.to_string(),
                )
            })?;
    }

    // Cancel event
    if form.cancel_last_event.is_some() {
        event = game_to_update.cancel_last_event().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(event) = &event {
            games::update_after_event_cancelled(&app_state, &profile, &game_to_update, event)
                .await
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.to_string(),
                    )
                })?;
        }
    }

    // Fans
    if let (Some(first_fan_factor), Some(second_fan_factor)) =
        (form.first_team_fan_factor, form.second_team_fan_factor)
    {
        if form.auto.is_some() {
            game_to_update.generate_fans().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;
        } else {
            let first_fan_factor: u8 = first_fan_factor.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state, &profile,
                    Some(&game_before_update), &competition,
                    "Veuillez remplir la valeur de fan factor (D3 + fans dévoués) ou bien générer en automatique".to_string(),
                )
            })?;

            let second_fan_factor: u8 = second_fan_factor.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state, &profile,
                    Some(&game_before_update), &competition,
                    "Veuillez remplir la valeur de fan factor (D3 + fans dévoués) ou bien générer en automatique".to_string(),
                )
            })?;

            game_to_update
                .set_team_fan_factor(game_to_update.first_team.clone(), first_fan_factor)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;

            game_to_update
                .set_team_fan_factor(game_to_update.second_team.clone(), second_fan_factor)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Weather
    if let Some(weather) = form.weather {
        if form.auto.is_some() {
            game_to_update.generate_weather().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;
        } else {
            game_to_update.push_weather(weather).map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Journey men
    if form.generate_journeymen.is_some() && !game_to_update.journeymen_ok() {
        let _ = game_to_update.generate_journeymen().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Inducements
    if let Some(inducement) = form.first_team_inducement {
        let inducement = serde_json::from_str(&inducement).map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.to_string(),
            )
        })?;

        game_to_update
            .team_buy_inducement(game_to_update.first_team.id.clone(), inducement)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(inducement) = form.second_team_inducement {
        let inducement = serde_json::from_str(&inducement).map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.to_string(),
            )
        })?;

        game_to_update
            .team_buy_inducement(game_to_update.second_team.id.clone(), inducement)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Prayers
    if let Some(mut prayer) = form.first_team_prayer {
        if form.auto.is_some() {
            prayer = PrayerToNuffle::roll(&game_to_update.version);
        }

        game_to_update
            .push_prayer(game_to_update.first_team.id.clone(), prayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(mut prayer) = form.second_team_prayer {
        if form.auto.is_some() {
            prayer = PrayerToNuffle::roll(&game_to_update.version);
        }

        game_to_update
            .push_prayer(game_to_update.second_team.id.clone(), prayer)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Toss
    if let Some(toss_winner) = form.toss_winner {
        if form.auto.is_some() {
            game_to_update.generate_toss_winner().map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;
        } else {
            game_to_update
                .push_toss_winner(toss_winner)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(kicking_team) = form.kicking_team {
        game_to_update
            .push_kicking_team(kicking_team)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Resurrection
    if let (Some(team_id), Some(true)) = (form.team_id, form.resurrection) {
        game_to_update
            .push_resurrection(team_id, 0, form.position)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            let last_event = last_event.clone();

            if matches!(last_event, GameEvent::Resurrection { .. }) {
                games::update_after_event_inserted(
                    &app_state,
                    &profile,
                    &mut game_to_update,
                    &last_event,
                )
                .await
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.to_string(),
                    )
                })?;

                event = Some(last_event.clone());
            }
        }
    }

    // Injuries
    if let (Some(team_id), Some(player_id), Some(injury)) =
        (form.team_id, form.player_id, form.injury)
    {
        game_to_update
            .push_injury(team_id, player_id, injury)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Hatred
    if let (Some(team_id), Some(player_id), Some(keyword)) =
        (form.team_id, form.player_id, form.hatred)
    {
        game_to_update
            .push_hatred(team_id, player_id, keyword)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Other player event
    if let (Some(team_id), Some(player_id), Some(other_player_event)) =
        (form.team_id, form.player_id, form.other_player_event)
    {
        // Sent-off
        if other_player_event.eq("sent_off") {
            game_to_update
                .push_sent_off(team_id, player_id)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;

            if let Some(last_event) = game_to_update.events.last() {
                event = Some(last_event.clone());
            }
        }
        // Sent-off
        else if other_player_event.eq("pushed_into_crowd") {
            game_to_update
                .push_pushed_into_crowd(team_id, player_id)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;

            if let Some(last_event) = game_to_update.events.last() {
                event = Some(last_event.clone());
            }
        }
        // Regeneration
        else if other_player_event.eq("regeneration") {
            game_to_update
                .push_player_skill(team_id, player_id, Skill::Regeneration)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;

            if let Some(last_event) = game_to_update.events.last() {
                event = Some(last_event.clone());
            }
        }
    }

    // Successes
    if let (Some(team_id), Some(player_id), Some(success)) =
        (form.team_id, form.player_id, form.success)
    {
        game_to_update
            .push_success(team_id, player_id, success)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Half-time
    if form.half_time.is_some() {
        game_to_update.end_first_half().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Extra-time
    if form.extra_time.is_some() {
        game_to_update.start_extra_time().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Penalties
    if let (Some(first_team_penalties_score), Some(second_team_penalties_score)) = (
        form.first_team_penalties_score,
        form.second_team_penalties_score,
    ) {
        game_to_update
            .push_penalties(first_team_penalties_score, second_team_penalties_score)
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Game end
    if form.end_game.is_some() {
        game_to_update.end_game().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Winnings
    if form.first_team_winnings.is_some() || form.second_team_winnings.is_some() {
        if form.auto.is_some() {
            let first_team_stalled = form.first_team_stalled.unwrap_or(false);
            let second_team_stalled = form.second_team_stalled.unwrap_or(false);

            game_to_update
                .generate_winnings(first_team_stalled, second_team_stalled)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        } else {
            if let Some(winnings) = form.first_team_winnings {
                let winnings: u32 = winnings.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state, &profile,
                        Some(&game_before_update), &competition,
                        "Veuillez remplir la valeur des gains (10000 * TD + Fans + 1 si non temporisé / 2) ou bien générer en automatique".to_string(),
                    )
                })?;

                game_to_update
                    .push_winnings(game_to_update.first_team.id, winnings)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game_before_update),
                            &competition,
                            err.name("fr"),
                        )
                    })?;
            }

            if let Some(winnings) = form.second_team_winnings {
                let winnings: u32 = winnings.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state, &profile,
                        Some(&game_before_update), &competition,
                        "Veuillez remplir la valeur des gains (10000 * TD + Fans / 2) ou bien générer en automatique".to_string(),
                    )
                })?;

                game_to_update
                    .push_winnings(game_to_update.second_team.id, winnings)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game_before_update),
                            &competition,
                            err.name("fr"),
                        )
                    })?;
            }
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Dedicated fans
    if form.first_team_dedicated_fans_delta.is_some()
        || form.second_team_dedicated_fans_delta.is_some()
    {
        if form.auto.is_some() {
            game_to_update
                .generate_dedicated_fans_updates()
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        } else {
            if let Some(delta) = form.first_team_dedicated_fans_delta {
                let delta: i8 = delta.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        "Veuillez remplir le delta en terme de fans dévoués (0, +1 ou -1)"
                            .to_string(),
                    )
                })?;

                game_to_update
                    .push_dedicated_fans_update(game_to_update.first_team.id, delta)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game_before_update),
                            &competition,
                            err.name("fr"),
                        )
                    })?;
            }

            if let Some(delta) = form.second_team_dedicated_fans_delta {
                let delta: i8 = delta.parse().map_err(|_| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        "Veuillez remplir le delta en terme de fans dévoués (0, +1 ou -1)"
                            .to_string(),
                    )
                })?;

                game_to_update
                    .push_dedicated_fans_update(game_to_update.second_team.id, delta)
                    .map_err(|err| {
                        redirect_when_update_ko(
                            &app_state,
                            &profile,
                            Some(&game_before_update),
                            &competition,
                            err.name("fr"),
                        )
                    })?;
            }
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // MVPs
    if let Some(mvp_id) = form.first_team_mvp {
        game_to_update
            .push_success(
                game_to_update.first_team.id,
                mvp_id,
                Success::MostValuablePlayer,
            )
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    if let Some(mvp_id) = form.second_team_mvp {
        game_to_update
            .push_success(
                game_to_update.second_team.id,
                mvp_id,
                Success::MostValuablePlayer,
            )
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    err.name("fr"),
                )
            })?;

        if let Some(last_event) = game_to_update.events.last() {
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
                    Some(&game_before_update),
                    &competition,
                    "Veuillez remplir la valeur des pertes liées aux erreurs coûteuses".to_string(),
                )
            })?;

            game_to_update
                .push_expensive_mistakes(game_to_update.first_team.id, loss)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        }

        if let Some(loss) = form.second_team_expensive_mistakes {
            let loss: i32 = loss.parse().map_err(|_| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
                    &competition,
                    "Veuillez remplir la valeur des pertes liées aux erreurs coûteuses".to_string(),
                )
            })?;

            game_to_update
                .push_expensive_mistakes(game_to_update.second_team.id, loss)
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.name("fr"),
                    )
                })?;
        }

        if let Some(last_event) = game_to_update.events.last() {
            event = Some(last_event.clone());
        }
    }

    // Game closure
    if form.close_game.is_some() {
        game_to_update.close_game().map_err(|err| {
            redirect_when_update_ko(
                &app_state,
                &profile,
                Some(&game_before_update),
                &competition,
                err.name("fr"),
            )
        })?;

        if let Some(last_event) = game_to_update.events.last() {
            let last_event = last_event.clone();

            if matches!(last_event, GameEvent::GameClosure) {
                games::update_after_event_inserted(
                    &app_state,
                    &profile,
                    &mut game_to_update,
                    &last_event,
                )
                .await
                .map_err(|err| {
                    redirect_when_update_ko(
                        &app_state,
                        &profile,
                        Some(&game_before_update),
                        &competition,
                        err.to_string(),
                    )
                })?;

                event = Some(last_event.clone());
            }
        }
    }

    // Update after event if some
    if let Some(event) = event {
        games::update_after_event(&app_state, &profile, &mut game_to_update, &event)
            .await
            .map_err(|err| {
                redirect_when_update_ko(
                    &app_state,
                    &profile,
                    Some(&game_before_update),
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
        if id < 0 { None } else { Some(id) }
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
                .map_err(|error| {
                    tracing::error!("{}", error);

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
                    .map_err(|error| {
                        tracing::error!("{}", error);

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
                .map_err(|error| {
                    tracing::error!("{}", error);

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
                tracing::error!("{}", error);

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
            Err(app_error.log_and_redirect(Redirect::to(&format!(
                "./game?id={}&message={}",
                form.id, app_error
            ))))
        })?;

    games::delete(&app_state, &profile, form.id.clone())
        .await
        .or_else(|app_error| {
            Err(app_error.log_and_redirect(Redirect::to(&format!(
                "./game?id={}&message={}",
                form.id, app_error
            ))))
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
