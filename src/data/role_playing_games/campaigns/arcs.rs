use crate::AppState;
use crate::data::role_playing_games::campaigns::sessions::GameSession;
use crate::data::role_playing_games::campaigns::{Campaign, sessions};
use crate::data::users::User;
use crate::errors::AppError;
use serde::Deserialize;

#[derive(Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct NarrativeArc {
    pub id: i32,
    pub position: i32,
    pub name: String,
    pub external_image_url: Option<String>,
    pub description: String,
    pub notes: String,
    pub campaign_id: i32,
}

impl NarrativeArc {
    pub fn indexed_name(&self) -> String {
        format!("{}. {}", self.position + 1, self.name)
    }
}

pub struct NarrativeArcWithGameSessions {
    pub arc: NarrativeArc,
    pub sessions: Vec<GameSession>,
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<NarrativeArc, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let arc: NarrativeArc = sqlx::query_as(
        "SELECT id,
                    position,
                    name,
                    external_image_url,
                    description,
                    notes,
                    campaign_id
            FROM rpg_arcs
            WHERE id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(arc)
}

pub async fn select_for_campaign(
    state: &AppState,
    campaign_id: i32,
) -> Result<Vec<NarrativeArc>, AppError> {
    tracing::debug!("select_for_campaign with id={}", campaign_id);

    let arcs: Vec<NarrativeArc> = sqlx::query_as(
        "SELECT id,
                    position,
                    name,
                    external_image_url,
                    description,
                    notes,
                    campaign_id
            FROM rpg_arcs
            WHERE campaign_id = $1
            ORDER BY position ASC",
    )
    .bind(campaign_id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(arcs)
}

pub async fn select_for_campaign_with_game_sessions(
    state: &AppState,
    campaign_id: i32,
) -> Result<Vec<NarrativeArcWithGameSessions>, AppError> {
    tracing::debug!(
        "select_for_campaign_with_game_sessions with id={}",
        campaign_id
    );

    let arcs: Vec<NarrativeArc> = select_for_campaign(state, campaign_id).await?;

    let mut arcs_with_sessions: Vec<NarrativeArcWithGameSessions> = Vec::with_capacity(arcs.len());

    for arc in arcs {
        arcs_with_sessions.push(NarrativeArcWithGameSessions {
            sessions: sessions::select_for_arc(state, arc.id).await?,
            arc,
        });
    }

    Ok(arcs_with_sessions)
}

pub async fn push_new_into_campaign(
    state: &AppState,
    connected_user: &User,
    campaign: &Campaign,
    arc_name: String,
) -> Result<i32, AppError> {
    tracing::debug!(
        "push_new_into_campaign for campaign with id={} with name={} by user={:?}",
        campaign.id,
        arc_name,
        connected_user
    );

    if !campaign.is_this_user_the_game_master(&Some(connected_user.clone())) {
        return Err(AppError::Unauthorized);
    }

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        let mut position = 0;

        let current_last_position: Option<i32> = sqlx::query_scalar(
            "SELECT max(position)
                FROM rpg_arcs
                WHERE campaign_id = $1",
        )
        .bind(campaign.id.clone())
        .fetch_one(&mut *transaction)
        .await?;

        if let Some(current_last_position) = current_last_position {
            position = current_last_position + 1;
        }

        let new_arc_id: i32 = sqlx::query_scalar(
            "INSERT INTO rpg_arcs (
                    campaign_id,
                    name,
                    position)
                VALUES ($1, $2, $3)
                RETURNING id",
        )
        .bind(campaign.id.clone())
        .bind(arc_name.clone())
        .bind(position.clone())
        .fetch_one(&mut *transaction)
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

        Ok(new_arc_id)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn update(
    state: &AppState,
    connected_user: &User,
    arc: &NarrativeArc,
) -> Result<(), AppError> {
    tracing::debug!("update with id={} by user={:?}", arc.id, connected_user);

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "UPDATE rpg_arcs
            SET name = $3,
                external_image_url = $4,
                description = $5,
                notes = $6,
                campaign_id = $7,
                last_updated = CURRENT_TIMESTAMP
            FROM rpg_campaigns
            WHERE rpg_arcs.id = $1
            AND rpg_campaigns.id = rpg_arcs.campaign_id
            AND rpg_campaigns.game_master_id = $2",
        )
        .bind(arc.id.clone())
        .bind(connected_user_id.clone())
        .bind(arc.name.clone())
        .bind(arc.external_image_url.clone())
        .bind(arc.description.clone())
        .bind(arc.notes.clone())
        .bind(arc.campaign_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "UPDATE rpg_campaigns
                SET last_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                AND game_master_id = $2",
        )
        .bind(arc.campaign_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(())
}
