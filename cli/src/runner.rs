use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
    #[error("Failed to execute binary: {0}")]
    NonZeroExit(i32),
}

pub struct BinaryRunner;

impl BinaryRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run_interactive(&self, binary_path: &Path, args: Vec<String>) -> Result<()> {
        if !binary_path.exists() {
            return Err(RunnerError::BinaryNotFound(binary_path.display().to_string()).into());
        }

        tracing::info!("Running interactive binary: {:?}", binary_path);

        // For interactive TUI applications, we need to handle terminal modes properly
        crossterm::terminal::disable_raw_mode().ok();

        let status = if cfg!(unix) {
            // On Unix, use exec-style to replace the current process temporarily
            Command::new(binary_path)
                .args(args)
                .status()
                .with_context(|| format!("Failed to execute binary: {}", binary_path.display()))?
        } else {
            // On Windows, just run normally
            Command::new(binary_path)
                .args(args)
                .status()
                .with_context(|| format!("Failed to execute binary: {}", binary_path.display()))?
        };

        if !status.success() {
            if let Some(code) = status.code() {
                if code != 0 && code != 130 {
                    // 130 is SIGINT (Ctrl+C)
                    tracing::warn!("Binary exited with code: {}", code);
                }
            }
        }

        Ok(())
    }

    pub fn check_binary_exists(&self, binary_path: &Path) -> bool {
        binary_path.exists() && binary_path.is_file()
    }

    pub fn which(&self, binary_name: &str) -> Option<PathBuf> {
        // Try to find the binary in PATH
        if let Ok(output) = Command::new("which").arg(binary_name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }

        // On Windows, try 'where' command
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("where").arg(binary_name).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .map(|s| s.trim().to_string());
                    if let Some(path) = path {
                        if !path.is_empty() {
                            return Some(PathBuf::from(path));
                        }
                    }
                }
            }
        }

        None
    }
}
