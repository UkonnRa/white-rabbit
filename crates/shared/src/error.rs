use std::fmt;

// ── Error Kind Trait ─────────────────────────────────────────────

/// Trait for error enums that can be wrapped in [`ContextualError`].
pub trait ErrorKindInfo: fmt::Debug + fmt::Display + Send + Sync + 'static {
    /// Default HTTP status code (RFC 9457 `status`).
    fn default_status(&self) -> u16;
    /// Stable, human-readable title (RFC 9457 `title`).
    fn title(&self) -> &'static str;
}

// ── Shared Error Kind ────────────────────────────────────────────

/// Universal error variants shared across all projects.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorKind {
    #[error("value must be non-empty")]
    NonEmpty,
    #[error("non-negative value expected, got {actual}")]
    NonNegative { actual: String },
    #[error("duplicate value: {value}")]
    DuplicateValues { value: String },
    #[error("values cannot coexist: {}", values.join(", "))]
    ConflictingValues { values: Vec<String> },
    #[error("cannot parse \"{value}\"")]
    InvalidFormat { value: String },
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict")]
    Conflict,
    #[error("internal error")]
    Internal,
}

impl ErrorKindInfo for ErrorKind {
    fn default_status(&self) -> u16 {
        match self {
            Self::NonEmpty
            | Self::NonNegative { .. }
            | Self::DuplicateValues { .. }
            | Self::ConflictingValues { .. }
            | Self::InvalidFormat { .. } => 422,
            Self::NotFound => 404,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::Internal => 500,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::NonEmpty => "Value must be non-empty",
            Self::NonNegative { .. } => "Value must be non-negative",
            Self::DuplicateValues { .. } => "Duplicate values",
            Self::ConflictingValues { .. } => "Conflicting values",
            Self::InvalidFormat { .. } => "Invalid format",
            Self::NotFound => "Not found",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::Conflict => "Conflict",
            Self::Internal => "Internal error",
        }
    }
}

// ── Error Source ─────────────────────────────────────────────────

/// Points to the origin of the error in the request.
/// Aligns with JSON:API `source` object and RFC 9457 extension members.
#[derive(Debug, Clone, Default)]
pub struct ErrorSource {
    pub pointer: Option<String>,
    pub parameter: Option<String>,
    pub header: Option<String>,
}

// ── Error Context (heap-allocated) ───────────────────────────────

/// Optional context fields, heap-allocated so that `ContextualError` stays small.
#[derive(Debug, Default)]
pub struct ErrorContext {
    pub resource_type: Option<&'static str>,
    pub field: Option<String>,
    pub detail: Option<String>,
    pub source: ErrorSource,
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

// ── Contextual Error (stack-friendly) ────────────────────────────

/// A small, stack-friendly error: the error kind + a boxed context.
///
/// `Display` delegates to the inner error. The context fields are accessed
/// programmatically by the endpoint layer when serializing to JSON:API / RFC 9457.
#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct ContextualError<E: ErrorKindInfo> {
    pub error: E,
    pub context: Box<ErrorContext>,
}

impl<E: ErrorKindInfo> ContextualError<E> {
    pub fn new(error: E) -> Self {
        Self {
            error,
            context: Box::new(ErrorContext::default()),
        }
    }

    pub fn with_resource_type(mut self, typ: &'static str) -> Self {
        self.context.resource_type = Some(typ);
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.context.field = Some(field.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.context.detail = Some(detail.into());
        self
    }

    pub fn with_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.context.source.pointer = Some(pointer.into());
        self
    }

    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.context.source.parameter = Some(parameter.into());
        self
    }

    pub fn with_cause(mut self, cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.context.cause = Some(Box::new(cause));
        self
    }

    pub fn map_error<F: ErrorKindInfo>(self, f: impl FnOnce(E) -> F) -> ContextualError<F> {
        ContextualError {
            error: f(self.error),
            context: self.context,
        }
    }

    /// Convert the error kind using `From`, preserving all context.
    /// The target type is inferred from the return position — no turbofish needed.
    pub fn convert<F: ErrorKindInfo + From<E>>(self) -> ContextualError<F> {
        self.map_error(F::from)
    }
}

// ── Convenience constructors ─────────────────────────────────────

impl ErrorKind {
    pub fn non_empty() -> Error {
        ContextualError::new(Self::NonEmpty)
    }

    pub fn non_negative(actual: impl fmt::Display) -> Error {
        ContextualError::new(Self::NonNegative {
            actual: actual.to_string(),
        })
    }

    pub fn invalid_format(value: impl fmt::Display) -> Error {
        ContextualError::new(Self::InvalidFormat {
            value: value.to_string(),
        })
    }

    pub fn duplicate_values(value: impl fmt::Display) -> Error {
        ContextualError::new(Self::DuplicateValues {
            value: value.to_string(),
        })
    }

    pub fn conflicting_values(values: &[impl fmt::Display]) -> Error {
        ContextualError::new(Self::ConflictingValues {
            values: values.iter().map(|v| v.to_string()).collect(),
        })
    }

    pub fn not_found() -> Error {
        ContextualError::new(Self::NotFound)
    }

    pub fn conflict() -> Error {
        ContextualError::new(Self::Conflict)
    }

    pub fn internal(msg: impl fmt::Display) -> Error {
        ContextualError::new(Self::Internal).with_detail(msg.to_string())
    }
}

// ── Type aliases (define complex, use simple) ────────────────────

/// `shared::Error` — use this everywhere, no Box needed.
pub type Error = ContextualError<ErrorKind>;

/// `shared::Result<T>` — clean return type.
pub type Result<T> = std::result::Result<T, Error>;
