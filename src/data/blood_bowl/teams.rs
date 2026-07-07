use crate::AppState;
use crate::data::blood_bowl::competitions::schedule::BYE;
use crate::data::blood_bowl::statistics::teams::TeamResults;
use crate::data::blood_bowl::{coaches, games, players, staff, teams};
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TypeName;
use blood_bowl_rs::versions::Version;
use serde::Deserialize;
use std::collections::HashMap;

pub struct TeamLogo {
    pub url: String,
}

impl TeamLogo {
    pub fn from_url_or_roster(external_logo_url: Option<String>, roster: Roster) -> Self {
        if let Some(url) = external_logo_url {
            Self { url }
        } else {
            Self {
                url: format!("/assets/images/blood_bowl/{}Logo.webp", roster.type_name()),
            }
        }
    }
}

impl From<&Team> for TeamLogo {
    fn from(team: &Team) -> Self {
        Self::from_url_or_roster(team.external_logo_url.clone(), team.roster.clone())
    }
}

impl From<TeamSummary> for TeamLogo {
    fn from(team: TeamSummary) -> Self {
        Self::from(&team)
    }
}

impl From<&TeamSummary> for TeamLogo {
    fn from(team: &TeamSummary) -> Self {
        Self::from_url_or_roster(team.external_logo_url.clone(), team.roster.clone())
    }
}

impl From<Team> for TeamLogo {
    fn from(team: Team) -> Self {
        Self::from(&team)
    }
}

pub struct TeamSummaryWithResults {
    pub team: TeamSummary,
    pub results: TeamResults,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct TeamSummary {
    pub id: i32,
    pub version: Version,
    pub name: String,
    pub roster: Roster,
    pub coach_id: Option<i32>,
    pub coach_name: String,
    pub external_logo_url: Option<String>,
    pub value: i32,
    pub current_value: i32,
    pub treasury: i32,
    pub dedicated_fans: i32,
    pub under_creation: bool,
}

impl PartialEq<Self> for TeamSummary {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialEq<BYE> for TeamSummary {
    fn eq(&self, bye: &BYE) -> bool {
        self.id.eq(&bye.id)
    }
}

impl PartialEq<Option<Self>> for TeamSummary {
    fn eq(&self, other: &Option<Self>) -> bool {
        if let Some(other) = other {
            self.id.eq(&other.id)
        } else {
            false
        }
    }
}

impl TeamSummary {
    pub fn list_into_list_with_option(team_list: &Vec<Self>) -> Vec<Option<Self>> {
        team_list.iter().map(|team| Some(team.clone())).collect()
    }

    pub async fn with_results(self, state: &AppState) -> Result<TeamSummaryWithResults, AppError> {
        let results = super::statistics::teams::select_results(state, self.id).await?;

        Ok(TeamSummaryWithResults {
            team: self,
            results,
        })
    }
}

pub async fn select_all(state: &AppState) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_all");

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users
            ON bb_teams.coach_id = users.id
            ORDER BY bb_teams.name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_all_with_results(
    state: &AppState,
) -> Result<Vec<TeamSummaryWithResults>, AppError> {
    tracing::debug!("select_all_with_results");

    let teams = select_all(state).await?;

    let mut teams_with_results = Vec::with_capacity(teams.len());

    for team in teams {
        teams_with_results.push(team.with_results(state).await?);
    }

    Ok(teams_with_results)
}

pub async fn select_all_filtered(
    state: &AppState,
    filter: String,
) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_all_filtered with filter={}", filter);

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE LOWER(bb_teams.name) LIKE $1
            OR LOWER(users.name) LIKE $1
            ORDER BY bb_teams.name ASC",
    )
    .bind(format!("%{}%", filter.to_lowercase()))
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_owned(state: &AppState, coach: User) -> Result<Vec<TeamSummary>, AppError> {
    tracing::debug!("select_owned for coach={:?}", coach);

    let teams: Vec<TeamSummary> = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE coach_id = $1
            ORDER BY value DESC",
    )
    .bind(coach.id.clone())
    .fetch_all(&state.db)
    .await?;

    Ok(teams)
}

pub async fn select_owned_with_results(
    state: &AppState,
    coach: User,
) -> Result<Vec<TeamSummaryWithResults>, AppError> {
    tracing::debug!("select_owned_with_results for coach={:?}", coach);

    let teams = select_owned(state, coach).await?;

    let mut teams_with_results = Vec::with_capacity(teams.len());

    for team in teams {
        teams_with_results.push(team.with_results(state).await?);
    }

    Ok(teams_with_results)
}

pub async fn select_summary_by_id(state: &AppState, id: i32) -> Result<TeamSummary, AppError> {
    tracing::debug!("select_summary_by_id with id={}", id);

    let team: TeamSummary = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE bb_teams.id = $1
            LIMIT 1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    Ok(team)
}

pub async fn select_by_id_with_staff_and_players(
    state: &AppState,
    id: i32,
) -> Result<Team, AppError> {
    select_by_id(state, id, true, true).await
}

pub async fn select_by_id_without_players(state: &AppState, id: i32) -> Result<Team, AppError> {
    select_by_id(state, id, true, false).await
}

pub async fn select_by_id_without_staff_nor_players(
    state: &AppState,
    id: i32,
) -> Result<Team, AppError> {
    select_by_id(state, id, false, false).await
}

async fn select_by_id(
    state: &AppState,
    id: i32,
    staff_needed: bool,
    players_needed: bool,
) -> Result<Team, AppError> {
    tracing::debug!("select_by_id with id={}", id);

    let team: TeamSummary = sqlx::query_as(
        "SELECT bb_teams.id,
                    bb_teams.version,
                    bb_teams.name,
                    bb_teams.roster,
                    bb_teams.coach_id,
                    users.name as coach_name,
                    bb_teams.external_logo_url,
                    bb_teams.value,
                    bb_teams.current_value,
                    bb_teams.treasury,
                    bb_teams.dedicated_fans,
                    bb_teams.under_creation
            FROM bb_teams
            LEFT JOIN users ON bb_teams.coach_id = users.id
            WHERE bb_teams.id = $1",
    )
    .bind(id.clone())
    .fetch_one(&state.db)
    .await?;

    let staff = if staff_needed {
        staff::select_for_team(state, id).await?
    } else {
        HashMap::new()
    };

    let players = if players_needed {
        players::select_under_contract_for_team(state, id).await?
    } else {
        vec![]
    };

    let coach = coaches::select_by_id(state, team.coach_id)
        .await?
        .unwrap_or(Coach {
            id: team.coach_id,
            name: team.coach_name,
            elo: None,
        });

    let team = Team {
        id: team.id,
        version: team.version,
        roster: team.roster,
        name: team.name,
        coach,
        treasury: team.treasury,
        external_logo_url: team.external_logo_url,
        staff,
        players,
        dedicated_fans: team.dedicated_fans as u8,
        under_creation: team.under_creation,
    };

    Ok(team)
}

pub async fn create(state: &AppState, coach: &User, bb_team: &Team) -> Result<i32, AppError> {
    tracing::debug!(
        "create for coach={:?} the following team={:?}",
        coach,
        bb_team
    );

    let mut transaction = state.db.begin().await?;

    let new_team_id: i32 = sqlx::query_scalar(
        "INSERT INTO bb_teams (
                version,
                name,
                roster,
                coach_id,
                treasury,
                dedicated_fans,
                value,
                current_value,
                under_creation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
            RETURNING id",
    )
    .bind(bb_team.version.clone())
    .bind(bb_team.name.clone())
    .bind(bb_team.roster.clone())
    .bind(coach.id.clone())
    .bind(bb_team.treasury.clone())
    .bind(bb_team.dedicated_fans.clone() as i32)
    .bind(bb_team.value()? as i32)
    .bind(bb_team.current_value()? as i32)
    .fetch_one(&mut *transaction)
    .await?;

    for (staff, quantity) in bb_team.staff.clone() {
        sqlx::query(
            "INSERT INTO bb_teams_staff (
                staff,
                number,
                team_id)
            VALUES ($1, $2, $3)",
        )
        .bind(staff.clone())
        .bind(quantity.clone() as i32)
        .bind(new_team_id.clone())
        .execute(&mut *transaction)
        .await?;
    }

    for (number, player) in bb_team.players.clone() {
        let new_player_id: i32 = sqlx::query_scalar(
            "INSERT INTO bb_players (
                version,
                name,
                position,
                is_captain)
            VALUES ($1, $2, $3, $4)
            RETURNING id",
        )
        .bind(player.version.clone())
        .bind(player.name.clone())
        .bind(player.position.clone())
        .bind(player.is_captain.clone())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO bb_teams_players (
                number,
                team_id,
                player_id)
            VALUES ($1, $2, $3)",
        )
        .bind(number.clone())
        .bind(new_team_id.clone())
        .bind(new_player_id.clone())
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(new_team_id)
}

pub async fn update_values(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_values by user={:?} for team_id={}",
        connected_user,
        team_id,
    );

    let team = teams::select_by_id_with_staff_and_players(state, team_id).await?;
    let team_value = team.value()?;
    let team_current_value = team.current_value()?;

    sqlx::query(
        "UPDATE bb_teams
        SET value = $1,
            current_value = $2,
            last_updated = CURRENT_TIMESTAMP
        WHERE id = $3",
    )
    .bind(team_value.clone() as i32)
    .bind(team_current_value.clone() as i32)
    .bind(team_id.clone())
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn update_name(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
    name: &String,
) -> Result<(), AppError> {
    tracing::debug!(
        "update_name by user={:?} for team_id={} with name={}",
        connected_user,
        team_id,
        name
    );

    if let Some(connected_user_id) = connected_user.id {
        sqlx::query(
            "UPDATE bb_teams
            SET name = $1,
                last_updated = CURRENT_TIMESTAMP
            WHERE id = $2
            AND coach_id = $3",
        )
        .bind(name.clone())
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&state.db)
        .await?;
    }

    Ok(())
}

pub async fn delete(
    state: &AppState,
    connected_user: &User,
    team_id: i32,
) -> Result<bool, AppError> {
    tracing::debug!(
        "delete by user={:?} for team_id={}",
        connected_user,
        team_id,
    );

    let games_played = games::select_played_by_team(state, team_id).await?;
    let games_scheduled = games::select_scheduled_for_team(state, team_id).await?;
    let game_playing = games::select_playing_by_team(state, team_id).await?;

    if games_played.len() > 0 || games_scheduled.len() > 0 || game_playing.is_some() {
        return Ok(false);
    }

    if let Some(connected_user_id) = connected_user.id {
        let mut transaction = state.db.begin().await?;

        sqlx::query(
            "DELETE
                FROM bb_players
                USING bb_teams_players, bb_teams
                WHERE bb_players.id = bb_teams_players.player_id
                AND bb_teams.id = bb_teams_players.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams_players
                USING bb_teams
                WHERE bb_teams.id = bb_teams_players.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams_staff
                USING bb_teams
                WHERE bb_teams.id = bb_teams_staff.team_id
                AND bb_teams.id = $1
                AND bb_teams.coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "DELETE
                FROM bb_teams
                WHERE id = $1
                AND coach_id = $2",
        )
        .bind(team_id.clone())
        .bind(connected_user_id.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(true)
}

pub async fn upgrade(
    state: &AppState,
    connected_user: &User,
    current_team: Team,
) -> Result<bool, AppError> {
    tracing::debug!(
        "delete by user={:?} for team_id={}",
        connected_user,
        current_team.id,
    );

    let game_playing = games::select_playing_by_team(state, current_team.id).await?;

    if game_playing.is_some() {
        return Ok(false);
    }

    if current_team
        .coach
        .eq(&connected_user.clone().try_into_coach(state).await?)
    {
        let Some(new_version) = current_team.version.next() else {
            return Ok(false);
        };

        let Some(new_roster) = current_team.roster_for_next_version() else {
            return Ok(false);
        };

        let Some(current_roster_definition) = current_team.roster_definition() else {
            return Ok(false);
        };

        let Some(new_roster_definition) = new_roster.definition(new_version) else {
            return Ok(false);
        };

        let mut new_team = current_team.clone();
        new_team.version = new_version;
        new_team.roster = new_roster;

        let mut transaction = state.db.begin().await?;

        for (staff, current_quantity) in current_team.staff {
            if current_quantity > 0 {
                match (
                    current_roster_definition.get_staff_information(&staff),
                    new_roster_definition.get_staff_information(&staff),
                ) {
                    (Some(current_staff_information), Some(new_staff_information)) => {
                        let new_quantity = if current_quantity > new_staff_information.maximum {
                            new_staff_information.maximum
                        } else {
                            current_quantity
                        };

                        sqlx::query(
                            "UPDATE bb_teams_staff
                                SET number = $3
                                WHERE team_id = $1
                                AND staff = $2",
                        )
                        .bind(new_team.id.clone())
                        .bind(staff.clone())
                        .bind(new_quantity.clone() as i32)
                        .execute(&mut *transaction)
                        .await?;

                        new_team.staff.insert(staff, new_quantity);
                        new_team.treasury += new_quantity as i32
                            * (current_staff_information.price as i32
                                - new_staff_information.price as i32);
                        new_team.treasury += current_staff_information.price as i32
                            * (current_quantity as i32 - new_quantity as i32);
                    }

                    (Some(current_staff_information), None) => {
                        sqlx::query(
                            "UPDATE bb_teams_staff
                                SET number = 0
                                WHERE team_id = $1
                                AND staff = $2",
                        )
                        .bind(new_team.id.clone())
                        .bind(staff.clone())
                        .execute(&mut *transaction)
                        .await?;

                        new_team.staff.remove(&staff);
                        new_team.treasury +=
                            current_staff_information.price as i32 * current_quantity as i32;
                    }

                    (_, _) => {}
                }
            }
        }

        new_team.players = Vec::with_capacity(current_team.players.len());

        for (player_number, current_player) in current_team.players.iter() {
            if let Some(new_position) = current_player.position_for_next_version() {
                let mut new_player = current_player.clone();
                new_player.version = new_version;
                new_player.roster = new_roster;
                new_player.position = new_position;

                for injury_index in 0..new_player.injuries.len() {
                    if let Some(new_injury) = new_player.injuries[injury_index]
                        .injury_in_next_version_with_same_impact(&current_player.version)
                    {
                        sqlx::query(
                            "UPDATE bb_players_injuries
                                SET injury = $3
                                WHERE player_id = $1
                                AND created_at = (
                                    SELECT created_at
                                    FROM bb_players_injuries
                                    WHERE player_id = $1
                                    ORDER BY created_at
                                    LIMIT 1 OFFSET $2
                                )",
                        )
                        .bind(new_player.id.clone())
                        .bind(injury_index.clone() as i32)
                        .bind(new_injury.clone())
                        .execute(&mut *transaction)
                        .await?;

                        if new_injury.ne(&new_player.injuries[injury_index]) {
                            new_player.injuries[injury_index] = new_injury;
                        }
                    }
                }

                match (
                    current_player.position_definition(),
                    new_player.position_definition(),
                ) {
                    (Some(current_position_definition), Some(new_position_definition)) => {
                        sqlx::query(
                            "UPDATE bb_players
                            SET version = $2,
                                position = $3,
                                last_updated = CURRENT_TIMESTAMP
                            WHERE id = $1",
                        )
                        .bind(new_player.id.clone())
                        .bind(new_player.version.clone())
                        .bind(new_player.position.clone())
                        .execute(&mut *transaction)
                        .await?;

                        new_team.treasury += current_position_definition.cost as i32
                            - new_position_definition.cost as i32;

                        sqlx::query(
                            "DELETE FROM bb_players_advancements
                                WHERE player_id = $1",
                        )
                        .bind(new_player.id.clone())
                        .execute(&mut *transaction)
                        .await?;

                        new_player.advancements = Vec::new();

                        new_team.players.push((player_number.clone(), new_player));
                    }

                    (Some(current_position_definition), None) => {
                        sqlx::query(
                            "UPDATE bb_teams_players
                            SET contract_end = CURRENT_TIMESTAMP
                            WHERE team_id = $1
                            AND player_id = $2",
                        )
                        .bind(new_team.id.clone())
                        .bind(current_player.id.clone())
                        .execute(&mut *transaction)
                        .await?;

                        new_team.treasury += current_position_definition.cost as i32;
                    }

                    _ => {
                        return Err(AppError::BloodBowlAppError(String::from(
                            "Un joueur actuel n'a pas de poste connu",
                        )));
                    }
                }
            }
        }

        for position in new_roster_definition.positions.iter() {
            if let Some(position_definition) = position.definition(new_version, new_roster) {
                if new_team.position_number_under_contract(&position)
                    > position_definition.maximum_quantity
                {
                    new_team.treasury += position_definition.cost as i32
                        * (new_team.position_number_under_contract(&position) as i32
                            - position_definition.maximum_quantity as i32);
                }
            }
        }

        if current_team.dedicated_fans > new_roster_definition.dedicated_fans_information.maximum {
            new_team.dedicated_fans = new_roster_definition.dedicated_fans_information.maximum;
        }

        sqlx::query(
            "UPDATE bb_teams
                    SET version = $2,
                        treasury = $3,
                        value = $4,
                        current_value = $5,
                        dedicated_fans = $6,
                        roster = $7,
                        last_updated = CURRENT_TIMESTAMP
                    WHERE id = $1",
        )
        .bind(new_team.id.clone())
        .bind(new_team.version.clone())
        .bind(new_team.treasury.clone())
        .bind(new_team.value()?.clone() as i32)
        .bind(new_team.current_value()?.clone() as i32)
        .bind(new_team.dedicated_fans.clone() as i32)
        .bind(new_team.roster.clone())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
    }

    Ok(true)
}
