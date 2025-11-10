use crate::app::templates::{BreadCrumb, NavigationBar};
use crate::data::blood_bowl::statistics::players::PlayersTopStatistics;
use crate::data::blood_bowl::statistics::teams::TeamsTopStatistics;
use crate::data::blood_bowl::statistics::{StatisticElement, Statistics};
use crate::data::blood_bowl::teams::TeamLogo;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::translation::TranslatedName;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/statistics_page.html")]
pub struct StatisticsPage {
    pub navigation_bar: NavigationBar,
    pub breadcrumb: BreadCrumb,
    pub teams_top_statistics: TeamsTopStatisticsLists,
    pub players_top_statistics: PlayersTopStatisticsLists,
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/teams_top_statistics_lists.html")]
pub struct TeamsTopStatisticsLists {
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
}

impl From<TeamsTopStatistics> for TeamsTopStatisticsLists {
    fn from(teams_top_statistics: TeamsTopStatistics) -> Self {
        Self {
            teams_top_victories: StatisticList::from(
                String::from("Victoires"),
                teams_top_statistics.teams_top_victories,
            ),
            teams_top_games: StatisticList::from(
                String::from("Nombre de matchs"),
                teams_top_statistics.teams_top_games,
            ),
            teams_top_star_player_points: StatisticList::from(
                String::from("Points de star players (PSP)"),
                teams_top_statistics.teams_top_star_player_points,
            ),
            teams_top_touchdowns: StatisticList::from(
                String::from("Touchdowns (TD)"),
                teams_top_statistics.teams_top_touchdowns,
            ),
            teams_top_casualties: StatisticList::from(
                String::from("Éliminations (ELI)"),
                teams_top_statistics.teams_top_casualties,
            ),
            teams_top_injuries: StatisticList::from(
                String::from("Blessures avec match raté (RPM)"),
                teams_top_statistics.teams_top_injuries,
            ),
            teams_top_interceptions: StatisticList::from(
                String::from("Interceptions (INT)"),
                teams_top_statistics.teams_top_interceptions,
            ),
            teams_top_deflections: StatisticList::from(
                String::from("Détournements (DET)"),
                teams_top_statistics.teams_top_deflections,
            ),
            teams_top_passing_completions: StatisticList::from(
                String::from("Passes (PAS)"),
                teams_top_statistics.teams_top_passing_completions,
            ),
            teams_top_throwing_completions: StatisticList::from(
                String::from("Lancers de coéquipier (LAN)"),
                teams_top_statistics.teams_top_throwing_completions,
            ),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/statistics/players_top_statistics_lists.html")]
pub struct PlayersTopStatisticsLists {
    pub players_top_star_player_points: StatisticList,
    pub players_top_touchdowns: StatisticList,
    pub players_top_casualties: StatisticList,
    pub players_top_injuries: StatisticList,
    pub players_top_interceptions: StatisticList,
    pub players_top_deflections: StatisticList,
    pub players_top_passing_completions: StatisticList,
    pub players_top_throwing_completions: StatisticList,
}

impl From<PlayersTopStatistics> for PlayersTopStatisticsLists {
    fn from(players_top_statistics: PlayersTopStatistics) -> Self {
        Self {
            players_top_star_player_points: StatisticList::from(
                String::from("Points de star players (PSP)"),
                players_top_statistics.players_top_star_player_points,
            ),
            players_top_touchdowns: StatisticList::from(
                String::from("Touchdowns (TD)"),
                players_top_statistics.players_top_touchdowns,
            ),
            players_top_casualties: StatisticList::from(
                String::from("Éliminations (ELI)"),
                players_top_statistics.players_top_casualties,
            ),
            players_top_injuries: StatisticList::from(
                String::from("Blessures avec match raté (RPM)"),
                players_top_statistics.players_top_injuries,
            ),
            players_top_interceptions: StatisticList::from(
                String::from("Interceptions (INT)"),
                players_top_statistics.players_top_interceptions,
            ),
            players_top_deflections: StatisticList::from(
                String::from("Détournements (DET)"),
                players_top_statistics.players_top_deflections,
            ),
            players_top_passing_completions: StatisticList::from(
                String::from("Passes (PAS)"),
                players_top_statistics.players_top_passing_completions,
            ),
            players_top_throwing_completions: StatisticList::from(
                String::from("Lancers de coéquipier (LAN)"),
                players_top_statistics.players_top_throwing_completions,
            ),
        }
    }
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
