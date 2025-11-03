use crate::data::blood_bowl::competitions::schedule::{
    GameSchedule, RoundSchedule, StageSchedule, BYE,
};
use crate::data::blood_bowl::competitions::standings::StageStandings;
use crate::data::blood_bowl::competitions::{stages, Competition};
use crate::data::blood_bowl::teams::TeamSummary;
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
    pub fn schedule_and_standings(
        &self,
        team_list: &Vec<Option<TeamSummary>>,
    ) -> (StageSchedule, StageStandings) {
        match self.stage_type {
            CompetitionStageType::Championship => {
                stages::round_robin_schedule_and_standings(team_list, self)
            }
            CompetitionStageType::Cup => stages::cup_schedule_and_standings(team_list, self),
        }
    }

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

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum CompetitionStageRule {
    HomeAndAway,
    WithRanking,
}

pub fn round_robin_schedule_and_standings(
    team_list: &Vec<Option<TeamSummary>>,
    stage: &CompetitionStage,
) -> (StageSchedule, StageStandings) {
    let mut home_schedule = StageSchedule::from(stage);
    let mut away_schedule = StageSchedule::from(stage);
    let mut home_standings = StageStandings::from_stage_with_teams(stage, team_list);
    let mut away_standings = StageStandings::from_stage_with_teams(stage, team_list);

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

            let mut home_round_standings = home_standings.new_round_standings();
            let mut away_round_standings = away_standings.new_round_standings();

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
                    let home_game_summary = None;

                    let home_game = GameSchedule {
                        home_team: home_team.clone(),
                        home_ranking_number: None,
                        away_team: away_team.clone(),
                        away_ranking_number: None,
                        game_summary: home_game_summary,
                    };

                    home_round_standings.process_game(&home_game);
                    home_round_schedule.push(home_game);

                    if home_and_away {
                        let away_game_summary = None;

                        let away_game = GameSchedule {
                            home_team: away_team.clone(),
                            home_ranking_number: None,
                            away_team: home_team.clone(),
                            away_ranking_number: None,
                            game_summary: away_game_summary,
                        };

                        away_round_standings.process_game(&away_game);
                        away_round_schedule.push(away_game);
                    }
                }
            }

            home_schedule.push(home_round_schedule);
            home_standings.process_round(home_round_standings);

            if home_and_away {
                away_schedule.push(away_round_schedule);
                away_standings.process_round(away_round_standings);
            }

            // Rotate teams (except fixed)
            let last = rotating.pop().unwrap();
            rotating.insert(0, last);
        }

        if home_and_away {
            home_schedule.extend(away_schedule);
            home_standings.extend(away_standings);
        }
    }

    (home_schedule, home_standings)
}

pub fn cup_schedule_and_standings(
    team_list: &Vec<Option<TeamSummary>>,
    stage: &CompetitionStage,
) -> (StageSchedule, StageStandings) {
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

        let mut teams = vec![teams];

        while teams[0].len() > 1 {
            let mut next_round_teams = Vec::with_capacity(teams.len() * 2);

            for cup_part_index in 0..teams.len() {
                let teams_number_competing_for_position = teams[cup_part_index].len();
                let position_teams_are_competing_for =
                    (cup_part_index * teams_number_competing_for_position) + 1;

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

                let mut winners = Vec::with_capacity(teams_number_competing_for_position / 2);
                let mut losers = Vec::with_capacity(teams_number_competing_for_position / 2);

                while first_team_index < second_team_index {
                    let game = GameSchedule {
                        home_team: teams[cup_part_index][first_team_index].clone(),
                        home_ranking_number: Some(
                            first_team_index + position_teams_are_competing_for,
                        ),
                        away_team: teams[cup_part_index][second_team_index].clone(),
                        away_ranking_number: Some(
                            second_team_index + position_teams_are_competing_for,
                        ),
                        game_summary: None,
                    };

                    winners.push(game.winner());

                    if with_ranking || teams_number_competing_for_position > 4 {
                        losers.push(game.loser());
                    }

                    round_schedule.push(game);

                    first_team_index += 1;
                    second_team_index -= 1;
                }

                next_round_teams.push(winners);

                if with_ranking || teams_number_competing_for_position > 4 {
                    next_round_teams.push(losers);
                }

                stage_schedule.push(round_schedule);
            }

            teams = next_round_teams;
        }
    }

    (stage_schedule, stage_standings)
}
