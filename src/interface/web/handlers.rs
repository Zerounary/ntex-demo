use ntex::http::StatusCode;
use ntex::web::types::{Json, Path, State};
use ntex::web::HttpResponse;

use crate::application::todo_usecase::TodoUseCase;
use crate::infrastructure::persistence::todo_repository::TodoRepositoryImpl;

use super::dto::{
    ApiResponse, CreateTodoRequest, MessageBody, TodoVO, UpdateTodoRequest,
};
use super::errors::AppError;
use super::AppState;

pub async fn list_todos(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let repo = TodoRepositoryImpl::new(&state.db);
    let usecase = TodoUseCase::new(repo);
    let todos = usecase.list().await?;
    let payload: Vec<TodoVO> = todos.into_iter().map(Into::into).collect();
    Ok(ApiResponse::success(payload).into_http(StatusCode::OK))
}

pub async fn get_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let repo = TodoRepositoryImpl::new(&state.db);
    let usecase = TodoUseCase::new(repo);
    let todo = usecase.get(path.into_inner()).await?;
    Ok(ApiResponse::success(TodoVO::from(todo)).into_http(StatusCode::OK))
}

pub async fn create_todo(
    state: State<AppState>,
    payload: Json<CreateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    let repo = TodoRepositoryImpl::new(&state.db);
    let usecase = TodoUseCase::new(repo);
    let todo = usecase.create(payload.into_inner().into()).await?;
    Ok(ApiResponse::success(TodoVO::from(todo)).into_http(StatusCode::CREATED))
}

pub async fn update_todo(
    state: State<AppState>,
    path: Path<i32>,
    payload: Json<UpdateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    let repo = TodoRepositoryImpl::new(&state.db);
    let usecase = TodoUseCase::new(repo);
    let result = usecase
        .update(path.into_inner(), payload.into_inner().into())
        .await?;
    Ok(ApiResponse::success(TodoVO::from(result)).into_http(StatusCode::OK))
}

pub async fn delete_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let repo = TodoRepositoryImpl::new(&state.db);
    let usecase = TodoUseCase::new(repo);
    usecase.delete(path.into_inner()).await?;

    Ok(ApiResponse::success(MessageBody {
        message: "Todo deleted".into(),
    })
    .into_http(StatusCode::OK))
}

