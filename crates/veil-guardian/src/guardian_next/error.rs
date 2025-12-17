use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardianNextError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status error: {status} {body}")]
    HttpStatus { status: u16, body: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("offline and no usable cache: {0}")]
    NoUsableCache(String),

    #[error("timeout budget exceeded")]
    TimeoutBudget,

    #[error("internal: {0}")]
    Internal(String),
}

impl GuardianNextError {
    pub fn http_status(status: reqwest::StatusCode, body: String) -> Self {
        Self::HttpStatus {
            status: status.as_u16(),
            body,
        }
    }
}
