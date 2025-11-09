use crate::errors::AppError;
use crate::AppState;

pub struct Session {}

impl Session {
    pub async fn upsert(
        state: &AppState,
        user_mail: String,
        session_id: String,
    ) -> Result<Self, AppError> {
        sqlx::query(
            "INSERT INTO sessions (
                    user_id,
                    session_id,
                    expires_at
                )
                VALUES (
                    (SELECT ID FROM USERS WHERE email = $1 LIMIT 1)
                    , $2
                    , CURRENT_TIMESTAMP + interval '4 hours'
                )
                ON CONFLICT (user_id) DO UPDATE SET
                session_id = excluded.session_id,
                expires_at = CURRENT_TIMESTAMP + interval '4 hours'",
        )
        .bind(user_mail)
        .bind(session_id)
        .execute(&state.db)
        .await?;

        Ok(Session {})
    }
}
