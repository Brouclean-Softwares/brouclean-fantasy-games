use crate::app::templates::{BreadCrumb, NavigationBar};
use crate::data::blood_bowl::statistics::TeamStatisticRow;
use crate::data::blood_bowl::teams::TeamLogo;
use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/statistics_page.html")]
pub struct StatisticsPage {
    pub navigation_bar: NavigationBar,
    pub breadcrumb: BreadCrumb,
    pub teams_top_victories: TeamsStatisticList,
    pub teams_top_games: TeamsStatisticList,
    pub teams_top_values: TeamsStatisticList,
    pub teams_top_star_player_points: TeamsStatisticList,
    pub teams_top_touchdowns: TeamsStatisticList,
    pub teams_top_casualties: TeamsStatisticList,
    pub teams_top_interceptions: TeamsStatisticList,
    pub teams_top_deflections: TeamsStatisticList,
    pub teams_top_passing_completions: TeamsStatisticList,
    pub teams_top_throwing_completions: TeamsStatisticList,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/teams_statistic_list.html")]
pub struct TeamsStatisticList {
    statistic_name: String,
    list_length: usize,
    teams_totals: Vec<TeamStatisticRow>,
}

impl TeamsStatisticList {
    pub fn from(
        statistic_name: String,
        list_length: usize,
        teams_totals: Vec<TeamStatisticRow>,
    ) -> Self {
        Self {
            statistic_name,
            list_length,
            teams_totals,
        }
    }
}
