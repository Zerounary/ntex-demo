use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("persistence error: {0}")]
    Persistence(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Error)]
pub enum UsecaseError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("operation expired")]
    Expired,
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
