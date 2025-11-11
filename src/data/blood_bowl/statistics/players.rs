use crate::data::blood_bowl::statistics::{StatisticElement, StatisticRow, Statistics};
use crate::errors::AppError;
use crate::AppState;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct PlayerStatistics {
    pub games_number: i64,
    pub passing_completions: i64,
    pub throwing_completions: i64,
    pub interceptions: i64,
    pub casualties: i64,
    pub touchdowns: i64,
    pub most_valuable_player: i64,
    pub star_player_points: i64,
}

impl PlayerStatistics {
    pub fn new() -> Self {
        Self {
            games_number: 0,
            passing_completions: 0,
            throwing_completions: 0,
            interceptions: 0,
            casualties: 0,
            touchdowns: 0,
            most_valuable_player: 0,
            star_player_points: 0,
        }
    }
}

pub struct PlayersTopStatistics {
    pub players_top_star_player_points: Statistics,
    pub players_top_touchdowns: Statistics,
    pub players_top_casualties: Statistics,
    pub players_top_injuries: Statistics,
    pub players_top_interceptions: Statistics,
    pub players_top_passing_completions: Statistics,
    pub players_top_throwing_completions: Statistics,
}

impl PlayersTopStatistics {
    pub fn empty() -> Self {
        Self {
            players_top_star_player_points: Statistics::empty(),
            players_top_touchdowns: Statistics::empty(),
            players_top_casualties: Statistics::empty(),
            players_top_injuries: Statistics::empty(),
            players_top_interceptions: Statistics::empty(),
            players_top_passing_completions: Statistics::empty(),
            players_top_throwing_completions: Statistics::empty(),
        }
    }

    pub async fn global(state: &AppState) -> Result<Self, AppError> {
        Ok(Self {
            players_top_star_player_points: select_players_star_player_points_top(state).await?,
            players_top_touchdowns: select_players_touchdowns_top(state).await?,
            players_top_casualties: select_players_casualties_top(state).await?,
            players_top_injuries: select_players_injuries_top(state).await?,
            players_top_interceptions: select_players_interceptions_top(state).await?,
            players_top_passing_completions: select_players_passing_completions_top(state).await?,
            players_top_throwing_completions: select_players_throwing_completions_top(state)
                .await?,
        })
    }

    pub async fn for_team_id(state: &AppState, team_id: i32) -> Result<Self, AppError> {
        Ok(Self {
            players_top_star_player_points: select_players_star_player_points_top_for_team_id(
                state, team_id,
            )
            .await?,
            players_top_touchdowns: select_players_touchdowns_top_for_team_id(state, team_id)
                .await?,
            players_top_casualties: select_players_casualties_top_for_team_id(state, team_id)
                .await?,
            players_top_injuries: select_players_injuries_top_for_team_id(state, team_id).await?,
            players_top_interceptions: select_players_interceptions_top_for_team_id(state, team_id)
                .await?,
            players_top_passing_completions: select_players_passing_completions_top_for_team_id(
                state, team_id,
            )
            .await?,
            players_top_throwing_completions: select_players_throwing_completions_top_for_team_id(
                state, team_id,
            )
            .await?,
        })
    }

    pub async fn for_competition_id(
        state: &AppState,
        competition_id: i32,
    ) -> Result<Self, AppError> {
        Ok(Self {
            players_top_star_player_points:
                select_players_star_player_points_top_for_competition_id(state, competition_id)
                    .await?,
            players_top_touchdowns: select_players_touchdowns_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            players_top_casualties: select_players_casualties_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            players_top_injuries: select_players_injuries_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            players_top_interceptions: select_players_interceptions_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            players_top_passing_completions:
                select_players_passing_completions_top_for_competition_id(state, competition_id)
                    .await?,
            players_top_throwing_completions:
                select_players_throwing_completions_top_for_competition_id(state, competition_id)
                    .await?,
        })
    }
}

pub async fn select_statistics(
    state: &AppState,
    player_id: i32,
) -> Result<PlayerStatistics, AppError> {
    tracing::debug!("select_statistics for player_id={}", player_id);

    let statistics: Option<PlayerStatistics> = sqlx::query_as(
        "SELECT COUNT(game_id) as games_number,
                    COALESCE(SUM(passing_completions), 0) as passing_completions,
                    COALESCE(SUM(throwing_completions), 0) as throwing_completions,
                    COALESCE(SUM(interceptions), 0) as interceptions,
                    COALESCE(SUM(casualties), 0) as casualties,
                    COALESCE(SUM(touchdowns), 0) as touchdowns,
                    COALESCE(SUM(most_valuable_player), 0) as most_valuable_player,
                    COALESCE(SUM(star_player_points), 0) as star_player_points
            FROM bb_games_teams_players
            WHERE player_id = $1",
    )
    .bind(player_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(statistics) = statistics {
        Ok(statistics.into())
    } else {
        Ok(PlayerStatistics::new())
    }
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
                    bb_players.position,
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

pub async fn select_players_star_player_points_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_star_player_points_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.star_player_points) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.star_player_points) > 0
            ORDER BY SUM(bb_games_teams_players.star_player_points) DESC
            LIMIT 5",
    )
        .bind(team_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_star_player_points_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_star_player_points_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.star_player_points) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.star_player_points) > 0
            ORDER BY SUM(bb_games_teams_players.star_player_points) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_touchdowns_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_touchdowns_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.touchdowns) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.touchdowns) > 0
            ORDER BY SUM(bb_games_teams_players.touchdowns) DESC
            LIMIT 5",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_touchdowns_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_touchdowns_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.touchdowns) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.touchdowns) > 0
            ORDER BY SUM(bb_games_teams_players.touchdowns) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_casualties_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_casualties_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.casualties) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.casualties) > 0
            ORDER BY SUM(bb_games_teams_players.casualties) DESC
            LIMIT 5",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_casualties_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_casualties_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.casualties) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.casualties) > 0
            ORDER BY SUM(bb_games_teams_players.casualties) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_injuries_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_injuries_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_injuries_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_injuries_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_players_injuries.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_interceptions_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_interceptions_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.interceptions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.interceptions) > 0
            ORDER BY SUM(bb_games_teams_players.interceptions) DESC
            LIMIT 5",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_interceptions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_interceptions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.interceptions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.interceptions) > 0
            ORDER BY SUM(bb_games_teams_players.interceptions) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_passing_completions_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_passing_completions_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.passing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.passing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.passing_completions) DESC
            LIMIT 5",
    )
        .bind(team_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_passing_completions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_passing_completions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.passing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.passing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.passing_completions) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
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
                    bb_players.position,
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

pub async fn select_players_throwing_completions_top_for_team_id(
    state: &AppState,
    team_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_throwing_completions_top_for_team_id with team_id={}",
        team_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.throwing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            WHERE bb_teams.id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.throwing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.throwing_completions) DESC
            LIMIT 5",
    )
        .bind(team_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}

pub async fn select_players_throwing_completions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_players_throwing_completions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_players.name,
                    bb_players.position,
                    CAST(SUM(bb_games_teams_players.throwing_completions) as VARCHAR) as statistic_value
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players.id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_players.id, bb_teams.id
            HAVING SUM(bb_games_teams_players.throwing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.throwing_completions) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Player,
        statistics_rows: stats,
    })
}
