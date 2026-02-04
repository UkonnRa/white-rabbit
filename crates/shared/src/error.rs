/// Shared error type for core domain and infrastructure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("value must be non empty")]
    NonEmpty,
    #[error("Non negative values expected, but got {0}")]
    NonNegativeValue(String),
}

/// Shared result type.
pub type Result<T> = std::result::Result<T, Error>;
