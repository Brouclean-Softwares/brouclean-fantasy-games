use crate::AppState;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink, role_playing_games};
use crate::data::role_playing_games::campaigns::arcs::{
    NarrativeArc, NarrativeArcWithGameSessions,
};
use crate::data::role_playing_games::campaigns::sessions::GameSession;
use crate::data::role_playing_games::campaigns::{Campaign, CampaignRow};
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;

pub fn breadcrumb() -> BreadCrumb {
    role_playing_games::breadcrumb()
        .plus_link(UrlLink::from("Campagnes", "/role_playing_games/campaigns"))
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/campaigns_page.html")]
pub struct CampaignsPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    campaigns: Vec<CampaignRow>,
    add_new_campaign_button: AddNewCampaignButton,
}

impl CampaignsPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        campaigns: Vec<CampaignRow>,
        games: Vec<Game>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: role_playing_games::breadcrumb(),
            campaigns,
            add_new_campaign_button: AddNewCampaignButton { profile, games },
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/add_new_campaign_button.html")]
pub struct AddNewCampaignButton {
    pub profile: Option<User>,
    pub games: Vec<Game>,
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/owned_campaigns_block.html")]
pub struct OwnedCampaignsBlock {
    pub owned_campaigns: Vec<CampaignRow>,
    pub add_new_campaign_button: AddNewCampaignButton,
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/campaign_page.html")]
pub struct CampaignPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    campaign: Campaign,
    tab_name: String,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    games: Vec<Game>,
    arcs_with_sessions: Vec<NarrativeArcWithGameSessions>,
}

impl CampaignPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        campaign: Campaign,
        tab_name: Option<String>,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
        games: Vec<Game>,
        arcs_with_sessions: Vec<NarrativeArcWithGameSessions>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            campaign,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            games,
            arcs_with_sessions,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/arc_page.html")]
pub struct NarrativeArcPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    arc: NarrativeArc,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
}

impl NarrativeArcPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        arc: NarrativeArc,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb().plus_link(UrlLink::from(
                "Campagne",
                &format!(
                    "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
                    arc.campaign_id
                ),
            )),
            arc,
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/session_pagination.html")]
pub struct GameSessionPagination {
    previous_session: Option<GameSession>,
    next_session: Option<GameSession>,
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/session_page.html")]
pub struct GameSessionPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    session: GameSession,
    session_pagination: GameSessionPagination,
    session_date_input: Option<String>,
    session_date: Option<String>,
    tab_name: String,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
}

impl GameSessionPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        session: GameSession,
        previous_session: Option<GameSession>,
        next_session: Option<GameSession>,
        tab_name: Option<String>,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
    ) -> Self {
        let session_date_input = if let Some(date) = session.playing_at {
            Some(date.format("%Y-%m-%dT%H:%M").to_string())
        } else {
            None
        };

        let session_date = if let Some(date) = session.playing_at {
            Some(date.format("%d/%m/%Y à %H:%M").to_string())
        } else {
            None
        };

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb().plus_link(UrlLink::from(
                "Campagne",
                &format!(
                    "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
                    session.campaign_id
                ),
            )),
            session,
            session_pagination: GameSessionPagination {
                previous_session,
                next_session,
            },
            session_date_input,
            session_date,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
        }
    }
}
