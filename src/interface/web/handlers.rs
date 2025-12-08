use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};

use crate::application::accelerator_usecase::AcceleratorUseCase;
use crate::application::auth_usecase::AuthUseCase;
use crate::application::cdk_usecase::CdkUseCase;
use crate::application::content_usecase::ContentUseCase;
use crate::application::node_usecase::NodeUseCase;
use crate::infrastructure::persistence::repositories::{
    AcceleratorRepositoryImpl, AuthRepositoryImpl, CdkRepositoryImpl, ConfigRepositoryImpl,
    NodeRepositoryImpl,
};

use super::AppState;
use super::dto::{
    AcceleratorBootstrapVO, AccountLoginRequestVO, AccountLoginResponseVO,
    AccountValidationRequestVO, AccountValidationResponseVO, CdkCodeVO, CdkGenerateRequestVO,
    CdkRedeemRequestVO, CdkRedeemResponseVO, DashboardVO, LibraryVO, NavigationVO,
    NodeRegisterRequest, ProfileSyncRequest, SettingsMetaVO, TicketRequestVO, TicketStatusQuery,
    WechatTicketVO,
};
use super::errors::{ApiResponse, AppError, MessageResponse};

#[web::get("/accelerator/bootstrap")]
pub async fn accelerator_bootstrap(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = AcceleratorRepositoryImpl::new(&state.db);
    let node_repo = NodeRepositoryImpl::new(&state.db);
    let usecase = AcceleratorUseCase::new(repo, node_repo);
    let payload = usecase.bootstrap().await?;
    Ok(ApiResponse::success(AcceleratorBootstrapVO::from(payload)).into_http(StatusCode::OK))
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
    Ok(ApiResponse::success(AccountValidationResponseVO::from(response))
        .into_http(StatusCode::OK))
}

#[web::post("/accelerator/start")]
pub async fn start_acceleration(
    state: State<AppState>,
    Json(body): Json<AccountValidationRequestVO>,
) -> Result<HttpResponse, AppError> {
    // 验证账号是否付费或过期
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
