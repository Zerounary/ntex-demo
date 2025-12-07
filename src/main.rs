mod entity;

use std::env;

use chrono::Utc;
use dotenv::dotenv;
use entity::todo;
use ntex::http::StatusCode;
use ntex::web::{
    self,
    types::{Json, Path, State},
    App,
    HttpRequest,
    HttpResponse,
    WebResponseError,
};
use ntex_files as fs;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbErr, EntityTrait,
    Schema, Set,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
    #[serde(default)]
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T>
where
    T: Serialize,
{
    success: bool,
    data: Option<T>,
    error: Option<ApiErrorBody>,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: String,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn into_http(self, status: StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(&self)
    }
}

impl ApiResponse<()> {
    fn error(code: &'static str, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorBody { code, message }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("validation error: {0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] DbErr),
}

impl AppError {
    fn code_and_status(&self) -> (&'static str, StatusCode) {
        match self {
            Self::NotFound(_) => ("NOT_FOUND", StatusCode::NOT_FOUND),
            Self::Validation(_) => ("BAD_REQUEST", StatusCode::BAD_REQUEST),
            Self::Database(_) => ("INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.code_and_status().1
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        let (code, status) = self.code_and_status();
        ApiResponse::error(code, self.to_string()).into_http(status)
    }
}

#[derive(Debug, Serialize)]
struct MessageBody {
    message: String,
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todo.db?mode=rwc".to_string());

    let db = Database::connect(&database_url)
        .await
        .map_err(to_io_error)?;
    init_db(&db).await.map_err(to_io_error)?;

    let state = AppState { db };

    web::HttpServer::new(move || {
        App::new()
            .state(state.clone())
            .configure(rest_routes)
            .service(
                fs::Files::new("/", "./static")
                    .show_files_listing()
                    .index_file("index.html"),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

fn to_io_error(err: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}

fn rest_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/todos")
                    .route(web::get().to(list_todos))
                    .route(web::post().to(create_todo)),
            )
            .service(
                web::resource("/todos/{id}")
                    .route(web::get().to(get_todo))
                    .route(web::put().to(update_todo))
                    .route(web::delete().to(delete_todo)),
            ),
    );
}

async fn init_db(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let mut stmt = schema.create_table_from_entity(todo::Entity);
    stmt.if_not_exists();
    db.execute(backend.build(&stmt)).await?;
    Ok(())
}

async fn list_todos(state: State<AppState>) -> Result<HttpResponse, AppError> {
    let todos = todo::Entity::find().all(&state.db).await?;
    Ok(ApiResponse::success(todos).into_http(StatusCode::OK))
}

async fn get_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let todo = todo::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Todo"))?;

    Ok(ApiResponse::success(todo).into_http(StatusCode::OK))
}

async fn create_todo(
    state: State<AppState>,
    payload: Json<CreateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    let title = validate_title(&payload.title)?;

    let active_model = todo::ActiveModel {
        title: Set(title),
        completed: Set(payload.completed),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    let todo = active_model.insert(&state.db).await?;
    Ok(ApiResponse::success(todo).into_http(StatusCode::CREATED))
}

async fn update_todo(
    state: State<AppState>,
    path: Path<i32>,
    payload: Json<UpdateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    if payload.title.is_none() && payload.completed.is_none() {
        return Err(AppError::Validation(
            "Provide at least one field to update.".into(),
        ));
    }

    let existing = todo::Entity::find_by_id(path.into_inner())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Todo"))?;

    let mut model: todo::ActiveModel = existing.into();

    if let Some(title) = &payload.title {
        model.title = Set(validate_title(title)?);
    }

    if let Some(completed) = payload.completed {
        model.completed = Set(completed);
    }

    let todo = model.update(&state.db).await?;
    Ok(ApiResponse::success(todo).into_http(StatusCode::OK))
}

async fn delete_todo(
    state: State<AppState>,
    path: Path<i32>,
) -> Result<HttpResponse, AppError> {
    let result = todo::Entity::delete_by_id(path.into_inner())
        .exec(&state.db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound("Todo"));
    }

    Ok(ApiResponse::success(MessageBody {
        message: "Todo deleted".into(),
    })
    .into_http(StatusCode::OK))
}

fn validate_title(input: &str) -> Result<String, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err(AppError::Validation("Title cannot be empty.".into()))
    } else {
        Ok(trimmed.to_string())
    }
}