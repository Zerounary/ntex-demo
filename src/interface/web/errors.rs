use ntex::http::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use serde::Serialize;

use crate::application::errors::UsecaseError;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AppError(#[from] UsecaseError);

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            UsecaseError::NotFound(_) => StatusCode::NOT_FOUND,
            UsecaseError::Validation(_) => StatusCode::BAD_REQUEST,
            UsecaseError::Unauthorized => StatusCode::UNAUTHORIZED,
            UsecaseError::Expired => StatusCode::BAD_REQUEST,
            UsecaseError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        let status = self.status_code();
        let code = match self.0 {
            UsecaseError::NotFound(_) => "NOT_FOUND",
            UsecaseError::Validation(_) => "BAD_REQUEST",
            UsecaseError::Unauthorized => "UNAUTHORIZED",
            UsecaseError::Expired => "EXPIRED",
            UsecaseError::Repository(_) => "INTERNAL_ERROR",
        };

        ApiResponse::<MessageResponse>::error(code, self.0.to_string()).into_http(status)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorBody>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &'static str, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiErrorBody { code, message }),
        }
    }

    pub fn into_http(self, status: StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(&self)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub message: String,
}
