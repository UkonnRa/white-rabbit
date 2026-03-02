use log::error;
use serde::Serialize;
use shared::ErrorKindInfo;

#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub field: Option<String>,
}

impl CommandError {
    pub fn from_domain(err: domain::error::Error) -> Self {
        let title = err.error.title().to_string();
        let status = err.error.default_status();
        let detail = err
            .context
            .detail
            .clone()
            .or_else(|| Some(err.error.to_string()));
        let field = err.context.field.clone();

        error!("status={status} title=\"{title}\" detail={detail:?} field={field:?}");

        Self {
            title,
            status,
            detail,
            field,
        }
    }

    pub fn from_shared(err: shared::Error) -> Self {
        Self::from_domain(err.convert())
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)?;
        if let Some(detail) = &self.detail {
            write!(f, ": {detail}")?;
        }
        Ok(())
    }
}

impl From<CommandError> for String {
    fn from(err: CommandError) -> String {
        serde_json::to_string(&err).unwrap_or_else(|_| err.to_string())
    }
}
