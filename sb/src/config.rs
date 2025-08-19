//! Configuration management for Saorsa Browser

use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Base directory for file operations
    pub base_directory: PathBuf,
    /// Maximum file size allowed for operations
    pub max_file_size: u64,
    /// Maximum preview size
    pub max_preview_size: u64,
    /// Whether to allow hidden files
    pub allow_hidden_files: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            max_file_size: crate::security::MAX_FILE_SIZE,
            max_preview_size: crate::security::MAX_PREVIEW_SIZE,
            allow_hidden_files: false,
        }
    }
}

impl Config {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base directory
    pub fn with_base_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.base_directory = path.into();
        self
    }

    /// Set the maximum file size
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Enable or disable hidden files
    pub fn with_hidden_files(mut self, allow: bool) -> Self {
        self.allow_hidden_files = allow;
        self
    }
}
