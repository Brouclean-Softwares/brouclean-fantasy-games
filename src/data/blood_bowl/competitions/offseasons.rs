use crate::AppState;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::blood_bowl::staff::StaffDetail;
use crate::data::blood_bowl::{coaches, players, staff, teams};
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::dices::Dice;
use blood_bowl_rs::errors::Error;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::positions::Position;
use blood_bowl_rs::staffs::Staff;
use blood_bowl_rs::teams::Team;
use serde::Deserialize;
use sqlx::Postgres;
use std::collections::HashMap;

pub const OFFSEASON_COMPETITION_ROUND_THRESHOLD: usize = 10;
pub const INITIAL_FUNDS_FOR_REDRAFT: i32 = 1000000;
pub const FUNDS_FOR_GAME_PLAYED: i32 = 20000;
pub const FUNDS_FOR_GAME_WON: i32 = 20000;
pub const FUNDS_FOR_GAME_DREW: i32 = 10000;
pub const PLAYER_ADDITIONAL_FEES_FOR_SEASON_PLAYED_WITH_EXPERIENCE: u32 = 20000;

pub async fn start_competition_offseason(
    state: &AppState,
    connected_user: &User,
    competition: &Competition,
) -> Result<(), AppError> {
    tracing::debug!(
        "start_competition_offseason by user={:?} for competition_id={}",
        connected_user,
        competition.id
    );

    if let Some(director) = &competition.director {
        if connected_user.eq(director) && competition.closed {
            let mut transaction = state.db.begin().await?;

            sqlx::query(
                "DELETE
                    FROM bb_redrafts_in_offseasons
                    WHERE competition_id = $1",
            )
            .bind(competition.id.clone())
            .execute(&mut *transaction)
            .await?;

            sqlx::query(
                "DELETE FROM bb_redrafting_players
                WHERE competition_id = $1",
            )
            .bind(competition.id.clone())
            .execute(&mut *transaction)
            .await?;

            sqlx::query(
                "DELETE FROM bb_redrafting_staff
                WHERE competition_id = $1",
            )
            .bind(competition.id.clone())
            .execute(&mut *transaction)
            .await?;

            sqlx::query(
                "DELETE FROM bb_redrafting_positions
                WHERE competition_id = $1",
            )
            .bind(competition.id.clone())
            .execute(&mut *transaction)
            .await?;

            for team in competition.select_playing_teams(state).await? {
                // Rest & relaxation & resign
                let team_players = players::select_under_contract_for_team(state, team.id).await?;

                for (_, player) in team_players {
                    let player_injuries = players::select_player_injuries(state, player.id).await?;

                    for player_injury in player_injuries {
                        if player_injury.injury.is_niggling_injury() && Dice::D6.roll() >= 4 {
                            players::update_player_who_recovered_from_injury(
                                &mut transaction,
                                player.id,
                                &player_injury,
                            )
                            .await?;
                        }
                    }

                    let player_hatred = players::select_player_hatred(state, player.id).await?;

                    for hatred in player_hatred {
                        if Dice::D6.roll() >= 4 {
                            players::update_player_who_recovered_from_hatred(
                                &mut transaction,
                                player.id,
                                &hatred,
                            )
                            .await?;
                        }
                    }

                    PlayerRedraft::insert_player_to_sign(
                        &mut transaction,
                        competition.id,
                        team.id,
                        player.id,
                        player.has_experience(),
                    )
                    .await?;
                }

                let staff = staff::select_for_team(state, team.id).await?;

                for (staff, number) in staff {
                    TeamRedraft::insert_staff_to_sign(
                        &mut transaction,
                        competition.id,
                        team.id,
                        staff,
                        number as i32,
                    )
                    .await?;
                }

                // Raised funds
                let team_results = competition.select_team_results(state, team.id).await?;

                let raised_funds = INITIAL_FUNDS_FOR_REDRAFT
                    + (FUNDS_FOR_GAME_PLAYED * team_results.total_played() as i32)
                    + (FUNDS_FOR_GAME_WON * team_results.victories as i32)
                    + (FUNDS_FOR_GAME_DREW * team_results.draws as i32);

                sqlx::query(
                    "INSERT INTO bb_redrafts_in_offseasons (
                            competition_id,
                            team_id,
                            raised_funds
                        )
                        VALUES ($1, $2, $3)",
                )
                .bind(competition.id.clone())
                .bind(team.id.clone())
                .bind(raised_funds.clone())
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
        }
    }

    Ok(())
}

#[derive(Clone)]
pub struct TeamRedraft {
    pub team: Team,
    pub raised_funds: i32,
    pub players_redrafted: Vec<PlayerRedraft>,
    pub players_not_redrafted: Vec<PlayerRedraft>,
    pub staff_to_sign: HashMap<Staff, u8>,
    pub positions_to_sign: HashMap<Position, u8>,
}

impl TeamRedraft {
    pub fn resigned_team(&self) -> Result<Team, AppError> {
        let mut players: Vec<(i32, Player)> = self
            .players_redrafted
            .iter()
            .map(|player_redraft| (player_redraft.number, player_redraft.player.clone()))
            .collect();

        for (position, quantity) in &self.positions_to_sign {
            for _ in 0..*quantity {
                let player = Player::new(
                    self.team.version.clone(),
                    position.clone(),
                    self.team.roster.clone(),
                );

                players.push((0, player));
            }
        }

        let treasury = self.remaining_funds()?;

        Ok(Team {
            players,
            treasury,
            staff: self.staff_to_sign.clone(),
            ..self.team.clone()
        })
    }

    pub fn remaining_funds(&self) -> Result<i32, AppError> {
        let mut remaining_funds = self.raised_funds + self.team.treasury;

        for player_redrafted in &self.players_redrafted {
            remaining_funds = remaining_funds - player_redrafted.redraft_cost()? as i32;
        }

        let roster_definition = self
            .team
            .roster_definition()
            .ok_or(AppError::BloodBowlError(Error::RosterNotExist))?;

        for staff_information in roster_definition.staff_information {
            remaining_funds = remaining_funds
                - (staff_information.price as i32
                    * self.staff_quantity_to_sign(&staff_information.staff) as i32);
        }

        for (position, quantity) in &self.positions_to_sign {
            let position_definition = position
                .definition(self.team.version.clone(), self.team.roster.clone())
                .ok_or(AppError::BloodBowlError(Error::PositionNotDefined))?;

            remaining_funds =
                remaining_funds - (position_definition.cost as i32 * quantity.clone() as i32);
        }

        Ok(remaining_funds)
    }

    pub async fn select_from_team(state: &AppState, team: Team) -> Result<Option<Self>, AppError> {
        if !team.in_offseason {
            return Ok(None);
        }

        let mut team_redraft = Self {
            team,
            raised_funds: 0,
            players_redrafted: vec![],
            players_not_redrafted: vec![],
            staff_to_sign: Default::default(),
            positions_to_sign: Default::default(),
        };

        team_redraft.populate_offseason_info(state).await?;
        team_redraft.populate_players_redrafts(&state).await?;
        team_redraft.populate_staff_to_sign(&state).await?;
        team_redraft.populate_positions_to_sign(&state).await?;

        Ok(Some(team_redraft))
    }

    async fn select_offseason_competition_id(
        &self,
        state: &AppState,
    ) -> Result<Option<i32>, AppError> {
        tracing::debug!(
            "select_offseason_competition_id for team_id={}",
            self.team.id
        );

        let offseason_competition_id: Option<i32> = sqlx::query_scalar(
            "SELECT competition_id
                FROM bb_redrafts_in_offseasons
                WHERE team_id = $1
                AND closed_at IS NULL
                ORDER BY created_at DESC
                LIMIT 1",
        )
        .bind(self.team.id.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(offseason_competition_id)
    }

    async fn populate_offseason_info(&mut self, state: &AppState) -> Result<(), AppError> {
        tracing::debug!("populate_offseason_info for team_id={}", self.team.id);

        let raised_funds: Option<i32> = sqlx::query_scalar(
            "SELECT raised_funds
                FROM bb_redrafts_in_offseasons
                WHERE team_id = $1
                AND closed_at IS NULL
                ORDER BY created_at DESC
                LIMIT 1",
        )
        .bind(self.team.id.clone())
        .fetch_optional(&state.db)
        .await?;

        self.raised_funds = raised_funds.unwrap_or(0);

        Ok(())
    }

    async fn populate_players_redrafts(&mut self, state: &AppState) -> Result<(), AppError> {
        for (number, player) in self.team.players.iter() {
            let player_redraft = PlayerRedraft::select_from_team_player(
                state,
                self.team.id,
                player.clone(),
                number.clone(),
            )
            .await?;

            if player_redraft.redrafted {
                self.players_redrafted.push(player_redraft);
            } else {
                self.players_not_redrafted.push(player_redraft);
            }
        }

        Ok(())
    }

    pub fn staff_quantity_to_sign(&self, staff: &Staff) -> u8 {
        self.staff_to_sign
            .get(&staff)
            .and_then(|&quantity| Some(quantity))
            .unwrap_or(0)
    }

    async fn populate_staff_to_sign(&mut self, state: &AppState) -> Result<(), AppError> {
        tracing::debug!("populate_staff_to_sign for team_id={}", self.team.id);

        let staff_detail: Vec<StaffDetail> = sqlx::query_as(
            "SELECT staff,
                    number
            FROM bb_redrafting_staff
            WHERE team_id = $1",
        )
        .bind(self.team.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut staff_to_sign: HashMap<Staff, u8> = HashMap::new();

        for staff_detail in staff_detail {
            staff_to_sign.insert(staff_detail.staff, staff_detail.number as u8);
        }

        self.staff_to_sign = staff_to_sign;

        Ok(())
    }

    pub async fn insert_staff_to_sign(
        transaction: &mut sqlx::Transaction<'_, Postgres>,
        competition_id: i32,
        team_id: i32,
        staff: Staff,
        number: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_staff_to_sign with competition_id={}, team_id={}, staff={:?} and number={}",
            competition_id,
            team_id,
            staff,
            number,
        );

        sqlx::query(
            "INSERT INTO bb_redrafting_staff (
                    competition_id,
                    team_id,
                    staff,
                    number
                )
                VALUES ($1, $2, $3, $4)",
        )
        .bind(competition_id.clone())
        .bind(team_id.clone())
        .bind(staff.clone())
        .bind(number.clone())
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }

    pub fn positions_quantity_to_sign(&self) -> u8 {
        self.positions_to_sign.values().sum()
    }

    pub fn position_quantity_to_sign(&self, position: &Position) -> u8 {
        self.positions_to_sign
            .get(&position)
            .and_then(|&quantity| Some(quantity))
            .unwrap_or(0)
    }

    async fn populate_positions_to_sign(&mut self, state: &AppState) -> Result<(), AppError> {
        tracing::debug!("populate_positions_to_sign for team_id={}", self.team.id);

        #[derive(Deserialize, sqlx::FromRow, Clone)]
        struct PositionDetail {
            position: Position,
            number: i32,
        }

        let positions_detail: Vec<PositionDetail> = sqlx::query_as(
            "SELECT position,
                    number
            FROM bb_redrafting_positions
            WHERE team_id = $1",
        )
        .bind(self.team.id.clone())
        .fetch_all(&state.db)
        .await?;

        let mut positions_to_sign: HashMap<Position, u8> = HashMap::new();

        for position_detail in positions_detail {
            positions_to_sign.insert(position_detail.position, position_detail.number as u8);
        }

        self.positions_to_sign = positions_to_sign;

        Ok(())
    }

    pub async fn insert_position_to_sign(
        transaction: &mut sqlx::Transaction<'_, Postgres>,
        competition_id: i32,
        team_id: i32,
        position: Position,
        number: i32,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_position_to_sign with competition_id={}, team_id={}, position={:?} and number={}",
            competition_id,
            team_id,
            position,
            number,
        );

        sqlx::query(
            "INSERT INTO bb_redrafting_positions (
                    competition_id,
                    team_id,
                    position,
                    number
                )
                VALUES ($1, $2, $3, $4)",
        )
        .bind(competition_id.clone())
        .bind(team_id.clone())
        .bind(position.clone())
        .bind(number.clone())
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }

    pub async fn save(&self, state: &AppState, connected_user: &User) -> Result<(), AppError> {
        tracing::debug!("save team_id={}", self.team.id);

        if connected_user.is_coach(&self.team.coach) && self.team.in_offseason {
            let offseason_competition_id = self.select_offseason_competition_id(state).await?;

            if let Some(offseason_competition_id) = offseason_competition_id {
                let mut transaction = state.db.begin().await?;

                sqlx::query(
                    "DELETE FROM bb_redrafting_staff
                        WHERE competition_id = $1
                        AND team_id = $2",
                )
                .bind(offseason_competition_id.clone())
                .bind(self.team.id.clone())
                .execute(&mut *transaction)
                .await?;

                for (staff_to_sign, quantity) in &self.staff_to_sign {
                    Self::insert_staff_to_sign(
                        &mut transaction,
                        offseason_competition_id,
                        self.team.id,
                        staff_to_sign.clone(),
                        quantity.clone() as i32,
                    )
                    .await?;
                }

                sqlx::query(
                    "DELETE FROM bb_redrafting_positions
                        WHERE competition_id = $1
                        AND team_id = $2",
                )
                .bind(offseason_competition_id.clone())
                .bind(self.team.id.clone())
                .execute(transaction.as_mut())
                .await?;

                for (position_to_sign, quantity) in &self.positions_to_sign {
                    Self::insert_position_to_sign(
                        &mut transaction,
                        offseason_competition_id,
                        self.team.id,
                        position_to_sign.clone(),
                        quantity.clone() as i32,
                    )
                    .await?;
                }

                transaction.commit().await?;
            }
        }

        Ok(())
    }

    pub async fn close(self, state: &AppState, connected_user: &User) -> Result<(), AppError> {
        tracing::debug!("close team_id={}", self.team.id);

        if connected_user.is_coach(&self.team.coach)
            && self.team.in_offseason
            && self.resigned_team()?.check_if_rules_compliant().is_ok()
        {
            let offseason_competition_id = self.select_offseason_competition_id(state).await?;

            if let Some(offseason_competition_id) = offseason_competition_id {
                let mut transaction = state.db.begin().await?;

                let new_treasury = self.remaining_funds()?;

                teams::update_treasury(
                    &mut transaction,
                    connected_user,
                    self.team.id,
                    new_treasury,
                )
                .await?;

                for (staff, quantity) in self.staff_to_sign {
                    staff::update_staff_for_team(
                        &mut transaction,
                        connected_user,
                        self.team.id,
                        staff,
                        quantity,
                    )
                    .await?;
                }

                for (position, quantity) in self.positions_to_sign {
                    for _ in 0..quantity {
                        let player = Player::new(
                            self.team.version.clone(),
                            position.clone(),
                            self.team.roster.clone(),
                        );

                        players::insert_new_player_for_team(
                            &mut transaction,
                            connected_user,
                            self.team.id,
                            (0, player),
                        )
                        .await?;
                    }
                }

                for player_not_redrafted in self.players_not_redrafted {
                    players::update_player_contract_end_for_team(
                        &mut transaction,
                        connected_user,
                        self.team.id,
                        player_not_redrafted.player.id,
                    )
                    .await?;
                }

                let result = sqlx::query(
                    "UPDATE bb_redrafts_in_offseasons
                        SET closed_at = CURRENT_TIMESTAMP
                        WHERE competition_id = $1
                        AND team_id = $2
                        AND closed_at IS NULL",
                )
                .bind(offseason_competition_id.clone())
                .bind(self.team.id.clone())
                .execute(&mut *transaction)
                .await?;

                if result.rows_affected() > 0 {
                    transaction.commit().await?;
                } else {
                    transaction.rollback().await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct PlayerRedraft {
    pub team_id: i32,
    pub number: i32,
    pub player: Player,
    pub seasons_played: usize,
    pub seasons_played_with_experience: usize,
    pub redrafted: bool,
}

impl PlayerRedraft {
    pub fn redraft_cost(&self) -> Result<u32, AppError> {
        Ok(self.player.value()?
            + (PLAYER_ADDITIONAL_FEES_FOR_SEASON_PLAYED_WITH_EXPERIENCE
                * self.seasons_played_with_experience as u32))
    }

    pub async fn select_from_team_player(
        state: &AppState,
        team_id: i32,
        player: Player,
        number: i32,
    ) -> Result<Self, AppError> {
        let offseason_competition_id =
            Self::select_offseason_competition_id(state, &player).await?;

        let seasons_played = Self::select_seasons_played_by_player(state, &player).await?;

        let seasons_played_without_experience =
            Self::select_seasons_played_without_experience_by_player(
                state,
                &player,
                offseason_competition_id.is_some(),
            )
            .await?;

        let seasons_played_with_experience = seasons_played - seasons_played_without_experience;

        let redrafted = Self::select_if_player_redrafted(state, &player).await?;

        Ok(Self {
            team_id,
            number,
            player,
            seasons_played,
            seasons_played_with_experience,
            redrafted,
        })
    }

    async fn select_offseason_competition_id(
        state: &AppState,
        player: &Player,
    ) -> Result<Option<i32>, AppError> {
        tracing::debug!(
            "select_offseason_competition_id for player_id={}",
            player.id
        );

        let offseason_competition_id: Option<i32> = sqlx::query_scalar(
            "SELECT bb_redrafts_in_offseasons.competition_id
                FROM bb_redrafts_in_offseasons
                INNER JOIN bb_teams_players
                ON bb_redrafts_in_offseasons.team_id = bb_teams_players.team_id
                AND bb_teams_players.contract_start < bb_redrafts_in_offseasons.created_at
                AND (
                    bb_teams_players.contract_end IS NULL
                    OR bb_teams_players.contract_end > bb_redrafts_in_offseasons.created_at
                )
                WHERE bb_teams_players.player_id = $1
                AND bb_redrafts_in_offseasons.closed_at IS NULL
                LIMIT 1",
        )
        .bind(player.id.clone())
        .fetch_optional(&state.db)
        .await?;

        Ok(offseason_competition_id)
    }

    async fn select_seasons_played_by_player(
        state: &AppState,
        player: &Player,
    ) -> Result<usize, AppError> {
        tracing::debug!(
            "select_seasons_played_by_player for player_id={}",
            player.id
        );

        let season_number: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
            FROM bb_redrafts_in_offseasons
            INNER JOIN bb_teams_players
            ON bb_redrafts_in_offseasons.team_id = bb_teams_players.team_id
            AND bb_teams_players.contract_start < bb_redrafts_in_offseasons.created_at
            AND (
                bb_teams_players.contract_end IS NULL
                OR bb_teams_players.contract_end > bb_redrafts_in_offseasons.created_at
            )
            WHERE bb_teams_players.player_id = $1",
        )
        .bind(player.id.clone())
        .fetch_one(&state.db)
        .await?;

        Ok(season_number as usize)
    }

    async fn select_seasons_played_without_experience_by_player(
        state: &AppState,
        player: &Player,
        player_in_offseason: bool,
    ) -> Result<usize, AppError> {
        tracing::debug!(
            "select_seasons_played_with_experience_by_player for player_id={}",
            player.id
        );

        let mut seasons_played_without_experience: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
            FROM bb_redrafting_players
            INNER JOIN bb_redrafts_in_offseasons
            ON bb_redrafts_in_offseasons.competition_id = bb_redrafting_players.competition_id
            WHERE bb_redrafting_players.player_id = $1
            AND bb_redrafting_players.has_experience = FALSE
            AND bb_redrafts_in_offseasons.closed_at IS NOT NULL",
        )
        .bind(player.id.clone())
        .fetch_one(&state.db)
        .await?;

        if !player.has_experience() && player_in_offseason {
            seasons_played_without_experience += 1;
        }

        Ok(seasons_played_without_experience as usize)
    }

    async fn select_if_player_redrafted(
        state: &AppState,
        player: &Player,
    ) -> Result<bool, AppError> {
        tracing::debug!("select_if_player_redrafted for player_id={}", player.id);

        let redrafted_in_progress: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
            FROM bb_redrafting_players
            INNER JOIN bb_redrafts_in_offseasons
            ON bb_redrafts_in_offseasons.competition_id = bb_redrafting_players.competition_id
            WHERE bb_redrafting_players.player_id = $1
            AND bb_redrafts_in_offseasons.closed_at IS NULL",
        )
        .bind(player.id.clone())
        .fetch_one(&state.db)
        .await?;

        Ok(redrafted_in_progress > 0)
    }

    pub async fn save(&self, state: &AppState, connected_user: &User) -> Result<(), AppError> {
        tracing::debug!(
            "save player_id={} for team_id={}",
            self.player.id,
            self.team_id
        );

        let team_coach = coaches::select_from_team(state, self.team_id).await?;

        if connected_user.is_option_coach(&team_coach) {
            let offseason_competition_id =
                Self::select_offseason_competition_id(state, &self.player).await?;

            if let Some(offseason_competition_id) = offseason_competition_id {
                let mut transaction = state.db.begin().await?;

                sqlx::query(
                    "DELETE FROM bb_redrafting_players
                WHERE competition_id = $1
                AND team_id = $2
                AND player_id = $3",
                )
                .bind(offseason_competition_id.clone())
                .bind(self.team_id.clone())
                .bind(self.player.id.clone())
                .execute(&mut *transaction)
                .await?;

                if self.redrafted {
                    Self::insert_player_to_sign(
                        &mut transaction,
                        offseason_competition_id,
                        self.team_id,
                        self.player.id,
                        self.player.has_experience(),
                    )
                    .await?;
                }

                transaction.commit().await?;
            }
        }

        Ok(())
    }

    pub async fn insert_player_to_sign(
        transaction: &mut sqlx::Transaction<'_, Postgres>,
        competition_id: i32,
        team_id: i32,
        player_id: i32,
        has_experience: bool,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "insert_player_to_sign with competition_id={}, team_id={}, player_id={} and has_experience={}",
            competition_id,
            team_id,
            player_id,
            has_experience,
        );

        sqlx::query(
            "INSERT INTO bb_redrafting_players (
                    competition_id,
                    team_id,
                    player_id,
                    has_experience
                )
                VALUES ($1, $2, $3, $4)",
        )
        .bind(competition_id.clone())
        .bind(team_id.clone())
        .bind(player_id.clone())
        .bind(has_experience.clone())
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }
}
