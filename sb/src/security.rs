//! Security utilities for safe file system operations
//!
//! This module provides functions to prevent common security vulnerabilities
//! such as path traversal attacks and resource exhaustion.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum file size allowed for operations (10MB)
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum preview size (1MB)
pub const MAX_PREVIEW_SIZE: u64 = 1024 * 1024;

/// Security-related errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Path contains directory traversal")]
    PathTraversal,
    #[error("File too large: {0} bytes")]
    FileTooLarge(u64),
    #[error("Access denied")]
    AccessDenied,
    #[error("Invalid path")]
    InvalidPath,
    #[error("Path contains null byte")]
    NullByte,
    #[error("Path is hidden")]
    HiddenPath,
    #[error("Path is absolute")]
    AbsolutePath,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Validates that a path is safe and within allowed boundaries
///
/// This function prevents path traversal attacks by ensuring paths are relative,
/// don't contain traversal patterns, and don't access hidden files.
pub fn validate_path(path: &Path, base_dir: &Path) -> Result<PathBuf, SecurityError> {
    let path_str = path.to_string_lossy();

    // Check for null bytes
    if path_str.contains('\0') {
        return Err(SecurityError::NullByte);
    }

    // Check if path is absolute
    if path.is_absolute() {
        return Err(SecurityError::AbsolutePath);
    }

    // Check for directory traversal patterns
    if path_str.contains("../") || path_str.contains("..\\") {
        return Err(SecurityError::PathTraversal);
    }

    // Check for exact ".." component
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(SecurityError::PathTraversal);
        }
    }

    // Check for hidden files/directories (starting with .)
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            let component_str = os_str.to_string_lossy();
            if component_str.starts_with('.') {
                return Err(SecurityError::HiddenPath);
            }
        }
    }

    // Resolve the path within the base directory
    let canonical_base = base_dir.canonicalize()?;
    let resolved_path = canonical_base.join(path);

    // Ensure the resolved path is still within base directory
    match resolved_path.canonicalize() {
        Ok(canonical_path) => {
            if canonical_path.starts_with(&canonical_base) {
                Ok(canonical_path)
            } else {
                Err(SecurityError::PathTraversal)
            }
        }
        Err(_) => {
            // If path doesn't exist, just validate it's within bounds without resolving
            if resolved_path.starts_with(&canonical_base) {
                Ok(resolved_path)
            } else {
                Err(SecurityError::PathTraversal)
            }
        }
    }
}

/// Checks if a file size is within allowed limits
///
/// Returns the file size if it's within limits, or an error if too large.
pub fn check_file_size(path: &Path) -> Result<u64, SecurityError> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    if size > MAX_FILE_SIZE {
        Err(SecurityError::FileTooLarge(size))
    } else {
        Ok(size)
    }
}

/// Sanitizes error messages to prevent information leakage
///
/// Removes sensitive path information from error messages that will be
/// displayed to users.
pub fn sanitize_error_message(error: &str, _path: &Path) -> String {
    let mut sanitized = error.to_string();

    // Remove common sensitive patterns
    let sensitive_patterns = [
        // Unix file paths
        (r"/[a-zA-Z0-9_./\-]+", "file"),
        // Windows file paths
        (r"[A-Za-z]:\\[a-zA-Z0-9_\\\.\-]+", "file"),
        // IP addresses
        (r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b", "[IP]"),
        // Passwords and credentials
        (r"password[=:]\s*[^\s]+", "password=[REDACTED]"),
        (r"pwd[=:]\s*[^\s]+", "pwd=[REDACTED]"),
        (r"token[=:]\s*[^\s]+", "token=[REDACTED]"),
        (r"key[=:]\s*[^\s]+", "key=[REDACTED]"),
        // Database connection strings
        (r"postgresql://[^\s]+", "postgresql://[REDACTED]"),
        (r"mysql://[^\s]+", "mysql://[REDACTED]"),
        (r"mongodb://[^\s]+", "mongodb://[REDACTED]"),
        // URLs with credentials
        (r"https?://[^@\s]+@[^\s]+", "https://[REDACTED]"),
        // Environment variable patterns
        (r"[A-Z_]+=[^\s]+", "[ENV_VAR]=[REDACTED]"),
    ];

    for (pattern, replacement) in &sensitive_patterns {
        if let Ok(re) = Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, *replacement).to_string();
        }
    }

    // Additional conservative sanitization for remaining path-like content
    let words: Vec<&str> = sanitized.split_whitespace().collect();
    let sanitized_words: Vec<String> = words
        .iter()
        .map(|word| {
            // If word contains path-like separators and looks like a path, redact it
            if (word.contains('/') && word.len() > 3)
                || (word.contains('\\') && word.len() > 3 && word.contains(':'))
            {
                "file".to_string()
            } else {
                word.to_string()
            }
        })
        .collect();

    sanitized_words.join(" ")
}
