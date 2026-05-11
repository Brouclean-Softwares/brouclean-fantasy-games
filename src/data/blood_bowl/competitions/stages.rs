use crate::AppState;
use crate::data::blood_bowl::competitions::schedule::{
    BYE, GameSchedule, RoundSchedule, StageSchedule,
};
use crate::data::blood_bowl::competitions::standings::{
    CompetingForPositionStandings, StageStandings,
};
use crate::data::blood_bowl::competitions::{Competition, stages};
use crate::data::blood_bowl::games;
use crate::data::blood_bowl::teams::TeamSummary;
use crate::data::users::User;
use crate::errors::AppError;
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

    pub async fn select_by_id(
        state: &AppState,
        id: i32,
    ) -> Result<Option<CompetitionStage>, AppError> {
        tracing::debug!("select_by_id for id={}", id);

        let row: Option<CompetitionStageRow> = sqlx::query_as(
            "SELECT id,
                    stage_name,
                    stage_type,
                    rules
            FROM bb_competitions_stages
            WHERE id = $1
            LIMIT 1",
        )
        .bind(id.clone())
        .fetch_optional(&state.db)
        .await?;

        if let Some(row) = row {
            Ok(Some(row.into_competition_stage().await?))
        } else {
            Ok(None)
        }
    }

    pub async fn select_for_game_id(
        state: &AppState,
        game_id: i32,
    ) -> Result<Option<CompetitionStage>, AppError> {
        tracing::debug!("select_for_game_id for game_id={}", game_id);

        let row: Option<CompetitionStageRow> = sqlx::query_as(
            "SELECT bb_competitions_stages.id,
                    bb_competitions_stages.stage_name,
                    bb_competitions_stages.stage_type,
                    bb_competitions_stages.rules
            FROM bb_competitions_stages
            INNER JOIN bb_competitions_stages_schedule
            ON bb_competitions_stages_schedule.stage_id = bb_competitions_stages.id
            WHERE bb_competitions_stages_schedule.game_id = $1
            LIMIT 1",
        )
        .bind(game_id.clone())
        .fetch_optional(&state.db)
        .await?;

        if let Some(row) = row {
            Ok(Some(row.into_competition_stage().await?))
        } else {
            Ok(None)
        }
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

    pub async fn update_for_competition(
        &self,
        state: &AppState,
        connected_user: &User,
        competition: &mut Competition,
    ) -> Result<(), AppError> {
        tracing::debug!(
            "update for competition_id={} and stage_id={}",
            competition.id,
            self.id,
        );

        if connected_user.eq(&competition.director) && !competition.closed {
            sqlx::query(
                "UPDATE bb_competitions_stages 
                    SET rules = $3
                    WHERE id = $1
                    AND competition_id = $2",
            )
            .bind(self.id.clone())
            .bind(competition.id.clone())
            .bind(serde_json::to_string(&self.rules)?.clone())
            .execute(&state.db)
            .await?;
        }

        competition.save(state, connected_user).await?;

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

    pub async fn schedule_and_standings(
        &self,
        state: &AppState,
        team_list: &Vec<Option<TeamSummary>>,
    ) -> Result<(StageSchedule, StageStandings), AppError> {
        match self.stage_type {
            CompetitionStageType::Championship => {
                stages::round_robin_schedule_and_standings(state, team_list, self).await
            }
            CompetitionStageType::Cup => {
                stages::cup_schedule_and_standings(state, team_list, self).await
            }
        }
    }

    pub fn available_rules(&self) -> Vec<CompetitionStageRule> {
        self.stage_type
            .available_rules()
            .iter()
            .filter(|&rule| !self.rules.contains(&rule))
            .map(|rule| rule.clone())
            .collect()
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

    pub fn available_rules(&self) -> Vec<CompetitionStageRule> {
        match self {
            CompetitionStageType::Championship => vec![CompetitionStageRule::HomeAndAway],
            CompetitionStageType::Cup => vec![CompetitionStageRule::WithRanking],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum CompetitionStageRule {
    HomeAndAway,
    WithRanking,
}

impl Display for CompetitionStageRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            CompetitionStageRule::HomeAndAway => "Matchs aller-retour",
            CompetitionStageRule::WithRanking => "Avec les matchs de classement",
        };

        write!(f, "{}", text)
    }
}

pub async fn round_robin_schedule_and_standings(
    state: &AppState,
    team_list: &Vec<Option<TeamSummary>>,
    stage: &CompetitionStage,
) -> Result<(StageSchedule, StageStandings), AppError> {
    let mut existing_games = games::select_all_for_competition_stage(state, stage.id).await?;

    let mut home_schedule = StageSchedule::from(stage);
    let mut away_schedule = StageSchedule::from(stage);
    let mut standings = StageStandings::from_stage_with_teams(stage, team_list);

    let home_and_away = stage.rules.contains(&CompetitionStageRule::HomeAndAway);

    if team_list.len() >= 2 {
        let mut teams = team_list.clone();

        if teams.len() % 2 != 0 {
            teams.push(Some(BYE.clone()));
        }

        let rounds_number = teams.len() - 1;
        let teams_half_number = teams.len() / 2;

        let fixed = teams[0].clone();
        let mut rotating = teams[1..].to_vec();

        for round in 0..rounds_number {
            let mut home_round_schedule =
                RoundSchedule::new_with_name(format!("Journée {}", round + 1));
            let mut away_round_schedule =
                RoundSchedule::new_with_name(format!("Journée {}", round + rounds_number + 1));

            for i in 0..teams_half_number {
                let home_team: Option<TeamSummary>;
                let away_team: Option<TeamSummary>;

                if i == 0 {
                    home_team = fixed.clone();
                    away_team = rotating[0].clone();
                } else {
                    home_team = rotating[i].clone();
                    away_team = rotating[rotating.len() - i].clone();
                }

                if home_team.ne(&Some(BYE.clone())) && away_team.ne(&Some(BYE.clone())) {
                    let mut home_game = GameSchedule {
                        home_team: home_team.clone(),
                        home_ranking_number: None,
                        away_team: away_team.clone(),
                        away_ranking_number: None,
                        game_summary: None,
                    };

                    home_game.pick_game_summary_from_list(&mut existing_games);

                    standings.process_game_for_position(&home_game, 1);
                    home_round_schedule.push(home_game);

                    if home_and_away {
                        let mut away_game = GameSchedule {
                            home_team: away_team.clone(),
                            home_ranking_number: None,
                            away_team: home_team.clone(),
                            away_ranking_number: None,
                            game_summary: None,
                        };

                        away_game.pick_game_summary_from_list(&mut existing_games);

                        standings.process_game_for_position(&away_game, 1);
                        away_round_schedule.push(away_game);
                    }
                }
            }

            home_schedule.push(home_round_schedule);

            if home_and_away {
                away_schedule.push(away_round_schedule);
            }

            // Rotate teams (except fixed)
            let last = rotating.pop().unwrap();
            rotating.insert(0, last);
        }

        if home_and_away {
            home_schedule.extend(away_schedule);
        }
    }

    Ok((home_schedule, standings))
}

pub async fn cup_schedule_and_standings(
    state: &AppState,
    team_list: &Vec<Option<TeamSummary>>,
    stage: &CompetitionStage,
) -> Result<(StageSchedule, StageStandings), AppError> {
    let mut existing_games = games::select_all_for_competition_stage(state, stage.id).await?;

    let mut stage_schedule = StageSchedule::from(stage);
    let mut stage_standings = StageStandings::from_stage_with_teams(stage, team_list);

    let with_ranking = stage.rules.contains(&CompetitionStageRule::WithRanking);

    if team_list.len() >= 2 {
        let mut teams = team_list.clone();

        let mut cup_teams_number = 2;

        while cup_teams_number < teams.len() {
            cup_teams_number *= 2;
        }

        while teams.len() < cup_teams_number {
            teams.push(Some(BYE.clone()));
        }

        stage_standings = StageStandings::from_stage_with_teams(stage, &teams);

        while stage_standings.teams_number_competing_for_first_position() > 1 {
            for position_standings in stage_standings.clone().positions_standings() {
                let position_teams_are_competing_for =
                    position_standings.position_teams_are_competing_for;

                if with_ranking || position_teams_are_competing_for <= 3 {
                    let teams_number_competing_for_position = position_standings.teams_len();

                    let round_name = match (
                        position_teams_are_competing_for,
                        teams_number_competing_for_position,
                    ) {
                        (1, 2) => "Finale 🏆".to_string(),
                        (1, 4) => "1/2 finale".to_string(),
                        (1, 8) => "1/4 de finale".to_string(),
                        (1, 16) => "1/8 de finale".to_string(),
                        (1, 32) => "1/16 de finale".to_string(),
                        (1, 64) => "1/32 de finale".to_string(),
                        (1, 128) => "1/64 de finale".to_string(),
                        (1, _) => "Tableau principal".to_string(),
                        (3, 2) => "Match pour la 3ème place 🥉".to_string(),
                        (number_for_part, 2) => {
                            format!("Match pour la {}ème place", number_for_part)
                        }
                        (number_for_part, _) => {
                            format!("Tableau pour la {}ème place", number_for_part)
                        }
                    };

                    let mut round_schedule = RoundSchedule::new_with_name_and_capacity(
                        round_name,
                        teams_number_competing_for_position / 2,
                    );

                    let mut first_team_index = 0;
                    let mut second_team_index = teams_number_competing_for_position - 1;

                    let mut winners = CompetingForPositionStandings::with_capacity_for_position(
                        teams_number_competing_for_position / 2,
                        position_teams_are_competing_for,
                    );
                    let mut losers = CompetingForPositionStandings::with_capacity_for_position(
                        teams_number_competing_for_position / 2,
                        position_teams_are_competing_for + teams_number_competing_for_position / 2,
                    );

                    while first_team_index < second_team_index {
                        let mut game = GameSchedule {
                            home_team: position_standings.team_at_index(first_team_index),
                            home_ranking_number: Some(
                                first_team_index + position_teams_are_competing_for,
                            ),
                            away_team: position_standings.team_at_index(second_team_index),
                            away_ranking_number: Some(
                                second_team_index + position_teams_are_competing_for,
                            ),
                            game_summary: None,
                        };

                        game.pick_game_summary_from_list(&mut existing_games);

                        winners.push_team(game.winner());
                        losers.push_team(game.loser());

                        round_schedule.push(game);

                        first_team_index += 1;
                        second_team_index -= 1;
                    }

                    stage_standings.process_position_standings(winners);
                    stage_standings.process_position_standings(losers);

                    stage_schedule.push(round_schedule);
                }
            }
        }
    }

    Ok((stage_schedule, stage_standings))
}
