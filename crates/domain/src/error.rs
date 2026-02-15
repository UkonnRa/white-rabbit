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
//
// DO NOT duplicate `shared::ErrorKind` convenience constructors here.
// For shared error variants (NotFound, DuplicateValues, Conflict, etc.),
// use `shared::ErrorKind::xxx().convert()` at the call site instead.
//
// Only domain-specific error kinds that don't exist in `shared` should
// have constructors here.

impl ErrorKind {
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
