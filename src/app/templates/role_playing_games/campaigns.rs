use crate::app::templates::{role_playing_games, BreadCrumb, NavigationBar, UrlLink};
use crate::data::role_playing_games::campaigns::{Campaign, CampaignRow};
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use crate::AppState;
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
    is_owner: bool,
    games: Vec<Game>,
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
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            campaign,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            is_owner: editable,
            games,
        }
    }
}
