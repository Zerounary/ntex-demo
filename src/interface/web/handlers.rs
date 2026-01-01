use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;
use log::{info, error};

use crate::application::accelerator_usecase::AcceleratorUseCase;
use crate::application::auth_usecase::AuthUseCase;
use crate::application::cdk_usecase::CdkUseCase;
use crate::application::content_usecase::ContentUseCase;
use crate::application::node_usecase::NodeUseCase;
use crate::application::errors::UsecaseError;
use crate::infrastructure::persistence::repositories::{
    AcceleratorRepositoryImpl, AuthRepositoryImpl, CdkRepositoryImpl, ConfigRepositoryImpl,
    NodeRepositoryImpl,
};
use crate::infrastructure::persistence::{
    accelerator_game, accelerator_game_node_binding, accelerator_node, acceleration_session,
    accelerator_user, accelerator_user_credential, accelerator_user_session, admin_node_config,
};

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
    AcceleratorUserUpdateProfileRequestVO, AcceleratorUserChangePasswordRequestVO, UserVO,
};
use super::errors::{ApiResponse, AppError, MessageResponse};

#[derive(Debug, Deserialize, Default)]
struct AdminUserDTO {
    pub id: u64,
    pub uuid: String,
    pub st: u64,
    pub dt: u64,
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
    info!(
        "[session_start] <- admin add_user success: node_id={} admin_user_id={}",
        node_id, data.id
    );
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
        .query(&[
            ("node_id", node_id.to_string()),
            ("act", "outbound".to_string()),
        ]);
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
        .map_err(|e| {
            UsecaseError::Validation(format!("main_admin query outbounds invalid response: {}", e))
        })?;

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

#[web::post("/accelerator/session/start")]
pub async fn session_start(
    state: State<AppState>,
    Json(body): Json<SessionStartRequestVO>,
) -> Result<HttpResponse, AppError> {
    let user_id = body.user_id;
    let game_id = body.game_id;
    let node_id = body.node_id;
    let desired_outbound_tag = format!("accel_{}_{}_{}", user_id, game_id, node_id);

    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);

    let validation = usecase
        .validate_account(AccountValidationRequestVO {
            user_id: user_id.clone(),
        }
        .into())
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

    // 节点离线时禁止启动（避免误停已有 active session）
    let node_cfg = admin_node_config::Entity::find_by_id(node_id)
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?;

    let offline_after_seconds = env::var("NODE_OFFLINE_AFTER_SECONDS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(60)
        .max(1);
    let offline_threshold = chrono::Utc::now() - chrono::Duration::seconds(offline_after_seconds);

    let is_online = node_cfg.as_ref().map(|n| n.is_online).unwrap_or(false);
    let last_seen_at = node_cfg.as_ref().and_then(|n| n.last_seen_at);
    let is_fresh = last_seen_at.map(|ts| ts >= offline_threshold).unwrap_or(false);

    if !is_online || !is_fresh {
        return Ok(ApiResponse::<MessageResponse>::error(
            "NODE_OFFLINE",
            format!("node {} is offline", node_id),
        )
        .into_http(StatusCode::SERVICE_UNAVAILABLE));
    }

    // 单用户同一时刻只允许一个 active 会话：存在则先 stop（best effort）
    if let Ok(Some(existing)) = acceleration_session::Entity::find()
        .filter(acceleration_session::Column::UserId.eq(user_id.as_str()))
        .filter(acceleration_session::Column::Status.eq("active"))
        .order_by_desc(acceleration_session::Column::StartedAt)
        .one(&state.db)
        .await
    {
        info!(
            "[session_start] best-effort stop existing session_id={} node_id={}",
            existing.session_id, existing.node_id
        );
        let _ = main_admin_delete_user(existing.node_id, existing.admin_user_id).await;

        let mut active: acceleration_session::ActiveModel = existing.into();
        active.status = Set("stopped".to_string());
        active.ended_at = Set(Some(chrono::Utc::now().into()));
        active.updated_at = Set(chrono::Utc::now().into());
        let _ = active.update(&state.db).await;
    }

    let uuid = uuid::Uuid::new_v4().to_string();
    let st = if validation.billing_mode == "pass" { 5u64 } else { 1u64 };

    // 1) 调用 main.rs Admin HTTP：下发用户到节点（add_user）
    let admin_user = main_admin_add_user(node_id, uuid, st, 0).await?;

    let admin_user_id = admin_user.id;
    let admin_uuid = admin_user.uuid;

    let outbounds = main_admin_get_outbound_tags(node_id).await?;
    let mut candidates: Vec<String> = outbounds
        .into_iter()
        .filter(|t| t != "block" && t != "direct" && t != "vmess_loopback")
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

        let mut best_tag: Option<String> = None;
        let mut best_count: u64 = u64::MAX;
        for t in &candidates {
            let c = counts.get(t).copied().unwrap_or(0);
            if c < best_count {
                best_count = c;
                best_tag = Some(t.clone());
            }
        }

        best_tag.ok_or_else(|| {
            UsecaseError::Validation(format!(
                "no available outbound tag for node_id={} (excluded block/direct/vmess_loopback)",
                node_id
            ))
        })?
    };

    info!(
        "[session_start] selected outbound_tag: node_id={} desired={} selected={}",
        node_id, desired_outbound_tag, mapped_outbound_tag
    );

    // 2) 调用 main.rs Admin HTTP：下发映射（add_mapping）
    main_admin_add_mapping(node_id, admin_uuid.clone(), mapped_outbound_tag.clone()).await?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let bill_type = validation.billing_mode.clone();

    let model = acceleration_session::ActiveModel {
        session_id: Set(session_id.clone()),
        user_id: Set(user_id),
        game_id: Set(game_id),
        node_id: Set(node_id),
        admin_user_id: Set(admin_user_id),
        uuid: Set(admin_uuid.clone()),
        outbound_tag: Set(mapped_outbound_tag),
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

    Ok(ApiResponse::success(SessionStartResponseVO {
        session_id,
        uuid: admin_uuid,
        bill_type,
        remaining_minutes,
    })
    .into_http(StatusCode::OK))
}

#[web::post("/accelerator/session/stop")]
pub async fn session_stop(
    state: State<AppState>,
    Json(body): Json<SessionStopRequestVO>,
) -> Result<HttpResponse, AppError> {
    let Some(model) = acceleration_session::Entity::find_by_id(body.session_id.clone())
            .one(&state.db)
            .await
        .map_err(|e| UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(e.to_string())))?
    else {
        return Err(UsecaseError::NotFound("session").into());
    };

    if model.status != "active" {
        return Ok(ApiResponse::success(SessionStopResponseVO {
            status: model.status,
            billed_minutes: model.billed_minutes,
        })
        .into_http(StatusCode::OK));
    }

    info!(
        "[session_stop] deleting admin user: session_id={} node_id={} admin_user_id={}",
        model.session_id, model.node_id, model.admin_user_id
    );
    let _ = main_admin_delete_user(model.node_id, model.admin_user_id).await;

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
    if body.user_id.trim().is_empty() {
        return Err(UsecaseError::Validation("user_id is required".to_string()).into());
    }
    if body.password.trim().is_empty() {
        return Err(UsecaseError::Validation("password is required".to_string()).into());
    }

    let exists = accelerator_user::Entity::find_by_id(body.user_id.clone())
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
    accelerator_user::ActiveModel {
        id: Set(body.user_id.clone()),
        name: Set(body.name),
        valid_until: Set(now.into()),
    }
    .insert(&state.db)
    .await
    .map_err(|e| {
        UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
            e.to_string(),
        ))
    })?;

    accelerator_user_credential::ActiveModel {
        user_id: Set(body.user_id),
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

    Ok(ApiResponse::success(MessageResponse {
        message: "User registered".into(),
    })
    .into_http(StatusCode::CREATED))
}

#[web::post("/auth/accelerator/login")]
pub async fn accelerator_user_login(
    state: State<AppState>,
    Json(body): Json<AcceleratorUserLoginRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cred = accelerator_user_credential::Entity::find_by_id(body.user_id.clone())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    if cred.password_hash != hash_password(&body.password) {
        return Err(UsecaseError::Unauthorized.into());
    }

    let user = accelerator_user::Entity::find_by_id(body.user_id.clone())
        .one(&state.db)
        .await
        .map_err(|e| {
            UsecaseError::Repository(crate::application::errors::RepositoryError::Persistence(
                e.to_string(),
            ))
        })?
        .ok_or(UsecaseError::Unauthorized)?;

    let now = chrono::Utc::now();
    let ttl_days = if body.remember { 30 } else { 1 };
    let expires_at = now + chrono::Duration::days(ttl_days);
    let token = Uuid::new_v4().to_string();

    accelerator_user_session::ActiveModel {
        token: Set(token.clone()),
        user_id: Set(body.user_id),
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

#[web::post("/cdk/generate")]
pub async fn generate_cdks(
    state: State<AppState>,
    Json(body): Json<CdkGenerateRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let cdks = usecase.generate_cdks(body.into()).await?;
    let cdks_vo: Vec<CdkCodeVO> = cdks.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(cdks_vo).into_http(StatusCode::CREATED))
}

#[web::post("/cdk/redeem")]
pub async fn redeem_cdk(
    state: State<AppState>,
    Json(body): Json<CdkRedeemRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let response = usecase.redeem_cdk(body.into()).await?;
    Ok(ApiResponse::success(CdkRedeemResponseVO::from(response)).into_http(StatusCode::OK))
}

#[web::get("/cdk/list")]
pub async fn list_cdks(
    state: State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let status = query.get("status").map(|s| s.as_str());
    let cdks = usecase.list_cdks(status).await?;
    let cdks_vo: Vec<CdkCodeVO> = cdks.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(cdks_vo).into_http(StatusCode::OK))
}

#[web::post("/account/validate")]
pub async fn validate_account(
    state: State<AppState>,
    Json(body): Json<AccountValidationRequestVO>,
) -> Result<HttpResponse, AppError> {
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let response = usecase.validate_account(body.into()).await?;
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
    Json(body): Json<AccountValidationRequestVO>,
) -> Result<HttpResponse, AppError> {
    // ... (rest of the code remains the same)
    let cdk_repo = CdkRepositoryImpl::new(&state.db);
    let auth_repo = AuthRepositoryImpl::new(&state.db);
    let usecase = CdkUseCase::new(cdk_repo, auth_repo);
    let validation = usecase.validate_account(body.into()).await?;

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

    let node_ids: Vec<String> = bindings.into_iter().map(|b| b.node_id).collect();
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
                vmess_uuid: n.vmess_uuid.clone(),
                vmess_server: n.vmess_server.clone(),
                vmess_port: n.vmess_port,
                vmess_email: n.vmess_email.clone(),
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
            node_id: Set(node_id),
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
