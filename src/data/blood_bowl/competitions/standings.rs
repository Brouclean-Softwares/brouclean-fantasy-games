use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct StageStandings {
    pub teams_standings: Vec<Option<TeamStandings>>,
}

impl Into<Vec<Option<TeamSummary>>> for StageStandings {
    fn into(self) -> Vec<Option<TeamSummary>> {
        let mut teams = Vec::new();

        for team_standing in self.teams_standings.iter() {
            if let Some(team_standing) = team_standing {
                teams.push(Some(team_standing.team.clone()));
            } else {
                teams.push(None);
            }
        }

        teams
    }
}

impl From<&Vec<Option<TeamSummary>>> for StageStandings {
    fn from(teams: &Vec<Option<TeamSummary>>) -> Self {
        let mut teams_standings: Vec<Option<TeamStandings>> = Vec::with_capacity(teams.len());

        for team in teams {
            teams_standings.push(TeamStandings::new_from_optional_team_summary(team));
        }

        Self { teams_standings }
    }
}

impl StageStandings {
    pub fn process_game_results(&mut self, game_summary: &GameSummary) {
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

            self.teams_standings.sort();
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
