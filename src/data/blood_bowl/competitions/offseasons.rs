use crate::AppState;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::players;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::dices::Dice;
use blood_bowl_rs::players::Player;

pub const OFFSEASON_COMPETITION_ROUND_THRESHOLD: usize = 10;
pub const INITIAL_FUNDS_FOR_REDRAFT: i32 = 1000000;
pub const FUNDS_FOR_GAME_PLAYED: i32 = 20000;
pub const FUNDS_FOR_GAME_WON: i32 = 20000;
pub const FUNDS_FOR_GAME_DREW: i32 = 10000;

pub async fn start_competition_offseason(
    state: &AppState,
    connected_user: &User,
    competition: &Competition,
) -> Result<(), AppError> {
    tracing::debug!(
        "start_competition_offseason by user={:?} for competition_id={}",
        connected_user,
        competition.id
    );

    if let Some(director) = &competition.director {
        if connected_user.eq(director) && competition.closed {
            let mut transaction = state.db.begin().await?;

            sqlx::query(
                "DELETE
                    FROM bb_competitions_teams_offseasons
                    WHERE competition_id = $1",
            )
            .bind(competition.id.clone())
            .execute(&mut *transaction)
            .await?;

            for team in competition.select_playing_teams(state).await? {
                // Rest & relaxation
                let team_players = players::select_under_contract_for_team(state, team.id).await?;

                for (_, player) in team_players {
                    let player_injuries = players::select_player_injuries(state, player.id).await?;

                    for player_injury in player_injuries {
                        if player_injury.injury.is_niggling_injury() && Dice::D6.roll() >= 4 {
                            players::update_player_who_recovered_from_injury(
                                &mut *transaction,
                                player.id,
                                &player_injury,
                            )
                            .await?;
                        }
                    }

                    let player_hatred = players::select_player_hatred(state, player.id).await?;

                    for hatred in player_hatred {
                        if Dice::D6.roll() >= 4 {
                            players::update_player_who_recovered_from_hatred(
                                &mut *transaction,
                                player.id,
                                &hatred,
                            )
                            .await?;
                        }
                    }
                }

                // Raised funds
                let team_results = competition.select_team_results(state, team.id).await?;

                let raised_funds = INITIAL_FUNDS_FOR_REDRAFT
                    + (FUNDS_FOR_GAME_PLAYED * team_results.total_played() as i32)
                    + (FUNDS_FOR_GAME_WON * team_results.victories as i32)
                    + (FUNDS_FOR_GAME_DREW * team_results.draws as i32);

                sqlx::query(
                    "INSERT INTO bb_competitions_teams_offseasons (
                            competition_id,
                            team_id,
                            raised_funds
                        )
                        VALUES ($1, $2, $3)",
                )
                .bind(competition.id.clone())
                .bind(team.id.clone())
                .bind(raised_funds.clone())
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
        }
    }

    Ok(())
}

pub async fn select_raised_fund_for_team(state: &AppState, team_id: &i32) -> Result<i32, AppError> {
    tracing::debug!("select_raised_fund_for_team for team_id={}", team_id);

    let raised_funds: Option<i32> = sqlx::query_scalar(
        "SELECT raised_funds
            FROM bb_competitions_teams_offseasons
            WHERE team_id = $1
            AND closed_at IS NULL
            ORDER BY created_at DESC
            LIMIT 1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(raised_funds.unwrap_or(0))
}

pub async fn select_if_player_in_offseason(
    state: &AppState,
    player: &Player,
) -> Result<bool, AppError> {
    tracing::debug!("select_if_player_in_offseason for player_id={}", player.id);

    let offseasons_still_in_progress: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
            FROM bb_competitions_teams_offseasons
            INNER JOIN bb_teams_players
            ON bb_competitions_teams_offseasons.team_id = bb_teams_players.team_id
            AND bb_teams_players.contract_start < bb_competitions_teams_offseasons.created_at
            AND (
                bb_teams_players.contract_end IS NULL
                OR bb_teams_players.contract_end > bb_competitions_teams_offseasons.created_at
            )
            WHERE bb_teams_players.player_id = $1
            AND bb_competitions_teams_offseasons.closed_at IS NULL",
    )
    .bind(player.id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(offseasons_still_in_progress > 0)
}

pub async fn select_seasons_played_by_player(
    state: &AppState,
    player: &Player,
) -> Result<usize, AppError> {
    tracing::debug!(
        "select_seasons_played_by_player for player_id={}",
        player.id
    );

    let season_number: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
            FROM bb_competitions_teams_offseasons
            INNER JOIN bb_teams_players
            ON bb_competitions_teams_offseasons.team_id = bb_teams_players.team_id
            AND bb_teams_players.contract_start < bb_competitions_teams_offseasons.created_at
            AND (
                bb_teams_players.contract_end IS NULL
                OR bb_teams_players.contract_end > bb_competitions_teams_offseasons.created_at
            )
            WHERE bb_teams_players.player_id = $1",
    )
    .bind(player.id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(season_number as usize)
}

pub async fn select_seasons_played_without_experience_by_player(
    state: &AppState,
    player: &Player,
) -> Result<usize, AppError> {
    tracing::debug!(
        "select_seasons_played_with_experience_by_player for player_id={}",
        player.id
    );

    let mut seasons_played_without_experience: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
            FROM bb_redrafting_players
            INNER JOIN bb_competitions_teams_offseasons
            ON bb_competitions_teams_offseasons.competition_id = bb_redrafting_players.competition_id
            WHERE bb_redrafting_players.player_id = $1
            AND bb_redrafting_players.has_experience = FALSE
            AND bb_competitions_teams_offseasons.closed_at IS NOT NULL",
    )
    .bind(player.id.clone())
    .fetch_one(&state.db)
    .await?;

    let player_in_offseason = select_if_player_in_offseason(state, player).await?;

    if !player.has_experience() && player_in_offseason {
        seasons_played_without_experience += 1;
    }

    Ok(seasons_played_without_experience as usize)
}
