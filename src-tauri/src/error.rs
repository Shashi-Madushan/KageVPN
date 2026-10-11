use serde::{Deserialize, Serialize, Serializer};

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
    /// The user asked to disconnect while the connection was still starting.
    #[error("cancelled")]
    Cancelled,
    #[error("config check failed: {0}")]
    ConfigCheck(String),
    #[error("administrator permission was denied")]
    PermissionDenied,
    #[error("{0}")]
    Launch(String),
    #[error("{0}")]
    ServerUnreachable(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

/// An error described for people: what happened, why, and what they can do about it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub title: String,
    pub detail: String,
    /// `retry`, `installEngine` or `diagnostics`; the UI offers the matching button.
    pub action: Option<String>,
}

impl AppError {
    /// A cancelled connection is something the user did, not a failure to report.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, AppError::Cancelled)
    }

    pub fn info(&self) -> ErrorInfo {
        let detail = self.to_string();
        let (code, title, action) = match self {
            AppError::SingBoxMissing => ("engineMissing", "The VPN engine is not installed", Some("installEngine")),
            AppError::Busy => ("busy", "Already connecting", None),
            AppError::Cancelled => ("cancelled", "Cancelled", None),
            AppError::ConfigCheck(_) => ("configInvalid", "The configuration was rejected", Some("diagnostics")),
            AppError::PermissionDenied => ("permissionDenied", "Administrator permission was denied", Some("retry")),
            AppError::ServerUnreachable(_) => ("serverUnreachable", "The server cannot be reached", Some("retry")),
            AppError::Launch(message) if message.contains("already in use") => ("portInUse", "The local port is busy", Some("retry")),
            AppError::Launch(message) if message.contains("did not start") => ("startTimeout", "The engine did not start", Some("diagnostics")),
            AppError::Launch(_) => ("launchFailed", "The connection could not start", Some("retry")),
            AppError::Invalid(_) => ("invalid", "Something needs attention", None),
            _ => ("internal", "Something went wrong", Some("diagnostics")),
        };
        ErrorInfo { code: code.into(), title: title.into(), detail, action: action.map(Into::into) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_actionable_info() {
        assert_eq!(AppError::SingBoxMissing.info().action.as_deref(), Some("installEngine"));
        let busy = AppError::Launch("port 2080 is already in use, is another sing-box running?".into()).info();
        assert_eq!(busy.code, "portInUse");
        assert_eq!(AppError::ServerUnreachable("x".into()).info().code, "serverUnreachable");
        assert_eq!(AppError::Cancelled.info().code, "cancelled");
        assert!(AppError::Cancelled.is_cancelled() && !AppError::Busy.is_cancelled());
    }
}
