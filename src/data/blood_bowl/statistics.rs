use blood_bowl_rs::positions::Position;
use blood_bowl_rs::rosters::Roster;
use serde::Deserialize;

pub mod players;
pub mod teams;

pub struct Statistics {
    pub statistic_element: StatisticElement,
    pub statistics_rows: Vec<StatisticRow>,
}

impl Statistics {
    pub fn empty() -> Self {
        Self {
            statistic_element: StatisticElement::Team,
            statistics_rows: vec![],
        }
    }
}

pub enum StatisticElement {
    Team,
    Player,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct StatisticRow {
    pub id: i32,
    pub team_id: i32,
    pub external_logo_url: Option<String>,
    pub roster: Roster,
    pub name: String,
    pub position: Option<Position>,
    pub statistic_value: String,
}
