use ntex::http::StatusCode;
use ntex::web::types::{Json, Query, State};
use ntex::web::{self, HttpResponse};

use crate::application::accelerator_usecase::AcceleratorUseCase;
use crate::application::auth_usecase::AuthUseCase;
use crate::application::content_usecase::ContentUseCase;
use crate::infrastructure::persistence::repositories::{
    AcceleratorRepositoryImpl, AuthRepositoryImpl, ConfigRepositoryImpl,
};

use super::AppState;
use super::dto::{
    AcceleratorBootstrapVO, AccountLoginRequestVO, AccountLoginResponseVO, DashboardVO, LibraryVO,
    NavigationVO, ProfileSyncRequest, SettingsMetaVO, TicketRequestVO, TicketStatusQuery,
    WechatTicketVO,
};
use super::errors::{ApiResponse, AppError, MessageResponse};

#[web::get("/api/accelerator/bootstrap")]
pub async fn accelerator_bootstrap(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = AcceleratorRepositoryImpl::new(&state.db);
    let usecase = AcceleratorUseCase::new(repo);
    let payload = usecase.bootstrap().await?;
    Ok(ApiResponse::success(AcceleratorBootstrapVO::from(payload)).into_http(StatusCode::OK))
}

#[web::post("/api/accelerator/profiles")]
pub async fn sync_profiles(
    state: State<AppState>,
    Json(body): Json<ProfileSyncRequest>,
) -> Result<HttpResponse, AppError> {
    let repo = AcceleratorRepositoryImpl::new(&state.db);
    let usecase = AcceleratorUseCase::new(repo);
    let profiles: Vec<_> = body.into();
    usecase.sync_profiles(profiles).await?;
    Ok(ApiResponse::success(MessageResponse {
        message: "Profiles updated".into(),
    })
    .into_http(StatusCode::OK))
}

#[web::get("/api/dashboard")]
pub async fn dashboard(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: DashboardVO = usecase.dashboard().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/api/library")]
pub async fn library(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: LibraryVO = usecase.library().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/api/settings/meta")]
pub async fn settings(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: SettingsMetaVO = usecase.settings().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::get("/api/navigation")]
pub async fn navigation(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let config = ConfigRepositoryImpl::new(&state.db);
    let usecase = ContentUseCase::new(config);
    let payload: NavigationVO = usecase.navigation().await?;
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

#[web::post("/api/auth/wechat/ticket")]
pub async fn create_wechat_ticket(
    state: State<AppState>,
    Json(body): Json<TicketRequestVO>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let ticket = usecase.create_ticket(body.scene).await?;
    Ok(ApiResponse::success(WechatTicketVO::from(ticket)).into_http(StatusCode::CREATED))
}

#[web::get("/api/auth/wechat/status")]
pub async fn wechat_status(
    state: State<AppState>,
    Query(query): Query<TicketStatusQuery>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let ticket = usecase.ticket_status(&query.ticket_id).await?;
    Ok(ApiResponse::success(WechatTicketVO::from(ticket)).into_http(StatusCode::OK))
}

#[web::post("/api/auth/account")]
pub async fn account_login(
    state: State<AppState>,
    Json(body): Json<AccountLoginRequestVO>,
) -> Result<HttpResponse, AppError> {
    let repo = AuthRepositoryImpl::new(&state.db);
    let usecase = AuthUseCase::new(repo);
    let response = usecase.account_login(body.into()).await?;
    Ok(ApiResponse::success(AccountLoginResponseVO::from(response)).into_http(StatusCode::OK))
}
