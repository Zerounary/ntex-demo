use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse, HttpRequest};
use ntex::util::Bytes;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;
use log::{info, error};
use rand::Rng;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use moka::future::Cache;
use crate::infrastructure::admin_config::AdminConfigStore;
use crate::interface::grpc_server::controlplane;
use tonic::transport::Channel;

use crate::application::accelerator_usecase::AcceleratorUseCase;
use crate::application::auth_usecase::AuthUseCase;
use crate::application::cdk_usecase::CdkUseCase;
use crate::application::content_usecase::ContentUseCase;
use crate::application::node_usecase::NodeUseCase;
use crate::application::ports::AuthRepository;
use crate::application::errors::{UsecaseError, RepositoryError};
use crate::domain::cdk::AccountValidationRequest;
use crate::grant_cdk_reward;
use crate::infrastructure::persistence::repositories::{
    AcceleratorRepositoryImpl, AuthRepositoryImpl, CdkRepositoryImpl, ConfigRepositoryImpl,
    NodeRepositoryImpl,
};
use crate::infrastructure::persistence::{
    accelerator_game, accelerator_game_node_binding, accelerator_node, accelerator_profile,
    acceleration_session, accelerator_user, accelerator_user_credential, accelerator_user_session,
    accelerator_user_login_log,
    accelerator_invite_reward_grant, admin_chain, admin_node_config, config_entry, user_wallet,
};
use crate::infrastructure::admin_config::ChainRouteEntry;

use super::AppState;
use super::auth::AuthedAcceleratorUser;
use super::dto::{
    AcceleratorBootstrapVO, AccountLoginRequestVO, AccountLoginResponseVO,
    AccountValidationRequestVO, AccountValidationResponseVO, CdkCodeVO, CdkGenerateRequestVO,
    CdkRedeemRequestVO, CdkRedeemResponseVO, DashboardVO, LibraryVO, NavigationVO,
    NodeRegisterRequest, ProfileSyncRequest, SettingsMetaVO, TicketRequestVO, TicketStatusQuery,
    WechatTicketVO,
    GameNodeBindingRequest, GameNodeVO,
    SessionStartRequestVO, SessionStartResponseVO, SessionStopRequestVO, SessionStopResponseVO,
    AcceleratorUserRegisterRequestVO, AcceleratorUserLoginRequestVO, AcceleratorUserLoginResponseVO,
    AcceleratorUserSendEmailCodeRequestVO,
    AcceleratorUserUpdateProfileRequestVO, AcceleratorUserChangePasswordRequestVO, UserVO,
    PagedResponseVO, SearchItemVO,
};
use super::errors::{ApiResponse, AppError, MessageResponse};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UserNotificationPayload {
    pub kind: String,
    pub message: String,
    pub ip: String,
    pub ts: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStreamQuery {
    pub token: String,
}

#[web::get("/auth/accelerator/notifications/stream")]
pub async fn accelerator_user_notifications_stream(
    state: State<AppState>,
    Query(query): Query<NotificationStreamQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id = user_id_by_token(&state, &query.token).await?;
    let rx = USER_NOTIFICATION_BUS.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(move |item| {
        match item {
            Ok(ev) if ev.user_id == user_id => {
                let json = serde_json::to_string(&ev.payload).ok()?;
                let frame = format!("data: {}\n\n", json);
                Some(Ok::<Bytes, ntex::web::Error>(Bytes::from(frame)))
            }
            _ => None,
        }
    });

    Ok(HttpResponse::Ok()
        .set_header("Content-Type", "text/event-stream")
        .set_header("Cache-Control", "no-cache")
        .set_header("Connection", "keep-alive")
        .streaming(stream))
}

#[derive(Debug, Clone)]
struct UserNotificationEvent {
    pub user_id: i64,
    pub payload: UserNotificationPayload,
}

static USER_NOTIFICATION_BUS: Lazy<broadcast::Sender<UserNotificationEvent>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(512);
    tx
});

static EMAIL_CODE_CACHE: Lazy<Cache<String, String>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(50_000)
        .time_to_live(std::time::Duration::from_secs(5 * 60))
        .build()
});

async fn send_email_code(to_email: &str, code: &str) -> Result<(), UsecaseError> {
    let body = format!("你的登录验证码是：{}（5分钟内有效）", code);
    crate::email::send_text_email(to_email, "登录验证码", &body)
        .await
        .map_err(map_email_error)
}

fn map_email_error(err: crate::email::EmailError) -> UsecaseError {
    match err {
        crate::email::EmailError::InvalidRecipient(_) =>
            UsecaseError::Validation("invalid email".to_string()),
        crate::email::EmailError::BuildEmail =>
            UsecaseError::Validation("build email failed".to_string()),
        crate::email::EmailError::MissingEnv(key) |
        crate::email::EmailError::InvalidEnv(key) =>
            UsecaseError::Repository(RepositoryError::Persistence(format!("smtp config error: {}", key))),
        crate::email::EmailError::Transport(msg) =>
            UsecaseError::Repository(RepositoryError::Persistence(msg)),
    }
}

fn generate_email_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1_000_000u32))
}

#[web::post("/auth/accelerator/email-code")]
pub async fn accelerator_user_send_email_code(
    _state: State<AppState>,
    Json(body): Json<AcceleratorUserSendEmailCodeRequestVO>,
) -> Result<HttpResponse, AppError> {
    let email = body.email.trim().to_string();
    if !email.contains('@') {
        return Ok(ApiResponse::<MessageResponse>::error(
            "BAD_REQUEST",
            "请输入正确的邮箱地址".to_string(),
        )
        .into_http(StatusCode::BAD_REQUEST));
    }

    let code = generate_email_code();
    EMAIL_CODE_CACHE.insert(email.clone(), code.clone()).await;
    send_email_code(&email, &code).await?;

    Ok(ApiResponse::success(MessageResponse {
        message: "验证码已发送".into(),
    })
    .into_http(StatusCode::OK))
}

async fn user_id_by_token(state: &AppState, token: &str) -> Result<i64, UsecaseError> {
    let now = chrono::Utc::now();
    let session = accelerator_user_session::Entity::find_by_id(token.to_string())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    let expires_at: chrono::DateTime<chrono::Utc> = session.expires_at.into();
    if expires_at <= now {
        return Err(UsecaseError::Unauthorized);
    }

    Ok(session.user_id)
}

fn extract_ip_from_headers(req: &HttpRequest) -> String {
    if let Some(v) = req.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        let ip = v.split(',').next().unwrap_or("").trim();
        if !ip.is_empty() {
            return ip.to_string();
        }
    }
    if let Some(v) = req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let ip = v.trim();
        if !ip.is_empty() {
            return ip.to_string();
        }
    }
    "unknown".to_string()
}

fn extract_user_agent(req: &HttpRequest) -> String {
    req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcceleratorInviteRewardTier {
    inviter_count: i32,
    cdk_type: String,
    num: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AcceleratorActivityConfig {
    enabled: bool,
    #[serde(default)]
    invite_rewards: Vec<AcceleratorInviteRewardTier>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserRegisterActivityConfig {
    enabled: bool,
    cdk_type: String,
    num: i64,
}

const ACCELERATOR_ACTIVITY_KEY: &str = "accelerator_activity";
const USER_REGISTER_ACTIVITY_KEY: &str = "user_register_activity";
const ENTRY_CONFIG_KEY: &str = "entry_config";

#[web::get("/entry_config")]
pub async fn entry_config(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let row = config_entry::Entity::find_by_id(ENTRY_CONFIG_KEY.to_string())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(RepositoryError::Persistence(format!(
                "query entry_config failed: {}",
                e
            )))
        })?;

    let payload = row
        .map(|m| m.payload)
        .unwrap_or_else(|| {
            json!({
                "client": {
                    "support": "--"
                },
                "business": {
                    "cooperation": "--"
                }
            })
        });

    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

fn extract_reality_client_params(
    inbound: &crate::infrastructure::admin_config::InboundConfig,
) -> Result<(String, String, String, String, String), UsecaseError> {
    let stream = inbound.stream_settings.as_ref().ok_or_else(|| {
        UsecaseError::Validation("missing streamSettings for inbound".to_string())
    })?;
    let rs = stream
        .get("realitySettings")
        .ok_or_else(|| UsecaseError::Validation("missing realitySettings".to_string()))?;

    let server_name = rs
        .get("serverName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| UsecaseError::Validation("missing realitySettings.serverName".to_string()))?;
    let public_key = rs
        .get("publicKey")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| UsecaseError::Validation("missing realitySettings.publicKey".to_string()))?;
    let short_id = rs
        .get("shortId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| UsecaseError::Validation("missing realitySettings.shortId".to_string()))?;
    let fingerprint = rs
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| UsecaseError::Validation("missing realitySettings.fingerprint".to_string()))?;
    let spider_x = rs
        .get("spiderX")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string());

    Ok((server_name, public_key, short_id, fingerprint, spider_x))
}

async fn apply_invite_rewards(
    db: &sea_orm::DatabaseConnection,
    inviter_id: i64,
    invitee_id: i64,
) -> Result<(), String> {
    let row = config_entry::Entity::find_by_id(ACCELERATOR_ACTIVITY_KEY.to_string())
        .one(db)
        .await
        .map_err(|e| format!("query accelerator_activity failed: {}", e))?;

    let cfg: AcceleratorActivityConfig = match row {
        Some(m) => serde_json::from_value(m.payload)
            .map_err(|e| format!("invalid accelerator_activity payload: {}", e))?,
        None => {
            return Ok(());
        }
    };

    if !cfg.enabled {
        return Ok(());
    }

    let inviter_count = accelerator_user::Entity::find()
        .filter(accelerator_user::Column::InviterId.eq(inviter_id))
        .count(db)
        .await
        .map_err(|e| format!("count inviter users failed: {}", e))? as i32;

    if inviter_count <= 0 {
        return Ok(());
    }

    let mut tiers = cfg
        .invite_rewards
        .into_iter()
        .filter(|t| t.inviter_count > 0)
        .collect::<Vec<_>>();
    tiers.sort_by_key(|t| t.inviter_count);

    let auth_repo = AuthRepositoryImpl::new(db);

    for tier in tiers {
        if inviter_count < tier.inviter_count {
            continue;
        }

        let already = accelerator_invite_reward_grant::Entity::find()
            .filter(accelerator_invite_reward_grant::Column::InviterId.eq(inviter_id))
            .filter(accelerator_invite_reward_grant::Column::Tier.eq(tier.inviter_count))
            .one(db)
            .await
            .map_err(|e| format!("query invite_reward_grant failed: {}", e))?
            .is_some();
        if already {
            continue;
        }

        let cdk_type = tier.cdk_type.trim().to_lowercase();
        let num = tier.num;
        if num <= 0 {
            return Err("invalid num for invite reward".to_string());
        }

        grant_cdk_reward(&auth_repo, inviter_id, &cdk_type, num).await?;

        let grant = accelerator_invite_reward_grant::ActiveModel {
            id: sea_orm::NotSet,
            inviter_id: Set(inviter_id),
            invitee_id: Set(invitee_id),
            tier: Set(tier.inviter_count),
            cdk_type: Set(cdk_type),
            num: Set(num),
            bandwidth_mbps: Set(None),
            granted_at: Set(chrono::Utc::now().into()),
        };
        grant
            .insert(db)
            .await
            .map_err(|e| format!("insert invite_reward_grant failed: {}", e))?;
    }

    Ok(())
}

async fn apply_user_register_activity_reward(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
) -> Result<(), String> {
    let row = config_entry::Entity::find_by_id(USER_REGISTER_ACTIVITY_KEY.to_string())
        .one(db)
        .await
        .map_err(|e| format!("query user_register_activity failed: {}", e))?;

    let cfg: UserRegisterActivityConfig = match row {
        Some(m) => serde_json::from_value(m.payload)
            .map_err(|e| format!("invalid user_register_activity payload: {}", e))?,
        None => {
            return Ok(());
        }
    };

    if !cfg.enabled {
        return Ok(());
    }

    let num = cfg.num;
    if num <= 0 {
        return Ok(());
    }

    let auth_repo = AuthRepositoryImpl::new(db);
    grant_cdk_reward(&auth_repo, user_id, &cfg.cdk_type, num).await?;

    Ok(())
}

#[derive(Debug, Deserialize, Default)]
struct AdminUserDTO {
    pub id: u64,
    pub uuid: String,
    pub st: u64,
    pub dt: u64,
}


async fn resolve_chain_exit_node_id(
    db: &sea_orm::DatabaseConnection,
    chain_id: i64,
) -> Result<u64, UsecaseError> {
    let chain = admin_chain::Entity::find_by_id(chain_id)
        .one(db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or_else(|| UsecaseError::NotFound("chain"))?;

    let mut routes: Vec<ChainRouteEntry> = serde_json::from_value(chain.routes).map_err(|e| {
        UsecaseError::Validation(format!("invalid chain routes json: {}", e))
    })?;

    if routes.is_empty() {
        return Err(UsecaseError::Validation("chain has no routes".to_string()));
    }
    routes.sort_by_key(|r| r.order);
    let last_to = routes
        .last()
        .map(|r| r.to_node_id)
        .ok_or_else(|| UsecaseError::Validation("chain routes empty".to_string()))?;
    Ok(last_to)
}

async fn resolve_chain_entry_node_id(
    db: &sea_orm::DatabaseConnection,
    chain_id: i64,
) -> Result<u64, UsecaseError> {
    let chain = admin_chain::Entity::find_by_id(chain_id)
        .one(db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or_else(|| UsecaseError::NotFound("chain"))?;

    let mut routes: Vec<ChainRouteEntry> = serde_json::from_value(chain.routes).map_err(|e| {
        UsecaseError::Validation(format!("invalid chain routes json: {}", e))
    })?;

    if routes.is_empty() {
        return Err(UsecaseError::Validation("chain has no routes".to_string()));
    }
    routes.sort_by_key(|r| r.order);
    let first_from = routes
        .first()
        .map(|r| r.from_node_id)
        .ok_or_else(|| UsecaseError::Validation("chain routes empty".to_string()))?;
    Ok(first_from)
}

async fn resolve_inbound_port_by_tag(
    config: &crate::infrastructure::admin_config::AdminConfigStore,
    node_id: u64,
    inbound_tag: &str,
) -> Result<i32, UsecaseError> {
    let inbounds = config
        .get_inbounds(node_id)
        .await
        .map_err(|e| UsecaseError::Validation(format!("get_inbounds failed: {}", e)))?;
    let port = inbounds
        .into_iter()
        .find(|i| i.tag == inbound_tag)
        .map(|i| i.port)
        .filter(|p| *p > 0)
        .ok_or_else(|| {
            UsecaseError::Validation(format!(
                "node_id={} inbound {} not found or invalid port",
                node_id, inbound_tag
            ))
        })?;
    Ok(port)
}

#[derive(Debug, Deserialize)]
struct AdminApiResponse<T> {
    pub msg: String,
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct AdminAddUserRequest {
    pub uuid: String,
    pub st: u64,
    pub dt: u64,
}

#[derive(Debug, serde::Serialize)]
struct AdminAddMappingRequest {
    pub uuid: String,
    pub outbound_tag: String,
}

#[derive(Debug, Deserialize)]
struct AdminOutboundConfigDTO {
    pub tag: String,
}

#[derive(Debug, Deserialize, Default)]
struct AdminOutboundsDataDTO {
    #[serde(default)]
    pub outbounds: Vec<AdminOutboundConfigDTO>,
}

fn main_admin_base_url() -> String {
    env::var("MAIN_ADMIN_BASE_URL")
        .or_else(|_| env::var("ADMIN_BASE_URL"))
        .unwrap_or_else(|_| "http://127.0.0.1:667".to_string())
}

fn main_admin_token() -> Option<String> {
    env::var("MAIN_ADMIN_TOKEN")
        .or_else(|_| env::var("ADMIN_TOKEN"))
        .ok()
        .filter(|v| !v.is_empty())
}

fn use_main_admin_http() -> bool {
    env::var("USE_MAIN_ADMIN_HTTP")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn main_grpc_channel(state: &AppState) -> Result<Channel, UsecaseError> {
    state
        .main_grpc
        .clone()
        .ok_or_else(|| UsecaseError::Validation("MAIN_GRPC_ADDR not configured or connect failed".to_string()))
}

async fn local_add_user(
    config: &AdminConfigStore,
    node_id: u64,
    uuid: String,
    st: u64,
    dt: u64,
) -> Result<AdminUserDTO, UsecaseError> {
    let _ = (config, node_id, uuid, st, dt);
    Err(UsecaseError::Validation("local_add_user is disabled; use control-plane gRPC".to_string()))
}

async fn local_delete_user(
    config: &AdminConfigStore,
    node_id: u64,
    admin_user_id: u64,
) -> Result<(), UsecaseError> {
    let _ = (config, node_id, admin_user_id);
    Err(UsecaseError::Validation("local_delete_user is disabled; use control-plane gRPC".to_string()))
}

async fn local_add_mapping(
    config: &AdminConfigStore,
    node_id: u64,
    uuid: String,
    outbound_tag: String,
) -> Result<(), UsecaseError> {
    let _ = (config, node_id, uuid, outbound_tag);
    Err(UsecaseError::Validation("local_add_mapping is disabled; use control-plane gRPC".to_string()))
}

async fn local_get_outbound_tags(
    config: &AdminConfigStore,
    node_id: u64,
) -> Result<Vec<String>, UsecaseError> {
    let _ = (config, node_id);
    Err(UsecaseError::Validation("local_get_outbound_tags is disabled; use control-plane gRPC".to_string()))
}

async fn cp_add_user(
    state: &AppState,
    node_id: u64,
    uuid: String,
    st: u64,
    dt: u64,
    sync: bool,
    sync_timeout_secs: u64,
) -> Result<AdminUserDTO, UsecaseError> {
    let mut client = controlplane::control_plane_client::ControlPlaneClient::new(main_grpc_channel(state)?);
    let rsp = client
        .add_user(controlplane::AddUserRequest {
            node_id,
            uuid,
            st,
            dt,
            sync,
            sync_timeout_secs,
        })
        .await
        .map_err(|e| UsecaseError::Validation(format!("controlplane add_user failed: {}", e)))?
        .into_inner();
    Ok(AdminUserDTO {
        id: rsp.user_id,
        uuid: rsp.uuid,
        st: rsp.st,
        dt: rsp.dt,
    })
}

async fn cp_add_mapping(
    state: &AppState,
    node_id: u64,
    uuid: String,
    outbound_tag: String,
    sync: bool,
    sync_timeout_secs: u64,
) -> Result<(), UsecaseError> {
    let mut client = controlplane::control_plane_client::ControlPlaneClient::new(main_grpc_channel(state)?);
    let _ = client
        .add_mapping(controlplane::AddMappingRequest {
            node_id,
            uuid,
            outbound_tag,
            sync,
            sync_timeout_secs,
        })
        .await
        .map_err(|e| UsecaseError::Validation(format!("controlplane add_mapping failed: {}", e)))?
        .into_inner();
    Ok(())
}

async fn cp_delete_user(
    state: &AppState,
    node_id: u64,
    user_id: u64,
    sync: bool,
    sync_timeout_secs: u64,
) -> Result<(), UsecaseError> {
    let mut client = controlplane::control_plane_client::ControlPlaneClient::new(main_grpc_channel(state)?);
    let _ = client
        .delete_user(controlplane::DeleteUserRequest {
            node_id,
            user_id,
            sync,
            sync_timeout_secs,
        })
        .await
        .map_err(|e| UsecaseError::Validation(format!("controlplane delete_user failed: {}", e)))?
        .into_inner();
    Ok(())
}

async fn cp_get_outbound_tags(state: &AppState, node_id: u64) -> Result<Vec<String>, UsecaseError> {
    let mut client = controlplane::control_plane_client::ControlPlaneClient::new(main_grpc_channel(state)?);
    let rsp = client
        .get_outbound_tags(controlplane::GetOutboundTagsRequest { node_id })
        .await
        .map_err(|e| UsecaseError::Validation(format!("controlplane get_outbound_tags failed: {}", e)))?
        .into_inner();
    Ok(rsp.tags)
}

async fn main_admin_add_user(
    node_id: u64,
    uuid: String,
    st: u64,
    dt: u64,
) -> Result<AdminUserDTO, UsecaseError> {
    let base = main_admin_base_url();
    info!(
        "[session_start] -> POST {}/api/admin/user?node_id={} uuid={} st={} dt={}",
        base, node_id, uuid, st, dt
    );
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}/api/admin/user", base))
        .query(&[("node_id", node_id)])
        .json(&AdminAddUserRequest {
            uuid: uuid.clone(),
            st,
            dt,
        });
    if let Some(token) = main_admin_token() {
        req = req.header("X-Admin-Token", token);
    }
    let resp = req.send().await.map_err(|e| {
        error!(
            "[session_start] admin add_user request failed: node_id={}, uuid={}, err={}",
            node_id, uuid, e
        );
        UsecaseError::Validation(format!("main_admin add_user request failed: {}", e))
    })?;
    let status = resp.status();
    let payload = resp
        .json::<AdminApiResponse<AdminUserDTO>>()
        .await
        .map_err(|e| UsecaseError::Validation(format!("main_admin add_user invalid response: {}", e)))?;

    if !status.is_success() {
        error!(
            "[session_start] admin add_user failed: status={} msg={:?} error={:?}",
            status, payload.message, payload.error
        );
        return Err(UsecaseError::Validation(format!(
            "main_admin add_user failed: status={}, msg={:?}, error={:?}",
            status,
            payload.message,
            payload.error
        )));
    }
    if payload.msg != "ok" {
        error!(
            "[session_start] admin add_user failed: msg={} error={:?}",
            payload.msg, payload.error
        );
        return Err(UsecaseError::Validation(format!(
            "main_admin add_user failed: msg={}, error={:?}",
            payload.msg, payload.error
        )));
    }
    let data = payload.data.ok_or_else(|| {
        error!("[session_start] admin add_user missing data");
        UsecaseError::Validation("main_admin add_user missing data".to_string())
    })?;

    Ok(data)
}

async fn main_admin_add_mapping(
    node_id: u64,
    uuid: String,
    outbound_tag: String,
) -> Result<(), UsecaseError> {
    let base = main_admin_base_url();
    info!(
        "[session_start] -> POST {}/api/admin/mapping?node_id={} uuid={} outbound_tag={}",
        base, node_id, uuid, outbound_tag
    );
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}/api/admin/mapping", base))
        .query(&[("node_id", node_id)])
        .json(&AdminAddMappingRequest {
            uuid: uuid.clone(),
            outbound_tag: outbound_tag.clone(),
        });
    if let Some(token) = main_admin_token() {
        req = req.header("X-Admin-Token", token);
    }
    let resp = req.send().await.map_err(|e| {
        error!(
            "[session_start] admin add_mapping request failed: node_id={} uuid={} err={}",
            node_id, uuid, e
        );
        UsecaseError::Validation(format!("main_admin add_mapping request failed: {}", e))
    })?;
    let status = resp.status();
    let payload = resp
        .json::<AdminApiResponse<serde_json::Value>>()
        .await
        .map_err(|e| UsecaseError::Validation(format!("main_admin add_mapping invalid response: {}", e)))?;

    if !status.is_success() || payload.msg != "ok" {
        error!(
            "[session_start] admin add_mapping failed: status={} msg={} error={:?}",
            status, payload.msg, payload.error
        );
        return Err(UsecaseError::Validation(format!(
            "main_admin add_mapping failed: status={}, msg={}, error={:?}",
            status, payload.msg, payload.error
        )));
    }
    info!(
        "[session_start] <- admin add_mapping success: node_id={} uuid={} outbound_tag={}",
        node_id, uuid, outbound_tag
    );
    Ok(())
}

async fn main_admin_get_outbound_tags(node_id: u64) -> Result<Vec<String>, UsecaseError> {
    let base = main_admin_base_url();
    let client = reqwest::Client::new();
    let mut req = client
        .get(format!("{}/api/admin/query", base))
        .query(&[("act", "outbound"), ("node_id", &node_id.to_string())]);
    if let Some(token) = main_admin_token() {
        req = req.header("X-Admin-Token", token);
    }
    let resp = req.send().await.map_err(|e| {
        error!(
            "[session_start] admin query outbounds request failed: node_id={}, err={}",
            node_id, e
        );
        UsecaseError::Validation(format!("main_admin query outbounds request failed: {}", e))
    })?;
    let status = resp.status();
    let payload = resp
        .json::<AdminApiResponse<AdminOutboundsDataDTO>>()
        .await
        .map_err(|e| UsecaseError::Validation(format!("main_admin query outbounds invalid response: {}", e)))?;

    if !status.is_success() || payload.msg != "ok" {
        error!(
            "[session_start] admin query outbounds failed: status={} msg={} error={:?}",
            status, payload.msg, payload.error
        );
        return Err(UsecaseError::Validation(format!(
            "main_admin query outbounds failed: status={}, msg={}, error={:?}",
            status, payload.msg, payload.error
        )));
    }

    let data = payload.data.unwrap_or_default();
    Ok(data.outbounds.into_iter().map(|o| o.tag).collect())
}

async fn main_admin_delete_user(node_id: u64, admin_user_id: u64) -> Result<(), UsecaseError> {
    let base = main_admin_base_url();
    info!(
        "[session] -> DELETE {}/api/admin/user/{}?node_id={}",
        base, admin_user_id, node_id
    );
    let client = reqwest::Client::new();
    let mut req = client
        .delete(format!("{}/api/admin/user/{}", base, admin_user_id))
        .query(&[("node_id", node_id)]);
    if let Some(token) = main_admin_token() {
        req = req.header("X-Admin-Token", token);
    }
    let resp = req.send().await.map_err(|e| {
        error!(
            "[session] admin delete_user request failed: node_id={} admin_user_id={} err={}",
            node_id, admin_user_id, e
        );
        UsecaseError::Validation(format!("main_admin delete_user request failed: {}", e))
    })?;
    let status = resp.status();
    let payload = resp
        .json::<AdminApiResponse<serde_json::Value>>()
        .await
        .map_err(|e| UsecaseError::Validation(format!("main_admin delete_user invalid response: {}", e)))?;
    if !status.is_success() || payload.msg != "ok" {
        error!(
            "[session] admin delete_user failed: status={} msg={} error={:?}",
            status, payload.msg, payload.error
        );
        return Err(UsecaseError::Validation(format!(
            "main_admin delete_user failed: status={}, msg={}, error={:?}",
            status, payload.msg, payload.error
        )));
    }
    info!(
        "[session] <- admin delete_user success: node_id={} admin_user_id={}",
        node_id, admin_user_id
    );
    Ok(())
}

fn hash_password(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[web::get("/accelerator/bootstrap")]
pub async fn accelerator_bootstrap(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = AcceleratorRepositoryImpl::new(&state.db);
    let node_repo = NodeRepositoryImpl::new(&state.db);
    let usecase = AcceleratorUseCase::new(repo, node_repo);
    let payload = usecase.bootstrap().await?;
    Ok(ApiResponse::success(AcceleratorBootstrapVO::from(payload)).into_http(StatusCode::OK))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GamesQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub keyword: Option<String>,
    pub region: Option<String>,
    pub status: Option<String>,
}

#[web::get("/accelerator/games")]
pub async fn accelerator_games(
    state: State<AppState>,
    Query(query): Query<GamesQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(12).clamp(1, 100);

    let mut cond = Condition::all();
    if let Some(keyword) = query.keyword.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(
            Condition::any()
                .add(accelerator_game::Column::Name.contains(keyword))
                .add(accelerator_game::Column::Region.contains(keyword))
                .add(accelerator_game::Column::Status.contains(keyword))
                .add(accelerator_game::Column::ProcessName.contains(keyword)),
        );
    }
    if let Some(region) = query.region.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(accelerator_game::Column::Region.contains(region));
    }
    if let Some(status) = query.status.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(accelerator_game::Column::Status.eq(status));
    }

    let base = accelerator_game::Entity::find()
        .filter(cond)
        .order_by_desc(accelerator_game::Column::Id);

    let total = base
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    let items = base
        .paginate(&state.db, page_size)
        .fetch_page(page - 1)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .into_iter()
        .map(|m| crate::interface::web::dto::GameVO {
            id: m.id,
            name: m.name,
            icon: m.icon,
            status: m.status,
            ping: m.ping,
        })
        .collect();

    Ok(ApiResponse::success(PagedResponseVO {
        items,
        page,
        page_size,
        total,
    })
    .into_http(StatusCode::OK))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub keyword: Option<String>,
}

#[web::get("/dashboard/announcements")]
pub async fn dashboard_announcements(
    state: State<AppState>,
    Query(query): Query<AnnouncementsQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);

    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: DashboardVO = usecase.dashboard().await?;

    let keyword = query.keyword.unwrap_or_default();
    let keyword = keyword.trim().to_lowercase();

    let mut items: Vec<crate::domain::content::Announcement> = payload
        .announcements
        .into_iter()
        .filter(|a| {
            if keyword.is_empty() {
                return true;
            }
            a.title.to_lowercase().contains(&keyword)
        })
        .collect();

    items.sort_by(|a, b| b.id.cmp(&a.id));
    let total = items.len() as u64;
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(items.len());
    let page_items = if start >= items.len() {
        vec![]
    } else {
        items[start..end].to_vec()
    };

    Ok(ApiResponse::success(PagedResponseVO {
        items: page_items,
        page,
        page_size,
        total,
    })
    .into_http(StatusCode::OK))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LibraryGamesQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub region: Option<String>,
    pub status: Option<String>,
}

#[web::get("/library/games")]
pub async fn library_games(
    state: State<AppState>,
    Query(query): Query<LibraryGamesQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(24).clamp(1, 100);

    let mut cond = Condition::all();
    if let Some(keyword) = query.keyword.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(
            Condition::any()
                .add(accelerator_game::Column::Name.contains(keyword))
                .add(accelerator_game::Column::Region.contains(keyword))
                .add(accelerator_game::Column::Status.contains(keyword))
                .add(accelerator_game::Column::ProcessName.contains(keyword)),
        );
    }
    if let Some(region) = query.region.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(accelerator_game::Column::Region.contains(region));
    }
    if let Some(status) = query.status.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cond = cond.add(accelerator_game::Column::Status.eq(status));
    }
    if let Some(category) = query.category.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if category != "全部" {
            cond = cond.add(
                Condition::any()
                    .add(accelerator_game::Column::Region.contains(category))
                    .add(accelerator_game::Column::Name.contains(category)),
            );
        }
    }

    let base = accelerator_game::Entity::find()
        .filter(cond)
        .order_by_desc(accelerator_game::Column::Id);

    let total = base
        .clone()
        .count(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    let items = base
        .paginate(&state.db, page_size)
        .fetch_page(page - 1)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .into_iter()
        .map(|m| crate::interface::web::dto::GameVO {
            id: m.id,
            name: m.name,
            icon: m.icon,
            status: m.status,
            ping: m.ping,
        })
        .collect();

    Ok(ApiResponse::success(PagedResponseVO {
        items,
        page,
        page_size,
        total,
    })
    .into_http(StatusCode::OK))
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub keyword: String,
    pub limit: Option<u64>,
}

#[web::get("/accelerator/search")]
pub async fn accelerator_search(
    state: State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let keyword = query.keyword.trim();
    if keyword.is_empty() {
        return Ok(ApiResponse::success(Vec::<SearchItemVO>::new()).into_http(StatusCode::OK));
    }
    let limit = query.limit.unwrap_or(10).clamp(1, 50) as usize;

    let mut results: Vec<SearchItemVO> = Vec::new();

    // games
    let games = accelerator_game::Entity::find()
        .filter(
            Condition::any()
                .add(accelerator_game::Column::Name.contains(keyword))
                .add(accelerator_game::Column::Region.contains(keyword)),
        )
        .order_by_desc(accelerator_game::Column::Id)
        .limit(limit as u64)
        .all(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    for g in games {
        results.push(SearchItemVO {
            kind: "game".to_string(),
            id: g.id.clone(),
            label: format!("{} · {}", g.name, g.region),
            game_id: Some(g.id),
            node_id: None,
            region: Some(g.region),
            mode: None,
        });
        if results.len() >= limit {
            break;
        }
    }

    if results.len() < limit {
        let profiles = accelerator_profile::Entity::find()
            .filter(
                Condition::any()
                    .add(accelerator_profile::Column::DisplayName.contains(keyword))
                    .add(accelerator_profile::Column::NodeId.contains(keyword)),
            )
            .order_by_desc(accelerator_profile::Column::Id)
            .limit((limit - results.len()) as u64)
            .all(&state.db)
            .await
            .map_err(|e| {
                UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                    e.to_string(),
                ))
            })?;

        // preload games/nodes for enrichment
        let game_models = accelerator_game::Entity::find()
            .all(&state.db)
            .await
            .map_err(|e| {
                UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                    e.to_string(),
                ))
            })?;
        let game_map: HashMap<String, accelerator_game::Model> =
            game_models.into_iter().map(|m| (m.id.clone(), m)).collect();

        let node_models = accelerator_node::Entity::find()
            .all(&state.db)
            .await
            .map_err(|e| {
                UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                    e.to_string(),
                ))
            })?;
        let node_map: HashMap<String, accelerator_node::Model> =
            node_models.into_iter().map(|m| (m.id.clone(), m)).collect();

        for p in profiles {
            let game = game_map.get(&p.game_id);
            let node = node_map.get(&p.node_id);
            results.push(SearchItemVO {
                kind: "profile".to_string(),
                id: p.id.clone(),
                label: match game {
                    Some(g) => format!("{} · {}", g.name, p.display_name),
                    None => p.display_name.clone(),
                },
                game_id: Some(p.game_id.clone()),
                node_id: Some(p.node_id.clone()),
                region: game.map(|g| g.region.clone()),
                mode: node.map(|n| n.mode.clone()),
            });
            if results.len() >= limit {
                break;
            }
        }
    }

    Ok(ApiResponse::success(results).into_http(StatusCode::OK))
}

#[web::post("/accelerator/session/start")]
pub async fn session_start(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<SessionStartRequestVO>,
) -> Result<HttpResponse, AppError> {
    let user_id = user.user.id;

    let binding = accelerator_game_node_binding::Entity::find_by_id(body.binding_id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or_else(|| UsecaseError::NotFound("binding"))?;

    let game_id = binding.game_id.clone();

    let (primary_node_id, tcp_entry_node_id, udp_entry_node_id, tcp_exit_node_id, udp_exit_node_id) = match binding.r#type.as_str() {
        "node" => {
            let node_id_str = binding
                .node_id
                .clone()
                .ok_or_else(|| UsecaseError::Validation("binding.node_id is required for type=node".to_string()))?;
            let node_id = node_id_str.parse::<u64>().map_err(|_| {
                UsecaseError::Validation(format!("invalid binding.node_id: {}", node_id_str))
            })?;
            (node_id, None, None, None, None)
        }
        "chain" => {
            let tcp_entry = match binding.tcp_chain_id {
                Some(tcp_chain_id) => Some(resolve_chain_entry_node_id(&state.db, tcp_chain_id).await?),
                None => None,
            };
            let udp_entry = match binding.udp_chain_id {
                Some(udp_chain_id) => Some(resolve_chain_entry_node_id(&state.db, udp_chain_id).await?),
                None => None,
            };
            let tcp_exit = match binding.tcp_chain_id {
                Some(tcp_chain_id) => Some(resolve_chain_exit_node_id(&state.db, tcp_chain_id).await?),
                None => None,
            };
            let udp_exit = match binding.udp_chain_id {
                Some(udp_chain_id) => Some(resolve_chain_exit_node_id(&state.db, udp_chain_id).await?),
                None => None,
            };

            let primary = tcp_entry.or(udp_entry).ok_or_else(|| {
                UsecaseError::Validation(
                    "binding.tcp_chain_id or binding.udp_chain_id is required for type=chain".to_string(),
                )
            })?;

            (primary, tcp_entry, udp_entry, tcp_exit, udp_exit)
        }
        other => {
            return Err(
                UsecaseError::Validation(format!("invalid binding.type: {}", other)).into(),
            );
        }
    };

    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);

    let validation = usecase
        .validate_account(AccountValidationRequest { user_id: user_id.clone() })
        .await?;

    if !validation.is_valid {
        return Ok(ApiResponse::<MessageResponse>::error(
            "ACCOUNT_VALIDATION_FAILED",
            validation.message,
        )
        .into_http(StatusCode::FORBIDDEN));
    }

    if validation.billing_mode == "minute" && validation.remaining_minutes < 1 {
        return Ok(ApiResponse::<MessageResponse>::error(
            "INSUFFICIENT_BALANCE",
            "remaining minutes is insufficient".to_string(),
        )
        .into_http(StatusCode::FORBIDDEN));
    }

    let offline_after_seconds = env::var("NODE_OFFLINE_AFTER_SECONDS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(60)
        .max(1);
    let offline_threshold = chrono::Utc::now() - chrono::Duration::seconds(offline_after_seconds);

    async fn ensure_node_online(
        db: &sea_orm::DatabaseConnection,
        node_id: u64,
        offline_threshold: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), UsecaseError> {
        let node_cfg = admin_node_config::Entity::find_by_id(node_id)
            .one(db)
            .await
            .map_err(|e| {
                UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                    e.to_string(),
                ))
            })?;

        let node_cfg = node_cfg.ok_or_else(|| UsecaseError::NotFound("node"))?;
        let is_online = node_cfg.is_online;
        let last_seen_at = node_cfg.last_seen_at;
        let is_fresh = last_seen_at.map(|ts| ts >= offline_threshold).unwrap_or(false);
        if !is_online || !is_fresh {
            return Err(UsecaseError::Validation(format!("node {} is offline", node_id)));
        }
        Ok(())
    }

    // 节点离线时禁止启动（避免误停已有 active session）
    if let Err(e) = ensure_node_online(&state.db, primary_node_id, offline_threshold).await {
        return Ok(ApiResponse::<MessageResponse>::error("NODE_OFFLINE", e.to_string())
            .into_http(StatusCode::SERVICE_UNAVAILABLE));
    }
    if let Some(tcp_node_id) = tcp_entry_node_id {
        if let Err(e) = ensure_node_online(&state.db, tcp_node_id, offline_threshold).await {
            return Ok(ApiResponse::<MessageResponse>::error("NODE_OFFLINE", e.to_string())
                .into_http(StatusCode::SERVICE_UNAVAILABLE));
        }
    }
    if let Some(udp_node_id) = udp_entry_node_id {
        if let Err(e) = ensure_node_online(&state.db, udp_node_id, offline_threshold).await {
            return Ok(ApiResponse::<MessageResponse>::error("NODE_OFFLINE", e.to_string())
                .into_http(StatusCode::SERVICE_UNAVAILABLE));
        }
    }

    if let Some(tcp_node_id) = tcp_exit_node_id {
        if let Err(e) = ensure_node_online(&state.db, tcp_node_id, offline_threshold).await {
            return Ok(ApiResponse::<MessageResponse>::error("NODE_OFFLINE", e.to_string())
                .into_http(StatusCode::SERVICE_UNAVAILABLE));
        }
    }
    if let Some(udp_node_id) = udp_exit_node_id {
        if let Err(e) = ensure_node_online(&state.db, udp_node_id, offline_threshold).await {
            return Ok(ApiResponse::<MessageResponse>::error("NODE_OFFLINE", e.to_string())
                .into_http(StatusCode::SERVICE_UNAVAILABLE));
        }
    }

    // 单用户同一时刻只允许一个 active 会话：存在则先 stop（best effort）
    if let Ok(Some(existing)) = acceleration_session::Entity::find()
        .filter(acceleration_session::Column::UserId.eq(user_id))
        .filter(acceleration_session::Column::Status.eq("active"))
        .order_by_desc(acceleration_session::Column::StartedAt)
        .one(&state.db)
        .await
    {
        info!(
            "[session_start] best-effort stop existing session_id={} node_id={} ",
            existing.session_id, existing.node_id
        );
        if existing.admin_user_id > 0 {
            if use_main_admin_http() {
                let _ = main_admin_delete_user(existing.node_id, existing.admin_user_id).await;
            } else {
                let _ = cp_delete_user(&*state, existing.node_id, existing.admin_user_id, false, 0).await;
            }
        }
        if let (Some(node_id), Some(admin_user_id)) = (existing.tcp_node_id, existing.tcp_admin_user_id) {
            if admin_user_id > 0 {
                if use_main_admin_http() {
                    let _ = main_admin_delete_user(node_id, admin_user_id).await;
                } else {
                    let _ = cp_delete_user(&*state, node_id, admin_user_id, false, 0).await;
                }
            }
        }
        if let (Some(node_id), Some(admin_user_id)) = (existing.udp_node_id, existing.udp_admin_user_id) {
            if admin_user_id > 0 {
                if use_main_admin_http() {
                    let _ = main_admin_delete_user(node_id, admin_user_id).await;
                } else {
                    let _ = cp_delete_user(&*state, node_id, admin_user_id, false, 0).await;
                }
            }
        }

        let mut active: acceleration_session::ActiveModel = existing.into();
        active.status = Set("stopped".to_string());
        active.ended_at = Set(Some(chrono::Utc::now().into()));
        active.updated_at = Set(chrono::Utc::now().into());
        let _ = active.update(&state.db).await;
    }

    let uuid = uuid::Uuid::new_v4().to_string();
    let st = {
        let wallet = user_wallet::Entity::find_by_id(user_id.clone())
            .one(&state.db)
            .await
            .map_err(|e| {
                UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                    e.to_string(),
                ))
            })?;
        let bw = wallet.and_then(|w| w.bandwidth).unwrap_or(3);
        if bw > 0 { bw as u64 } else { 3u64 }
    };

    async fn select_outbound_tag(
        state: &AppState,
        node_id: u64,
        desired_outbound_tag: String,
    ) -> Result<String, UsecaseError> {
        let outbounds = if use_main_admin_http() {
            main_admin_get_outbound_tags(node_id).await?
        } else {
            cp_get_outbound_tags(state, node_id).await?
        };
        let mut candidates: Vec<String> = outbounds
            .into_iter()
            .filter(|t| t != "block")
            .collect();
        candidates.sort();

        let mapped_outbound_tag = if candidates.iter().any(|t| t == &desired_outbound_tag) {
            desired_outbound_tag.clone()
        } else {
            let active_sessions = acceleration_session::Entity::find()
                .filter(acceleration_session::Column::NodeId.eq(node_id))
                .filter(acceleration_session::Column::Status.eq("active"))
                .all(&state.db)
                .await
                .map_err(|e| {
                    UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                        e.to_string(),
                    ))
                })?;

            let mut counts: HashMap<String, u64> = HashMap::new();
            for s in active_sessions {
                *counts.entry(s.outbound_tag).or_insert(0) += 1;
            }

            if candidates.is_empty() {
                return Err(UsecaseError::Validation(format!(
                    "no available outbound tag for node_id={}",
                    node_id
                )));
            }

            let has_only_direct = !candidates.iter().any(|t| t != "direct");
            if has_only_direct {
                candidates
                    .iter()
                    .find(|t| *t == "direct")
                    .cloned()
                    .ok_or_else(|| {
                        UsecaseError::Validation(format!(
                            "no available outbound tag for node_id={} (only direct expected)",
                            node_id
                        ))
                    })?
            } else {
                let mut best_tag: Option<String> = None;
                let mut best_count: u64 = u64::MAX;
                for t in candidates.iter().filter(|t| *t != "direct") {
                    let c = counts.get(t).copied().unwrap_or(0);
                    if c < best_count {
                        best_count = c;
                        best_tag = Some(t.clone());
                    }
                }

                best_tag.ok_or_else(|| {
                    UsecaseError::Validation(format!(
                        "no available non-direct outbound tag for node_id={}",
                        node_id
                    ))
                })?
            }
        };

        Ok(mapped_outbound_tag)
    }

    let mut primary_admin_user_id: u64 = 0;
    let mut primary_admin_uuid: String = uuid.clone();
    let mut primary_outbound_tag: String = "".to_string();

    let mut tcp_admin_user_id: Option<u64> = None;
    let mut tcp_outbound_tag: Option<String> = None;
    let mut udp_admin_user_id: Option<u64> = None;
    let mut udp_outbound_tag: Option<String> = None;

    match binding.r#type.as_str() {
        "node" => {
            let desired_outbound_tag = format!("accel_{}_{}_{}", user_id, game_id, primary_node_id);
            let admin_user = if use_main_admin_http() {
                main_admin_add_user(primary_node_id, uuid.clone(), st, 0).await?
            } else {
                let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(8);
                cp_add_user(&*state, primary_node_id, uuid.clone(), st, 0, true, sync_timeout).await?
            };
            primary_admin_user_id = admin_user.id;
            primary_admin_uuid = admin_user.uuid;
            primary_outbound_tag = select_outbound_tag(&*state, primary_node_id, desired_outbound_tag).await?;

            info!(
                "[session_start] selected outbound_tag: node_id={} selected={} ",
                primary_node_id, primary_outbound_tag
            );

            if use_main_admin_http() {
                main_admin_add_mapping(primary_node_id, primary_admin_uuid.clone(), primary_outbound_tag.clone()).await?;
            } else {
                let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(8);
                cp_add_mapping(&*state, primary_node_id, primary_admin_uuid.clone(), primary_outbound_tag.clone(), true, sync_timeout).await?;
            }
        }
        "chain" => {
            if let Some(tcp_node_id) = tcp_exit_node_id {
                let desired_outbound_tag = format!("accel_{}_{}_{}_tcp", user_id, game_id, tcp_node_id);
                let admin_user = if use_main_admin_http() {
                    main_admin_add_user(tcp_node_id, uuid.clone(), st, 0).await?
                } else {
                    let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(8);
                    cp_add_user(&*state, tcp_node_id, uuid.clone(), st, 0, true, sync_timeout).await?
                };
                tcp_admin_user_id = Some(admin_user.id);
                primary_admin_uuid = admin_user.uuid;
                let tag = select_outbound_tag(&*state, tcp_node_id, desired_outbound_tag).await?;
                if use_main_admin_http() {
                    main_admin_add_mapping(tcp_node_id, primary_admin_uuid.clone(), tag.clone()).await?;
                } else {
                    let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(8);
                    cp_add_mapping(&*state, tcp_node_id, primary_admin_uuid.clone(), tag.clone(), true, sync_timeout).await?;
                }
                tcp_outbound_tag = Some(tag);
            }
            if let Some(udp_node_id) = udp_exit_node_id {
                let desired_outbound_tag = format!("accel_{}_{}_{}_udp", user_id, game_id, udp_node_id);
                let admin_user = if use_main_admin_http() {
                    main_admin_add_user(udp_node_id, uuid.clone(), st, 0).await?
                } else {
                    let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(8);
                    cp_add_user(&*state, udp_node_id, uuid.clone(), st, 0, true, sync_timeout).await?
                };
                udp_admin_user_id = Some(admin_user.id);
                primary_admin_uuid = admin_user.uuid;
                let tag = select_outbound_tag(&*state, udp_node_id, desired_outbound_tag).await?;
                if use_main_admin_http() {
                    main_admin_add_mapping(udp_node_id, primary_admin_uuid.clone(), tag.clone()).await?;
                } else {
                    let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(8);
                    cp_add_mapping(&*state, udp_node_id, primary_admin_uuid.clone(), tag.clone(), true, sync_timeout).await?;
                }
                udp_outbound_tag = Some(tag);
            }

            // primary fields pick tcp first (if exists) else udp
            if let (Some(node_id), Some(admin_id), Some(tag)) = (tcp_exit_node_id, tcp_admin_user_id, tcp_outbound_tag.clone()) {
                primary_admin_user_id = admin_id;
                primary_outbound_tag = tag;
            } else if let (Some(node_id), Some(admin_id), Some(tag)) = (udp_exit_node_id, udp_admin_user_id, udp_outbound_tag.clone()) {
                primary_admin_user_id = admin_id;
                primary_outbound_tag = tag;
            } else {
                return Err(UsecaseError::Validation(
                    "missing tcp_chain_id/udp_chain_id for chain binding".to_string(),
                )
                .into());
            }
        }
        _ => {}
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let bill_type = validation.billing_mode.clone();

    let game_id_clone = game_id.clone();
    let model = acceleration_session::ActiveModel {
        session_id: Set(session_id.clone()),
        user_id: Set(user_id.clone()),
        game_id: Set(game_id_clone.clone()),
        node_id: Set(primary_node_id),
        admin_user_id: Set(primary_admin_user_id),
        uuid: Set(primary_admin_uuid.clone()),
        outbound_tag: Set(primary_outbound_tag.clone()),
        tcp_node_id: Set(tcp_exit_node_id),
        tcp_admin_user_id: Set(tcp_admin_user_id),
        tcp_outbound_tag: Set(tcp_outbound_tag.clone()),
        udp_node_id: Set(udp_exit_node_id),
        udp_admin_user_id: Set(udp_admin_user_id),
        udp_outbound_tag: Set(udp_outbound_tag.clone()),
        status: Set("active".to_string()),
        bill_type: Set(bill_type.clone()),
        started_at: Set(now.into()),
        last_activity_at: Set(now.into()),
        last_accounted_at: Set(now.into()),
        billed_minutes: Set(0),
        ended_at: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };

    model
        .insert(&state.db)
        .await
        .map_err(|e| UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(e.to_string())))?;

    let remaining_minutes = if bill_type == "minute" {
        Some(validation.remaining_minutes)
    } else {
        None
    };

    let build_profile = |
        node_id_str: String,
        vless_id: String,
        vless_server: String,
        vless_port: i32,
        reality_server_name: String,
        reality_public_key: String,
        reality_short_id: String,
        reality_fingerprint: String,
        reality_spider_x: String,
        udp_server_override: Option<String>,
        udp_port: i32,
        process_name: String,
        region: String,
    | {
        let udp_host = udp_server_override.unwrap_or_else(|| vless_server.clone());
        crate::interface::web::dto::ProfileVO {
            id: binding.id.to_string(),
            game_id: binding.game_id.clone(),
            display_name: binding
                .display_name
                .clone()
                .unwrap_or_else(|| node_id_str.clone()),
            node_id: node_id_str,
            process_name,
            vless_id,
            vless_server: vless_server.clone(),
            vless_port,
            vless_encryption: "none".to_string(),
            reality_server_name,
            reality_public_key,
            reality_short_id,
            reality_fingerprint,
            reality_spider_x,
            udp_proxy: format!("{}:{}", udp_host, udp_port),
            mode: binding.mode.clone().unwrap_or_else(|| "进程模式".to_string()),
            status: binding.status.clone().unwrap_or_else(|| "active".to_string()),
            region,
            ping: binding.ping.unwrap_or(5),
        }
    };

    let game = accelerator_game::Entity::find_by_id(game_id_clone.clone())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or_else(|| UsecaseError::NotFound("game"))?;

    let routing_rules = {
        let raw = game.routing_rules.as_deref().unwrap_or("").trim();
        if raw.is_empty() {
            Vec::new()
        } else {
            let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
                UsecaseError::Validation(format!("invalid game.routing_rules json: {e}"))
            })?;
            let arr = v.as_array().ok_or_else(|| {
                UsecaseError::Validation("game.routing_rules must be a json array".to_string())
            })?;

            let mut out: Vec<serde_json::Value> = Vec::with_capacity(arr.len());
            for (idx, item) in arr.iter().enumerate() {
                if !item.is_object() {
                    return Err(UsecaseError::Validation(format!(
                        "game.routing_rules[{idx}] must be a json object"
                    ))
                    .into());
                }
                out.push(item.clone());
            }
            out
        }
    };

    let sniff_domains_excluded = {
        let raw = game
            .sniff_domains_excluded
            .as_deref()
            .unwrap_or("")
            .trim();
        if raw.is_empty() {
            Vec::new()
        } else {
            let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
                UsecaseError::Validation(format!("invalid game.sniff_domains_excluded json: {e}"))
            })?;
            let arr = v.as_array().ok_or_else(|| {
                UsecaseError::Validation(
                    "game.sniff_domains_excluded must be a json array".to_string(),
                )
            })?;

            let mut out: Vec<String> = Vec::with_capacity(arr.len());
            for (idx, item) in arr.iter().enumerate() {
                let s = item.as_str().ok_or_else(|| {
                    UsecaseError::Validation(format!(
                        "game.sniff_domains_excluded[{idx}] must be a string"
                    ))
                })?;
                let s = s.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
            }
            out
        }
    };

    let mut profile_vo: Option<crate::interface::web::dto::ProfileVO> = None;
    let mut tcp_profile_vo: Option<crate::interface::web::dto::ProfileVO> = None;
    let mut udp_profile_vo: Option<crate::interface::web::dto::ProfileVO> = None;

    match binding.r#type.as_str() {
        "node" => {
            let server = state
                .admin_config
                .get_node_public_ip(primary_node_id)
                .await
                .map_err(|e| UsecaseError::Validation(format!("get_node_public_ip failed: {}", e)))?;
            let port = state
                .admin_config
                .select_default_forward_port(primary_node_id)
                .await
                .map_err(|e| {
                    UsecaseError::Validation(format!("select_default_forward_port failed: {}", e))
                })?;

            let inbounds = state
                .admin_config
                .get_inbounds(primary_node_id)
                .await
                .map_err(|e| UsecaseError::Validation(format!("get_inbounds failed: {}", e)))?;
            let inbound = inbounds
                .iter()
                .find(|i| i.tag == "entrydoor")
                .ok_or_else(|| UsecaseError::Validation("missing inbound tag entrydoor".to_string()))?;
            let (reality_server_name, reality_public_key, reality_short_id, reality_fingerprint, reality_spider_x) =
                extract_reality_client_params(inbound)?;

            profile_vo = Some(build_profile(
                primary_node_id.to_string(),
                primary_admin_uuid.clone(),
                server,
                port,
                reality_server_name,
                reality_public_key,
                reality_short_id,
                reality_fingerprint,
                reality_spider_x,
                None,
                port,
                game.process_name,
                binding.region.clone().unwrap_or_else(|| game.region),
            ));
        }
        "chain" => {
            let mut tcp_entry_endpoint: Option<(String, i32)> = None;
            let mut udp_entry_endpoint: Option<(String, i32)> = None;
            if let Some(tcp_chain_id) = binding.tcp_chain_id {
                let entry_node_id = resolve_chain_entry_node_id(&state.db, tcp_chain_id).await?;
                let exit_node_id = resolve_chain_exit_node_id(&state.db, tcp_chain_id).await?;
                let server = state
                    .admin_config
                    .get_node_public_ip(entry_node_id)
                    .await
                    .map_err(|e| {
                        UsecaseError::Validation(format!("get_node_public_ip failed: {}", e))
                    })?;
                let tag = format!("chain_{}_1", tcp_chain_id);
                let inbounds = state
                    .admin_config
                    .get_inbounds(exit_node_id)
                    .await
                    .map_err(|e| {
                        UsecaseError::Validation(format!("get_inbounds failed: {}", e))
                    })?;
                let inbound = inbounds
                    .iter()
                    .find(|i| i.tag == "entrydoor")
                    .ok_or_else(|| {
                        UsecaseError::Validation(
                            "missing inbound tag entrydoor for chain binding".to_string(),
                        )
                    })?;
                let (reality_server_name, reality_public_key, reality_short_id, reality_fingerprint, reality_spider_x) =
                    extract_reality_client_params(inbound)?;
                let tcp_port = match resolve_inbound_port_by_tag(&state.admin_config, entry_node_id, &tag).await {
                    Ok(p) => p,
                    Err(e) if entry_node_id == exit_node_id => state
                        .admin_config
                        .select_default_forward_port(entry_node_id)
                        .await
                        .map_err(|err| {
                            UsecaseError::Validation(format!(
                                "select_default_forward_port failed: {} (previous resolve_inbound_port_by_tag error: {})",
                                err, e
                            ))
                        })?,
                    Err(e) => return Err(e.into()),
                };
                tcp_entry_endpoint = Some((server.clone(), tcp_port));

                tcp_profile_vo = Some(build_profile(
                    entry_node_id.to_string(),
                    primary_admin_uuid.clone(),
                    server.clone(),
                    tcp_port,
                    reality_server_name,
                    reality_public_key,
                    reality_short_id,
                    reality_fingerprint,
                    reality_spider_x,
                    None,
                    tcp_port,
                    game.process_name.clone(),
                    binding.region.clone().unwrap_or_else(|| game.region.clone()),
                ));

                if profile_vo.is_none() {
                    profile_vo = tcp_profile_vo.take();
                }
            }

            if let Some(udp_chain_id) = binding.udp_chain_id {
                let entry_node_id = resolve_chain_entry_node_id(&state.db, udp_chain_id).await?;
                let exit_node_id = resolve_chain_exit_node_id(&state.db, udp_chain_id).await?;
                let server = state
                    .admin_config
                    .get_node_public_ip(entry_node_id)
                    .await
                    .map_err(|e| {
                        UsecaseError::Validation(format!("get_node_public_ip failed: {}", e))
                    })?;
                let first_hop_tag = format!("chain_{}_1", udp_chain_id);
                let inbounds = state
                    .admin_config
                    .get_inbounds(exit_node_id)
                    .await
                    .map_err(|e| {
                        UsecaseError::Validation(format!("get_inbounds failed: {}", e))
                    })?;
                let inbound = inbounds
                    .iter()
                    .find(|i| i.tag == "entrydoor")
                    .ok_or_else(|| {
                        UsecaseError::Validation(
                            "missing inbound tag entrydoor for chain binding".to_string(),
                        )
                    })?;
                let (reality_server_name, reality_public_key, reality_short_id, reality_fingerprint, reality_spider_x) =
                    extract_reality_client_params(inbound)?;
                let udp_port = match resolve_inbound_port_by_tag(&state.admin_config, entry_node_id, &first_hop_tag).await {
                    Ok(p) => p,
                    Err(e) => {
                        // fallback: look for matching inbounds on entry node tagged as chain_{id}_*
                        let socks_tag = format!("chain_{}_socks", udp_chain_id);
                        if let Ok(port) = resolve_inbound_port_by_tag(&state.admin_config, exit_node_id, &socks_tag).await {
                            port
                        } else if entry_node_id == exit_node_id {
                            // fallback to first matching hop inbound on same node
                            let candidate = format!("chain_{}_", udp_chain_id);
                            let entry_inbounds = state
                                .admin_config
                                .get_inbounds(entry_node_id)
                                .await
                                .map_err(|err| {
                                    UsecaseError::Validation(format!(
                                        "get_inbounds failed for entry node: {}",
                                        err
                                    ))
                                })?;
                            let hop_port = entry_inbounds
                                .iter()
                                .find(|ib| ib.tag.starts_with(&candidate))
                                .map(|ib| ib.port)
                                .and_then(|p| if p > 0 { Some(p) } else { None })
                                .ok_or_else(|| {
                                    UsecaseError::Validation(format!(
                                        "resolve_inbound_port_by_tag failed: {} and no fallback inbound found",
                                        e
                                    ))
                                })?;
                            hop_port
                        } else {
                            return Err(e.into());
                        }
                    }
                };
                udp_entry_endpoint = Some((server.clone(), udp_port));

                udp_profile_vo = Some(build_profile(
                    entry_node_id.to_string(),
                    primary_admin_uuid.clone(),
                    server.clone(),
                    udp_port,
                    reality_server_name,
                    reality_public_key,
                    reality_short_id,
                    reality_fingerprint,
                    reality_spider_x,
                    None,
                    udp_port,
                    game.process_name.clone(),
                    binding.region.clone().unwrap_or_else(|| game.region.clone()),
                ));

                if profile_vo.is_none() {
                    profile_vo = udp_profile_vo.take();
                }
            }

            if profile_vo.is_none() {
                return Err(UsecaseError::Validation(
                    "missing tcp_chain_id/udp_chain_id for chain binding".to_string(),
                )
                .into());
            }
            if let (Some((udp_host, udp_port)), Some(profile)) = (udp_entry_endpoint.clone(), profile_vo.as_mut()) {
                profile.udp_proxy = format!("{}:{}", udp_host, udp_port);
            }
        }
        other => {
            return Err(UsecaseError::Validation(format!("invalid binding.type: {}", other)).into());
        }
    }

    let profile_vo = profile_vo.ok_or_else(|| UsecaseError::Validation("missing profile".to_string()))?;

    Ok(ApiResponse::success(SessionStartResponseVO {
        session_id,
        uuid: profile_vo.vless_id.clone(),
        bill_type,
        remaining_minutes,
        profile: profile_vo,
        tcp_profile: None,
        udp_profile: None,
        routing_rules,
        sniff_domains_excluded,
    })
    .into_http(StatusCode::OK))
}

#[web::post("/accelerator/session/stop")]
pub async fn session_stop(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<SessionStopRequestVO>,
) -> Result<HttpResponse, AppError> {
    let Some(model) = acceleration_session::Entity::find_by_id(body.session_id.clone())
            .one(&state.db)
            .await
        .map_err(|e| UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(e.to_string())))?
    else {
        return Err(UsecaseError::NotFound("session").into());
    };

    if model.user_id != user.user.id {
        return Err(UsecaseError::Unauthorized.into());
    }

    if model.status != "active" {
        return Ok(ApiResponse::success(SessionStopResponseVO {
            status: model.status,
            billed_minutes: model.billed_minutes,
        })
        .into_http(StatusCode::OK));
    }

    if model.admin_user_id > 0 {
        info!(
            "[session_stop] deleting admin user: session_id={} node_id={} admin_user_id={}",
            model.session_id, model.node_id, model.admin_user_id
        );
        if use_main_admin_http() {
            let _ = main_admin_delete_user(model.node_id, model.admin_user_id).await;
        } else {
            let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(8);
            let _ = cp_delete_user(&*state, model.node_id, model.admin_user_id, true, sync_timeout).await;
        }
    }

    if let (Some(node_id), Some(admin_user_id)) = (model.tcp_node_id, model.tcp_admin_user_id) {
        if admin_user_id > 0 {
            info!(
                "[session_stop] deleting tcp admin user: session_id={} node_id={} admin_user_id={}",
                model.session_id, node_id, admin_user_id
            );
            if use_main_admin_http() {
                let _ = main_admin_delete_user(node_id, admin_user_id).await;
            } else {
                let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(8);
                let _ = cp_delete_user(&*state, node_id, admin_user_id, true, sync_timeout).await;
            }
        }
    }

    if let (Some(node_id), Some(admin_user_id)) = (model.udp_node_id, model.udp_admin_user_id) {
        if admin_user_id > 0 {
            info!(
                "[session_stop] deleting udp admin user: session_id={} node_id={} admin_user_id={}",
                model.session_id, node_id, admin_user_id
            );
            if use_main_admin_http() {
                let _ = main_admin_delete_user(node_id, admin_user_id).await;
            } else {
                let sync_timeout = env::var("SESSION_SYNC_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(8);
                let _ = cp_delete_user(&*state, node_id, admin_user_id, true, sync_timeout).await;
            }
        }
    }

    let mut active: acceleration_session::ActiveModel = model.into();
    let now = chrono::Utc::now();
    active.status = Set("stopped".to_string());
    active.last_activity_at = Set(now.into());
    active.ended_at = Set(Some(now.into()));
    active.updated_at = Set(now.into());

    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(e.to_string())))?;

    Ok(ApiResponse::success(SessionStopResponseVO {
        status: updated.status,
        billed_minutes: updated.billed_minutes,
    })
    .into_http(StatusCode::OK))
}

#[web::post("/accelerator/profiles")]
pub async fn sync_profiles(
    state: State<AppState>,
    Json(body): Json<ProfileSyncRequest>,
) -> Result<HttpResponse, AppError> {
    let repo = AcceleratorRepositoryImpl::new(&state.db);
    let node_repo = NodeRepositoryImpl::new(&state.db);
    let usecase = AcceleratorUseCase::new(repo, node_repo);
    
    // 将 ProfileVO 转换为 ProfileVOData
    let profile_vos: Vec<crate::application::accelerator_usecase::ProfileVOData> = body
        .profiles
        .into_iter()
        .map(|p| crate::application::accelerator_usecase::ProfileVOData {
            id: p.id,
            game_id: p.game_id,
            display_name: p.display_name,
            process_name: p.process_name,
            vless_id: p.vless_id,
            vless_server: p.vless_server,
            vless_port: p.vless_port,
            vless_encryption: p.vless_encryption,
            reality_server_name: p.reality_server_name,
            reality_public_key: p.reality_public_key,
            reality_short_id: p.reality_short_id,
            reality_fingerprint: p.reality_fingerprint,
            reality_spider_x: p.reality_spider_x,
            udp_proxy: p.udp_proxy,
            mode: p.mode,
            status: p.status,
            region: p.region,
            ping: p.ping,
        })
        .collect();
    
    usecase.sync_profiles_from_vo(profile_vos).await?;
    Ok(ApiResponse::success(MessageResponse {
        message: "Profiles updated".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::get("/dashboard")]
pub async fn dashboard(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: DashboardVO = usecase.dashboard().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/games/{game_id}/bindings")]
pub async fn list_game_bindings(
    state: State<AppState>,
    path: web::types::Path<String>,
) -> Result<HttpResponse, AppError> {
    let game_id = path.into_inner();

    let bindings = accelerator_game_node_binding::Entity::find()
        .filter(accelerator_game_node_binding::Column::GameId.eq(game_id.as_str()))
        .order_by_asc(accelerator_game_node_binding::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| {
            crate::application::errors::UsecaseError::Repository(
                crate::application::errors::RepositoryError::Persistence(e.to_string()),
            )
        })?;

    let payload: Vec<crate::interface::web::dto::ProfileListVO> = bindings
        .into_iter()
        .map(|b| crate::interface::web::dto::ProfileListVO {
            id: b.id.to_string(),
            game_id: b.game_id,
            display_name: b
                .display_name
                .clone()
                .or_else(|| b.node_id.clone())
                .unwrap_or_else(|| format!("binding_{}", b.id)),
            node_id: b.node_id.unwrap_or_default(),
            mode: b.mode.unwrap_or_else(|| "进程模式".to_string()),
            status: b.status.unwrap_or_else(|| "active".to_string()),
            region: b.region.unwrap_or_else(|| "".to_string()),
            ping: b.ping.unwrap_or(5),
        })
        .collect();

    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/library")]
pub async fn library(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: LibraryVO = usecase.library().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/settings/meta")]
pub async fn settings(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: SettingsMetaVO = usecase.settings().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/navigation")]
pub async fn navigation(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: NavigationVO = usecase.navigation().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::post("/auth/wechat/ticket")]
pub async fn create_wechat_ticket(
    state: State<AppState>,
    Json(body): Json<TicketRequestVO>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let ticket = usecase.create_ticket(body.scene).await?;
    Ok(ApiResponse::success(WechatTicketVO::from(ticket)).into_http(StatusCode::CREATED))
}

#[web::get("/auth/wechat/status")]
pub async fn wechat_status(
    state: State<AppState>,
    Query(query): Query<TicketStatusQuery>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let ticket = usecase.ticket_status(&query.ticket_id).await?;
    Ok(ApiResponse::success(WechatTicketVO::from(ticket)).into_http(StatusCode::OK))
}

#[web::post("/auth/account")]
pub async fn account_login(
    state: State<AppState>,
    Json(body): Json<AccountLoginRequestVO>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let response = usecase.account_login(body.into()).await?;
    Ok(ApiResponse::success(AccountLoginResponseVO::from(response)).into_http(StatusCode::OK))
}

#[web::post("/auth/accelerator/register")]
pub async fn accelerator_user_register(
    state: State<AppState>,
    Json(body): Json<AcceleratorUserRegisterRequestVO>,
) -> Result<HttpResponse, AppError> {
    if body.email.trim().is_empty() {
        return Err(UsecaseError::Validation("email is required".to_string()).into());
    }
    if body.password.trim().is_empty() {
        return Err(UsecaseError::Validation("password is required".to_string()).into());
    }

    // 注册时邮箱验证码校验
    let cached = EMAIL_CODE_CACHE.get(&body.email).await;
    if cached.as_deref() != Some(body.email_code.trim()) {
        return Ok(ApiResponse::<MessageResponse>::error(
            "EMAIL_CODE_INVALID",
            "邮箱验证码错误或已过期".to_string(),
        )
        .into_http(StatusCode::BAD_REQUEST));
    }
    EMAIL_CODE_CACHE.invalidate(&body.email).await;

    let email = body.email.trim().to_string();
    let exists = accelerator_user::Entity::find()
        .filter(accelerator_user::Column::Email.eq(email.as_str()))
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .is_some();
    if exists {
        return Err(UsecaseError::Validation("user already exists".to_string()).into());
    }

    let now = chrono::Utc::now();

    let invite_code = generate_invite_code();
    let inviter_id: Option<i64> = match body.invite_code.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(code) => {
            let inviter = accelerator_user::Entity::find()
                .filter(accelerator_user::Column::InviteCode.eq(code))
                .one(&state.db)
                .await
                .map_err(|e| {
                    UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                        e.to_string(),
                    ))
                })?;
            inviter.map(|u| u.id)
        }
        None => None,
    };

    let inserted = accelerator_user::ActiveModel {
        id: sea_orm::NotSet,
        email: Set(email.clone()),
        name: Set(body.name),
        invite_code: Set(invite_code),
        inviter_id: Set(inviter_id.clone()),
        valid_until: Set(now.into()),
    }
    .insert(&state.db)
    .await
    .map_err(|e| {
        UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
            e.to_string(),
        ))
    })?;

    let new_user_id = inserted.id;

    accelerator_user_credential::ActiveModel {
        user_id: Set(new_user_id),
        password_hash: Set(hash_password(&body.password)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&state.db)
    .await
    .map_err(|e| {
        UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
            e.to_string(),
        ))
    })?;

    if let Some(inviter_id) = inviter_id {
        if let Err(e) = apply_invite_rewards(&state.db, inviter_id, new_user_id).await {
            log::warn!("apply_invite_rewards failed (ignored): {}", e);
        }
    }

    if let Err(e) = apply_user_register_activity_reward(&state.db, new_user_id).await {
        log::warn!("apply_user_register_activity_reward failed (ignored): {}", e);
    }

    Ok(ApiResponse::success(MessageResponse {
        message: "User registered".into(),
    })
    .into_http(StatusCode::CREATED))
}

fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    const LEN: usize = 8;
    let mut rng = rand::thread_rng();
    (0..LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[web::post("/auth/accelerator/login")]
pub async fn accelerator_user_login(
    state: State<AppState>,
    req: HttpRequest,
    Json(body): Json<AcceleratorUserLoginRequestVO>,
) -> Result<HttpResponse, AppError> {
    let ip = extract_ip_from_headers(&req);
    let ua = extract_user_agent(&req);
    let now = chrono::Utc::now();

    let email = body.email.trim().to_string();
    let user = accelerator_user::Entity::find()
        .filter(accelerator_user::Column::Email.eq(email.as_str()))
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    let user_id = user.id;
    let cred = accelerator_user_credential::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    if cred.password_hash != hash_password(&body.password) {
        let _ = accelerator_user_login_log::ActiveModel {
            user_id: Set(user_id),
            ip: Set(ip),
            user_agent: Set(ua),
            success: Set(false),
            reason_code: Set("UNAUTHORIZED".to_string()),
            reason_message: Set("invalid password".to_string()),
            created_at: Set(now.into()),
            ..Default::default()
        }
        .insert(&state.db)
        .await;
        return Err(UsecaseError::Unauthorized.into());
    }

    // 如果该账号存在 active 加速会话，则禁止再次登录（避免 A 抢占 B 的加速中账号）
    if let Ok(Some(_existing)) = acceleration_session::Entity::find()
        .filter(acceleration_session::Column::UserId.eq(user_id))
        .filter(acceleration_session::Column::Status.eq("active"))
        .one(&state.db)
        .await
    {
        let _ = USER_NOTIFICATION_BUS.send(UserNotificationEvent {
            user_id,
            payload: UserNotificationPayload {
                kind: "SECURITY_WARNING".to_string(),
                message: "当前账号在IP处登录，请检查账号是否泄露".to_string(),
                ip: ip.clone(),
                ts: now.timestamp(),
            },
        });

        let _ = accelerator_user_login_log::ActiveModel {
            user_id: Set(user_id),
            ip: Set(ip),
            user_agent: Set(ua),
            success: Set(false),
            reason_code: Set("ACCOUNT_IN_USE".to_string()),
            reason_message: Set("account is accelerating".to_string()),
            created_at: Set(now.into()),
            ..Default::default()
        }
        .insert(&state.db)
        .await;

        return Ok(ApiResponse::<MessageResponse>::error(
            "ACCOUNT_IN_USE",
            "当前账号正在登录加速， 禁止登录。".to_string(),
        )
        .into_http(StatusCode::FORBIDDEN));
    }

    let ttl_days = if body.remember { 30 } else { 1 };
    let expires_at = now + chrono::Duration::days(ttl_days);
    let token = Uuid::new_v4().to_string();

    accelerator_user_session::ActiveModel {
        token: Set(token.clone()),
        user_id: Set(user_id),
        expires_at: Set(expires_at.into()),
        created_at: Set(now.into()),
    }
    .insert(&state.db)
    .await
    .map_err(|e| {
        UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
            e.to_string(),
        ))
    })?;

    let domain_user: crate::domain::accelerator::AcceleratorUser = user.into();

    let _ = accelerator_user_login_log::ActiveModel {
        user_id: Set(domain_user.id),
        ip: Set(ip),
        user_agent: Set(ua),
        success: Set(true),
        reason_code: Set("OK".to_string()),
        reason_message: Set("".to_string()),
        created_at: Set(now.into()),
        ..Default::default()
    }
    .insert(&state.db)
    .await;

    Ok(ApiResponse::success(AcceleratorUserLoginResponseVO {
        success: true,
        token,
        user: UserVO::from(domain_user),
    })
    .into_http(StatusCode::OK))
}

#[web::get("/auth/accelerator/me")]
pub async fn accelerator_user_me(user: AuthedAcceleratorUser) -> Result<HttpResponse, AppError> {
    Ok(ApiResponse::success(UserVO::from(user.user)).into_http(StatusCode::OK))
}

#[web::post("/auth/accelerator/profile")]
pub async fn accelerator_user_update_profile(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<AcceleratorUserUpdateProfileRequestVO>,
) -> Result<HttpResponse, AppError> {
    let model = accelerator_user::Entity::find_by_id(user.user.id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    let mut active: accelerator_user::ActiveModel = model.into();
    active.name = Set(body.name);
    let updated = active
        .update(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    let domain_user: crate::domain::accelerator::AcceleratorUser = updated.into();
    Ok(ApiResponse::success(UserVO::from(domain_user)).into_http(StatusCode::OK))
}

#[web::post("/nodes/register")]
pub async fn register_node(
    state: State<AppState>,
    Json(body): Json<NodeRegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let repo = NodeRepositoryImpl::new(&state.db);
    let usecase = NodeUseCase::new(repo);
    let node: crate::domain::accelerator::Node = body.into();
    usecase.register_node(node).await?;
    Ok(ApiResponse::success(MessageResponse {
        message: "Node registered".into(),
    })
    .into_http(StatusCode::CREATED))
}

#[web::post("/nodes/{node_id}/unregister")]
pub async fn unregister_node(
    state: State<AppState>,
    path: web::types::Path<String>,
) -> Result<HttpResponse, AppError> {
    let node_id = path.into_inner();
    let repo = NodeRepositoryImpl::new(&state.db);
    let usecase = NodeUseCase::new(repo);
    usecase.unregister_node(&node_id).await?;
    Ok(ApiResponse::success(MessageResponse {
        message: "Node unregistered".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::post("/nodes/{node_id}/heartbeat")]
pub async fn node_heartbeat(
    state: State<AppState>,
    path: web::types::Path<String>,
) -> Result<HttpResponse, AppError> {
    let node_id = path.into_inner();
    let repo = NodeRepositoryImpl::new(&state.db);
    let usecase = NodeUseCase::new(repo);
    usecase.heartbeat(&node_id).await?;
    Ok(ApiResponse::success(MessageResponse {
        message: "Heartbeat updated".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::get("/nodes")]
pub async fn list_nodes(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = NodeRepositoryImpl::new(&state.db);
    let usecase = NodeUseCase::new(repo);
    let nodes = usecase.list_nodes().await?;
    Ok(ApiResponse::success(nodes).into_http(StatusCode::OK))
}

#[web::get("/nodes/active")]
pub async fn list_active_nodes(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = NodeRepositoryImpl::new(&state.db);
    let usecase = NodeUseCase::new(repo);
    let nodes = usecase.list_active_nodes().await?;
    Ok(ApiResponse::success(nodes).into_http(StatusCode::OK))
}

#[web::post("/cdk/redeem")]
pub async fn redeem_cdk(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<CdkRedeemRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let mut request: crate::domain::cdk::CdkRedeemRequest = body.into();
    request.user_id = user.user.id;
    let response = usecase.redeem_cdk(request).await?;
    Ok(ApiResponse::success(CdkRedeemResponseVO::from(response)).into_http(StatusCode::OK))
}

#[web::post("/account/validate")]
pub async fn validate_account(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<AccountValidationRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    
    let mut request: AccountValidationRequest = body.into();
    request.user_id = user.user.id;
    
    let response = usecase.validate_account(request).await?;
    Ok(ApiResponse::success(AccountValidationResponseVO::from(response)).into_http(StatusCode::OK))
}

#[web::post("/auth/accelerator/password")]
pub async fn accelerator_user_change_password(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<AcceleratorUserChangePasswordRequestVO>,
) -> Result<HttpResponse, AppError> {
    let model = accelerator_user_credential::Entity::find_by_id(user.user.id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    if model.password_hash != hash_password(&body.old_password) {
        return Err(UsecaseError::Unauthorized.into());
    }

    let now = chrono::Utc::now();
    let mut active: accelerator_user_credential::ActiveModel = model.into();
    active.password_hash = Set(hash_password(&body.new_password));
    active.updated_at = Set(now.into());
    active
        .update(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    Ok(ApiResponse::success(MessageResponse {
        message: "Password updated".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::post("/auth/accelerator/logout")]
pub async fn accelerator_user_logout(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
) -> Result<HttpResponse, AppError> {
    accelerator_user_session::Entity::delete_by_id(user.token)
        .exec(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    Ok(ApiResponse::success(MessageResponse {
        message: "Logged out".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::post("/accelerator/start")]
pub async fn start_acceleration(
    state: State<AppState>,
    user: AuthedAcceleratorUser,
    Json(body): Json<AccountValidationRequestVO>,
) -> Result<HttpResponse, AppError> {
    // ... (rest of the code remains the same)
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    
    let mut request: AccountValidationRequest = body.into();
    request.user_id = user.user.id;
    
    let validation = usecase.validate_account(request).await?;

    if !validation.is_valid {
        return Ok(ApiResponse::<MessageResponse>::error(
            "ACCOUNT_VALIDATION_FAILED",
            validation.message,
        )
        .into_http(StatusCode::FORBIDDEN));
    }

    Ok(ApiResponse::success(MessageResponse {
        message: "Acceleration started successfully".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::get("/games/{game_id}/nodes")]
pub async fn list_game_nodes(
    state: State<AppState>,
    path: web::types::Path<String>,
) -> Result<HttpResponse, AppError> {
    let game_id = path.into_inner();

    let bindings = accelerator_game_node_binding::Entity::find()
        .filter(accelerator_game_node_binding::Column::GameId.eq(game_id.as_str()))
        .order_by_asc(accelerator_game_node_binding::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(|e| crate::application::errors::UsecaseError::Repository(
            crate::application::errors::RepositoryError::Persistence(e.to_string()),
        ))?;

    let node_ids: Vec<String> = bindings
        .into_iter()
        .filter(|b| b.r#type == "node")
        .filter_map(|b| b.node_id)
        .collect();
    if node_ids.is_empty() {
        return Ok(ApiResponse::success(Vec::<GameNodeVO>::new()).into_http(StatusCode::OK));
    }

    let nodes = accelerator_node::Entity::find()
        .filter(accelerator_node::Column::Id.is_in(node_ids.clone()))
        .all(&state.db)
        .await
        .map_err(|e| crate::application::errors::UsecaseError::Repository(
            crate::application::errors::RepositoryError::Persistence(e.to_string()),
        ))?;

    let node_map: std::collections::HashMap<String, accelerator_node::Model> =
        nodes.into_iter().map(|n| (n.id.clone(), n)).collect();

    let payload: Vec<GameNodeVO> = node_ids
        .into_iter()
        .filter_map(|id| {
            let n = node_map.get(&id)?;
            Some(GameNodeVO {
                node_id: n.id.clone(),
                vless_id: n.vless_id.clone(),
                vless_server: n.vless_server.clone(),
                vless_port: n.vless_port,
                vless_encryption: n.vless_encryption.clone(),
                reality_server_name: n.reality_server_name.clone(),
                reality_public_key: n.reality_public_key.clone(),
                reality_short_id: n.reality_short_id.clone(),
                reality_fingerprint: n.reality_fingerprint.clone(),
                reality_spider_x: n.reality_spider_x.clone(),
                udp_proxy: n.udp_proxy.clone(),
                mode: n.mode.clone(),
                ping: n.ping,
                status: n.status.clone(),
            })
        })
        .collect();

    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::post("/games/{game_id}/nodes")]
pub async fn set_game_nodes(
    state: State<AppState>,
    path: web::types::Path<String>,
    Json(body): Json<GameNodeBindingRequest>,
) -> Result<HttpResponse, AppError> {
    let game_id = path.into_inner();

    let game_exists = accelerator_game::Entity::find_by_id(game_id.clone())
        .one(&state.db)
        .await
        .map_err(|e| crate::application::errors::UsecaseError::Repository(
            crate::application::errors::RepositoryError::Persistence(e.to_string()),
        ))?
        .is_some();

    if !game_exists {
        return Err(crate::application::errors::UsecaseError::NotFound("game").into());
    }

    let node_ids = body.node_ids;

    if !node_ids.is_empty() {
        let existing_nodes = accelerator_node::Entity::find()
            .filter(accelerator_node::Column::Id.is_in(node_ids.clone()))
            .all(&state.db)
            .await
            .map_err(|e| crate::application::errors::UsecaseError::Repository(
                crate::application::errors::RepositoryError::Persistence(e.to_string()),
            ))?;

        if existing_nodes.len() != node_ids.len() {
            return Err(crate::application::errors::UsecaseError::Validation(
                "some node_ids do not exist".to_string(),
            )
            .into());
        }
    }

    accelerator_game_node_binding::Entity::delete_many()
        .filter(accelerator_game_node_binding::Column::GameId.eq(game_id.as_str()))
        .exec(&state.db)
        .await
        .map_err(|e| crate::application::errors::UsecaseError::Repository(
            crate::application::errors::RepositoryError::Persistence(e.to_string()),
        ))?;

    for node_id in node_ids {
        accelerator_game_node_binding::ActiveModel {
            game_id: Set(game_id.clone()),
            r#type: Set("node".to_string()),
            node_id: Set(Some(node_id)),
            ..Default::default()
        }
        .insert(&state.db)
        .await
        .map_err(|e| crate::application::errors::UsecaseError::Repository(
            crate::application::errors::RepositoryError::Persistence(e.to_string()),
        ))?;
    }

    Ok(ApiResponse::success(MessageResponse {
        message: "Game nodes updated".into(),
    })
    .into_http(StatusCode::OK))
}
