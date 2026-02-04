#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Duplicate values found for {typ} {field}: {value}")]
    DuplicateValues {
        typ: &'static str,
        field: String,
        value: String,
    },
    #[error("{typ} {field} must be non empty")]
    NonEmpty { typ: &'static str, field: String },
    #[error("Non negative values required for {typ} {field}, the current value is {current}")]
    NonNegativeValue {
        typ: &'static str,
        field: String,
        current: String,
    },
    #[error("The values cannot exist same time for {typ} {field}: {values:?}")]
    CannotExistSameTime {
        typ: &'static str,
        field: String,
        values: Vec<String>,
    },
    #[error("Invalid date: {value}")]
    InvalidDate {
        value: String,
        source: chrono::ParseError,
    },
    #[error("Invalid number: {value}")]
    InvalidNumber {
        value: String,
        source: rust_decimal::Error,
    },
}

impl From<Error> for shared::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::NonEmpty { .. } => shared::Error::NonEmpty,
            Error::NonNegativeValue { current, .. } => shared::Error::NonNegativeValue(current),
            _ => {
                // For other error types that don't map directly, we can't convert
                // This is a fallback that shouldn't normally be used
                shared::Error::NonEmpty
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
