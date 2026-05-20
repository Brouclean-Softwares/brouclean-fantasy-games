use crate::AppState;
use crate::app::templates::role_playing_games::characters::CharacterSelector;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink, role_playing_games};
use crate::data::role_playing_games::campaigns::arcs::{
    NarrativeArc, NarrativeArcWithGameSessions,
};
use crate::data::role_playing_games::campaigns::sessions::{GameSession, GameSessionWithCampaign};
use crate::data::role_playing_games::campaigns::{Campaign, CampaignRow};
use crate::data::role_playing_games::characters::CharacterRow;
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;
use http::Uri;

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
        uri: &Uri,
        campaigns: Vec<CampaignRow>,
        games: Vec<Game>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
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
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    games: Vec<Game>,
    arcs_with_sessions: Vec<NarrativeArcWithGameSessions>,
    characters: Vec<CharacterRow>,
}

impl CampaignPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        campaign: Campaign,
        tab_name: Option<String>,
        deletable: bool,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
        games: Vec<Game>,
        arcs_with_sessions: Vec<NarrativeArcWithGameSessions>,
        characters: Vec<CharacterRow>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: breadcrumb(),
            campaign,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            deletable,
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            games,
            arcs_with_sessions,
            characters,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/arc_page.html")]
pub struct NarrativeArcPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    campaign: Campaign,
    arc: NarrativeArc,
    sessions: Vec<GameSession>,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    characters: Vec<CharacterRow>,
}

impl NarrativeArcPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        campaign: Campaign,
        arc: NarrativeArc,
        sessions: Vec<GameSession>,
        deletable: bool,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
        characters: Vec<CharacterRow>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: breadcrumb().plus_link(UrlLink::from(
                "Campagne",
                &format!(
                    "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
                    arc.campaign_id
                ),
            )),
            campaign,
            arc,
            sessions,
            deletable,
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            characters,
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
    campaign: Campaign,
    session: GameSession,
    session_pagination: GameSessionPagination,
    session_date_input: Option<String>,
    session_date: Option<String>,
    tab_name: String,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    characters: Vec<CharacterRow>,
    character_selector: CharacterSelector,
}

impl GameSessionPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        uri: &Uri,
        campaign: Campaign,
        session: GameSession,
        previous_session: Option<GameSession>,
        next_session: Option<GameSession>,
        tab_name: Option<String>,
        deletable: bool,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
        characters: Vec<CharacterRow>,
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

        let game_id = campaign.game_id;

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile, uri),
            breadcrumb: breadcrumb().plus_link(UrlLink::from(
                "Campagne",
                &format!(
                    "/role_playing_games/campaigns/campaign?id={}&tab_name=sessions",
                    session.campaign_id
                ),
            )),
            campaign,
            session,
            session_pagination: GameSessionPagination {
                previous_session,
                next_session,
            },
            session_date_input,
            session_date,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            deletable,
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            characters,
            character_selector: CharacterSelector::get("character_to_link_id".to_string(), game_id),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/campaigns/campaign_sessions_table.html")]
pub struct CampaignSessionTable {
    pub campaign_sessions: Vec<GameSessionWithCampaign>,
}

impl CampaignSessionTable {
    pub fn from_campaign_sessions(campaign_sessions: Vec<GameSessionWithCampaign>) -> Self {
        Self { campaign_sessions }
    }
}
