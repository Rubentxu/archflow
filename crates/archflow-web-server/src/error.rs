// ═══════════════════════════════════════════════════════════════════════════════
// Error types for ArchFlow Web Server
// ═══════════════════════════════════════════════════════════════════════════════

use std::path::PathBuf;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the web server
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// React UI dist directory not found
    #[error("React UI dist directory not found: {0:?}")]
    DistPathNotFound(PathBuf),

    /// Diagram not found
    #[error("Diagram not found: {0}")]
    DiagramNotFound(String),

    /// Invalid request data
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),
}

// Implement From conversions for common error types
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(format!("JSON error: {}", err))
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(format!("Error: {}", err))
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Error::DistPathNotFound(path) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("UI dist directory not found: {:?}", path),
            ),
            Error::DiagramNotFound(id) => (
                axum::http::StatusCode::NOT_FOUND,
                format!("Diagram '{}' not found", id),
            ),
            Error::InvalidRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            Error::Internal(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", msg),
            ),
        };

        let body = serde_json::json!({
            "error": message,
        });

        (status, axum::Json(body)).into_response()
    }
}
