use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;

pub async fn select_by_id(state: &AppState, id: i32) -> Result<User, AppError> {
    tracing::debug!("select_by_id with id={:?}", id);

    let coach: User = sqlx::query_as(
        "SELECT users.id,
                    users.email,
                    users.name,
                    users.given_name,
                    users.family_name,
                    users.picture
            FROM users
            INNER JOIN bb_teams
            ON users.id = bb_teams.coach_id
            WHERE users.id = $1
            LIMIT 1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(coach)
}
