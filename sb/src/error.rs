use std::fmt;
use std::path::PathBuf;

/// Custom error type for Saorsa Browser
#[derive(Debug)]
pub enum SbError {
    /// File system related errors
    Io {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },
    /// Git related errors
    Git {
        operation: String,
        source: git2::Error,
    },
    /// Tree widget errors
    TreeWidget(String),
}

impl fmt::Display for SbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SbError::Io {
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
            SbError::Git { operation, source } => {
                write!(f, "Git error during '{}': {}", operation, source)
            }
            SbError::TreeWidget(msg) => {
                write!(f, "Tree widget error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SbError::Io { source, .. } => Some(source),
            SbError::Git { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SbError {
    fn from(err: std::io::Error) -> Self {
        SbError::Io {
            operation: "unknown".to_string(),
            path: PathBuf::from("unknown"),
            source: err,
        }
    }
}

impl From<git2::Error> for SbError {
    fn from(err: git2::Error) -> Self {
        SbError::Git {
            operation: "unknown".to_string(),
            source: err,
        }
    }
}

/// Helper functions for creating specific error types
impl SbError {
    pub fn tree_widget<S: Into<String>>(message: S) -> Self {
        SbError::TreeWidget(message.into())
    }
}
