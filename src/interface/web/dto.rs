use serde::{Deserialize, Serialize};
use struct_convert::Convert;

use crate::domain::{
    accelerator::{AcceleratorUser, BootstrapPayload, Game, Profile},
    auth::{AccountLoginResponse, WechatTicket},
    cdk::{AccountValidationRequest, AccountValidationResponse, CdkCode, CdkGenerateRequest, CdkRedeemRequest, CdkRedeemResponse, CdkType},
    content::{DashboardPayload, LibraryPayload, NavigationConfig, SettingsMeta},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameNodeBindingRequest {
    pub node_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameNodeVO {
    pub node_id: String,
    pub vless_id: String,
    pub vless_server: String,
    pub vless_port: i32,
    pub vless_encryption: String,
    pub reality_server_name: String,
    pub reality_public_key: String,
    pub reality_short_id: String,
    pub reality_fingerprint: String,
    pub reality_spider_x: String,
    pub udp_proxy: String,
    pub mode: String,
    pub ping: i32,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorUserRegisterRequestVO {
    pub user_id: String,
    pub name: String,
    pub password: String,
    #[serde(default)]
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorUserLoginRequestVO {
    pub user_id: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorUserLoginResponseVO {
    pub success: bool,
    pub token: String,
    pub user: UserVO,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorUserUpdateProfileRequestVO {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorUserChangePasswordRequestVO {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratorBootstrapVO {
    pub games: Vec<GameVO>,
    pub profiles: Vec<ProfileListVO>,
    pub user: Option<UserVO>,
}

impl From<BootstrapPayload> for AcceleratorBootstrapVO {
    fn from(value: BootstrapPayload) -> Self {
        // 创建节点和游戏的查找映射
        let node_map: std::collections::HashMap<_, _> = value
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n.clone()))
            .collect();
        let game_map: std::collections::HashMap<_, _> = value
            .games
            .iter()
            .map(|g| (g.id.clone(), g.clone()))
            .collect();

        // 组装 ProfileListVO：仅用于前端节点列表展示（不包含 vmess_server 等敏感配置）
        let profiles: Vec<ProfileListVO> = value
            .profiles
            .into_iter()
            .filter_map(|p| {
                let node = node_map.get(&p.node_id)?;
                let game = game_map.get(&p.game_id)?;
                Some(ProfileListVO {
                    id: p.id,
                    game_id: p.game_id,
                    display_name: p.display_name,
                    node_id: p.node_id,
                    mode: node.mode.clone(),
                    status: p.status,
                    region: game.region.clone(),
                    ping: node.ping,
                })
            })
            .collect();

        Self {
            games: value.games.into_iter().map(GameVO::from).collect(),
            profiles,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileVO {
    pub id: String,
    pub game_id: String,
    pub display_name: String,
    #[serde(default)]
    pub node_id: String,
    pub process_name: String,
    pub vless_id: String,
    pub vless_server: String,
    pub vless_port: i32,
    pub vless_encryption: String,
    pub reality_server_name: String,
    pub reality_public_key: String,
    pub reality_short_id: String,
    pub reality_fingerprint: String,
    pub reality_spider_x: String,
    pub udp_proxy: String,
    pub mode: String,
    pub status: String,
    pub region: String,
    pub ping: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileListVO {
    pub id: String,
    pub game_id: String,
    pub display_name: String,
    #[serde(default)]
    pub node_id: String,
    pub mode: String,
    pub status: String,
    pub region: String,
    pub ping: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponseVO<T>
where
    T: Serialize,
{
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItemVO {
    pub kind: String,
    pub id: String,
    pub label: String,
    pub game_id: Option<String>,
    pub node_id: Option<String>,
    pub region: Option<String>,
    pub mode: Option<String>,
}

// ProfileVO 现在通过 BootstrapPayload 的 From 实现来组装
// 这里保留一个简单的实现用于向后兼容（如果需要）
impl From<Profile> for ProfileVO {
    fn from(_value: Profile) -> Self {
        // 这个实现不应该被直接使用，因为 ProfileVO 需要从 Profile + Node + Game 组合
        // 如果被调用，返回一个默认值（这种情况不应该发生）
        panic!("ProfileVO should be created from BootstrapPayload, not directly from Profile")
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

// ProfileSyncRequest 现在需要同时创建/更新 Node 和 Profile
// 这个转换需要访问 NodeRepository，所以我们在 UseCase 层处理
// 这里保留一个标记类型，实际转换在 UseCase 中完成
impl From<ProfileSyncRequest> for Vec<Profile> {
    fn from(_value: ProfileSyncRequest) -> Self {
        // 这个实现不应该被直接使用
        // ProfileSyncRequest 的转换应该在 UseCase 层完成，因为需要同时处理 Node
        panic!("ProfileSyncRequest conversion should be handled in UseCase layer")
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRegisterRequest {
    pub id: String,
    pub vless_id: String,
    pub vless_server: String,
    pub vless_port: i32,
    pub vless_encryption: String,
    pub reality_server_name: String,
    pub reality_public_key: String,
    pub reality_short_id: String,
    pub reality_fingerprint: String,
    pub reality_spider_x: String,
    pub udp_proxy: String,
    pub mode: String,
    pub ping: i32,
    pub status: String,
}

impl From<NodeRegisterRequest> for crate::domain::accelerator::Node {
    fn from(value: NodeRegisterRequest) -> Self {
        use chrono::Utc;
        Self {
            id: value.id,
            vless_id: value.vless_id,
            vless_server: value.vless_server,
            vless_port: value.vless_port,
            vless_encryption: value.vless_encryption,
            reality_server_name: value.reality_server_name,
            reality_public_key: value.reality_public_key,
            reality_short_id: value.reality_short_id,
            reality_fingerprint: value.reality_fingerprint,
            reality_spider_x: value.reality_spider_x,
            udp_proxy: value.udp_proxy,
            mode: value.mode,
            ping: value.ping,
            status: value.status,
            last_heartbeat: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdkGenerateRequestVO {
    pub cdk_type: String,
    pub count: u32,
    pub duration_minutes: Option<i64>,
    pub bandwidth_mbps: Option<i64>,
    pub expires_at: Option<String>,
}

impl From<CdkGenerateRequestVO> for CdkGenerateRequest {
    fn from(value: CdkGenerateRequestVO) -> Self {
        use chrono::DateTime;
        Self {
            cdk_type: CdkType::from_str(&value.cdk_type).unwrap_or(CdkType::Day),
            count: value.count,
            duration_minutes: value.duration_minutes,
            bandwidth_mbps: value.bandwidth_mbps,
            expires_at: value.expires_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdkRedeemRequestVO {
    pub code: String,
}

impl From<CdkRedeemRequestVO> for CdkRedeemRequest {
    fn from(value: CdkRedeemRequestVO) -> Self {
        Self {
            code: value.code,
            user_id: "".to_string(), // This will be overridden in the handler
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdkRedeemResponseVO {
    pub success: bool,
    pub message: String,
    pub duration_minutes: i64,
    pub valid_until: Option<String>,
    pub remaining_minutes: Option<i64>,
}

impl From<CdkRedeemResponse> for CdkRedeemResponseVO {
    fn from(value: CdkRedeemResponse) -> Self {
        Self {
            success: value.success,
            message: value.message,
            duration_minutes: value.duration_minutes,
            valid_until: value
                .valid_until
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            remaining_minutes: value.remaining_minutes,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdkCodeVO {
    pub id: String,
    pub code: String,
    pub cdk_type: String,
    pub duration_minutes: i64,
    pub status: String,
    pub used_by: Option<String>,
    pub used_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

impl From<CdkCode> for CdkCodeVO {
    fn from(value: CdkCode) -> Self {
        Self {
            id: value.id,
            code: value.code,
            cdk_type: value.cdk_type.as_str().to_string(),
            duration_minutes: value.duration_minutes,
            status: value.status.as_str().to_string(),
            used_by: value.used_by,
            used_at: value.used_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            expires_at: value.expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            created_at: value.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountValidationRequestVO {
}

impl From<AccountValidationRequestVO> for AccountValidationRequest {
    fn from(_value: AccountValidationRequestVO) -> Self {
        Self {
            user_id: "".to_string(), // This will be overridden in the handler
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartRequestVO {
    pub binding_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartResponseVO {
    pub session_id: String,
    pub uuid: String,
    pub bill_type: String,
    pub remaining_minutes: Option<i64>,
    pub profile: ProfileVO,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_profile: Option<ProfileVO>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_profile: Option<ProfileVO>,
    pub routing_rules: Vec<serde_json::Value>,
    pub sniff_domains_excluded: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStopRequestVO {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStopResponseVO {
    pub status: String,
    pub billed_minutes: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountValidationResponseVO {
    pub is_valid: bool,
    pub is_paid: bool,
    pub valid_until: Option<String>,
    pub remaining_minutes: i64,
    pub billing_mode: String,
    pub message: String,
}

impl From<AccountValidationResponse> for AccountValidationResponseVO {
    fn from(value: AccountValidationResponse) -> Self {
        Self {
            is_valid: value.is_valid,
            is_paid: value.is_paid,
            valid_until: value.valid_until.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            remaining_minutes: value.remaining_minutes,
            billing_mode: value.billing_mode,
            message: value.message,
        }
    }
}
