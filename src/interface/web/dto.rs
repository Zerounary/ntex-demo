use serde::{Deserialize, Serialize};
use struct_convert::Convert;

use crate::domain::{
    accelerator::{AcceleratorUser, BootstrapPayload, Game, Profile},
    auth::{AccountLoginResponse, WechatTicket},
    content::{DashboardPayload, LibraryPayload, NavigationConfig, SettingsMeta},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorBootstrapVO {
    pub games: Vec<GameVO>,
    pub profiles: Vec<ProfileVO>,
    pub user: Option<UserVO>,
}

impl From<BootstrapPayload> for AcceleratorBootstrapVO {
    fn from(value: BootstrapPayload) -> Self {
        Self {
            games: value.games.into_iter().map(GameVO::from).collect(),
            profiles: value.profiles.into_iter().map(ProfileVO::from).collect(),
            user: value.user.map(UserVO::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVO {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub status: String,
    pub ping: i32,
}

impl From<Game> for GameVO {
    fn from(value: Game) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon: value.icon,
            status: value.status,
            ping: value.ping,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileVO {
    pub id: String,
    pub game_id: String,
    pub display_name: String,
    pub process_name: String,
    pub vmess_uuid: String,
    pub vmess_server: String,
    pub vmess_port: i32,
    pub vmess_email: String,
    pub udp_proxy: String,
    pub mode: String,
    pub status: String,
    pub region: String,
    pub ping: i32,
}

impl From<Profile> for ProfileVO {
    fn from(value: Profile) -> Self {
        Self {
            id: value.id,
            game_id: value.game_id,
            display_name: value.display_name,
            process_name: value.process_name,
            vmess_uuid: value.vmess_uuid,
            vmess_server: value.vmess_server,
            vmess_port: value.vmess_port,
            vmess_email: value.vmess_email,
            udp_proxy: value.udp_proxy,
            mode: value.mode,
            status: value.status,
            region: value.region,
            ping: value.ping,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVO {
    pub id: String,
    pub name: String,
    pub valid_until: String,
}

impl From<AcceleratorUser> for UserVO {
    fn from(value: AcceleratorUser) -> Self {
        Self {
            id: value.id,
            name: value.name,
            valid_until: value.valid_until.format("%Y/%m/%d").to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketRequestVO {
    pub scene: String,
}

#[derive(Debug, Deserialize, Convert)]
#[serde(rename_all = "camelCase")]
#[convert(into = "crate::domain::auth::AccountLoginRequest")]
pub struct AccountLoginRequestVO {
    pub phone: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSyncRequest {
    pub profiles: Vec<ProfileVO>,
}

impl From<ProfileSyncRequest> for Vec<Profile> {
    fn from(value: ProfileSyncRequest) -> Self {
        value
            .profiles
            .into_iter()
            .map(|p| Profile {
                id: p.id,
                game_id: p.game_id,
                display_name: p.display_name,
                process_name: p.process_name,
                vmess_uuid: p.vmess_uuid,
                vmess_server: p.vmess_server,
                vmess_port: p.vmess_port,
                vmess_email: p.vmess_email,
                udp_proxy: p.udp_proxy,
                mode: p.mode,
                status: p.status,
                region: p.region,
                ping: p.ping,
            })
            .collect()
    }
}

pub type DashboardVO = DashboardPayload;
pub type LibraryVO = LibraryPayload;
pub type SettingsMetaVO = SettingsMeta;
pub type NavigationVO = NavigationConfig;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WechatTicketVO {
    pub ticket_id: String,
    pub qr_code_url: String,
    pub expires_in: i64,
    pub status: String,
    pub scene: String,
    pub success: bool,
    pub user: Option<UserVO>,
}

impl From<WechatTicket> for WechatTicketVO {
    fn from(value: WechatTicket) -> Self {
        Self {
            ticket_id: value.ticket_id,
            qr_code_url: value.qr_code_url,
            expires_in: value.expires_in,
            status: value.status.as_str().to_string(),
            scene: value.scene,
            success: value.success,
            user: value.user.map(UserVO::from),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TicketStatusQuery {
    #[serde(rename = "ticketId")]
    pub ticket_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginResponseVO {
    pub success: bool,
    pub token: String,
    pub user: UserVO,
}

impl From<AccountLoginResponse> for AccountLoginResponseVO {
    fn from(value: AccountLoginResponse) -> Self {
        Self {
            success: value.success,
            token: value.token,
            user: value.user.into(),
        }
    }
}
