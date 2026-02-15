use std::fmt;

/// Domain-specific error kinds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorKind {
    /// A universal error from the shared crate.
    #[error("{0}")]
    Shared(#[from] shared::ErrorKind),

    /// A value does not match the expected value from a related entity.
    #[error("expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
}

impl shared::ErrorKindInfo for ErrorKind {
    fn default_status(&self) -> u16 {
        match self {
            Self::Shared(e) => e.default_status(),
            Self::Mismatch { .. } => 422,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Shared(e) => e.title(),
            Self::Mismatch { .. } => "Value mismatch",
        }
    }
}

// ── Convenience constructors ─────────────────────────────────────

impl ErrorKind {
    pub fn non_empty() -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::NonEmpty))
    }

    pub fn non_negative(actual: impl fmt::Display) -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::NonNegative {
            actual: actual.to_string(),
        }))
    }

    pub fn invalid_format(value: impl fmt::Display) -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::InvalidFormat {
            value: value.to_string(),
        }))
    }

    pub fn conflicting_values(values: &[impl fmt::Display]) -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::ConflictingValues {
            values: values.iter().map(|v| v.to_string()).collect(),
        }))
    }

    pub fn not_found() -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::NotFound))
    }

    pub fn internal(msg: impl fmt::Display) -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::Internal))
            .with_detail(msg.to_string())
    }

    pub fn duplicate_values(value: impl fmt::Display) -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::DuplicateValues {
            value: value.to_string(),
        }))
    }

    pub fn conflict() -> Error {
        shared::ContextualError::new(Self::Shared(shared::ErrorKind::Conflict))
    }

    pub fn mismatch(expected: impl fmt::Display, actual: impl fmt::Display) -> Error {
        shared::ContextualError::new(Self::Mismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

// ── Type aliases (define complex, use simple) ────────────────────

/// `domain::Error` — use this everywhere, no Box needed.
pub type Error = shared::ContextualError<ErrorKind>;

/// `domain::Result<T>` — clean return type.
pub type Result<T> = std::result::Result<T, Error>;
