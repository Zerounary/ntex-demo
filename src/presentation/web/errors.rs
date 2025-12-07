use ntex::http::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use sea_orm::DbErr;

use super::dto::ApiResponse;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
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

