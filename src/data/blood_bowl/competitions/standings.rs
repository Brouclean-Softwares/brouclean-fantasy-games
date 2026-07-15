use crate::data::blood_bowl::competitions::schedule::{BYE, GameSchedule};
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use std::cmp::Ordering;
use std::ops::Index;

#[derive(Clone)]
pub struct CompetitionStandings {
    pub stages_standings: Vec<StageStandings>,
}

impl From<Vec<StageStandings>> for CompetitionStandings {
    fn from(stages_standings: Vec<StageStandings>) -> Self {
        Self { stages_standings }
    }
}

impl CompetitionStandings {
    pub fn standings(&self) -> Vec<CompetingForPositionStandings> {
        let last_stage_standings = self.stages_standings.last();

        if let Some(last_stage_standings) = last_stage_standings {
            last_stage_standings.standings()
        } else {
            Vec::new()
        }
    }

    pub fn teams_final_standings(&self) -> Vec<(usize, Option<TeamStandings>)> {
        let last_stage_standings = self.stages_standings.last();

        if let Some(last_stage_standings) = last_stage_standings {
            last_stage_standings.teams_standings()
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone)]
pub struct StageStandings {
    pub stage: CompetitionStage,
    positions_standings: Vec<CompetingForPositionStandings>,
}

impl From<&StageStandings> for StageStandings {
    fn from(other: &StageStandings) -> Self {
        Self {
            stage: other.stage.clone(),
            positions_standings: Vec::new(),
        }
    }
}

impl Into<Vec<Option<TeamSummary>>> for StageStandings {
    fn into(self) -> Vec<Option<TeamSummary>> {
        let mut teams = Vec::new();

        for position_standings in self.clone().standings() {
            for team_standing in position_standings.clone().standings() {
                if let Some(team_standing) = team_standing {
                    teams.push(Some(team_standing.team.clone()));
                } else {
                    teams.push(None);
                }
            }
        }

        teams
    }
}

impl StageStandings {
    pub fn from_stage_with_teams(
        stage: &CompetitionStage,
        teams: &Vec<Option<TeamSummary>>,
    ) -> Self {
        let standings_for_first_position =
            CompetingForPositionStandings::from_teams_for_position(teams, 1);
        Self {
            stage: stage.clone(),
            positions_standings: vec![standings_for_first_position],
        }
    }

    pub fn positions_standings(self) -> Vec<CompetingForPositionStandings> {
        self.positions_standings
    }

    pub fn process_position_standings(
        &mut self,
        position_standings: CompetingForPositionStandings,
    ) {
        if let Some(position) =
            self.positions_standings
                .iter()
                .position(|current_round_standings| {
                    position_standings
                        .position_teams_are_competing_for
                        .eq(&current_round_standings.position_teams_are_competing_for)
                })
        {
            self.positions_standings[position] = position_standings;
        } else {
            self.positions_standings.push(position_standings);
        }

        self.positions_standings.sort_by(|a, b| {
            b.position_teams_are_competing_for
                .cmp(&a.position_teams_are_competing_for)
        })
    }

    pub fn process_game_for_position(
        &mut self,
        game_schedule: &GameSchedule,
        position_teams_are_competing_for: usize,
    ) {
        if let Some(position) = self.positions_standings.iter().position(|round_standings| {
            round_standings.position_teams_are_competing_for == position_teams_are_competing_for
        }) {
            self.positions_standings[position].process_game(game_schedule);
        }
    }

    pub fn teams_number_competing_for_first_position(&self) -> usize {
        if let Some(position) = self
            .positions_standings
            .iter()
            .position(|round_standings| round_standings.position_teams_are_competing_for.eq(&1))
        {
            self.positions_standings[position].teams_len()
        } else {
            0
        }
    }

    pub fn standings(&self) -> Vec<CompetingForPositionStandings> {
        let mut standings = self.positions_standings.clone();

        standings.sort_by(|a, b| {
            a.position_teams_are_competing_for
                .cmp(&b.position_teams_are_competing_for)
        });

        standings
    }

    pub fn teams_standings(&self) -> Vec<(usize, Option<TeamStandings>)> {
        let mut teams_ranking = Vec::new();

        for competing_for_position_standings in self.standings() {
            for (team_index, team_standings) in competing_for_position_standings
                .standings()
                .iter()
                .enumerate()
            {
                let position = match self.stage.stage_type {
                    CompetitionStageType::Championship => {
                        competing_for_position_standings.position_teams_are_competing_for
                            + team_index
                    }

                    CompetitionStageType::Cup => {
                        competing_for_position_standings.position_teams_are_competing_for
                    }
                };

                teams_ranking.push((position, team_standings.clone()));
            }
        }

        teams_ranking
    }
}

#[derive(Clone)]
pub struct CompetingForPositionStandings {
    teams_standings: Vec<Option<TeamStandings>>,
    pub position_teams_are_competing_for: usize,
}

impl CompetingForPositionStandings {
    pub fn teams_len(&self) -> usize {
        self.teams_standings.len()
    }

    pub fn team_at_index(&self, index: usize) -> Option<TeamSummary> {
        self.teams_standings
            .index(index)
            .clone()
            .and_then(|team_standings| Some(team_standings.team))
    }

    pub fn from_teams_for_position(
        teams: &Vec<Option<TeamSummary>>,
        position_teams_are_competing_for: usize,
    ) -> Self {
        let mut standings =
            Self::with_capacity_for_position(teams.len(), position_teams_are_competing_for);

        for team in teams {
            standings.push_team(team.clone());
        }

        standings
    }

    pub fn with_capacity_for_position(
        capacity: usize,
        position_teams_are_competing_for: usize,
    ) -> Self {
        Self {
            teams_standings: Vec::with_capacity(capacity),
            position_teams_are_competing_for,
        }
    }

    pub fn push_team(&mut self, team: Option<TeamSummary>) {
        self.teams_standings
            .push(TeamStandings::new_from_optional_team_summary(team))
    }

    pub fn standings(&self) -> Vec<Option<TeamStandings>> {
        let mut standings: Vec<Option<TeamStandings>> = self
            .teams_standings
            .iter()
            .filter(|&team_standing| {
                if let Some(team_standing) = team_standing {
                    team_standing.team.ne(&BYE)
                } else {
                    true
                }
            })
            .map(|team_standing| team_standing.clone())
            .collect();

        standings.sort_by(|a, b| match (a, b) {
            (Some(a), Some(b)) => {
                if a.points.ne(&b.points) {
                    b.points.cmp(&a.points)
                } else if a.victories.ne(&b.victories) {
                    b.victories.cmp(&a.victories)
                } else if a.touchdowns.ne(&b.touchdowns) {
                    b.touchdowns.cmp(&a.touchdowns)
                } else {
                    b.casualties.cmp(&a.casualties)
                }
            }

            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });

        standings
    }

    fn add_teams_standings(&mut self, teams_standings_to_add: Vec<Option<TeamStandings>>) {
        for standings_to_add in teams_standings_to_add {
            if let Some(standings_to_add) = standings_to_add {
                let team_position = self.teams_standings.iter().position(|team_standings| {
                    if let Some(team_standings) = team_standings {
                        team_standings.team.eq(&standings_to_add.team)
                    } else {
                        false
                    }
                });

                if let Some(team_position) = team_position {
                    if let Some(team_standings) = &mut self.teams_standings[team_position] {
                        team_standings.add(standings_to_add);
                    } else {
                        self.teams_standings.push(Some(standings_to_add));
                    }
                } else {
                    self.teams_standings.push(Some(standings_to_add));
                }
            }
        }
    }

    pub fn process_game(&mut self, game_schedule: &GameSchedule) {
        if let Some(game_summary) = &game_schedule.game_summary {
            self.add_teams_standings(TeamStandings::from_game_summary(game_summary));
        }
    }
}

#[derive(Clone)]
pub struct TeamStandings {
    pub team: TeamSummary,
    pub points: usize,
    pub victories: usize,
    pub draws: usize,
    pub losses: usize,
    pub touchdowns: usize,
    pub casualties: usize,
}

impl TeamStandings {
    fn new_from_optional_team_summary(optional_team_summary: Option<TeamSummary>) -> Option<Self> {
        if let Some(team_summary) = optional_team_summary {
            Some(Self {
                team: team_summary,
                points: 0,
                victories: 0,
                draws: 0,
                losses: 0,
                touchdowns: 0,
                casualties: 0,
            })
        } else {
            None
        }
    }

    pub fn games_played(&self) -> usize {
        self.victories + self.draws + self.losses
    }

    fn from_game_summary(game_summary: &GameSummary) -> Vec<Option<Self>> {
        if game_summary.finished {
            let first_team_touchdowns = game_summary.first_team_score as usize;
            let second_team_touchdowns = game_summary.second_team_score as usize;
            let first_team_casualties = game_summary.first_team_casualties as usize;
            let second_team_casualties = game_summary.second_team_casualties as usize;

            let (victories, draws, losses) = match (
                game_summary.first_team_is_winner,
                game_summary.second_team_is_winner,
            ) {
                (true, false) => (1, 0, 0),
                (false, true) => (0, 0, 1),
                (_, _) => (0, 1, 0),
            };

            let points = (victories * 3)
                + draws
                + if first_team_touchdowns >= 3 { 1 } else { 0 }
                + if second_team_touchdowns == 0 { 1 } else { 0 }
                + if first_team_casualties >= 3 { 1 } else { 0 };

            let first_team_standings = Self {
                team: game_summary.first_team.clone(),
                points,
                victories,
                draws,
                losses,
                touchdowns: first_team_touchdowns,
                casualties: first_team_casualties,
            };

            let (victories, draws, losses) = match (
                game_summary.first_team_is_winner,
                game_summary.second_team_is_winner,
            ) {
                (true, false) => (0, 0, 1),
                (false, true) => (1, 0, 0),
                (_, _) => (0, 1, 0),
            };

            let points = (victories * 3)
                + draws
                + if second_team_touchdowns >= 3 { 1 } else { 0 }
                + if first_team_touchdowns == 0 { 1 } else { 0 }
                + if second_team_casualties >= 3 { 1 } else { 0 };

            let second_team_standings = Self {
                team: game_summary.second_team.clone(),
                points,
                victories,
                draws,
                losses,
                touchdowns: second_team_touchdowns,
                casualties: second_team_casualties,
            };

            vec![Some(first_team_standings), Some(second_team_standings)]
        } else {
            Vec::new()
        }
    }

    fn add(&mut self, other: TeamStandings) {
        if self.team.eq(&other.team) {
            self.points += other.points;
            self.victories += other.victories;
            self.draws += other.draws;
            self.losses += other.losses;
            self.touchdowns += other.touchdowns;
            self.casualties += other.casualties;
        }
    }
}
