use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::rosters::Roster;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamStatisticRow {
    pub id: i32,
    pub external_logo_url: Option<String>,
    pub roster: Roster,
    pub name: String,
    pub statistic_value: String,
}

pub async fn select_teams_victories_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_victories_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (
                (bb_games.first_team_id = bb_teams.id AND bb_games.first_team_is_winner)
                OR
                (bb_games.second_team_id = bb_teams.id AND bb_games.second_team_is_winner)
            )
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_games.id) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_games_top_5(state: &AppState) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_victories_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (bb_games.first_team_id = bb_teams.id OR bb_games.second_team_id = bb_teams.id)
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_games.id) DESC
            LIMIT 5",
    )
        .fetch_all(&state.db)
        .await?;

    Ok(stats)
}

pub async fn select_teams_value_top_5(state: &AppState) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_value_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(bb_teams.value / 1000 as VARCHAR) || 'k' as statistic_value
            FROM bb_teams
            ORDER BY bb_teams.value DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_star_player_points_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_star_player_points_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.star_player_points) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.star_player_points) > 0
            ORDER BY SUM(bb_games_teams_players.star_player_points) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_touchdowns_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_touchdowns_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.touchdowns) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.touchdowns) > 0
            ORDER BY SUM(bb_games_teams_players.touchdowns) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_casualties_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_casualties_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.casualties) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.casualties) > 0
            ORDER BY SUM(bb_games_teams_players.casualties) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_interceptions_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_interceptions_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.interceptions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.interceptions) > 0
            ORDER BY SUM(bb_games_teams_players.interceptions) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_deflections_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_deflections_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.deflections) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.deflections) > 0
            ORDER BY SUM(bb_games_teams_players.deflections) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(stats)
}

pub async fn select_teams_passing_completions_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_passing_completions_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.passing_completions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.passing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.passing_completions) DESC
            LIMIT 5",
    )
        .fetch_all(&state.db)
        .await?;

    Ok(stats)
}

pub async fn select_teams_throwing_completions_top_5(
    state: &AppState,
) -> Result<Vec<TeamStatisticRow>, AppError> {
    tracing::debug!("select_teams_throwing_completions_top_5");

    let stats: Vec<TeamStatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(SUM(bb_games_teams_players.throwing_completions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.throwing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.throwing_completions) DESC
            LIMIT 5",
    )
        .fetch_all(&state.db)
        .await?;

    Ok(stats)
}
