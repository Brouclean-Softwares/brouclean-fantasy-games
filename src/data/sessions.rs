use crate::errors::AppError;
use crate::AppState;
use chrono::NaiveDateTime;

pub struct Session {}

impl Session {
    pub async fn upsert(
        state: &AppState,
        user_mail: String,
        session_id: String,
        expires_at: NaiveDateTime,
    ) -> Result<Self, AppError> {
        sqlx::query(
            "INSERT INTO sessions (user_id, session_id, expires_at) VALUES (
        (SELECT ID FROM USERS WHERE email = $1 LIMIT 1),
         $2, $3)
        ON CONFLICT (user_id) DO UPDATE SET
        session_id = excluded.session_id,
        expires_at = excluded.expires_at",
        )
        .bind(user_mail)
        .bind(session_id)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

        Ok(Session {})
    }
}
