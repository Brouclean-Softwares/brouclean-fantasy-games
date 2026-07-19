use crate::AppState;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::games;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::elo;
use blood_bowl_rs::games::Game;
use serde::Deserialize;

pub async fn select_by_id(state: &AppState, id: Option<i32>) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_by_id with id={:?}", id);

    let user = User::select_by_id(state, id).await?;

    if let Some(user) = user {
        Ok(Some(user.try_into_coach(state).await?))
    } else {
        Ok(None)
    }
}

pub async fn select_from_team(state: &AppState, team_id: i32) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_from_team with team_id={}", team_id);

    let user: Option<User> = sqlx::query_as(
        "SELECT users.id,
                users.email,
                users.name,
                users.given_name,
                users.family_name,
                users.picture
        FROM users
        INNER JOIN bb_teams
        ON users.id = bb_teams.coach_id
        WHERE bb_teams.id = $1
        LIMIT 1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    if let Some(user) = user {
        Ok(Some(user.try_into_coach(state).await?))
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Debug, sqlx::FromRow, Clone)]
pub struct EloRanking {
    pub id: i32,
    pub name: String,
    pub elo: f64,
}

impl EloRanking {
    pub fn rounded_elo(&self) -> i32 {
        self.elo.round() as i32
    }
}

pub async fn select_elo_ranking(state: &AppState) -> Result<Vec<EloRanking>, AppError> {
    tracing::debug!("select_elo_ranking");

    let ranking: Vec<EloRanking> = sqlx::query_as(
        "WITH ranked AS (
                SELECT
                    u.id,
                    u.name,
                    bce.elo,
                    bg.closed_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY bce.coach_id
                        ORDER BY bg.closed_at DESC
                    ) AS rn
                FROM bb_coaches_elo bce
                INNER JOIN users u
                ON u.id = bce.coach_id
                INNER JOIN bb_games bg
                ON bg.id = bce.game_id
            )
            SELECT id, name, elo
            FROM ranked
            WHERE rn = 1
            ORDER BY elo DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(ranking)
}

pub async fn select_elo_for_user(
    state: &AppState,
    user_id: &Option<i32>,
) -> Result<Option<f64>, AppError> {
    tracing::debug!("select_elo_for_user for user_id={:?}", user_id);

    if let Some(user_id) = user_id {
        let elo: Option<f64> = sqlx::query_scalar(
            "SELECT bb_coaches_elo.elo
            FROM bb_coaches_elo
            INNER JOIN bb_games
            ON bb_games.id = bb_coaches_elo.game_id
            WHERE bb_coaches_elo.coach_id = $1
            ORDER BY bb_games.closed_at DESC
            LIMIT 1",
        )
        .bind(user_id.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(elo)
    } else {
        Ok(None)
    }
}

pub async fn insert_elo_ranking_after_game(state: &AppState, game: &Game) -> Result<(), AppError> {
    tracing::debug!("insert_elo_ranking_after_game");

    if game.game_finished() {
        let competition_team_number =
            if let Some(competition) = Competition::select_for_game_id(state, game.id).await? {
                Some(competition.select_playing_teams(state).await?.len())
            } else {
                None
            };

        let (first_coach_new_elo, second_coach_new_elo) =
            elo::new_naf_elo_from_game(game, competition_team_number, competition_team_number);

        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "DELETE FROM bb_coaches_elo
                WHERE game_id = $1",
        )
        .bind(game.id.clone())
        .execute(&mut *transaction)
        .await?;

        if let Some(coach_id) = game.first_team.coach.id {
            sqlx::query(
                "INSERT INTO bb_coaches_elo (coach_id, game_id, elo)
            VALUES ($1, $2, $3)",
            )
            .bind(coach_id.clone())
            .bind(game.id.clone())
            .bind(first_coach_new_elo.clone())
            .execute(&mut *transaction)
            .await?;
        }

        if let Some(coach_id) = game.second_team.coach.id {
            sqlx::query(
                "INSERT INTO bb_coaches_elo (coach_id, game_id, elo)
            VALUES ($1, $2, $3)",
            )
            .bind(coach_id.clone())
            .bind(game.id.clone())
            .bind(second_coach_new_elo.clone())
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn regenerate_elo_ranking(state: &AppState) -> Result<(), AppError> {
    tracing::debug!("regenerate_elo_ranking");

    sqlx::query("DELETE FROM bb_coaches_elo")
        .execute(&state.db)
        .await?;

    let games = games::select_all_played(state).await?;

    for game_summary in games {
        let game = games::select_by_id(state, game_summary.id).await?;

        insert_elo_ranking_after_game(state, &game).await?;
    }

    Ok(())
}
