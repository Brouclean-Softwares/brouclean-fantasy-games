use crate::AppState;
use crate::data::blood_bowl::teams;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::staffs::Staff;
use serde::Deserialize;
use sqlx::Postgres;
use std::collections::HashMap;

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct StaffDetail {
    pub staff: Staff,
    pub number: i32,
}

pub async fn select_for_team(
    state: &AppState,
    team_id: i32,
) -> Result<HashMap<Staff, u8>, AppError> {
    tracing::debug!("select_for_team with team_id={}", team_id);

    let staff_detail: Vec<StaffDetail> = sqlx::query_as(
        "SELECT staff,
                    number
            FROM bb_teams_staff
            WHERE team_id = $1",
    )
    .bind(team_id.clone())
    .fetch_all(&state.db)
    .await?;

    let mut staff: HashMap<Staff, u8> = HashMap::new();

    for staff_detail in staff_detail {
        staff.insert(staff_detail.staff, staff_detail.number as u8);
    }

    Ok(staff)
}

pub async fn update_staff_for_team(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    connected_user: &User,
    team_id: i32,
    staff: Staff,
    new_staff_quantity: u8,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_staff_for_team by user={:?} for team_id={} with staff={:?}",
        connected_user,
        team_id,
        staff
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_teams_staff
            SET number = $1
            FROM bb_teams
            WHERE bb_teams.id = bb_teams_staff.team_id
            AND bb_teams.id = $2
            AND bb_teams.coach_id = $3
            AND bb_teams_staff.staff = $4",
        )
        .bind(new_staff_quantity.clone() as i32)
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .bind(staff.clone())
        .execute(transaction.as_mut())
        .await?;
    }

    Ok(())
}

pub async fn buy_staff_for_team(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    staff: Staff,
) -> Result<(), AppError> {
    tracing::debug!(
        "buy_staff_for_team by user={:?} for team_id={} with staff={:?}",
        connected_user,
        team_id,
        staff
    );

    if let Some(connected_user_id) = connected_user.id {
        let mut team = teams::select_by_id_with_staff_and_players(state, team_id).await?;

        if !team.is_drafting() {
            let new_staff_quantity = team.buy_staff(&staff)?;

            let team_value = team.value()?;
            let team_current_value = team.current_value()?;

            let mut transaction = state.db.begin().await?;

            update_staff_for_team(
                &mut transaction,
                connected_user,
                team.id,
                staff,
                new_staff_quantity,
            )
            .await?;

            sqlx::query(
                "UPDATE bb_teams
                SET treasury = $1,
                    value = $2,
                    current_value = $3
                WHERE id = $4
                AND coach_id = $5",
            )
            .bind(team.treasury.clone())
            .bind(team_value.clone() as i32)
            .bind(team_current_value.clone() as i32)
            .bind(team_id.clone())
            .bind(connected_user_id.clone())
            .execute(&mut *transaction)
            .await?;

            transaction.commit().await?;
        }
    }

    Ok(())
}
