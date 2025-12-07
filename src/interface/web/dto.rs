use chrono::{DateTime, Utc};
use ntex::http::StatusCode;
use ntex::web::HttpResponse;
use serde::{Deserialize, Serialize};
use struct_convert::Convert;

#[derive(Debug, Deserialize, Convert)]
#[convert(into = "crate::domain::todo::NewTodo")]
pub struct CreateTodoRequest {
    pub title: String,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Deserialize, Convert)]
#[convert(into = "crate::domain::todo::UpdateTodo")]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, Convert)]
#[convert(from = "crate::domain::todo::Todo")]
pub struct TodoVO {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MessageBody {
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn into_http(self, status: StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(&self)
    }
}

impl ApiResponse<()> {
    pub fn error(code: &'static str, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorBody { code, message }),
        }
    }
}

