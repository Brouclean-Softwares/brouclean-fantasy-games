use crate::AppState;
use crate::data::role_playing_games::campaigns;
use crate::data::role_playing_games::campaigns::arcs::NarrativeArc;
use crate::data::users::User;
use crate::errors::AppError;
use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct GameSession {
    pub id: i32,
    pub position: i32,
    pub arc_position: i32,
    pub name: String,
    pub playing_at: Option<NaiveDateTime>,
    pub external_image_url: Option<String>,
    pub description: String,
    pub notes: String,
    pub campaign_id: i32,
    pub arc_id: i32,
}

impl GameSession {
    pub fn indexed_name(&self) -> String {
        session_indexed_name(self.arc_position + 1, self.position + 1, &self.name)
    }
}

#[derive(Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct CampaignSession {
    pub id: i32,
    pub playing_at: Option<NaiveDateTime>,
    pub campaign_name: String,
    pub game_name: String,
    pub game_external_logo_url: Option<String>,
    pub game_master_name: Option<String>,
}

pub fn session_indexed_name(
    arc_position: i32,
    session_position: i32,
    session_name: &String,
) -> String {
    format!(
        "{}.{}. {}",
        arc_position + 1,
        session_position + 1,
        session_name
    )
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<GameSession, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let session: GameSession = sqlx::query_as(
        "SELECT rpg_sessions.id,
                    rpg_sessions.position,
                    rpg_arcs.position as arc_position,
                    rpg_sessions.name,
                    rpg_sessions.playing_at,
                    rpg_sessions.external_image_url,
                    rpg_sessions.description,
                    rpg_sessions.notes,
                    rpg_arcs.campaign_id,
                    rpg_sessions.arc_id
            FROM rpg_sessions
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            WHERE rpg_sessions.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(session)
}

pub async fn select_for_arc(state: &AppState, arc_id: i32) -> Result<Vec<GameSession>, AppError> {
    tracing::debug!("select_for_arc with id={}", arc_id);

    let sessions: Vec<GameSession> = sqlx::query_as(
        "SELECT rpg_sessions.id,
                    rpg_sessions.position,
                    rpg_arcs.position as arc_position,
                    rpg_sessions.name,
                    rpg_sessions.playing_at,
                    rpg_sessions.external_image_url,
                    rpg_sessions.description,
                    rpg_sessions.notes,
                    rpg_arcs.campaign_id,
                    rpg_sessions.arc_id
            FROM rpg_sessions
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            WHERE rpg_arcs.id = $1
            ORDER BY rpg_sessions.position ASC",
    )
    .bind(arc_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(sessions)
}

pub async fn select_previous_session(
    state: &AppState,
    session: &GameSession,
) -> Result<Option<GameSession>, AppError> {
    tracing::debug!("select_previous_session for session id={}", session.id);

    let session: Option<GameSession> = sqlx::query_as(
        "SELECT rpg_sessions.id,
                    rpg_sessions.position,
                    rpg_arcs.position as arc_position,
                    rpg_sessions.name,
                    rpg_sessions.playing_at,
                    rpg_sessions.external_image_url,
                    rpg_sessions.description,
                    rpg_sessions.notes,
                    rpg_arcs.campaign_id,
                    rpg_sessions.arc_id
            FROM rpg_sessions
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            WHERE (rpg_sessions.position < $1 AND rpg_arcs.position = $2)
            OR rpg_arcs.position < $2
            ORDER BY rpg_arcs.position DESC,
                     rpg_sessions.position DESC
            LIMIT 1",
    )
    .bind(session.position.clone())
    .bind(session.arc_position.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(session)
}

pub async fn select_next_session(
    state: &AppState,
    session: &GameSession,
) -> Result<Option<GameSession>, AppError> {
    tracing::debug!("select_next_session for session id={}", session.id);

    let session: Option<GameSession> = sqlx::query_as(
        "SELECT rpg_sessions.id,
                    rpg_sessions.position,
                    rpg_arcs.position as arc_position,
                    rpg_sessions.name,
                    rpg_sessions.playing_at,
                    rpg_sessions.external_image_url,
                    rpg_sessions.description,
                    rpg_sessions.notes,
                    rpg_arcs.campaign_id,
                    rpg_sessions.arc_id
            FROM rpg_sessions
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            WHERE (rpg_sessions.position > $1 AND rpg_arcs.position = $2)
            OR rpg_arcs.position > $2
            ORDER BY rpg_arcs.position ASC,
                     rpg_sessions.position ASC
            LIMIT 1",
    )
    .bind(session.position.clone())
    .bind(session.arc_position.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(session)
}

pub async fn select_schedule_sessions_for_user(
    state: &AppState,
    user_id: i32,
) -> Result<Vec<CampaignSession>, AppError> {
    tracing::debug!("select_schedule_sessions_for_user for user_id={}", user_id);

    let sessions: Vec<CampaignSession> = sqlx::query_as(
        "SELECT rpg_sessions.id,
                    rpg_sessions.playing_at,
                    rpg_campaigns.name as campaign_name,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.name as game_master_name
            FROM rpg_sessions
            INNER JOIN rpg_arcs
            ON rpg_sessions.arc_id = rpg_arcs.id
            INNER JOIN rpg_campaigns
            ON rpg_arcs.campaign_id = rpg_campaigns.id
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_campaigns.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_campaigns.game_master_id
            WHERE rpg_campaigns.game_master_id = $1
            AND rpg_sessions.playing_at >= CURRENT_TIMESTAMP
            ORDER BY rpg_sessions.playing_at",
    )
    .bind(user_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(sessions)
}

pub async fn push_new_into_arc(
    state: &AppState,
    connected_user: &User,
    arc: &NarrativeArc,
    session_name: String,
) -> Result<i32, AppError> {
    tracing::debug!(
        "push_new_into_arc for arc with id={} with name={} by user={:?}",
        arc.id,
        session_name,
        connected_user
    );

    let campaign = campaigns::select_by_id(state, arc.campaign_id).await?;

    if !campaign.is_this_user_the_game_master(&Some(connected_user.clone())) {
        return Err(AppError::Unauthorized);
    }

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        let mut position = 0;

        let current_last_position: Option<i32> = sqlx::query_scalar(
            "SELECT max(position)
                FROM rpg_sessions
                WHERE arc_id = $1",
        )
        .bind(arc.id.clone())
        .fetch_one(&mut *transaction)
        .await?;

        if let Some(current_last_position) = current_last_position {
            position = current_last_position + 1;
        }

        let new_session_id: i32 = sqlx::query_scalar(
            "INSERT INTO rpg_sessions (
                    arc_id,
                    name,
                    position)
                VALUES ($1, $2, $3)
                RETURNING id",
        )
        .bind(arc.id.clone())
        .bind(session_name.clone())
        .bind(position.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_arcs
                SET last_updated = CURRENT_TIMESTAMP
                FROM rpg_campaigns
                WHERE rpg_arcs.id = $1
                AND rpg_campaigns.game_master_id = $2
                AND rpg_campaigns.id = rpg_arcs.campaign_id",
        )
        .bind(arc.id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_campaigns
                SET last_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                AND game_master_id = $2",
        )
        .bind(campaign.id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(new_session_id)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn update(
    state: &AppState,
    connected_user: &User,
    session: &GameSession,
) -> Result<(), AppError> {
    tracing::debug!("update with id={} by user={:?}", session.id, connected_user);

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "UPDATE rpg_sessions
            SET name = $3,
                playing_at = $4,
                external_image_url = $5,
                description = $6,
                notes = $7,
                arc_id = $8,
                last_updated = CURRENT_TIMESTAMP
            FROM rpg_campaigns, rpg_arcs
            WHERE rpg_sessions.id = $1
            AND rpg_arcs.id = rpg_sessions.arc_id
            AND rpg_campaigns.id = rpg_arcs.campaign_id
            AND rpg_campaigns.game_master_id = $2",
        )
        .bind(session.id.clone())
        .bind(connected_user_id.clone())
        .bind(session.name.clone())
        .bind(session.playing_at.clone())
        .bind(session.external_image_url.clone())
        .bind(session.description.clone())
        .bind(session.notes.clone())
        .bind(session.arc_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_arcs
                SET last_updated = CURRENT_TIMESTAMP
                FROM rpg_campaigns
                WHERE rpg_arcs.id = $1
                AND rpg_campaigns.game_master_id = $2
                AND rpg_campaigns.id = rpg_arcs.campaign_id",
        )
        .bind(session.arc_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_campaigns
                SET last_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                AND game_master_id = $2",
        )
        .bind(session.campaign_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn delete(
    state: &AppState,
    connected_user: &User,
    session_id: i32,
) -> Result<bool, AppError> {
    tracing::debug!(
        "delete by user={:?} for session_id={}",
        connected_user,
        session_id,
    );

    if let Some(connected_user_id) = connected_user.id {
        let session = select_by_id(state, session_id).await?;

        let mut transaction = state.db.begin().await?;

        let position_deleted: i32 = sqlx::query_scalar(
            "DELETE
                FROM rpg_sessions
                USING rpg_arcs, rpg_campaigns
                WHERE rpg_arcs.id = rpg_sessions.arc_id
                AND rpg_campaigns.id = rpg_arcs.campaign_id
                AND rpg_sessions.id = $1
                AND rpg_campaigns.game_master_id = $2
                RETURNING rpg_sessions.position",
        )
        .bind(session_id.clone())
        .bind(connected_user_id.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_sessions
            SET position = rpg_sessions.position - 1
            FROM rpg_campaigns, rpg_arcs
            WHERE rpg_sessions.position > $1
            AND rpg_arcs.id = rpg_sessions.arc_id
            AND rpg_campaigns.id = rpg_arcs.campaign_id
            AND rpg_campaigns.game_master_id = $2
            AND rpg_sessions.arc_id = $3",
        )
        .bind(position_deleted.clone())
        .bind(connected_user_id.clone())
        .bind(session.arc_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(true)
}
