use crate::app::templates::{BreadCrumb, NavigationBar};
use crate::data::blood_bowl::statistics::{StatisticElement, Statistics};
use crate::data::blood_bowl::teams::TeamLogo;
use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/statistics_page.html")]
pub struct StatisticsPage {
    pub navigation_bar: NavigationBar,
    pub breadcrumb: BreadCrumb,
    pub teams_top_victories: StatisticList,
    pub teams_top_games: StatisticList,
    pub teams_top_star_player_points: StatisticList,
    pub teams_top_touchdowns: StatisticList,
    pub teams_top_casualties: StatisticList,
    pub teams_top_injuries: StatisticList,
    pub teams_top_interceptions: StatisticList,
    pub teams_top_deflections: StatisticList,
    pub teams_top_passing_completions: StatisticList,
    pub teams_top_throwing_completions: StatisticList,
    pub players_top_star_player_points: StatisticList,
    pub players_top_touchdowns: StatisticList,
    pub players_top_casualties: StatisticList,
    pub players_top_injuries: StatisticList,
    pub players_top_interceptions: StatisticList,
    pub players_top_deflections: StatisticList,
    pub players_top_passing_completions: StatisticList,
    pub players_top_throwing_completions: StatisticList,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/statistic_list.html")]
pub struct StatisticList {
    statistic_name: String,
    statistics: Statistics,
    element_url: String,
}

impl StatisticList {
    pub fn from(statistic_name: String, statistics: Statistics) -> Self {
        let element_url = match statistics.statistic_element {
            StatisticElement::Team => String::from("/blood_bowl/teams/team?id"),
            StatisticElement::Player => String::from("/blood_bowl/players/player?player_id"),
        };

        Self {
            statistic_name,
            statistics,
            element_url,
        }
    }
}
