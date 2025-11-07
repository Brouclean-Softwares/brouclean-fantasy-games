use crate::data::blood_bowl::competitions::stages::CompetitionStage;
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;

pub struct StageSchedule {
    pub stage: CompetitionStage,
    pub rounds_schedule: Vec<RoundSchedule>,
    pub finished: bool,
}

impl From<&CompetitionStage> for StageSchedule {
    fn from(stage: &CompetitionStage) -> Self {
        Self {
            stage: stage.clone(),
            rounds_schedule: Vec::new(),
            finished: true,
        }
    }
}

impl StageSchedule {
    pub fn push(&mut self, round_schedule: RoundSchedule) {
        if !round_schedule.is_empty() {
            self.finished = self.finished && round_schedule.finished;
            self.rounds_schedule.push(round_schedule);
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.finished = self.finished && other.finished;
        self.rounds_schedule.extend(other.rounds_schedule);
    }
}

pub struct RoundSchedule {
    pub name: String,
    pub games_schedule: Vec<GameSchedule>,
    pub all_games_created: bool,
    pub finished: bool,
}

impl RoundSchedule {
    pub fn new_with_name(name: String) -> Self {
        Self {
            name,
            games_schedule: Vec::new(),
            all_games_created: true,
            finished: true,
        }
    }

    pub fn new_with_name_and_capacity(name: String, capacity: usize) -> Self {
        Self {
            name,
            games_schedule: Vec::with_capacity(capacity),
            all_games_created: true,
            finished: true,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.games_schedule.is_empty()
    }

    pub fn push(&mut self, game_schedule: GameSchedule) {
        if game_schedule.home_team.ne(&Some(BYE.clone()))
            && game_schedule.away_team.ne(&Some(BYE.clone()))
        {
            self.all_games_created = self.all_games_created && game_schedule.created();
            self.finished = self.finished && game_schedule.finished();

            self.games_schedule.push(game_schedule);
        }
    }
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

#[derive(Clone)]
pub struct GameSchedule {
    pub home_team: Option<TeamSummary>,
    pub home_ranking_number: Option<usize>,
    pub away_team: Option<TeamSummary>,
    pub away_ranking_number: Option<usize>,
    pub game_summary: Option<GameSummary>,
}

impl From<GameSummary> for GameSchedule {
    fn from(game_summary: GameSummary) -> Self {
        Self {
            home_team: Some(game_summary.first_team.clone()),
            home_ranking_number: None,
            away_team: Some(game_summary.second_team.clone()),
            away_ranking_number: None,
            game_summary: Some(game_summary),
        }
    }
}

impl GameSchedule {
    pub fn created(&self) -> bool {
        self.game_summary.is_some()
    }

    pub fn finished(&self) -> bool {
        if let Some(game) = &self.game_summary {
            game.finished
        } else {
            false
        }
    }

    pub fn score(&self) -> Option<(usize, usize)> {
        if let Some(game_summary) = &self.game_summary {
            if game_summary.started {
                Some((
                    game_summary.first_team_score as usize,
                    game_summary.second_team_score as usize,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn casualties(&self) -> Option<(usize, usize)> {
        if let Some(game_summary) = &self.game_summary {
            if game_summary.started {
                Some((
                    game_summary.first_team_casualties as usize,
                    game_summary.second_team_casualties as usize,
                ))
            } else {
                None
            }
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
}
