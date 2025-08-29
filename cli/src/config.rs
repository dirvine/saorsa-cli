use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for the Saorsa CLI
///
/// This struct contains all configurable options for the CLI application,
/// organized into logical sections for GitHub integration, caching, and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// GitHub repository configuration for binary downloads
    pub github: GitHubConfig,
    /// Cache directory and binary storage configuration
    pub cache: CacheConfig,
    /// User behavior preferences and settings
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub check_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub directory: Option<PathBuf>,
    pub auto_clean: bool,
    pub max_versions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub auto_update_check: bool,
    pub use_system_binaries: bool,
    pub prefer_local_build: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig {
                owner: "dirvine".to_string(),
                repo: "saorsa-cli".to_string(),
                check_prerelease: false,
            },
            cache: CacheConfig {
                directory: None,
                auto_clean: false,
                max_versions: 3,
            },
            behavior: BehaviorConfig {
                auto_update_check: true,
                use_system_binaries: false,
                prefer_local_build: false,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;

            toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config from {:?}", config_path))
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to find config directory")?;

        Ok(config_dir.join("saorsa-cli").join("config.toml"))
    }

    pub fn cache_dir(&self) -> Result<PathBuf> {
        if let Some(ref dir) = self.cache.directory {
            Ok(dir.clone())
        } else {
            let cache_dir = dirs::cache_dir()
                .context("Failed to find cache directory")?
                .join("saorsa-cli")
                .join("binaries");
            Ok(cache_dir)
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        let cache_dir = self.cache_dir()?;
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;

        let config_dir = Self::config_path()?
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        if !config_dir.as_os_str().is_empty() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
        }

        Ok(())
    }

    pub fn update_from_cli(&mut self, no_update_check: bool, use_system: bool) {
        // Update config based on command-line arguments
        if no_update_check {
            self.behavior.auto_update_check = false;
        }
        if use_system {
            self.behavior.use_system_binaries = true;
        }
    }
}
