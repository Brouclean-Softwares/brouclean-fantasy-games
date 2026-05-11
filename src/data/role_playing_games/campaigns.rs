use crate::AppState;
use crate::data::users::User;
use crate::errors::AppError;
use serde::Deserialize;

pub mod arcs;
pub mod sessions;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct CampaignRow {
    pub id: i32,
    pub name: String,
    pub external_image_url: Option<String>,
    pub description: String,
    pub notes: String,
    pub game_id: i32,
    pub game_name: String,
    pub game_external_logo_url: Option<String>,
    pub game_master_id: Option<i32>,
    pub game_master_name: Option<String>,
}

impl CampaignRow {
    pub async fn into_campaign(self, state: &AppState) -> Result<Campaign, AppError> {
        let game_master = User::select_by_id(state, self.game_master_id).await?;

        Ok(Campaign {
            id: self.id,
            name: self.name,
            external_image_url: self.external_image_url,
            description: self.description,
            notes: self.notes,
            game_id: self.game_id,
            game_name: self.game_name,
            game_external_logo_url: self.game_external_logo_url,
            game_master,
        })
    }
}

#[derive(Debug)]
pub struct Campaign {
    pub id: i32,
    pub name: String,
    pub external_image_url: Option<String>,
    pub description: String,
    pub notes: String,
    pub game_id: i32,
    pub game_name: String,
    pub game_external_logo_url: Option<String>,
    pub game_master: Option<User>,
}

impl Campaign {
    pub fn is_this_user_the_game_master(&self, user: &Option<User>) -> bool {
        User::optional_user_eq_other(&self.game_master, user)
    }
}

pub async fn select_all(state: &AppState) -> Result<Vec<CampaignRow>, AppError> {
    tracing::debug!("select_all");

    let campaign_rows: Vec<CampaignRow> = sqlx::query_as(
        "SELECT rpg_campaigns.id,
                    rpg_campaigns.name,
                    rpg_campaigns.external_image_url,
                    rpg_campaigns.description,
                    rpg_campaigns.notes,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as game_master_id,
                    users.name as game_master_name
            FROM rpg_campaigns
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_campaigns.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_campaigns.game_master_id
            ORDER BY rpg_campaigns.name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(campaign_rows)
}

pub async fn select_owned(state: &AppState, user: &User) -> Result<Vec<CampaignRow>, AppError> {
    tracing::debug!("select_owned for user={:?}", user);

    let campaign_rows: Vec<CampaignRow> = sqlx::query_as(
        "SELECT rpg_campaigns.id,
                    rpg_campaigns.name,
                    rpg_campaigns.external_image_url,
                    rpg_campaigns.description,
                    rpg_campaigns.notes,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as game_master_id,
                    users.name as game_master_name
            FROM rpg_campaigns
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_campaigns.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_campaigns.game_master_id
            WHERE rpg_campaigns.game_master_id = $1
            ORDER BY rpg_campaigns.name ASC",
    )
    .bind(user.id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(campaign_rows)
}

pub async fn select_by_id(state: &AppState, id: i32) -> Result<Campaign, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let campaign_row: CampaignRow = sqlx::query_as(
        "SELECT rpg_campaigns.id,
                    rpg_campaigns.name,
                    rpg_campaigns.external_image_url,
                    rpg_campaigns.description,
                    rpg_campaigns.notes,
                    rpg_games.id as game_id,
                    rpg_games.name as game_name,
                    rpg_games.external_logo_url as game_external_logo_url,
                    users.id as game_master_id,
                    users.name as game_master_name
            FROM rpg_campaigns
            INNER JOIN rpg_games
            ON rpg_games.id = rpg_campaigns.game_id
            LEFT OUTER JOIN users
            ON users.id = rpg_campaigns.game_master_id
            WHERE rpg_campaigns.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let campaign = campaign_row.into_campaign(state).await?;

    Ok(campaign)
}

pub async fn create(state: &AppState, user: &User, campaign: &Campaign) -> Result<i32, AppError> {
    tracing::debug!(
        "create for user={:?} the following campaign={:?}",
        user,
        campaign,
    );

    let new_campaign_id: i32 = sqlx::query_scalar(
        "INSERT INTO rpg_campaigns (
                game_id,
                name,
                game_master_id)
            VALUES ($1, $2, $3)
            RETURNING id",
    )
    .bind(campaign.game_id.clone())
    .bind(campaign.name.clone())
    .bind(user.id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(new_campaign_id)
}

pub async fn update(
    state: &AppState,
    connected_user: &User,
    campaign: &Campaign,
) -> Result<(), AppError> {
    tracing::debug!(
        "update campaign with id={} by user={:?}",
        campaign.id,
        connected_user
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE rpg_campaigns
            SET name = $3,
                external_image_url = $4,
                description = $5,
                notes = $6,
                game_id = $7,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $1
            AND game_master_id = $2",
        )
        .bind(campaign.id.clone())
        .bind(connected_user_id.clone())
        .bind(campaign.name.clone())
        .bind(campaign.external_image_url.clone())
        .bind(campaign.description.clone())
        .bind(campaign.notes.clone())
        .bind(campaign.game_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}
