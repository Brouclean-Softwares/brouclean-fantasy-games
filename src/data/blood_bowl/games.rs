use crate::auth::profile;
use crate::data::blood_bowl::{coaches, players, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::games::{Game, GameSummary};
use blood_bowl_rs::players::Player;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::{Team, TeamSummary};
use blood_bowl_rs::versions::Version;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

pub async fn create(
    state: &AppState,
    coach: &User,
    team_a: &Team,
    team_b: &Team,
    played_at: NaiveDateTime,
) -> Result<i32, AppError> {
    tracing::debug!(
        "create by coach={:?} to play at {} for the following teams: team_a_id={} and team_b_id={}",
        coach,
        played_at,
        team_a.id,
        team_b.id,
    );

    let game = Game::create(
        None,
        Some(coach.clone().into()),
        team_a.version,
        played_at,
        &team_a,
        &team_b,
    )?;

    let mut transaction = state.db.begin().await?;

    let new_game_id: Id = sqlx::query_as(
        "INSERT INTO bb_games (
                version,
                played_at,
                created_by)
            VALUES ($1, $2, $3)
            RETURNING id",
    )
    .bind(game.version.clone())
    .bind(game.played_at.clone())
    .bind(coach.id.clone())
    .fetch_one(&mut *transaction)
    .await?;

    sqlx::query(
        "INSERT INTO bb_games_teams (
                game_id,
                coach_id,
                team_id,
                coach_name,
                team_name,
                team_roster,
                score,
                casualties,
                winner)
            SELECT $1,
                   bb_teams.coach_id,
                   bb_teams.id,
                   users.name,
                   bb_teams.name,
                   bb_teams.roster,
                   0,
                   0,
                   false
            FROM bb_teams
            LEFT JOIN users
            ON users.id = bb_teams.coach_id
            WHERE bb_teams.id = $2",
    )
    .bind(new_game_id.id.clone())
    .bind(team_a.id.clone())
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        "INSERT INTO bb_games_teams (
                game_id,
                coach_id,
                team_id,
                coach_name,
                team_name,
                team_roster,
                score,
                casualties,
                winner)
            SELECT $1,
                   bb_teams.coach_id,
                   bb_teams.id,
                   users.name,
                   bb_teams.name,
                   bb_teams.roster,
                   0,
                   0,
                   false
            FROM bb_teams
            LEFT JOIN users
            ON users.id = bb_teams.coach_id
            WHERE bb_teams.id = $2",
    )
    .bind(new_game_id.id.clone())
    .bind(team_b.id.clone())
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(new_game_id.id)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameRow {
    id: i32,
    version: Version,
    played_at: NaiveDateTime,
    created_by: Option<i32>,
    closed_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct GameTeamRow {
    coach_id: Option<i32>,
    team_id: i32,
    coach_name: String,
    team_name: String,
    team_roster: Roster,
    score: i32,
    casualties: i32,
    winner: bool,
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let game_row: GameRow = sqlx::query_as(
        "SELECT id,
                    version,
                    played_at,
                    created_by,
                    closed_at
            FROM bb_games
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let game_team_rows: Vec<GameTeamRow> = sqlx::query_as(
        "SELECT coach_id,
                    team_id,
                    coach_name,
                    team_name,
                    team_roster,
                    score,
                    casualties,
                    winner
            FROM bb_games_teams
            WHERE game_id = $1
            LIMIT 2",
    )
    .bind(id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut teams: Vec<Team> = Vec::new();

    for game_team_row in game_team_rows.clone() {
        let mut team: Team = Team {
            id: game_team_row.team_id,
            version: game_row.version,
            roster: game_team_row.team_roster,
            name: game_team_row.team_name,
            coach: Coach {
                id: game_team_row.coach_id,
                name: game_team_row.coach_name,
            },
            treasury: 0,
            external_logo_url: None,
            staff: Default::default(),
            players: vec![],
            games_played: vec![],
            game_playing: None,
            dedicated_fans: 0,
            under_creation: false,
        };

        if let Ok(team_by_id) = teams::select_by_id(&state, team.id).await {
            team = team_by_id;
        }

        teams.push(team)
    }

    let first_team = teams
        .get(0)
        .ok_or(AppError::from(
            blood_bowl_rs::errors::Error::GameShouldHaveTwoTeams,
        ))?
        .clone();

    let second_team = teams
        .get(1)
        .ok_or(AppError::from(
            blood_bowl_rs::errors::Error::GameShouldHaveTwoTeams,
        ))?
        .clone();

    let mut created_by = None;

    if let Some(coach_id) = game_row.created_by {
        created_by = coaches::select_by_id(state, coach_id).await?;
    }

    let game = Game {
        id: Some(game_row.id),
        version: game_row.version,
        created_by,
        played_at: game_row.played_at,
        closed_at: game_row.closed_at,
        first_team,
        second_team,
        first_team_playing_players: vec![],
        second_team_playing_players: vec![],
        events: vec![],
    };

    Ok(game)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerIdWithNumber {
    player_id: i32,
    number: i32,
}

pub async fn select_playing_players_in_game_for_team(
    state: &AppState,
    game_id: i32,
    team_id: i32,
) -> Result<Vec<(i32, Player)>, AppError> {
    tracing::debug!(
        "select_playing_players_in_game_for_team with game_id={} and team_id={}",
        game_id,
        team_id
    );

    let players_id_with_number: Vec<PlayerIdWithNumber> = sqlx::query_as(
        "SELECT player_id,
                    number
            FROM bb_games_teams_players
            WHERE game_id = $1
            AND team_id = $2",
    )
    .bind(game_id.clone())
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut players: Vec<(i32, Player)> = Vec::with_capacity(players_id_with_number.len());

    for player_id_with_number in players_id_with_number {
        let number = player_id_with_number.number;
        let player = players::select_by_id(state, player_id_with_number.player_id).await?;

        players.push((number, player));
    }

    Ok(players)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct TeamGameRow {
    id: i32,
    version: Version,
    played_at: NaiveDateTime,
    closed_at: Option<NaiveDateTime>,
    created_by: Option<i32>,
    opponent_team_id: i32,
    opponent_team_roster: Roster,
    opponent_team_name: String,
    opponent_id: Option<i32>,
    opponent_name: String,
}

pub async fn select_played_by_team(
    state: &AppState,
    team: &Team,
) -> Result<Vec<GameSummary>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team.id);

    let mut game_list: Vec<GameSummary> = Vec::new();

    let game_opponent_rows: Vec<TeamGameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.closed_at,
                    bb_games.created_by,
                    opponent.team_id AS opponent_team_id,
                    opponent.team_roster AS opponent_team_roster,
                    opponent.team_name AS opponent_team_name,
                    opponent.coach_id AS opponent_id,
                    opponent.coach_name AS opponent_name
            FROM bb_games
            INNER JOIN bb_games_teams AS mine
            ON mine.game_id = bb_games.id
            AND mine.team_id = $1
            INNER JOIN bb_games_teams AS opponent
            ON opponent.game_id = bb_games.id
            AND opponent.team_id <> $1
            WHERE bb_games.closed_at IS NOT NULL
            ORDER BY bb_games.closed_at",
    )
    .bind(team.id.clone())
    .fetch_all(&state.db)
    .await?;

    for game_opponent_row in game_opponent_rows {
        let mut created_by = None;

        if let Some(coach_id) = game_opponent_row.created_by {
            created_by = coaches::select_by_id(state, coach_id).await?;
        }

        game_list.push(GameSummary {
            id: game_opponent_row.id,
            version: game_opponent_row.version,
            played_at: game_opponent_row.played_at,
            closed_at: game_opponent_row.closed_at,
            first_team: TeamSummary::from(team.clone()),
            second_team: TeamSummary {
                id: game_opponent_row.opponent_team_id,
                version: game_opponent_row.version,
                roster: game_opponent_row.opponent_team_roster,
                name: game_opponent_row.opponent_team_name,
                coach: Coach {
                    id: game_opponent_row.opponent_id,
                    name: game_opponent_row.opponent_name,
                },
                external_logo_url: None,
                value: 0,
                current_value: 0,
                treasury: 0,
                last_game_played_date_time: None,
                is_playing_a_game: false,
            },
            first_team_score: 0,
            second_team_score: 0,
            first_team_casualties: 0,
            created_by,
            second_team_casualties: 0,
        })
    }

    Ok(game_list)
}

pub async fn select_playing_by_team(
    state: &AppState,
    team: &Team,
) -> Result<Option<GameSummary>, AppError> {
    tracing::debug!("select_playing_by_team for team_id={:?}", team.id);

    let game_opponent_row: Option<TeamGameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.closed_at,
                    bb_games.created_by,
                    opponent.team_id AS opponent_team_id,
                    opponent.team_roster AS opponent_team_roster,
                    opponent.team_name AS opponent_team_name,
                    opponent.coach_id AS opponent_id,
                    opponent.coach_name AS opponent_name
            FROM bb_games
            INNER JOIN bb_games_teams AS mine
            ON mine.game_id = bb_games.id
            AND mine.team_id = $1
            INNER JOIN bb_games_teams AS opponent
            ON opponent.game_id = bb_games.id
            AND opponent.team_id <> $1
            INNER JOIN bb_games_events
            ON bb_games_events.game_id = bb_games.id
            WHERE bb_games.closed_at IS NULL
            LIMIT 1",
    )
    .bind(team.id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(game_opponent_row) = game_opponent_row {
        let mut created_by = None;

        if let Some(coach_id) = game_opponent_row.created_by {
            created_by = coaches::select_by_id(state, coach_id).await?;
        }

        let game_summary = GameSummary {
            id: game_opponent_row.id,
            version: game_opponent_row.version,
            played_at: game_opponent_row.played_at,
            closed_at: game_opponent_row.closed_at,
            first_team: TeamSummary::from(team.clone()),
            second_team: TeamSummary {
                id: game_opponent_row.opponent_team_id,
                version: game_opponent_row.version,
                roster: game_opponent_row.opponent_team_roster,
                name: game_opponent_row.opponent_team_name,
                coach: Coach {
                    id: game_opponent_row.opponent_id,
                    name: game_opponent_row.opponent_name,
                },
                external_logo_url: None,
                value: 0,
                current_value: 0,
                treasury: 0,
                last_game_played_date_time: None,
                is_playing_a_game: false,
            },
            first_team_score: 0,
            second_team_score: 0,
            first_team_casualties: 0,
            created_by,
            second_team_casualties: 0,
        };

        Ok(Some(game_summary))
    } else {
        Ok(None)
    }
}
