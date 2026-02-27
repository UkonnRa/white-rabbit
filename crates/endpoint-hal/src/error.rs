use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// RFC 9457 Problem Details response body.
#[derive(Debug, Clone, Serialize)]
pub struct ProblemDetail {
    pub r#type: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl ProblemDetail {
    pub fn from_domain_error(err: domain::error::Error, instance: impl Into<String>) -> Self {
        let status = shared::ErrorKindInfo::default_status(&err.error);
        let title = shared::ErrorKindInfo::title(&err.error).to_string();
        let detail = err
            .context
            .detail
            .clone()
            .or_else(|| Some(err.error.to_string()));
        let field = err.context.field.clone();

        Self {
            r#type: "about:blank".to_string(),
            title,
            status,
            detail,
            instance: Some(instance.into()),
            field,
        }
    }

    pub fn not_found(instance: impl Into<String>) -> Self {
        Self {
            r#type: "about:blank".to_string(),
            title: "Not found".to_string(),
            status: 404,
            detail: None,
            instance: Some(instance.into()),
            field: None,
        }
    }
}

impl IntoResponse for ProblemDetail {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(&self).unwrap_or_default();

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response()
    }
}
