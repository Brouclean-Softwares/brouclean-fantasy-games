use crate::AppState;
use crate::auth::SESSION_ID;
use crate::data::blood_bowl::coaches;
use crate::errors::AppError;
use axum::extract::{FromRef, FromRequestParts};
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

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let MayBeUser(profile) = MayBeUser::from_request_parts(parts, state).await?;

        profile.ok_or(AppError::Unauthorized)
    }
}

pub struct MayBeUser(pub Option<User>);

impl<S> FromRequestParts<S> for MayBeUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let cookie_jar: PrivateCookieJar<AppState> =
            PrivateCookieJar::from_request_parts(parts, &state).await?;

        let cookie = cookie_jar.get(SESSION_ID).map(|c| c.value().to_owned());

        if let Some(cookie) = cookie {
            let user = User::select_connected_user(&state, &cookie).await?;

            if let Some(user) = &user {
                user.extend_session(&state, &cookie).await?;
            }

            Ok(MayBeUser(user))
        } else {
            Ok(MayBeUser(None))
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

    pub fn has_optional_id(&self, optional_id: &Option<i32>) -> bool {
        Self::optional_user_has_optional_id(&Some(self.clone()), optional_id)
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

    pub async fn try_into_coach(self, state: &AppState) -> Result<Coach, AppError> {
        let elo = coaches::select_elo_for_user(state, &self.id).await?;

        Ok(Coach {
            id: self.id,
            name: self.name,
            elo,
        })
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
                SET expires_at = CURRENT_TIMESTAMP + interval '48 hours'
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
