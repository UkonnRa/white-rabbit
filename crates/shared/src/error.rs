use thiserror::Error;

/// Shared error type for core domain and infrastructure.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Permission denied: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("External error: {0}")]
    External(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Shared result type.
pub type Result<T> = std::result::Result<T, Error>;
