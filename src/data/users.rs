use crate::auth::SESSION_ID;
use crate::errors::AppError;
use crate::AppState;
use axum::extract::FromRequestParts;
use axum_extra::extract::PrivateCookieJar;
use blood_bowl_rs::coaches::Coach;
use http::request::Parts;
use serde::Deserialize;

#[derive(Deserialize, Debug, sqlx::FromRow, Clone)]
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

    pub fn is_coach(&self, coach: &Coach) -> bool {
        match (self.id, coach.id) {
            (Some(id), Some(coach_id)) => id.eq(&coach_id),
            _ => false,
        }
    }

    pub fn is_option_coach(&self, coach: &Option<Coach>) -> bool {
        if let Some(coach) = coach {
            self.is_coach(coach)
        } else {
            false
        }
    }

    pub async fn select_connected_user(state: &AppState, cookie: String) -> Result<Self, AppError> {
        let connected_user: User = sqlx::query_as(
            "SELECT users.id,
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

    pub async fn select_by_id(state: &AppState, id: Option<i32>) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_by_id with id={:?}", id);

        if let Some(user_id) = id {
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
            WHERE users.id = $1
            LIMIT 1",
            )
            .bind(user_id.clone())
            .fetch_optional(&state.db)
            .await?;

            Ok(user)
        } else {
            Ok(None)
        }
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

impl Into<Coach> for User {
    fn into(self) -> Coach {
        Coach {
            id: self.id,
            name: self.name,
        }
    }
}

impl PartialEq<Coach> for User {
    fn eq(&self, other: &Coach) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialEq<Option<Coach>> for User {
    fn eq(&self, other: &Option<Coach>) -> bool {
        if let Some(other_coach) = other.clone() {
            self.eq(&other_coach)
        } else {
            false
        }
    }
}

impl PartialEq<User> for User {
    fn eq(&self, other: &User) -> bool {
        if let (Some(id), Some(other_id)) = (self.id.clone(), other.id.clone()) {
            id.eq(&other_id)
        } else {
            false
        }
    }
}

impl PartialEq<Option<User>> for User {
    fn eq(&self, other: &Option<User>) -> bool {
        if let Some(other_user) = other.clone() {
            self.eq(&other_user)
        } else {
            false
        }
    }
}
