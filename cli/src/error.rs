use std::fmt;
use std::path::PathBuf;

/// Custom error type for better error messages
#[derive(Debug)]
pub enum SaorsaError {
    /// File system errors
    Io {
        operation: String,
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    /// Network related errors
    Network { url: String, source: reqwest::Error },
}

impl fmt::Display for SaorsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaorsaError::Io {
                operation,
                path,
                source,
            } => {
                if let Some(p) = path {
                    write!(
                        f,
                        "I/O error during '{}' on '{}': {}",
                        operation,
                        p.display(),
                        source
                    )
                } else {
                    write!(f, "I/O error during '{}': {}", operation, source)
                }
            }
            SaorsaError::Network { url, source } => {
                write!(f, "Network error accessing '{}': {}", url, source)
            }
        }
    }
}

impl std::error::Error for SaorsaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaorsaError::Io { source, .. } => Some(source),
            SaorsaError::Network { source, .. } => Some(source),
        }
    }
}

impl From<std::io::Error> for SaorsaError {
    fn from(err: std::io::Error) -> Self {
        SaorsaError::Io {
            operation: "unknown".to_string(),
            path: None,
            source: err,
        }
    }
}

impl From<reqwest::Error> for SaorsaError {
    fn from(err: reqwest::Error) -> Self {
        SaorsaError::Network {
            url: "unknown".to_string(),
            source: err,
        }
    }
}

/// Helper functions for creating specific error types
#[allow(dead_code)]
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaorsaError {
    #[error("I/O error during '{operation}' on '{path:?}': {source}")]
    Io {
        operation: String,
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    #[error("Network error for url '{url}': {source}")]
    Network { url: String, source: reqwest::Error },
}

impl fmt::Display for SaorsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaorsaError::Io {
                operation,
                path,
                source,
            } => {
                if let Some(p) = path {
                    write!(
                        f,
                        "I/O error during '{}' on '{}': {}",
                        operation,
                        p.display(),
                        source
                    )
                } else {
                    write!(f, "I/O error during '{}': {}", operation, source)
                }
            }
            SaorsaError::Network { url, source } => {
                write!(f, "Network error for url '{}': {}", url, source)
            }
        }
    }
}

impl SaorsaError {
    pub fn io(operation: &str, source: std::io::Error) -> Self {
        SaorsaError::Io {
            operation: operation.to_string(),
            path: None,
            source,
        }
    }

    pub fn io_with_context<P: Into<PathBuf>>(
        operation: &str,
        path: P,
        source: std::io::Error,
    ) -> Self {
        SaorsaError::Io {
            operation: operation.to_string(),
            path: Some(path.into()),
            source,
        }
    }

    pub fn network(url: &str, source: reqwest::Error) -> Self {
        SaorsaError::Network {
            url: url.to_string(),
            source,
        }
    }

    pub fn network_with_url<U: Into<String>>(url: U, source: reqwest::Error) -> Self {
        SaorsaError::Network {
            url: url.into(),
            source,
        }
    }
}
