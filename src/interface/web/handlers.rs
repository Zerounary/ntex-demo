use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use sha2::{Digest, Sha256};
use uuid::Uuid;

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
    accelerator_user, accelerator_user_credential, accelerator_user_session,
};
use crate::interface::admin::chain_ops;

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
    let outbound_tag = body.outbound_tag;
    let chain_id = body.chain_id;
    let chain_base_port = body.chain_base_port;

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

    // 单用户同一时刻只允许一个 active 会话：存在则先 stop（best effort）
    if let Ok(Some(existing)) = acceleration_session::Entity::find()
        .filter(acceleration_session::Column::UserId.eq(user_id.as_str()))
        .filter(acceleration_session::Column::Status.eq("active"))
        .order_by_desc(acceleration_session::Column::StartedAt)
        .one(&state.db)
        .await
    {
        let _ = state
            .admin_config
            .delete_user(existing.node_id, existing.admin_user_id)
            .await;

        if let Some(mqtt) = state.mqtt_publisher.as_ref() {
            let _ = mqtt.publish_update_notification(existing.node_id, "user").await;
            let _ = mqtt.publish_update_notification(existing.node_id, "inbound").await;
            let _ = mqtt.publish_update_notification(existing.node_id, "outbound").await;
        }

        let mut active: acceleration_session::ActiveModel = existing.into();
        active.status = Set("stopped".to_string());
        active.ended_at = Set(Some(chrono::Utc::now().into()));
        active.updated_at = Set(chrono::Utc::now().into());
        let _ = active.update(&state.db).await;
    }

    // 可选：确保链路端口存在
    if let Some(chain_id) = &chain_id {
        let _ = chain_ops::apply_chain(
            &state.admin_config,
            None,
            chain_id,
            chain_base_port.unwrap_or(40000),
        )
        .await;

        // 链路相关的 inbound/config 变更，需要通知节点刷新
        if let Some(mqtt) = state.mqtt_publisher.as_ref() {
            let _ = mqtt.publish_update_notification(node_id, "inbound").await;
            let _ = mqtt.publish_update_notification(node_id, "config").await;
        }
    }

    let uuid = uuid::Uuid::new_v4().to_string();
    let st = if validation.billing_mode == "pass" { 5u64 } else { 1u64 };

    // 1) 写 DB 下发用户到节点（add_user）
    let admin_user = state
        .admin_config
        .add_user(node_id, uuid, st, 0)
        .await
        .map_err(|e| UsecaseError::Validation(format!("add_user failed: {}", e)))?;

    let admin_user_id = admin_user.id;
    let admin_uuid = admin_user.uuid;

    // 2) 写 DB 下发映射（add_mapping）
    state
        .admin_config
        .add_mapping(node_id, admin_uuid.clone(), outbound_tag.clone())
        .await
        .map_err(|e| UsecaseError::Validation(format!("add_mapping failed: {}", e)))?;

    // 3) 推送节点刷新通知（仅服务端）
    if let Some(mqtt) = state.mqtt_publisher.as_ref() {
        // 参考 admin 接口行为：add_user -> user 更新，add_mapping -> outbound 更新
        let _ = mqtt.publish_update_notification(node_id, "user").await;
        let _ = mqtt.publish_update_notification(node_id, "outbound").await;
        // 某些节点实现对 inbound/config 也敏感，额外触发一次（best effort）
        let _ = mqtt.publish_update_notification(node_id, "inbound").await;
        let _ = mqtt.publish_update_notification(node_id, "config").await;
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let mapped_outbound_tag = outbound_tag;

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

    let _ = state
        .admin_config
        .delete_user(model.node_id, model.admin_user_id)
        .await;

    if let Some(mqtt) = state.mqtt_publisher.as_ref() {
        let _ = mqtt.publish_update_notification(model.node_id, "user").await;
        let _ = mqtt.publish_update_notification(model.node_id, "inbound").await;
        let _ = mqtt.publish_update_notification(model.node_id, "outbound").await;
        let _ = mqtt.publish_update_notification(model.node_id, "config").await;
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
