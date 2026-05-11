use crate::data::role_playing_games::campaigns::Campaign;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
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

pub async fn select_for_campaigns(
    state: &AppState,
    campaign: &Campaign,
) -> Result<Vec<NarrativeArc>, AppError> {
    tracing::debug!("select_for_campaigns with id={}", campaign.id);

    let arc_rows: Vec<NarrativeArc> = sqlx::query_as(
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
    .bind(campaign.id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(arc_rows)
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
                and game_master_id = $2",
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
