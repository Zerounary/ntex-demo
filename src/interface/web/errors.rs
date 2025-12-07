use ntex::http::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};

use crate::application::errors::UsecaseError;

use super::dto::ApiResponse;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AppError(#[from] UsecaseError);

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            UsecaseError::NotFound(_) => StatusCode::NOT_FOUND,
            UsecaseError::Validation(_) => StatusCode::BAD_REQUEST,
            UsecaseError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        let status = self.status_code();
        let code = match self.0 {
            UsecaseError::NotFound(_) => "NOT_FOUND",
            UsecaseError::Validation(_) => "BAD_REQUEST",
            UsecaseError::Repository(_) => "INTERNAL_ERROR",
        };

        ApiResponse::error(code, self.0.to_string()).into_http(status)
    }
}

