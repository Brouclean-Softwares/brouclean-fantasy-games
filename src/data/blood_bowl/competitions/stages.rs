use crate::data::blood_bowl::competitions::Competition;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct CompetitionStageRow {
    id: i32,
    stage_name: String,
    stage_type: String,
    rules: String,
}

impl CompetitionStageRow {
    async fn into_competition_stage(self) -> Result<CompetitionStage, AppError> {
        Ok(CompetitionStage {
            id: self.id,
            stage_name: self.stage_name,
            stage_type: serde_json::from_str(&self.stage_type)?,
            rules: serde_json::from_str(&self.rules)?,
        })
    }
}

#[derive(Clone)]
pub struct CompetitionStage {
    pub id: i32,
    pub stage_name: String,
    pub stage_type: CompetitionStageType,
    pub rules: Vec<CompetitionStageRule>,
}

impl CompetitionStage {
    pub async fn select_for_competition(
        state: &AppState,
        competition: &Competition,
    ) -> Result<Vec<CompetitionStage>, AppError> {
        tracing::debug!(
            "select_for_competition for competition_id={}",
            competition.id
        );

        let rows: Vec<CompetitionStageRow> = sqlx::query_as(
            "SELECT id,
                    stage_name,
                    stage_type,
                    rules
            FROM bb_competitions_stages
            WHERE competition_id = $1
            ORDER BY created_at ASC",
        )
        .bind(competition.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut competition_stages: Vec<CompetitionStage> = Vec::with_capacity(rows.len());

        for competition_stage_row in rows {
            competition_stages.push(competition_stage_row.into_competition_stage().await?);
        }

        Ok(competition_stages)
    }

    pub async fn insert_for_competition(
        state: &AppState,
        connected_user: &User,
        competition: &mut Competition,
        stage_type_to_add: CompetitionStageType,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_for_competition for competition_id={} and stage_type_to_add={:?}",
            competition.id,
            stage_type_to_add,
        );

        if connected_user.eq(&competition.director) && !competition.closed {
            let rules: Vec<CompetitionStageRule> = vec![];

            sqlx::query(
                "INSERT INTO bb_competitions_stages (
                            competition_id,
                            stage_type,
                            stage_name,
                            rules)
                        VALUES ($1, $2, $3, $4)",
            )
            .bind(competition.id.clone())
            .bind(serde_json::to_string(&stage_type_to_add)?.clone())
            .bind(stage_type_to_add.to_string().clone())
            .bind(serde_json::to_string(&rules)?.clone())
            .execute(&state.db)
            .await?;

            competition.save(state, connected_user).await?;
        }

        Ok(())
    }

    pub async fn delete_for_competition(
        state: &AppState,
        connected_user: &User,
        competition: &mut Competition,
        stage_id: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "delete_for_competition for competition_id={} and stage_id={}",
            competition.id,
            stage_id,
        );

        if connected_user.eq(&competition.director) && !competition.started {
            sqlx::query(
                "DELETE
                    FROM bb_competitions_stages
                    WHERE id = $1",
            )
            .bind(stage_id.clone())
            .execute(&state.db)
            .await?;

            competition.save(state, connected_user).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum CompetitionStageType {
    Championship,
    Cup,
}

impl Display for CompetitionStageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            CompetitionStageType::Championship => "Championnat",
            CompetitionStageType::Cup => "Coupe",
        };

        write!(f, "{}", text)
    }
}

impl CompetitionStageType {
    pub fn available_list() -> Vec<Self> {
        let mut list = vec![Self::Championship, Self::Cup];

        list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

        list
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum CompetitionStageRule {}
