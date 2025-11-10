use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::rosters::Roster;
use serde::Deserialize;

pub struct Statistics {
    pub statistic_element: StatisticElement,
    pub statistics_rows: Vec<StatisticRow>,
}

pub enum StatisticElement {
    Team,
    Player,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct StatisticRow {
    pub id: i32,
    pub team_id: i32,
    pub external_logo_url: Option<String>,
    pub roster: Roster,
    pub name: String,
    pub statistic_value: String,
}

pub async fn select_teams_victories_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_victories_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_teams_games_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_games_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_teams_star_player_points_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_star_player_points_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_star_player_points_top(
    state: &AppState,
) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_star_player_points_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.star_player_points) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.star_player_points) > 0
            ORDER BY SUM(bb_games_teams_players.star_player_points) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_touchdowns_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_touchdowns_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_touchdowns_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_touchdowns_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.touchdowns) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.touchdowns) > 0
            ORDER BY SUM(bb_games_teams_players.touchdowns) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_casualties_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_casualties_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_casualties_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_casualties_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.casualties) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.casualties) > 0
            ORDER BY SUM(bb_games_teams_players.casualties) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_injuries_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_injuries_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_teams_players
            ON bb_teams_players.team_id = bb_teams.id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_injuries_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_injuries_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            GROUP BY bb_players.id, bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_interceptions_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_interceptions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_interceptions_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_interceptions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.interceptions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.interceptions) > 0
            ORDER BY SUM(bb_games_teams_players.interceptions) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_deflections_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_deflections_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_deflections_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_deflections_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.deflections) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.deflections) > 0
            ORDER BY SUM(bb_games_teams_players.deflections) DESC
            LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_passing_completions_top(
    state: &AppState,
) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_passing_completions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_passing_completions_top(
    state: &AppState,
) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_passing_completions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.passing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.passing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.passing_completions) DESC
            LIMIT 5",
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_teams_throwing_completions_top(
    state: &AppState,
) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_throwing_completions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
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

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_players_throwing_completions_top(
    state: &AppState,
) -> Result<Statistics, AppError> {
    tracing::debug!("select_players_throwing_completions_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    CAST(SUM(bb_games_teams_players.throwing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.throwing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.throwing_completions) DESC
            LIMIT 5",
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}
