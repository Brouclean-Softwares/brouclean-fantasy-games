use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;

pub struct RoundSchedule {
    pub name: String,
    pub games: Vec<GameSchedule>,
}

pub struct GameSchedule {
    pub home_team: Option<TeamSummary>,
    pub home_ranking_number: Option<usize>,
    pub away_team: Option<TeamSummary>,
    pub away_ranking_number: Option<usize>,
    pub game_summary: Option<GameSummary>,
}

impl GameSchedule {
    pub fn score(&self) -> Option<(usize, usize)> {
        if let Some(game_summary) = &self.game_summary {
            Some((
                game_summary.first_team_score as usize,
                game_summary.second_team_score as usize,
            ))
        } else {
            None
        }
    }

    pub fn winner(&self) -> Option<TeamSummary> {
        if let Some(game_summary) = &self.game_summary {
            game_summary.winner()
        } else {
            None
        }
    }

    pub fn loser(&self) -> Option<TeamSummary> {
        if let Some(game_summary) = &self.game_summary {
            game_summary.loser()
        } else {
            None
        }
    }

    fn reverse(&self) -> Self {
        Self {
            home_team: self.away_team.clone(),
            home_ranking_number: self.away_ranking_number,
            away_team: self.home_team.clone(),
            away_ranking_number: self.home_ranking_number,
            game_summary: None,
        }
    }

    fn reverse_all_games(games: &Vec<Self>) -> Vec<Self> {
        let mut reversed_games = Vec::with_capacity(games.len());

        for game in games.iter() {
            reversed_games.push(game.reverse());
        }

        reversed_games
    }
}

pub fn round_robin_schedule(
    team_list: &Vec<TeamSummary>,
    home_and_away: bool,
) -> Vec<RoundSchedule> {
    let mut home_schedule = Vec::new();
    let mut away_schedule = Vec::new();

    if team_list.len() >= 2 {
        let mut teams = TeamSummary::list_into_list_with_option(team_list);

        if teams.len() % 2 != 0 {
            teams.push(None);
        }

        let rounds_number = teams.len() - 1;
        let teams_half_number = teams.len() / 2;

        let fixed = teams[0].clone();
        let mut rotating = teams[1..].to_vec();

        for round in 0..rounds_number {
            let mut round_games = Vec::new();

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

                if home_team.is_some() && away_team.is_some() {
                    round_games.push(GameSchedule {
                        home_team,
                        home_ranking_number: None,
                        away_team,
                        away_ranking_number: None,
                        game_summary: None,
                    });
                }
            }

            if home_and_away {
                away_schedule.push(RoundSchedule {
                    name: format!("Journée {}", round + rounds_number + 1),
                    games: GameSchedule::reverse_all_games(&round_games),
                });
            }

            home_schedule.push(RoundSchedule {
                name: format!("Journée {}", round + 1),
                games: round_games,
            });

            // Rotate teams (except fixed)
            let last = rotating.pop().unwrap();
            rotating.insert(0, last);
        }

        if home_and_away {
            home_schedule.extend(away_schedule);
        }
    }

    home_schedule
}

pub fn cup_schedule(team_list: &Vec<TeamSummary>, with_ranking: bool) -> Vec<RoundSchedule> {
    let mut schedule = Vec::new();

    if team_list.len() >= 2 {
        let mut teams = TeamSummary::list_into_list_with_option(team_list);

        let mut cup_teams_number = 2;

        while cup_teams_number < teams.len() {
            cup_teams_number *= 2;
        }

        while teams.len() < cup_teams_number {
            teams.push(None);
        }

        let mut teams = vec![teams];

        while teams[0].len() > 1 {
            let mut next_round_teams = Vec::with_capacity(teams.len() * 2);

            for cup_part_index in 0..teams.len() {
                let number_for_part = (cup_part_index * teams[cup_part_index].len()) + 1;

                let mut round_games = Vec::with_capacity(teams[cup_part_index].len() / 2);
                let mut first_team_index = 0;
                let mut second_team_index = teams[cup_part_index].len() - 1;

                let mut winners = Vec::with_capacity(teams[cup_part_index].len() / 2);
                let mut losers = Vec::with_capacity(teams[cup_part_index].len() / 2);

                while first_team_index < second_team_index {
                    let game = GameSchedule {
                        home_team: teams[cup_part_index][first_team_index].clone(),
                        home_ranking_number: Some(first_team_index + number_for_part),
                        away_team: teams[cup_part_index][second_team_index].clone(),
                        away_ranking_number: Some(second_team_index + number_for_part),
                        game_summary: None,
                    };

                    winners.push(game.winner());

                    if with_ranking {
                        losers.push(game.loser());
                    }

                    round_games.push(game);

                    first_team_index += 1;
                    second_team_index -= 1;
                }

                next_round_teams.push(winners);

                if with_ranking {
                    next_round_teams.push(losers);
                }

                let round_name = match (number_for_part, round_games.len()) {
                    (1, 1) => "Finale".to_string(),
                    (1, 2) => "1/2 finale".to_string(),
                    (1, 4) => "1/4 de finale".to_string(),
                    (1, 8) => "1/8 de finale".to_string(),
                    (1, 16) => "1/16 de finale".to_string(),
                    (1, 32) => "1/32 de finale".to_string(),
                    (1, _) => "Tableau principal".to_string(),
                    (number_for_part, 1) => format!("Match pour la {}ème place", number_for_part),
                    (number_for_part, _) => format!("Tableau pour la {}ème place", number_for_part),
                };

                schedule.push(RoundSchedule {
                    name: round_name,
                    games: round_games,
                });
            }

            teams = next_round_teams;
        }
    }

    schedule
}
