use crate::data::blood_bowl::games::select_playing_team_player_for_game;
use crate::data::blood_bowl::{coaches, games, teams};
use crate::data::users::User;
use crate::data::{Id, IsTrue, Total};
use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::advancements::{Advancement, AdvancementChoice};
use blood_bowl_rs::injuries::Injury;
use blood_bowl_rs::players::{Player, PlayerType};
use blood_bowl_rs::positions::{Keyword, Position};
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerDetail {
    id: i32,
    version: Version,
    name: String,
    position: Position,
    roster: Roster,
    number: i32,
}

impl PlayerDetail {
    async fn into_player(self, state: &AppState) -> Result<Player, AppError> {
        let player_injuries = select_player_injuries(state, self.id).await?;
        let star_player_points = select_remaining_star_player_points(state, self.id).await?;
        let advancements = select_advancements(state, self.id).await?;
        let hatred = select_player_hatred(state, self.id).await?;

        Ok(Player {
            id: self.id,
            version: self.version,
            position: self.position,
            roster: self.roster,
            name: self.name,
            star_player_points,
            player_type: PlayerType::FromRoster,
            miss_next_game: PlayerInjury::extract_miss_next_game(&player_injuries),
            advancements,
            injuries: PlayerInjury::extract_current_injuries(&player_injuries),
            hatred,
        })
    }
}

pub async fn select_by_id_for_team(
    state: &AppState,
    player_id: i32,
    team_id: i32,
) -> Result<Option<(i32, Player)>, AppError> {
    tracing::debug!(
        "select_by_id with player_id={} for team_id={}",
        player_id,
        team_id
    );

    let player_detail: Option<PlayerDetail> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_players.version,
                    bb_players.name,
                    bb_players.position,
                    bb_teams.roster,
                    bb_teams_players.number
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_players.id = bb_teams_players.player_id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            WHERE bb_teams_players.team_id = $2
            AND bb_teams_players.player_id = $1
            LIMIT 1",
    )
    .bind(player_id.clone())
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(player_detail) = player_detail {
        Ok(Some((
            player_detail.number,
            player_detail.into_player(state).await?,
        )))
    } else {
        Ok(None)
    }
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
                    bb_teams.roster,
                    bb_teams_players.number
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_players.id = bb_teams_players.player_id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
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
            player_detail.into_player(state).await?,
        ));
    }

    Ok(players)
}

pub async fn is_under_contract_for_team(
    state: &AppState,
    player_id: i32,
    team_id: i32,
) -> Result<bool, AppError> {
    tracing::debug!(
        "is_under_contract_for_team with player_id={} for team_id={}",
        player_id,
        team_id
    );

    let result: Option<IsTrue> = sqlx::query_as(
        "SELECT contract_end IS NULL as is_true
            FROM bb_teams_players
            WHERE team_id = $1
            AND player_id = $2
            LIMIT 1",
    )
    .bind(team_id.clone())
    .bind(player_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(result) = result {
        Ok(result.is_true)
    } else {
        Ok(false)
    }
}

pub async fn select_former_for_team(
    state: &AppState,
    team_id: i32,
) -> Result<Vec<(i32, Player)>, AppError> {
    tracing::debug!("select_former_for_team with team_id={}", team_id);

    let players_detail: Vec<PlayerDetail> = sqlx::query_as(
        "SELECT bb_players.id,
                    bb_players.version,
                    bb_players.name,
                    bb_players.position,
                    bb_teams.roster,
                    bb_teams_players.number
            FROM bb_players
            INNER JOIN bb_teams_players
            ON bb_players.id = bb_teams_players.player_id
            INNER JOIN bb_teams
            ON bb_teams.id = bb_teams_players.team_id
            WHERE bb_teams_players.team_id = $1
            AND bb_teams_players.contract_end IS NOT NULL
            ORDER BY bb_teams_players.contract_end DESC",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut players: Vec<(i32, Player)> = Vec::new();

    for player_detail in players_detail {
        players.push((
            player_detail.number,
            player_detail.into_player(state).await?,
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
        number,
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
        let mut team = teams::select_by_id_with_staff_and_players(state, team_id).await?;
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

pub async fn buy_journeyman_in_game_for_team(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    player_id_in_game: i32,
    game_id: i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "buy_journeyman_in_game_for_team by user={:?} for team_id={}, game_id={} and player_id={}",
        connected_user,
        team_id,
        game_id,
        player_id_in_game,
    );

    if let Some(connected_user_id) = connected_user.id {
        let mut team = teams::select_by_id_with_staff_and_players(state, team_id).await?;
        let game = games::select_by_id(state, game_id).await?;
        let journey_man =
            select_playing_team_player_for_game(state, &game, team_id, player_id_in_game).await?;

        if let Some(journey_man) = journey_man {
            if let Some((number, player)) = team.buy_journeyman(journey_man)? {
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
                    "UPDATE bb_games_teams_players
                    SET player_id = $1
                    WHERE player_id_in_game = $2
                    AND game_id = $3
                    AND team_id = $4",
                )
                .bind(new_player_id.id.clone())
                .bind(player_id_in_game.clone())
                .bind(game_id.clone())
                .bind(team_id.clone())
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
        }
    }

    Ok(())
}

pub async fn buyout_for_team(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    player_id: i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "buyout_player by user={:?} for team_id={} and player_id={}",
        connected_user,
        team_id,
        player_id
    );

    sqlx::query(
        "UPDATE bb_teams_players
        SET contract_end = CURRENT_TIMESTAMP
        FROM bb_teams
        WHERE bb_teams_players.player_id = $1
        AND bb_teams_players.team_id = $2
        AND bb_teams.id = bb_teams_players.team_id
        AND bb_teams.coach_id = $3",
    )
    .bind(player_id.clone())
    .bind(team_id.clone())
    .bind(connected_user.id.clone())
    .execute(&state.db)
    .await?;

    teams::update_values(state, connected_user, team_id).await?;

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
                    | (Injury::DislocatedHip, false)
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
                    | (Injury::DislocatedHip, true)
                    | (Injury::DislocatedShoulder, true) => false,
                }
            })
            .count()
            > 0
    }

    fn extract_current_injuries(player_injuries: &Vec<Self>) -> Vec<Injury> {
        let mut injuries = vec![];

        for player_injury in player_injuries.iter() {
            match (
                player_injury.injury.clone(),
                player_injury.before_last_game.clone(),
            ) {
                (Injury::SeriouslyHurt, false)
                | (Injury::SeriousInjury, _)
                | (Injury::HeadInjury, _)
                | (Injury::SmashedKnee, _)
                | (Injury::BrokenArm, _)
                | (Injury::NeckInjury, _)
                | (Injury::DislocatedShoulder, _)
                | (Injury::DislocatedHip, _)
                | (Injury::Dead, _) => injuries.push(player_injury.injury.clone()),

                (Injury::Stunned, _)
                | (Injury::KO, _)
                | (Injury::BadlyHurt, _)
                | (Injury::SeriouslyHurt, true) => {}
            };
        }

        injuries
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
            INNER JOIN bb_teams_players
            ON bb_teams_players.player_id = bb_players_injuries.player_id
            INNER JOIN bb_games
            ON (bb_teams_players.team_id = bb_games.first_team_id OR bb_teams_players.team_id = bb_games.second_team_id)
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

async fn select_remaining_star_player_points(
    state: &AppState,
    player_id: i32,
) -> Result<i32, AppError> {
    tracing::debug!("select_remaining_star_player_points with id={}", player_id);

    let points_won: Total = sqlx::query_as(
        "SELECT SUM(star_player_points) as total
            FROM bb_games_teams_players
            WHERE player_id = $1",
    )
    .bind(player_id.clone())
    .fetch_one(&state.db)
    .await?;

    let points_spent: Total = sqlx::query_as(
        "SELECT SUM(star_player_points) as total
            FROM bb_players_advancements
            WHERE player_id = $1",
    )
    .bind(player_id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok((points_won.total.unwrap_or(0) - points_spent.total.unwrap_or(0)) as i32)
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct PlayerAdvancement {
    advancement: Option<String>,
    choice: Option<String>,
    star_player_points: Option<i32>,
    options_to_choose: Option<String>,
}

impl PlayerAdvancement {
    pub fn advancement(&self) -> Result<Option<Advancement>, AppError> {
        if let Some(advancement) = self.advancement.clone() {
            let advancement: Advancement = serde_json::from_str(&advancement)?;

            Ok(Some(advancement))
        } else {
            Ok(None)
        }
    }

    pub fn choice(&self) -> Result<Option<AdvancementChoice>, AppError> {
        if let Some(choice) = self.choice.clone() {
            let choice: AdvancementChoice = serde_json::from_str(&choice)?;

            Ok(Some(choice))
        } else {
            Ok(None)
        }
    }

    pub fn star_player_points_cost(&self) -> Option<i32> {
        self.star_player_points
    }

    pub fn options_to_choose(&self) -> Result<Option<Vec<Advancement>>, AppError> {
        if let Some(options_to_choose) = self.options_to_choose.clone() {
            let options_to_choose: Vec<Advancement> = serde_json::from_str(&options_to_choose)?;

            Ok(Some(options_to_choose))
        } else {
            Ok(None)
        }
    }
}

async fn select_advancements(
    state: &AppState,
    player_id: i32,
) -> Result<Vec<Advancement>, AppError> {
    tracing::debug!("select_advancements with id={}", player_id);

    let player_advancements: Vec<PlayerAdvancement> = sqlx::query_as(
        "SELECT advancement,
                    choice,
                    star_player_points,
                    options_to_choose
            FROM bb_players_advancements
            WHERE player_id = $1
            AND advancement IS NOT NULL
            ORDER BY added_at",
    )
    .bind(player_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut advancements = vec![];

    for player_advancement in player_advancements {
        if let Some(advancement) = player_advancement.advancement()? {
            advancements.push(advancement);
        }
    }

    Ok(advancements)
}

pub async fn select_advancements_with_choices(
    state: &AppState,
    player_id: i32,
) -> Result<Vec<PlayerAdvancement>, AppError> {
    tracing::debug!("select_advancements with id={}", player_id);

    let advancements: Vec<PlayerAdvancement> = sqlx::query_as(
        "SELECT advancement,
                    choice,
                    star_player_points,
                    options_to_choose
            FROM bb_players_advancements
            WHERE player_id = $1
            ORDER BY added_at",
    )
    .bind(player_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(advancements)
}

pub async fn add_advancement_choice(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    player_id: i32,
    advancement_choice: AdvancementChoice,
) -> Result<(), AppError> {
    tracing::debug!(
        "add_advancement_choice by user={:?} for team_id={} and player_id={} with advancement_choice={}",
        connected_user,
        team_id,
        player_id,
        advancement_choice.type_name(),
    );

    if let Some((_, player)) = select_by_id_for_team(state, player_id, team_id).await? {
        if let Some(team_coach) = coaches::select_from_team(state, team_id).await? {
            let choice_cost = advancement_choice.star_player_points_cost_for_player(&player) as i32;

            if team_coach.id.eq(&connected_user.id) && player.star_player_points >= choice_cost {
                sqlx::query(
                    "INSERT INTO bb_players_advancements (
                    player_id,
                    choice,
                    star_player_points,
                    options_to_choose
                )
                VALUES ($1, $2, $3, $4)",
                )
                .bind(player_id.clone())
                .bind(serde_json::to_string(&advancement_choice)?)
                .bind(choice_cost.clone())
                .bind(serde_json::to_string(
                    &advancement_choice.roll_advancements_to_choose_for_player(&player),
                )?)
                .execute(&state.db)
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn add_advancement(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    player_id: i32,
    advancement_to_add: Advancement,
) -> Result<(), AppError> {
    tracing::debug!(
        "add_advancement by user={:?} for team_id={} and player_id={} with advancement_to_add={}",
        connected_user,
        team_id,
        player_id,
        advancement_to_add.type_name(),
    );

    sqlx::query(
        "UPDATE bb_players_advancements
        SET advancement = $4,
            options_to_choose = NULL,
            added_at = CURRENT_TIMESTAMP
        FROM bb_teams_players, bb_teams
        WHERE bb_players_advancements.player_id = bb_teams_players.player_id
        AND bb_teams.id = bb_teams_players.team_id
        AND bb_players_advancements.player_id = $1
        AND bb_teams.id = $2
        AND bb_teams.coach_id = $3
        AND bb_players_advancements.advancement IS NULL",
    )
    .bind(player_id.clone())
    .bind(team_id.clone())
    .bind(connected_user.id.clone())
    .bind(serde_json::to_string(&advancement_to_add)?)
    .execute(&state.db)
    .await?;

    teams::update_values(state, connected_user, team_id).await?;

    Ok(())
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct PlayerHatred {
    keyword: Keyword,
}

async fn select_player_hatred(state: &AppState, player_id: i32) -> Result<Vec<Keyword>, AppError> {
    tracing::debug!("select_player_hatred with id={}", player_id);

    let hatred: Vec<PlayerHatred> = sqlx::query_as(
        "SELECT DISTINCT keyword
            FROM bb_players_hatred
            WHERE player_id = $1
            AND recovered_at IS NULL",
    )
    .bind(player_id.clone())
    .fetch_all(&state.db)
    .await?;

    let keywords = hatred
        .iter()
        .map(|player_hatred| player_hatred.keyword)
        .collect();

    Ok(keywords)
}
