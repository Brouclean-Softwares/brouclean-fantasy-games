use crate::errors::AppError;
use crate::AppState;
use blood_bowl_rs::rosters::Staff;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct StaffDetail {
    staff: Staff,
    number: i32,
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
