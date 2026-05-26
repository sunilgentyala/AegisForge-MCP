use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum AegisError {
    #[error("invalid target: {0}")]
    InvalidTarget(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("sandbox error: {0}")]
    SandboxError(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("sanitization failed: {0}")]
    SanitizationFailed(String),
}

impl From<std::io::Error> for AegisError {
    fn from(e: std::io::Error) -> Self {
        AegisError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AegisError {
    fn from(e: serde_json::Error) -> Self {
        AegisError::Parse(e.to_string())
    }
}

pub type AegisResult<T> = Result<T, AegisError>;

// Tauri commands must return Result<T, String> — this impl enables the
// '?' operator inside commands to convert automatically.
impl From<AegisError> for String {
    fn from(e: AegisError) -> Self {
        e.to_string()
    }
}
