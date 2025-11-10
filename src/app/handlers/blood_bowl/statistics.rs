use crate::app::templates::blood_bowl::statistics::{StatisticsPage, TeamsStatisticList};
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

    let lists_length = 5;

    let teams_top_victories = statistics::select_teams_victories_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_games = statistics::select_teams_games_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_values = statistics::select_teams_value_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_star_player_points =
        statistics::select_teams_star_player_points_top_5(&app_state)
            .await
            .or_else(error_redirect)?;

    let teams_top_touchdowns = statistics::select_teams_touchdowns_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_casualties = statistics::select_teams_casualties_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_interceptions = statistics::select_teams_interceptions_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_deflections = statistics::select_teams_deflections_top_5(&app_state)
        .await
        .or_else(error_redirect)?;

    let teams_top_passing_completions =
        statistics::select_teams_passing_completions_top_5(&app_state)
            .await
            .or_else(error_redirect)?;

    let teams_top_throwing_completions =
        statistics::select_teams_throwing_completions_top_5(&app_state)
            .await
            .or_else(error_redirect)?;

    Ok(StatisticsPage {
        navigation_bar: NavigationBar::get(&app_state, &profile),
        breadcrumb: blood_bowl::breadcrumb(),
        teams_top_victories: TeamsStatisticList::from(
            String::from("Victoires"),
            lists_length,
            teams_top_victories,
        ),
        teams_top_games: TeamsStatisticList::from(
            String::from("Nombre de matchs"),
            lists_length,
            teams_top_games,
        ),
        teams_top_values: TeamsStatisticList::from(
            String::from("Valeur d'équipe (VE)"),
            lists_length,
            teams_top_values,
        ),
        teams_top_star_player_points: TeamsStatisticList::from(
            String::from("Points de star players (PSP)"),
            lists_length,
            teams_top_star_player_points,
        ),
        teams_top_touchdowns: TeamsStatisticList::from(
            String::from("Touchdowns (TD)"),
            lists_length,
            teams_top_touchdowns,
        ),
        teams_top_casualties: TeamsStatisticList::from(
            String::from("Éliminations (ELI)"),
            lists_length,
            teams_top_casualties,
        ),
        teams_top_interceptions: TeamsStatisticList::from(
            String::from("Interceptions (INT)"),
            lists_length,
            teams_top_interceptions,
        ),
        teams_top_deflections: TeamsStatisticList::from(
            String::from("Détournements (DET)"),
            lists_length,
            teams_top_deflections,
        ),
        teams_top_passing_completions: TeamsStatisticList::from(
            String::from("Passes (PAS)"),
            lists_length,
            teams_top_passing_completions,
        ),
        teams_top_throwing_completions: TeamsStatisticList::from(
            String::from("Lancers de coéquipier (LAN)"),
            lists_length,
            teams_top_throwing_completions,
        ),
    })
}
