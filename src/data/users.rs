use crate::auth::SESSION_ID;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::FromRequestParts;
use axum_extra::extract::PrivateCookieJar;
use http::request::Parts;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub email: String,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();

        let cookie_jar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(&mut parts.clone(), &state).await?;

        let Some(cookie) = cookie_jar
            .get(SESSION_ID)
            .map(|cookie| cookie.value().to_owned())
        else {
            return Err(AppError::Unauthorized);
        };

        let connected_user = User::select_connected_user(&state, cookie).await?;

        Ok(connected_user)
    }
}

impl User {
    pub fn is_admin(&self, state: &AppState) -> bool {
        state.admin_email.eq(&self.email)
    }

    pub async fn select_connected_user(state: &AppState, cookie: String) -> Result<Self, AppError> {
        let connected_user: User = sqlx::query_as(
            "SELECT
                    users.id,
                    users.email,
                    users.name,
                    users.given_name,
                    users.family_name,
                    users.picture
                FROM sessions
                LEFT JOIN USERS ON sessions.user_id = users.id
                WHERE sessions.session_id = $1
                LIMIT 1",
        )
        .bind(cookie)
        .fetch_one(&state.db)
        .await?;

        Ok(connected_user)
    }

    pub async fn upsert(&self, state: &AppState) -> Result<Self, AppError> {
        let upserted_user: User = sqlx::query_as(
            "INSERT INTO users (email, name, given_name, family_name, picture)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (email) DO UPDATE SET
                    name = excluded.name,
                    given_name = excluded.given_name,
                    family_name = excluded.family_name,
                    picture = excluded.picture,
                    last_updated = CURRENT_TIMESTAMP
                RETURNING users.id, users.email, users.name, given_name, family_name, users.picture",
        )
        .bind(self.email.clone())
        .bind(self.name.clone())
        .bind(self.given_name.clone())
        .bind(self.family_name.clone())
        .bind(self.picture.clone())
        .fetch_one(&state.db)
        .await?;

        Ok(upserted_user)
    }
}
