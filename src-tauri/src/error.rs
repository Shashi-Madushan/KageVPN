use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Invalid(String),
    #[error("profile not found")]
    ProfileNotFound,
    #[error("sing-box not found, install it with `brew install sing-box`")]
    SingBoxMissing,
    #[error("already connecting or connected")]
    Busy,
    #[error("config check failed: {0}")]
    ConfigCheck(String),
    #[error("administrator permission was denied")]
    PermissionDenied,
    #[error("{0}")]
    Launch(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
