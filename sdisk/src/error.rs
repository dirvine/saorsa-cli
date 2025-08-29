use std::fmt;
use std::path::PathBuf;

/// Custom error type for Saorsa Disk
#[derive(Debug)]
pub enum SdiskError {
    /// File system related errors
    Io {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },
    /// Progress bar creation errors
    ProgressBar(String),
    /// Directory traversal errors
    WalkDir {
        path: PathBuf,
        source: walkdir::Error,
    },
}

impl fmt::Display for SdiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdiskError::Io {
                operation,
                path,
                source,
            } => {
                write!(
                    f,
                    "File system error during '{}' on '{}': {}",
                    operation,
                    path.display(),
                    source
                )
            }
            SdiskError::ProgressBar(msg) => {
                write!(f, "Progress bar error: {}", msg)
            }
            SdiskError::WalkDir { path, source } => {
                write!(
                    f,
                    "Directory traversal error at '{}': {}",
                    path.display(),
                    source
                )
            }
        }
    }
}

impl std::error::Error for SdiskError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SdiskError::Io { source, .. } => Some(source),
            SdiskError::WalkDir { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SdiskError {
    fn from(err: std::io::Error) -> Self {
        SdiskError::Io {
            operation: "unknown".to_string(),
            path: PathBuf::from("unknown"),
            source: err,
        }
    }
}

impl From<walkdir::Error> for SdiskError {
    fn from(err: walkdir::Error) -> Self {
        SdiskError::WalkDir {
            path: PathBuf::from("unknown"),
            source: err,
        }
    }
}

/// Helper functions for creating specific error types
impl SdiskError {
    pub fn progress_bar<S: Into<String>>(message: S) -> Self {
        SdiskError::ProgressBar(message.into())
    }
}
