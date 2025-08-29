use anyhow::{Context, Result};
#[allow(dead_code)]
use anyhow::{Context, Result};
use std::path::Path;
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

        let status = if cfg!(unix) {
            use std::os::unix::process::CommandExt;
            Command::new(binary_path)
                .args(args)
                .exec();
            // This part is tricky because exec replaces the current process.
            // We might not get here if exec is successful.
            // Consider using a different approach if you need to get the status.
            return Ok(());
        } else {
            // On Windows, just run normally
            Command::new(binary_path)
                .args(args)
                .status()
                .with_context(|| format!("Failed to execute binary: {}", binary_path.display()))?
        };

        if !status.success() {
            // The process has exited with a non-zero status code.
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
        binary_path.exists()
    }

    pub fn which(&self, binary_name: &str) -> Option<PathBuf> {
        use std::env;
        use std::path::PathBuf;

        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(binary_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
        })
    }
}
