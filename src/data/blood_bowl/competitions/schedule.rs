use crate::data::blood_bowl::competitions::stages::CompetitionStage;
use crate::data::blood_bowl::games::GameSummary;
use crate::data::blood_bowl::teams::TeamSummary;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::versions::Version;

pub struct CompetitionSchedule {
    pub stages_schedule: Vec<StageSchedule>,
}

impl From<Vec<StageSchedule>> for CompetitionSchedule {
    fn from(stages_schedule: Vec<StageSchedule>) -> Self {
        Self { stages_schedule }
    }
}

impl CompetitionSchedule {
    pub fn get_stage_round(&self, stage_id: i32, round_index: usize) -> Option<RoundSchedule> {
        let stage_index = self
            .stages_schedule
            .iter()
            .position(|stage_schedule| stage_schedule.stage.id.eq(&stage_id));

        if let Some(stage_index) = stage_index {
            if let Some(round_schedule) = self.stages_schedule[stage_index]
                .rounds_schedule
                .get(round_index)
            {
                Some(round_schedule.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_finished(&self) -> bool {
        self.stages_schedule
            .iter()
            .filter(|&stage_schedule| !stage_schedule.is_finished())
            .count()
            == 0
    }

    pub fn round_number(&self) -> usize {
        let mut round_number = 0;

        for stage_schedule in &self.stages_schedule {
            round_number += stage_schedule.round_number();
        }

        round_number
    }

    pub fn should_imply_offseason(&self) -> bool {
        self.round_number() >= super::offseasons::OFFSEASON_COMPETITION_ROUND_THRESHOLD
    }
}

pub struct StageSchedule {
    pub stage: CompetitionStage,
    pub rounds_schedule: Vec<RoundSchedule>,
    pub all_games_created: bool,
    pub finished: bool,
}

impl From<&CompetitionStage> for StageSchedule {
    fn from(stage: &CompetitionStage) -> Self {
        Self {
            stage: stage.clone(),
            rounds_schedule: Vec::new(),
            all_games_created: true,
            finished: true,
        }
    }
}

impl StageSchedule {
    pub fn push(&mut self, round_schedule: RoundSchedule) {
        if !round_schedule.is_empty() {
            self.all_games_created = self.all_games_created && round_schedule.all_games_created;
            self.finished = self.finished && round_schedule.finished;

            self.rounds_schedule.push(round_schedule);
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.all_games_created = self.all_games_created && other.all_games_created;
        self.finished = self.finished && other.finished;

        self.rounds_schedule.extend(other.rounds_schedule);
    }

    pub fn is_finished(&self) -> bool {
        self.rounds_schedule
            .iter()
            .filter(|&round_schedule| !round_schedule.is_finished())
            .count()
            == 0
    }

    pub fn round_number(&self) -> usize {
        self.rounds_schedule.len()
    }
}

#[derive(Clone)]
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

    pub fn games_that_can_be_created(&self) -> Vec<GameSchedule> {
        self.games_schedule
            .iter()
            .map(|game_schedule| game_schedule.clone())
            .filter(|game_schedule| game_schedule.can_be_created())
            .collect()
    }

    pub fn games_can_be_created(&self) -> bool {
        self.games_schedule
            .iter()
            .filter(|&game_schedule| game_schedule.can_be_created())
            .count()
            > 0
    }

    pub fn is_finished(&self) -> bool {
        !self.games_can_be_created()
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
        in_offseason: false,
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

    pub fn can_be_created(&self) -> bool {
        self.game_summary.is_none()
            && self.home_team.is_some()
            && self.home_team.ne(&Some(BYE.clone()))
            && self.away_team.is_some()
            && self.away_team.ne(&Some(BYE.clone()))
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

    pub fn pick_game_summary_from_list(&mut self, games: &mut Vec<GameSummary>) {
        if self.game_summary.is_none() {
            let game_position = games.iter().position(|game_summary| {
                game_summary.first_team.eq(&self.home_team)
                    && game_summary.second_team.eq(&self.away_team)
            });

            if let Some(game_position) = game_position {
                self.game_summary = Some(games.remove(game_position));
            }
        }
    }
}
