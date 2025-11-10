use crate::app::templates::blood_bowl::statistics::{StatisticList, StatisticsPage};
use crate::app::templates::{blood_bowl, NavigationBar};
use crate::data::blood_bowl::statistics;
use crate::data::users::User;
use crate::AppState;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(statistics))
}

pub async fn statistics(
    State(app_state): State<AppState>,
    profile: Option<User>,
) -> Result<StatisticsPage, Redirect> {
    let error_redirect = |error| {
        tracing::debug!("Error : {}", error);
        Err(Redirect::to("/blood_bowl"))
    };

    let teams_top_victories = statistics::select_teams_victories_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_games = statistics::select_teams_games_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_star_player_points = statistics::select_teams_star_player_points_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_touchdowns = statistics::select_teams_touchdowns_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_casualties = statistics::select_teams_casualties_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_injuries = statistics::select_teams_injuries_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_interceptions = statistics::select_teams_interceptions_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_deflections = statistics::select_teams_deflections_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_passing_completions =
        statistics::select_teams_passing_completions_top(&app_state)
            .await
            .or_else(error_redirect)?;

    let teams_top_throwing_completions =
        statistics::select_teams_throwing_completions_top(&app_state)
            .await
            .or_else(error_redirect)?;

    let players_top_star_player_points =
        statistics::select_players_star_player_points_top(&app_state)
            .await
            .or_else(error_redirect)?;

    let players_top_touchdowns = statistics::select_players_touchdowns_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let players_top_casualties = statistics::select_players_casualties_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let players_top_injuries = statistics::select_players_injuries_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let players_top_interceptions = statistics::select_players_interceptions_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let players_top_deflections = statistics::select_players_deflections_top(&app_state)
        .await
        .or_else(error_redirect)?;

    let players_top_passing_completions =
        statistics::select_players_passing_completions_top(&app_state)
            .await
            .or_else(error_redirect)?;

    let players_top_throwing_completions =
        statistics::select_players_throwing_completions_top(&app_state)
            .await
            .or_else(error_redirect)?;

    Ok(StatisticsPage {
        navigation_bar: NavigationBar::get(&app_state, &profile),
        breadcrumb: blood_bowl::breadcrumb(),
        teams_top_victories: StatisticList::from(String::from("Victoires"), teams_top_victories),
        teams_top_games: StatisticList::from(String::from("Nombre de matchs"), teams_top_games),
        teams_top_star_player_points: StatisticList::from(
            String::from("Points de star players (PSP)"),
            teams_top_star_player_points,
        ),
        teams_top_touchdowns: StatisticList::from(
            String::from("Touchdowns (TD)"),
            teams_top_touchdowns,
        ),
        teams_top_casualties: StatisticList::from(
            String::from("Éliminations (ELI)"),
            teams_top_casualties,
        ),
        teams_top_injuries: StatisticList::from(String::from("Blessures"), teams_top_injuries),
        teams_top_interceptions: StatisticList::from(
            String::from("Interceptions (INT)"),
            teams_top_interceptions,
        ),
        teams_top_deflections: StatisticList::from(
            String::from("Détournements (DET)"),
            teams_top_deflections,
        ),
        teams_top_passing_completions: StatisticList::from(
            String::from("Passes (PAS)"),
            teams_top_passing_completions,
        ),
        teams_top_throwing_completions: StatisticList::from(
            String::from("Lancers de coéquipier (LAN)"),
            teams_top_throwing_completions,
        ),
        players_top_star_player_points: StatisticList::from(
            String::from("Points de star players (PSP)"),
            players_top_star_player_points,
        ),
        players_top_touchdowns: StatisticList::from(
            String::from("Touchdowns (TD)"),
            players_top_touchdowns,
        ),
        players_top_casualties: StatisticList::from(
            String::from("Éliminations (ELI)"),
            players_top_casualties,
        ),
        players_top_injuries: StatisticList::from(String::from("Blessures"), players_top_injuries),
        players_top_interceptions: StatisticList::from(
            String::from("Interceptions (INT)"),
            players_top_interceptions,
        ),
        players_top_deflections: StatisticList::from(
            String::from("Détournements (DET)"),
            players_top_deflections,
        ),
        players_top_passing_completions: StatisticList::from(
            String::from("Passes (PAS)"),
            players_top_passing_completions,
        ),
        players_top_throwing_completions: StatisticList::from(
            String::from("Lancers de coéquipier (LAN)"),
            players_top_throwing_completions,
        ),
    })
}
