use crate::data::blood_bowl::competitions::schedule::{GameSchedule, RoundSchedule, BYE};
use crate::data::blood_bowl::competitions::standings::{StageStandings, TeamStandings};
use crate::data::blood_bowl::competitions::Competition;
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

pub fn round_robin_schedule_and_standings(
    team_list: &Vec<Option<TeamSummary>>,
    home_and_away: bool,
) -> (Vec<RoundSchedule>, StageStandings) {
    let mut home_schedule = Vec::new();
    let mut away_schedule = Vec::new();
    let mut stage_standings = StageStandings::from(team_list);

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
            let mut home_round_games = Vec::new();
            let mut away_round_games = Vec::new();
            let mut home_round_is_finished = true;
            let mut away_round_is_finished = true;

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

                    if let Some(home_game_summary) = &home_game_summary {
                        stage_standings.process_game_results(&home_game_summary);

                        home_round_is_finished =
                            home_round_is_finished && home_game_summary.finished;
                    } else {
                        home_round_is_finished = false;
                    }

                    let home_game = GameSchedule {
                        home_team: home_team.clone(),
                        home_ranking_number: None,
                        away_team: away_team.clone(),
                        away_ranking_number: None,
                        game_summary: home_game_summary,
                    };

                    home_round_games.push(home_game);

                    if home_and_away {
                        let away_game_summary = None;

                        if let Some(away_game_summary) = &away_game_summary {
                            stage_standings.process_game_results(&away_game_summary);

                            away_round_is_finished =
                                away_round_is_finished && away_game_summary.finished;
                        } else {
                            away_round_is_finished = false;
                        }

                        let away_game = GameSchedule {
                            home_team: away_team.clone(),
                            home_ranking_number: None,
                            away_team: home_team.clone(),
                            away_ranking_number: None,
                            game_summary: away_game_summary,
                        };

                        away_round_games.push(away_game);
                    }
                }
            }

            home_schedule.push(RoundSchedule {
                name: format!("Journée {}", round + 1),
                games: home_round_games,
                finished: home_round_is_finished,
            });

            if home_and_away {
                away_schedule.push(RoundSchedule {
                    name: format!("Journée {}", round + rounds_number + 1),
                    games: away_round_games,
                    finished: away_round_is_finished,
                });
            }

            // Rotate teams (except fixed)
            let last = rotating.pop().unwrap();
            rotating.insert(0, last);
        }

        if home_and_away {
            home_schedule.extend(away_schedule);
        }
    }

    (home_schedule, stage_standings)
}

pub fn cup_schedule_and_standings(
    team_list: &Vec<Option<TeamSummary>>,
    with_ranking: bool,
) -> (Vec<RoundSchedule>, StageStandings) {
    let mut schedule = Vec::new();
    let mut stage_standings = StageStandings::from(team_list);

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

                let mut round_games = Vec::with_capacity(teams_number_competing_for_position / 2);
                let mut round_is_finished = true;

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

                    let winner = game.winner();
                    round_is_finished = round_is_finished && winner.is_some();
                    winners.push(winner);

                    if with_ranking || teams_number_competing_for_position > 4 {
                        losers.push(game.loser());
                    }

                    if game.home_team.ne(&Some(BYE.clone()))
                        && game.away_team.ne(&Some(BYE.clone()))
                    {
                        round_games.push(game);
                    }

                    first_team_index += 1;
                    second_team_index -= 1;
                }

                let round_is_finished = !winners.contains(&None);
                next_round_teams.push(winners);

                if with_ranking || teams_number_competing_for_position > 4 {
                    next_round_teams.push(losers);
                }

                if round_games.len() > 0 {
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

                    schedule.push(RoundSchedule {
                        name: round_name,
                        games: round_games,
                        finished: round_is_finished,
                    });
                }
            }

            teams = next_round_teams;
        }
    }

    (schedule, stage_standings)
}
