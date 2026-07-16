use crate::AppState;
use crate::data::blood_bowl::statistics::{StatisticElement, StatisticRow, Statistics};
use crate::errors::AppError;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamResults {
    pub victories: i64,
    pub draws: i64,
    pub losses: i64,
}

impl TeamResults {
    pub fn new() -> Self {
        Self {
            victories: 0,
            draws: 0,
            losses: 0,
        }
    }

    pub fn total_played(&self) -> i64 {
        self.victories + self.draws + self.losses
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamStatistics {
    pub passing_completions: i64,
    pub throwing_completions: i64,
    pub interceptions: i64,
    pub casualties: i64,
    pub touchdowns: i64,
    pub star_player_points: i64,
}

impl TeamStatistics {
    pub fn new() -> Self {
        Self {
            passing_completions: 0,
            throwing_completions: 0,
            interceptions: 0,
            casualties: 0,
            touchdowns: 0,
            star_player_points: 0,
        }
    }
}

pub struct TeamsTopStatistics {
    pub teams_top_victories: Statistics,
    pub teams_top_games: Statistics,
    pub teams_top_star_player_points: Statistics,
    pub teams_top_touchdowns: Statistics,
    pub teams_top_casualties: Statistics,
    pub teams_top_interceptions: Statistics,
    pub teams_top_passing_completions: Statistics,
    pub teams_top_throwing_completions: Statistics,
    pub teams_top_injuries: Statistics,
    pub teams_top_deaths: Statistics,
}

impl TeamsTopStatistics {
    pub fn empty() -> Self {
        Self {
            teams_top_victories: Statistics::empty(),
            teams_top_games: Statistics::empty(),
            teams_top_star_player_points: Statistics::empty(),
            teams_top_touchdowns: Statistics::empty(),
            teams_top_casualties: Statistics::empty(),
            teams_top_interceptions: Statistics::empty(),
            teams_top_passing_completions: Statistics::empty(),
            teams_top_throwing_completions: Statistics::empty(),
            teams_top_injuries: Statistics::empty(),
            teams_top_deaths: Statistics::empty(),
        }
    }

    pub async fn global(state: &AppState) -> Result<Self, AppError> {
        Ok(Self {
            teams_top_victories: select_teams_victories_top(state).await?,
            teams_top_games: select_teams_games_top(state).await?,
            teams_top_star_player_points: select_teams_star_player_points_top(state).await?,
            teams_top_touchdowns: select_teams_touchdowns_top(state).await?,
            teams_top_casualties: select_teams_casualties_top(state).await?,
            teams_top_interceptions: select_teams_interceptions_top(state).await?,
            teams_top_passing_completions: select_teams_passing_completions_top(state).await?,
            teams_top_throwing_completions: select_teams_throwing_completions_top(state).await?,
            teams_top_injuries: select_teams_injuries_top(state).await?,
            teams_top_deaths: select_teams_deaths_top(state).await?,
        })
    }

    pub async fn for_competition_id(
        state: &AppState,
        competition_id: i32,
    ) -> Result<Self, AppError> {
        Ok(Self {
            teams_top_victories: select_teams_victories_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_games: select_teams_games_top_for_competition_id(state, competition_id)
                .await?,
            teams_top_star_player_points: select_teams_star_player_points_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_touchdowns: select_teams_touchdowns_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_casualties: select_teams_casualties_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_interceptions: select_teams_interceptions_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_passing_completions: select_teams_passing_completions_top_for_competition_id(
                state,
                competition_id,
            )
            .await?,
            teams_top_throwing_completions:
                select_teams_throwing_completions_top_for_competition_id(state, competition_id)
                    .await?,
            teams_top_injuries: select_teams_injuries_top_for_competition_id(state, competition_id)
                .await?,
            teams_top_deaths: select_teams_deaths_top_for_competition_id(state, competition_id)
                .await?,
        })
    }
}

pub async fn select_results(state: &AppState, team_id: i32) -> Result<TeamResults, AppError> {
    tracing::debug!("select_results for team_id={}", team_id);

    let results: Option<TeamResults> = sqlx::query_as(
        "SELECT
                COALESCE(SUM(
                    CASE
                        WHEN (first_team_id = $1 AND first_team_is_winner = TRUE)
                            OR (second_team_id = $1 AND second_team_is_winner = TRUE)
                        THEN 1 ELSE 0
                    END
                ), 0) AS victories,
            
                COALESCE(SUM(
                    CASE
                        WHEN (
                            (first_team_id = $1 OR second_team_id = $1)
                            AND first_team_is_winner = FALSE
                            AND second_team_is_winner = FALSE
                        )
                        THEN 1 ELSE 0
                    END
                ), 0) AS draws,
            
                COALESCE(SUM(
                    CASE
                        WHEN (first_team_id = $1 AND second_team_is_winner = TRUE)
                            OR (second_team_id = $1 AND first_team_is_winner = TRUE)
                        THEN 1 ELSE 0
                    END
                ), 0) AS losses
            
            FROM bb_games
            WHERE closed_at IS NOT NULL
            AND (first_team_id = $1 OR second_team_id = $1)",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(results) = results {
        Ok(results.into())
    } else {
        Ok(TeamResults::new())
    }
}

pub async fn select_results_for_competition(
    state: &AppState,
    team_id: i32,
    competition_id: i32,
) -> Result<TeamResults, AppError> {
    tracing::debug!(
        "select_results_for_competition for team_id={} and competition_id={}",
        team_id,
        competition_id
    );

    let results: Option<TeamResults> = sqlx::query_as(
        "SELECT
                COALESCE(SUM(
                    CASE
                        WHEN (bb_games.first_team_id = $1 AND bb_games.first_team_is_winner = TRUE)
                            OR (bb_games.second_team_id = $1 AND bb_games.second_team_is_winner = TRUE)
                        THEN 1 ELSE 0
                    END
                ), 0) AS victories,
            
                COALESCE(SUM(
                    CASE
                        WHEN (
                            (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)
                            AND bb_games.first_team_is_winner = FALSE
                            AND bb_games.second_team_is_winner = FALSE
                        )
                        THEN 1 ELSE 0
                    END
                ), 0) AS draws,
            
                COALESCE(SUM(
                    CASE
                        WHEN (bb_games.first_team_id = $1 AND bb_games.second_team_is_winner = TRUE)
                            OR (bb_games.second_team_id = $1 AND bb_games.first_team_is_winner = TRUE)
                        THEN 1 ELSE 0
                    END
                ), 0) AS losses
            
            FROM bb_games
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            WHERE bb_competitions_stages_schedule.competition_id = $2
            AND bb_games.closed_at IS NOT NULL
            AND (bb_games.first_team_id = $1 OR bb_games.second_team_id = $1)",
    )
        .bind(team_id.clone())
        .fetch_optional(&state.db)
        .await?;

    if let Some(results) = results {
        Ok(results.into())
    } else {
        Ok(TeamResults::new())
    }
}

pub async fn select_statistics(state: &AppState, team_id: i32) -> Result<TeamStatistics, AppError> {
    tracing::debug!("select_statistics for team_id={}", team_id);

    let statistics: Option<TeamStatistics> = sqlx::query_as(
        "SELECT COALESCE(SUM(passing_completions), 0) as passing_completions,
                    COALESCE(SUM(throwing_completions), 0) as throwing_completions,
                    COALESCE(SUM(interceptions), 0) as interceptions,
                    COALESCE(SUM(casualties), 0) as casualties,
                    COALESCE(SUM(touchdowns), 0) as touchdowns,
                    COALESCE(SUM(star_player_points), 0) as star_player_points
            FROM bb_games_teams_players
            WHERE team_id = $1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(statistics) = statistics {
        Ok(statistics.into())
    } else {
        Ok(TeamStatistics::new())
    }
}

pub async fn select_teams_victories_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_victories_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (
                (bb_games.first_team_id = bb_teams.id AND bb_games.first_team_is_winner)
                OR
                (bb_games.second_team_id = bb_teams.id AND bb_games.second_team_is_winner)
            )
            AND bb_games.started_at IS NOT NULL
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

pub async fn select_teams_victories_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_victories_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (
                (bb_games.first_team_id = bb_teams.id AND bb_games.first_team_is_winner)
                OR
                (bb_games.second_team_id = bb_teams.id AND bb_games.second_team_is_winner)
            )
            AND bb_games.started_at IS NOT NULL
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_games.id) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
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
                    NULL as position,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (bb_games.first_team_id = bb_teams.id OR bb_games.second_team_id = bb_teams.id)
            AND bb_games.started_at IS NOT NULL
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

pub async fn select_teams_games_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_games_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_games.id) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games
            ON (bb_games.first_team_id = bb_teams.id OR bb_games.second_team_id = bb_teams.id)
            AND bb_games.started_at IS NOT NULL
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games.id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_games.id) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
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
                    NULL as position,
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

pub async fn select_teams_star_player_points_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_star_player_points_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.star_player_points) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.star_player_points) > 0
            ORDER BY SUM(bb_games_teams_players.star_player_points) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_touchdowns_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_touchdowns_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.touchdowns) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.touchdowns) > 0
            ORDER BY SUM(bb_games_teams_players.touchdowns) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_casualties_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_casualties_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.casualties) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.casualties) > 0
            ORDER BY SUM(bb_games_teams_players.casualties) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_deaths_top(state: &AppState) -> Result<Statistics, AppError> {
    tracing::debug!("select_teams_deaths_top");

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_teams_players
            ON bb_teams_players.team_id = bb_teams.id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            WHERE bb_players_injuries.injury = 'Dead'
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

pub async fn select_teams_injuries_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_injuries_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_teams_players
            ON bb_teams_players.team_id = bb_teams.id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_players_injuries.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}

pub async fn select_teams_deaths_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_deaths_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(COUNT(bb_players_injuries.injury) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_teams_players
            ON bb_teams_players.team_id = bb_teams.id
            INNER JOIN bb_players_injuries
            ON bb_players_injuries.player_id = bb_teams_players.player_id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_players_injuries.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            AND bb_players_injuries.injury = 'Dead'
            GROUP BY bb_teams.id
            ORDER BY COUNT(bb_players_injuries.injury) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_interceptions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_interceptions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.interceptions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.interceptions) > 0
            ORDER BY SUM(bb_games_teams_players.interceptions) DESC
            LIMIT 5",
    )
    .bind(competition_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_passing_completions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_passing_completions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.passing_completions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.passing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.passing_completions) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
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
                    NULL as position,
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

pub async fn select_teams_throwing_completions_top_for_competition_id(
    state: &AppState,
    competition_id: i32,
) -> Result<Statistics, AppError> {
    tracing::debug!(
        "select_teams_throwing_completions_top_for_competition_id with competition_id={}",
        competition_id
    );

    let stats: Vec<StatisticRow> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.id as team_id,
                    bb_teams.external_logo_url,
                    bb_teams.roster,
                    bb_teams.name,
                    NULL as position,
                    CAST(SUM(bb_games_teams_players.throwing_completions) as VARCHAR) as statistic_value
            FROM bb_teams
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.team_id = bb_teams.id
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.game_id = bb_games_teams_players.game_id
            WHERE bb_competitions_stages_schedule.competition_id = $1
            GROUP BY bb_teams.id
            HAVING SUM(bb_games_teams_players.throwing_completions) > 0
            ORDER BY SUM(bb_games_teams_players.throwing_completions) DESC
            LIMIT 5",
    )
        .bind(competition_id.clone())
        .fetch_all(&state.db)
        .await?;

    Ok(Statistics {
        statistic_element: StatisticElement::Team,
        statistics_rows: stats,
    })
}
