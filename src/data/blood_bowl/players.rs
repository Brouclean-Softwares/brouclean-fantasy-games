use crate::data::blood_bowl::teams;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerDetail {
    id: i32,
    version: Version,
    name: String,
    position: Position,
    number: i32,
    star_player_points: i32,
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Player, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let player_detail: PlayerDetail = sqlx::query_as(
        "SELECT id,
                    version,
                    name,
                    position,
                    star_player_points,
                    0 AS number
            FROM bb_players
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let player_injuries = select_player_injuries(state, id).await?;

    Ok(Player {
        id: player_detail.id,
        version: player_detail.version,
        position: player_detail.position,
        name: player_detail.name,
        star_player_points: player_detail.star_player_points,
        is_journeyman: false,
        is_star_player: false,
        miss_next_game: PlayerInjury::extract_miss_next_game(&player_injuries),
        advancements: vec![],
        injuries: PlayerInjury::extract_remaining_injuries(&player_injuries),
    })
}

pub async fn select_under_contract_for_team(
    state: &AppState,
    team_id: i32,
) -> Result<Vec<(i32, Player)>, AppError> {
    tracing::debug!("select_under_contract_for_team with team_id={}", team_id);

    let players_detail: Vec<PlayerDetail> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_players.version,
                    bb_players.name,
                    bb_players.position,
                    bb_players.star_player_points,
                    bb_teams_players.number
            FROM bb_players
            INNER JOIN bb_teams_players ON bb_players.id = bb_teams_players.player_id
            WHERE bb_teams_players.team_id = $1
            AND bb_teams_players.contract_end IS NULL
            ORDER BY bb_teams_players.number ASC",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut players: Vec<(i32, Player)> = Vec::new();

    for player_detail in players_detail {
        players.push((
            player_detail.number,
            Player {
                id: player_detail.id,
                version: player_detail.version,
                position: player_detail.position,
                name: player_detail.name,
                star_player_points: player_detail.star_player_points,
                is_journeyman: false,
                is_star_player: false,
                miss_next_game: false,
                advancements: vec![],
                injuries: vec![],
            },
        ));
    }

    Ok(players)
}

pub async fn update_name(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
    player_id: &i32,
    name: &String,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_name by user={:?} for team_id={} and player_id={} with name={}",
        connected_user,
        team_id,
        player_id,
        name
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_players
                SET name = $1,
                    last_updated = CURRENT_TIMESTAMP
                FROM bb_teams_players, bb_teams
                WHERE bb_players.id = bb_teams_players.player_id
                AND bb_teams.id = bb_teams_players.team_id
                AND bb_players.id = $2
                AND bb_teams.id = $3
                AND bb_teams.coach_id = $4",
        )
        .bind(name.clone())
        .bind(player_id.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

pub async fn update_number(
    state: &AppState,
    connected_user: &User,
    team_id: &i32,
    player_id: &i32,
    number: &i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_number by user={:?} for team_id={} and player_id={} with number={}",
        connected_user,
        team_id,
        player_id,
        number
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_teams_players
                SET number = $1
                FROM bb_teams
                WHERE bb_teams.id = bb_teams_players.team_id
                AND bb_teams_players.player_id = $2
                AND bb_teams.id = $3
                AND bb_teams.coach_id = $4",
        )
        .bind(number.clone())
        .bind(player_id.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct Id {
    id: i32,
}

pub async fn buy_position_for_team(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    position: Position,
) -> Result<(), AppError> {
    tracing::debug!(
        "buy_position_for_team by user={:?} for team_id={} with position={:?}",
        connected_user,
        team_id,
        position
    );

    if let Some(connected_user_id) = connected_user.id {
        let mut team = teams::select_by_id(state, team_id).await?;
        let (number, player) = team.buy_position(&position)?;
        let team_value = team.value()?;
        let team_current_value = team.current_value()?;

        let mut transaction = state.db.begin().await?;

        let new_player_id: Id = sqlx::query_as(
            "INSERT INTO bb_players (
                version,
                name,
                position)
            VALUES ($1, $2, $3)
            RETURNING id",
        )
        .bind(player.version.clone())
        .bind(player.name.clone())
        .bind(player.position.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO bb_teams_players (
                number,
                team_id,
                player_id)
            VALUES ($1, $2, $3)",
        )
        .bind(number.clone())
        .bind(team_id.clone())
        .bind(new_player_id.id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE bb_teams
            SET treasury = $1,
                value = $2,
                current_value = $3,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $4
            AND coach_id = $5",
        )
        .bind(team.treasury.clone())
        .bind(team_value.clone() as i32)
        .bind(team_current_value.clone() as i32)
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(())
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerInjury {
    injury: Injury,
    before_last_game: bool,
}

impl PlayerInjury {
    fn extract_miss_next_game(player_injuries: &Vec<Self>) -> bool {
        player_injuries
            .iter()
            .filter(|&player_injury| {
                match (
                    player_injury.injury.clone(),
                    player_injury.before_last_game.clone(),
                ) {
                    (Injury::SeriouslyHurt, false)
                    | (Injury::SeriousInjury, false)
                    | (Injury::HeadInjury, false)
                    | (Injury::SmashedKnee, false)
                    | (Injury::BrokenArm, false)
                    | (Injury::NeckInjury, false)
                    | (Injury::DislocatedShoulder, false)
                    | (Injury::Dead, _) => true,

                    (Injury::Stunned, _)
                    | (Injury::KO, _)
                    | (Injury::BadlyHurt, _)
                    | (Injury::SeriouslyHurt, true)
                    | (Injury::SeriousInjury, true)
                    | (Injury::HeadInjury, true)
                    | (Injury::SmashedKnee, true)
                    | (Injury::BrokenArm, true)
                    | (Injury::NeckInjury, true)
                    | (Injury::DislocatedShoulder, true) => false,
                }
            })
            .count()
            > 0
    }

    fn extract_remaining_injuries(player_injuries: &Vec<Self>) -> Vec<Injury> {
        let mut remaining_injuries = vec![];

        for player_injury in player_injuries.iter() {
            match (
                player_injury.injury.clone(),
                player_injury.before_last_game.clone(),
            ) {
                (Injury::SeriouslyHurt, false)
                | (Injury::SeriousInjury, false)
                | (Injury::HeadInjury, _)
                | (Injury::SmashedKnee, _)
                | (Injury::BrokenArm, _)
                | (Injury::NeckInjury, _)
                | (Injury::DislocatedShoulder, _)
                | (Injury::Dead, _) => remaining_injuries.push(player_injury.injury.clone()),

                (Injury::Stunned, _)
                | (Injury::KO, _)
                | (Injury::BadlyHurt, _)
                | (Injury::SeriouslyHurt, true)
                | (Injury::SeriousInjury, true) => {}
            };
        }

        remaining_injuries
    }
}

async fn select_player_injuries(
    state: &AppState,
    player_id: i32,
) -> Result<Vec<PlayerInjury>, AppError> {
    tracing::debug!("select_player_injuries with id={}", player_id);

    let injuries: Vec<PlayerInjury> = sqlx::query_as(
        "SELECT bb_players_injuries.injury,
                    bb_players_injuries.created_at < MAX(bb_games.started_at) as before_last_game
            FROM bb_players_injuries
            INNER JOIN bb_games_teams_players
            ON bb_games_teams_players.player_id = bb_players_injuries.player_id
            INNER JOIN bb_games
            ON bb_games.id = bb_games_teams_players.game_id
            WHERE bb_players_injuries.player_id = $1
            AND bb_players_injuries.recovered_at IS NULL
            GROUP BY bb_players_injuries.injury,
            bb_players_injuries.created_at",
    )
    .bind(player_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(injuries)
}
