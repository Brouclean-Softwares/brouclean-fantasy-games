use crate::data::blood_bowl::teams::TeamSummary;

pub struct GameSchedule {
    pub home_team: TeamSummary,
    pub away_team: TeamSummary,
}

impl GameSchedule {
    pub fn round_robin_schedule(team_list: &Vec<TeamSummary>) -> Vec<Vec<GameSchedule>> {
        let mut teams: Vec<Option<TeamSummary>> =
            team_list.iter().map(|team| Some(team.clone())).collect();

        let n = teams.len();

        if n >= 2 {
            let has_bye = if n % 2 != 0 {
                teams.push(None);
                true
            } else {
                false
            };

            let n = teams.len();
            let rounds = n - 1;
            let half = n / 2;

            let mut schedule = Vec::new();

            // Split first team (fixed) from others
            let fixed = teams[0].clone();
            let mut rotating = teams[1..].to_vec();

            for _round in 0..rounds {
                let mut round_games = Vec::new();

                // Pairs construction
                for i in 0..half {
                    let home_team: Option<TeamSummary>;
                    let away_team: Option<TeamSummary>;

                    if i == 0 {
                        home_team = fixed.clone();
                        away_team = rotating[0].clone();
                    } else {
                        home_team = rotating[i].clone();
                        away_team = rotating[rotating.len() - i].clone();
                    }

                    // Ignore games with None
                    if let (Some(home_team), Some(away_team)) = (home_team, away_team) {
                        round_games.push(GameSchedule {
                            home_team,
                            away_team,
                        });
                    }
                }

                schedule.push(round_games);

                // Rotate teams (except first)
                let last = rotating.pop().unwrap();
                rotating.insert(0, last);
            }

            // remove days with only None
            if has_bye {
                schedule.into_iter().filter(|day| !day.is_empty()).collect()
            } else {
                schedule
            }
        } else {
            vec![]
        }
    }
}
