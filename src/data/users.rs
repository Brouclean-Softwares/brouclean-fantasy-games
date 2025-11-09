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

        if let Some(connected_user) = User::select_connected_user(&state, &cookie).await? {
            connected_user.extend_session(&state, &cookie).await?;

            Ok(connected_user)
        } else {
            Err(AppError::Unauthorized)
        }
    }
}

impl User {
    pub fn optional_user_eq_other(optional_user: &Option<User>, other: &Option<User>) -> bool {
        if let Some(other) = other {
            Self::optional_user_has_optional_id(optional_user, &other.id)
        } else {
            false
        }
    }

    pub fn optional_user_has_optional_id(
        optional_user: &Option<User>,
        optional_id: &Option<i32>,
    ) -> bool {
        if let (Some(user), Some(id)) = (optional_user, optional_id) {
            if let Some(user_id) = user.id {
                user_id.eq(id)
            } else {
                false
            }
        } else {
            false
        }
    }

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

    pub async fn select_connected_user(
        state: &AppState,
        cookie: &String,
    ) -> Result<Option<Self>, AppError> {
        let connected_user: Option<User> = sqlx::query_as(
            "SELECT users.id,
                        users.email,
                        users.name,
                        users.given_name,
                        users.family_name,
                        users.picture
                FROM sessions
                LEFT JOIN USERS
                ON sessions.user_id = users.id
                WHERE sessions.session_id = $1
                AND sessions.expires_at > CURRENT_TIMESTAMP
                LIMIT 1",
        )
        .bind(cookie.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(connected_user)
    }

    async fn extend_session(&self, state: &AppState, cookie: &String) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE sessions
                SET expires_at = CURRENT_TIMESTAMP + interval '4 hours'
                WHERE session_id = $1
                AND user_id = $2
                AND expires_at > CURRENT_TIMESTAMP",
        )
        .bind(cookie.clone())
        .bind(self.id)
        .execute(&state.db)
        .await?;

        Ok(())
    }

    pub async fn select_by_id(state: &AppState, id: Option<i32>) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_by_id with id={:?}", id);

        if let Some(user_id) = id {
            let user: Option<User> = sqlx::query_as(
                "SELECT id,
                            email,
                            name,
                            given_name,
                            family_name,
                            picture
                    FROM users
                    WHERE id = $1
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

    pub async fn select_by_mail(state: &AppState, mail: &String) -> Result<Option<Self>, AppError> {
        tracing::debug!("select_by_mail with id={}", mail);

        let user: Option<User> = sqlx::query_as(
            "SELECT id,
                        email,
                        name,
                        given_name,
                        family_name,
                        picture
                FROM users
                WHERE email = $1
                LIMIT 1",
        )
        .bind(mail.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(user)
    }

    pub async fn upsert(&self, state: &AppState) -> Result<Self, AppError> {
        let existing_user = Self::select_by_mail(state, &self.email).await?;

        if let Some(user_id) = existing_user.and_then(|user| user.id) {
            let updated_user: User = sqlx::query_as(
                "UPDATE users
                    SET name = $2,
                        given_name = $3,
                        family_name = $4,
                        picture = $5,
                        last_updated = CURRENT_TIMESTAMP
                    WHERE id = $1
                    RETURNING users.id, users.email, users.name, given_name, family_name, users.picture",
            )
                .bind(user_id.clone())
                .bind(self.name.clone())
                .bind(self.given_name.clone())
                .bind(self.family_name.clone())
                .bind(self.picture.clone())
                .fetch_one(&state.db)
                .await?;

            Ok(updated_user)
        } else {
            let inserted_user: User = sqlx::query_as(
                "INSERT INTO users (email, name, given_name, family_name, picture)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (email) DO NOTHING
                RETURNING users.id, users.email, users.name, given_name, family_name, users.picture",
            )
                .bind(self.email.clone())
                .bind(self.name.clone())
                .bind(self.given_name.clone())
                .bind(self.family_name.clone())
                .bind(self.picture.clone())
                .fetch_one(&state.db)
                .await?;

            Ok(inserted_user)
        }
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
