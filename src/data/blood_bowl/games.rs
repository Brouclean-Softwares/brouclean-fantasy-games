use crate::data::blood_bowl::{players, teams};
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::games::Game;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::Team;
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
        team_a.id.unwrap_or_default(),
        team_b.id.unwrap_or_default(),
    );

    let game = Game::create(
        None,
        team_a.version,
        played_at,
        team_a.clone(),
        team_b.clone(),
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
            WHERE bb_teams.id in ($2, $3)",
    )
    .bind(new_game_id.id.clone())
    .bind(team_a.id.unwrap().clone())
    .bind(team_b.id.unwrap().clone())
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
    closed_at: Option<NaiveDateTime>,
    team_id: i32,
    team_roster: Roster,
    team_name: String,
    coach_id: Option<i32>,
    coach_name: String,
    opponent_team_id: i32,
    opponent_roster: Roster,
    opponent_team_name: String,
    opponent_id: Option<i32>,
    opponent_name: String,
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Game, AppError> {
    tracing::debug!("select_by_id for id={}", id);

    let game_row: GameRow = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.closed_at,
                    first_team.team_id,
                    first_team.team_roster,
                    first_team.team_name,
                    first_team.coach_id,
                    first_team.coach_name,
                    opponent.team_id AS opponent_team_id,
                    opponent.team_roster AS opponent_roster,
                    opponent.team_name AS opponent_team_name,
                    opponent.coach_id AS opponent_id,
                    opponent.coach_name AS opponent_name
            FROM bb_games
            INNER JOIN bb_games_teams AS first_team
            ON first_team.game_id = bb_games.id
            INNER JOIN bb_games_teams AS opponent
            ON opponent.game_id = bb_games.id
            WHERE bb_games.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let first_team = teams::select_by_id(&state, game_row.team_id)
        .await
        .unwrap_or(Team {
            id: Some(game_row.team_id),
            version: game_row.version,
            roster: game_row.team_roster,
            name: game_row.team_name,
            coach_id: game_row.coach_id,
            coach_name: game_row.coach_name,
            treasury: 0,
            external_logo_url: None,
            staff: Default::default(),
            players: vec![],
            games_played: vec![],
            dedicated_fans: 0,
            under_creation: false,
        });

    let opponent_team = teams::select_by_id(&state, game_row.opponent_team_id)
        .await
        .unwrap_or(Team {
            id: Some(game_row.opponent_team_id),
            version: game_row.version,
            roster: game_row.opponent_roster,
            name: game_row.opponent_team_name,
            coach_id: game_row.opponent_id,
            coach_name: game_row.opponent_name,
            treasury: 0,
            external_logo_url: None,
            staff: Default::default(),
            players: vec![],
            games_played: vec![],
            dedicated_fans: 0,
            under_creation: false,
        });

    let game = Game {
        id: Some(game_row.id),
        version: game_row.version,
        played_at: game_row.played_at,
        closed_at: game_row.closed_at,
        teams: vec![first_team, opponent_team],
        playing_players: Default::default(),
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

pub async fn select_played_by_team(state: &AppState, team: &Team) -> Result<Vec<Game>, AppError> {
    tracing::debug!("select_played_by_team for team_id={:?}", team.id);

    let mut game_list: Vec<Game> = Vec::new();

    if team.id.is_none() {
        return Ok(game_list);
    }

    let game_rows: Vec<GameRow> = sqlx::query_as(
        "SELECT bb_games.id,
                    bb_games.version,
                    bb_games.played_at,
                    bb_games.closed_at,
                    mine.team_id,
                    mine.team_roster,
                    mine.team_name,
                    mine.coach_id,
                    mine.coach_name,
                    opponent.team_id AS opponent_team_id,
                    opponent.team_roster AS opponent_roster,
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
    .bind(team.id.unwrap().clone())
    .fetch_all(&state.db)
    .await?;

    for game_row in game_rows {
        game_list.push(Game {
            id: Some(game_row.id),
            version: game_row.version,
            played_at: game_row.played_at,
            closed_at: game_row.closed_at,
            teams: vec![
                team.clone(),
                Team {
                    id: Some(game_row.opponent_team_id),
                    version: game_row.version,
                    roster: game_row.opponent_roster,
                    name: game_row.opponent_team_name,
                    coach_id: game_row.opponent_id,
                    coach_name: game_row.opponent_name,
                    treasury: 0,
                    external_logo_url: None,
                    staff: Default::default(),
                    players: vec![],
                    games_played: vec![],
                    dedicated_fans: 0,
                    under_creation: false,
                },
            ],
            playing_players: Default::default(),
            events: vec![],
        })
    }

    Ok(game_list)
}
