use crate::data::blood_bowl::competitions::schedule::GameSchedule;
use crate::data::blood_bowl::competitions::stages::{CompetitionStage, CompetitionStageType};
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct StageStandings {
    pub stage: CompetitionStage,
    teams: Vec<Option<TeamSummary>>,
    rounds_standings: Vec<RoundStandings>,
}

impl From<&StageStandings> for StageStandings {
    fn from(other: &StageStandings) -> Self {
        Self {
            stage: other.stage.clone(),
            teams: other.teams.clone(),
            rounds_standings: Vec::new(),
        }
    }
}

impl Into<Vec<Option<TeamSummary>>> for StageStandings {
    fn into(self) -> Vec<Option<TeamSummary>> {
        let mut teams = Vec::new();

        for team_standing in self.standings().iter() {
            if let Some(team_standing) = team_standing {
                teams.push(Some(team_standing.team.clone()));
            } else {
                teams.push(None);
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
        Self {
            stage: stage.clone(),
            teams: teams.clone(),
            rounds_standings: Vec::new(),
        }
    }

    pub fn new_round_standings(&self) -> RoundStandings {
        RoundStandings::from(&self.teams)
    }

    pub fn process_round(&mut self, round_standings: RoundStandings) {
        match self.stage.stage_type {
            CompetitionStageType::Championship => {
                self.rounds_standings = vec![round_standings];
            }
            CompetitionStageType::Cup => {}
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.rounds_standings.extend(other.rounds_standings);
    }

    pub fn standings(&self) -> Vec<Option<TeamStandings>> {
        match self.stage.stage_type {
            CompetitionStageType::Championship => {
                if let Some(round_standings) = self.rounds_standings.last() {
                    round_standings.teams_standings.clone()
                } else {
                    self.new_round_standings().teams_standings
                }
            }
            CompetitionStageType::Cup => self.new_round_standings().teams_standings,
        }
    }
}

#[derive(Clone)]
pub struct RoundStandings {
    teams_standings: Vec<Option<TeamStandings>>,
}

impl From<&Vec<Option<TeamSummary>>> for RoundStandings {
    fn from(teams: &Vec<Option<TeamSummary>>) -> Self {
        let mut teams_standings: Vec<Option<TeamStandings>> = Vec::with_capacity(teams.len());

        for team in teams {
            teams_standings.push(TeamStandings::new_from_optional_team_summary(team));
        }

        Self { teams_standings }
    }
}

impl RoundStandings {
    pub fn standings(&mut self) -> Vec<Option<TeamStandings>> {
        self.teams_standings.sort();
        self.teams_standings.clone()
    }

    pub fn process_game(&mut self, game_schedule: &GameSchedule) {
        if let Some(game_summary) = &game_schedule.game_summary {
            if let Some((first_team_results, second_team_results)) =
                TeamStandings::from_game_summary(game_summary)
            {
                for team_standings in self.teams_standings.iter_mut() {
                    if let Some(team_standings) = team_standings {
                        if team_standings.team.eq(&first_team_results.team) {
                            team_standings.add(first_team_results.clone());
                        }

                        if team_standings.team.eq(&second_team_results.team) {
                            team_standings.add(second_team_results.clone());
                        }
                    }
                }
            }
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

impl Eq for TeamStandings {}

impl PartialEq<Self> for TeamStandings {
    fn eq(&self, other: &Self) -> bool {
        self.team.eq(&other.team)
    }
}

impl PartialOrd<Self> for TeamStandings {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.points.ne(&other.points) {
            self.points.partial_cmp(&other.points)
        } else if self.victories.ne(&other.victories) {
            self.victories.partial_cmp(&other.victories)
        } else if self.touchdowns.ne(&other.touchdowns) {
            self.touchdowns.partial_cmp(&other.touchdowns)
        } else {
            self.casualties.partial_cmp(&other.casualties)
        }
    }
}

impl Ord for TeamStandings {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.points.ne(&other.points) {
            self.points.cmp(&other.points)
        } else if self.victories.ne(&other.victories) {
            self.victories.cmp(&other.victories)
        } else if self.touchdowns.ne(&other.touchdowns) {
            self.touchdowns.cmp(&other.touchdowns)
        } else {
            self.casualties.cmp(&other.casualties)
        }
    }
}

impl TeamStandings {
    fn new_from_optional_team_summary(optional_team_summary: &Option<TeamSummary>) -> Option<Self> {
        if let Some(team_summary) = optional_team_summary {
            Some(Self {
                team: team_summary.clone(),
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

    fn from_game_summary(game_summary: &GameSummary) -> Option<(Self, Self)> {
        if game_summary.finished {
            let first_team_standings = Self {
                team: game_summary.first_team.clone(),
                points: 0,
                victories: if game_summary.first_team_is_winner {
                    1
                } else {
                    0
                },
                draws: if !game_summary.first_team_is_winner && !game_summary.second_team_is_winner
                {
                    1
                } else {
                    0
                },
                losses: if game_summary.second_team_is_winner {
                    1
                } else {
                    0
                },
                touchdowns: game_summary.first_team_score as usize,
                casualties: game_summary.first_team_casualties as usize,
            };

            let second_team_standings = Self {
                team: game_summary.second_team.clone(),
                points: 0,
                victories: if game_summary.second_team_is_winner {
                    1
                } else {
                    0
                },
                draws: if !game_summary.first_team_is_winner && !game_summary.second_team_is_winner
                {
                    1
                } else {
                    0
                },
                losses: if game_summary.first_team_is_winner {
                    1
                } else {
                    0
                },
                touchdowns: game_summary.second_team_score as usize,
                casualties: game_summary.second_team_casualties as usize,
            };

            Some((first_team_standings, second_team_standings))
        } else {
            None
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
