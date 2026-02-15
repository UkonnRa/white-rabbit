use std::fmt;

// ── What went wrong ──────────────────────────────────────────────

/// Classification of the error, carrying the error's own data.
/// Reusable across all projects. Maps to RFC 9457 `type` / JSON:API `code`.
///
/// This carries **what** went wrong and **what value** triggered it,
/// but NOT **where** (resource type, field, pointer) — that context
/// is on [`Error`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    // Validation constraints
    NonEmpty,
    NonNegative { actual: String },
    DuplicateValues { value: String },
    ConflictingValues { values: Vec<String> },
    InvalidFormat { value: String },

    // Resource / access
    NotFound,
    Unauthorized,
    Forbidden,
    Conflict,

    // Infrastructure
    Internal,
}

impl ErrorKind {
    /// Default HTTP status code for this kind (RFC 9457 `status`).
    pub fn default_status(&self) -> u16 {
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

    /// Stable, human-readable title (RFC 9457 `title` / JSON:API `title`).
    /// SHOULD NOT change from occurrence to occurrence of the problem.
    pub fn title(&self) -> &'static str {
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

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonEmpty => write!(f, "value must be non-empty"),
            Self::NonNegative { actual } => {
                write!(f, "non-negative value expected, got {actual}")
            }
            Self::DuplicateValues { value } => write!(f, "duplicate value: {value}"),
            Self::ConflictingValues { values } => {
                write!(f, "values cannot coexist: {}", values.join(", "))
            }
            Self::InvalidFormat { value } => write!(f, "cannot parse \"{value}\""),
            Self::NotFound => write!(f, "not found"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Forbidden => write!(f, "forbidden"),
            Self::Conflict => write!(f, "conflict"),
            Self::Internal => write!(f, "internal error"),
        }
    }
}

// ── Where it went wrong ──────────────────────────────────────────

/// Points to the origin of the error in the request.
/// Aligns with JSON:API `source` object and RFC 9457 extension members.
#[derive(Debug, Clone, Default)]
pub struct ErrorSource {
    /// JSON Pointer (RFC 6901) into the request document,
    /// e.g. "/data/attributes/amount".
    pub pointer: Option<String>,
    /// URI query parameter, e.g. "filter[status]".
    pub parameter: Option<String>,
    /// Request header name.
    pub header: Option<String>,
}

// ── The unified error ────────────────────────────────────────────

/// A single problem occurrence.
/// Modeled after RFC 9457 Problem Details and JSON:API Error Objects.
///
/// - [`ErrorKind`] carries **what** went wrong and its intrinsic data.
/// - The context fields here carry **where** it went wrong.
/// - `detail` is an optional human-readable override (RFC 9457 `detail`);
///   when absent, `ErrorKind::Display` provides the occurrence-specific message.
pub struct Error {
    // ─ identity ─
    /// The classification of the error, with its intrinsic data.
    pub kind: ErrorKind,

    // ─ domain-level context (set by domain layer) ─
    /// The resource type, e.g. "Record".
    pub resource_type: Option<&'static str>,
    /// The field name, e.g. "amount".
    pub field: Option<String>,

    // ─ occurrence-specific detail ─
    /// Optional human-readable detail override (RFC 9457 `detail`).
    /// When absent, `ErrorKind`'s `Display` is used instead.
    pub detail: Option<String>,

    // ─ request-level source (set by endpoint layer) ─
    /// Where in the request the error originated (JSON:API `source`).
    pub source: ErrorSource,

    // ─ cause chain ─
    /// The underlying cause, if any.
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            resource_type: None,
            field: None,
            detail: None,
            source: ErrorSource::default(),
            cause: None,
        }
    }

    // ── Convenience constructors ────────────────────────────────

    pub fn non_empty() -> Self {
        Self::new(ErrorKind::NonEmpty)
    }

    pub fn non_negative(actual: impl fmt::Display) -> Self {
        Self::new(ErrorKind::NonNegative {
            actual: actual.to_string(),
        })
    }

    pub fn invalid_format(value: impl fmt::Display) -> Self {
        Self::new(ErrorKind::InvalidFormat {
            value: value.to_string(),
        })
    }

    pub fn duplicate_values(value: impl fmt::Display) -> Self {
        Self::new(ErrorKind::DuplicateValues {
            value: value.to_string(),
        })
    }

    pub fn conflicting_values(values: &[impl fmt::Display]) -> Self {
        Self::new(ErrorKind::ConflictingValues {
            values: values.iter().map(|v| v.to_string()).collect(),
        })
    }

    pub fn not_found() -> Self {
        Self::new(ErrorKind::NotFound)
    }

    pub fn internal(msg: impl fmt::Display) -> Self {
        Self::new(ErrorKind::Internal).with_detail(msg.to_string())
    }

    // ── Builder methods for enriching with context ──────────────

    pub fn with_resource_type(mut self, typ: &'static str) -> Self {
        self.resource_type = Some(typ);
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.source.pointer = Some(pointer.into());
        self
    }

    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.source.parameter = Some(parameter.into());
        self
    }

    pub fn with_cause(mut self, cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Error");
        d.field("kind", &self.kind);
        if let Some(typ) = self.resource_type {
            d.field("resource_type", &typ);
        }
        if let Some(field) = &self.field {
            d.field("field", field);
        }
        if let Some(detail) = &self.detail {
            d.field("detail", detail);
        }
        if self.source.pointer.is_some()
            || self.source.parameter.is_some()
            || self.source.header.is_some()
        {
            d.field("source", &self.source);
        }
        if let Some(cause) = &self.cause {
            d.field("cause", cause);
        }
        d.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Context prefix: "Record.amount: "
        if let Some(typ) = self.resource_type {
            write!(f, "{typ}")?;
            if let Some(field) = &self.field {
                write!(f, ".{field}")?;
            }
            write!(f, ": ")?;
        } else if let Some(field) = &self.field {
            write!(f, "{field}: ")?;
        }

        // ErrorKind carries the occurrence-specific message
        write!(f, "{}", self.kind)?;

        // Optional detail override (appended if present)
        if let Some(detail) = &self.detail {
            write!(f, " ({detail})")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Shared result type.
pub type Result<T> = std::result::Result<T, Error>;
