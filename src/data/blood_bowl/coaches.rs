use crate::AppState;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::coaches::Coach;

pub async fn select_by_id(state: &AppState, id: Option<i32>) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_by_id with id={:?}", id);

    let coach = User::select_by_id(state, id).await?;
    Ok(coach.and_then(|user| Some(user.into())))
}

pub async fn select_from_team(state: &AppState, team_id: i32) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_from_team with team_id={}", team_id);

    let coach: Option<User> = sqlx::query_as(
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

    Ok(coach.and_then(|user| Some(user.into())))
}
