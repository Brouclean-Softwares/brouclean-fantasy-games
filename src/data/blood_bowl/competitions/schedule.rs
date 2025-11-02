use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;

pub struct RoundSchedule {
    pub name: String,
    pub games: Vec<GameSchedule>,
    pub finished: bool,
}

lazy_static::lazy_static! {
    pub static ref BYE: TeamSummary = TeamSummary {
        id: -1000,
        version: Version::V4,
        name: "BYE".to_string(),
        roster: Roster::Amazon,
        coach_id: None,
        coach_name: "".to_string(),
        external_logo_url: None,
        value: 0,
        current_value: 0,
        treasury: 0,
        dedicated_fans: 0,
        under_creation: false,
    };
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
        } else if self.home_team.eq(&Some(BYE.clone())) {
            self.away_team.clone()
        } else if self.away_team.eq(&Some(BYE.clone())) {
            self.home_team.clone()
        } else {
            None
        }
    }

    pub fn loser(&self) -> Option<TeamSummary> {
        if let Some(game_summary) = &self.game_summary {
            game_summary.loser()
        } else if self.home_team.eq(&Some(BYE.clone())) {
            self.home_team.clone()
        } else if self.away_team.eq(&Some(BYE.clone())) {
            self.away_team.clone()
        } else {
            None
        }
    }

    pub fn reverse(&self) -> Self {
        Self {
            home_team: self.away_team.clone(),
            home_ranking_number: self.away_ranking_number,
            away_team: self.home_team.clone(),
            away_ranking_number: self.home_ranking_number,
            game_summary: None,
        }
    }

    pub fn reverse_all_games(games: &Vec<Self>) -> Vec<Self> {
        let mut reversed_games = Vec::with_capacity(games.len());

        for game in games.iter() {
            reversed_games.push(game.reverse());
        }

        reversed_games
    }
}
