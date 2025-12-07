use ntex::http::StatusCode;
use ntex::web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
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

