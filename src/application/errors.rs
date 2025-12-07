use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("persistence error: {0}")]
    Persistence(String),
}

#[derive(Debug, Error)]
pub enum UsecaseError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("validation error: {0}")]
    Validation(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

