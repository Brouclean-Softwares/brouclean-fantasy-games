use crate::AppState;
use crate::app::templates::CampaignSessionTable;
use crate::app::templates::role_playing_games::campaigns::{
    AddNewCampaignButton, OwnedCampaignsBlock,
};
use crate::app::templates::role_playing_games::characters::{
    AddNewCharacterButton, OwnedCharactersBlock,
};
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink};
use crate::data::role_playing_games::campaigns::CampaignRow;
use crate::data::role_playing_games::campaigns::sessions::CampaignSession;
use crate::data::role_playing_games::characters::CharacterRow;
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use crate::errors::AppError;
use askama::Template;
use askama_web::WebTemplate;
use http::Uri;

pub mod campaigns;
pub mod characters;
pub mod games;

pub fn breadcrumb() -> BreadCrumb {
    BreadCrumb::only_home().plus_link(UrlLink::from("Jeux de rôle", "/role_playing_games"))
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    scheduled_campaign_sessions: Vec<CampaignSession>,
    owned_characters_block: Option<OwnedCharactersBlock>,
    owned_campaigns_block: Option<OwnedCampaignsBlock>,
}

impl HomePage {
    pub async fn get(
        app_state: &AppState,
        profile: &User,
        uri: &Uri,
        scheduled_campaign_sessions: Vec<CampaignSession>,
        owned_characters: Vec<CharacterRow>,
        owned_campaigns: Vec<CampaignRow>,
        games: Vec<Game>,
    ) -> Result<Self, AppError> {
        let owned_characters_block = if owned_characters.is_empty() {
            None
        } else {
            Some(OwnedCharactersBlock {
                owned_characters,
                add_new_character_button: AddNewCharacterButton {
                    profile: Some(profile.clone()),
                    games: games.clone(),
                },
            })
        };

        let owned_campaigns_block = if owned_campaigns.is_empty() {
            None
        } else {
            Some(OwnedCampaignsBlock {
                owned_campaigns,
                add_new_campaign_button: AddNewCampaignButton {
                    profile: Some(profile.clone()),
                    games: games.clone(),
                },
            })
        };

        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile.clone()), uri),
            breadcrumb: BreadCrumb::only_home(),
            scheduled_campaign_sessions,
            owned_characters_block,
            owned_campaigns_block,
        })
    }
}
