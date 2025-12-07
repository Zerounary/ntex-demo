use ntex::http::StatusCode;
use ntex::web::types::{Json, Path, State};
use ntex::web::HttpResponse;

use crate::application::todo_service::TodoService;

use super::dto::{ApiResponse, CreateTodoRequest, MessageBody, UpdateTodoRequest};
use super::errors::AppError;
use super::AppState;

pub async fn list_todos(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let service = TodoService::new(&state.db);
    let todos = service.list().await?;
    Ok(ApiResponse::success(todos).into_http(StatusCode::OK))
}

pub async fn get_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let service = TodoService::new(&state.db);
    let todo = service
        .find(path.into_inner())
        .await?
        .ok_or(AppError::NotFound("Todo"))?;

    Ok(ApiResponse::success(todo).into_http(StatusCode::OK))
}

pub async fn create_todo(
    state: State<AppState>,
    payload: Json<CreateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    let service = TodoService::new(&state.db);
    let title =
        TodoService::validate_title(&payload.title).map_err(AppError::Validation)?;

    let todo = service.create(title, payload.completed).await?;
    Ok(ApiResponse::success(todo).into_http(StatusCode::CREATED))
}

pub async fn update_todo(
    state: State<AppState>,
    path: Path<i32>,
    payload: Json<UpdateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    if payload.title.is_none() && payload.completed.is_none() {
        return Err(AppError::Validation(
            "Provide at least one field to update.".into(),
        ));
    }

    let service = TodoService::new(&state.db);
    let title = match &payload.title {
        Some(title) => Some(
            TodoService::validate_title(title).map_err(AppError::Validation)?,
        ),
        None => None,
    };

    let todo = service
        .update(path.into_inner(), title, payload.completed)
        .await?
        .ok_or(AppError::NotFound("Todo"))?;

    Ok(ApiResponse::success(todo).into_http(StatusCode::OK))
}

pub async fn delete_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let service = TodoService::new(&state.db);
    let deleted = service.delete(path.into_inner()).await?;

    if !deleted {
        return Err(AppError::NotFound("Todo"));
    }

    Ok(ApiResponse::success(MessageBody {
        message: "Todo deleted".into(),
    })
    .into_http(StatusCode::OK))
}

