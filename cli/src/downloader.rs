use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use futures::StreamExt;
use thiserror::Error;
use tokio::io::AsyncWriteExt;

use crate::platform::Platform;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("No matching asset found for platform")]
    NoMatchingAsset,
    #[error("No releases found")]
    NoReleases,
    #[error("Checksum verification failed")]
    ChecksumMismatch,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub assets: Vec<GitHubAsset>,
    pub published_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct Downloader {
    client: Client,
    repo_owner: String,
    repo_name: String,
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new(repo_owner: String, repo_name: String) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to find cache directory")?
            .join("saorsa-cli")
            .join("binaries");

        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        let client = Client::builder()
            .user_agent("saorsa-cli/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            repo_owner,
            repo_name,
            cache_dir,
        })
    }

    pub async fn get_latest_release(&self) -> Result<GitHubRelease, DownloadError> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            // Try to get all releases if latest doesn't exist
            let url = format!(
                "https://api.github.com/repos/{}/{}/releases",
                self.repo_owner, self.repo_name
            );
            
            let releases: Vec<GitHubRelease> = self.client
                .get(&url)
                .send()
                .await?
                .json()
                .await?;

            releases.into_iter()
                .next()
                .ok_or(DownloadError::NoReleases)
        } else {
            Ok(response.json().await?)
        }
    }

    pub fn binary_path(&self, binary_name: &str, platform: &Platform) -> PathBuf {
        self.cache_dir.join(format!(
            "{}{}",
            binary_name,
            platform.binary_extension()
        ))
    }

    pub async fn download_binary(
        &self,
        binary_name: &str,
        platform: &Platform,
        force: bool,
    ) -> Result<PathBuf> {
        let binary_path = self.binary_path(binary_name, platform);

        if binary_path.exists() && !force {
            tracing::info!("Binary already exists at {:?}", binary_path);
            return Ok(binary_path);
        }

        let release = self.get_latest_release().await
            .context("Failed to get latest release")?;

        let asset_name = platform.asset_name(binary_name);
        let asset = release.assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or(DownloadError::NoMatchingAsset)?;

        tracing::info!("Downloading {} from {}", asset.name, asset.browser_download_url);

        let archive_path = self.download_asset(asset).await
            .context("Failed to download asset")?;

        self.extract_binary(&archive_path, binary_name, platform)
            .await
            .context("Failed to extract binary")?;

        // Clean up archive
        fs::remove_file(&archive_path).ok();

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    async fn download_asset(&self, asset: &GitHubAsset) -> Result<PathBuf> {
        let archive_path = self.cache_dir.join(&asset.name);

        let response = self.client
            .get(&asset.browser_download_url)
            .send()
            .await
            .context("Failed to start download")?;

        let total_size = response.content_length().unwrap_or(asset.size);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );

        let mut file = tokio::fs::File::create(&archive_path).await
            .context("Failed to create archive file")?;
        
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to download chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        
        Ok(archive_path)
    }

    async fn extract_binary(
        &self,
        archive_path: &Path,
        binary_name: &str,
        platform: &Platform,
    ) -> Result<()> {
        let binary_path = self.binary_path(binary_name, platform);

        match platform.archive_extension() {
            ".tar.gz" => {
                use flate2::read::GzDecoder;
                use tar::Archive;

                let file = File::open(archive_path)
                    .context("Failed to open archive")?;
                let gz = GzDecoder::new(file);
                let mut archive = Archive::new(gz);

                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path = entry.path()?;
                    
                    if let Some(name) = path.file_name() {
                        if name == binary_name {
                            let mut output = File::create(&binary_path)
                                .context("Failed to create binary file")?;
                            io::copy(&mut entry, &mut output)
                                .context("Failed to extract binary")?;
                            return Ok(());
                        }
                    }
                }

                anyhow::bail!("Binary {} not found in archive", binary_name);
            }
            ".zip" => {
                use zip::ZipArchive;

                let file = File::open(archive_path)
                    .context("Failed to open archive")?;
                let mut archive = ZipArchive::new(file)?;

                let binary_name_with_ext = format!("{}{}", binary_name, platform.binary_extension());
                
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    if let Some(name) = Path::new(file.name()).file_name() {
                        if name == binary_name_with_ext.as_str() || name == binary_name {
                            let mut output = File::create(&binary_path)
                                .context("Failed to create binary file")?;
                            io::copy(&mut file, &mut output)
                                .context("Failed to extract binary")?;
                            return Ok(());
                        }
                    }
                }

                anyhow::bail!("Binary {} not found in archive", binary_name);
            }
            _ => anyhow::bail!("Unsupported archive format"),
        }
    }

    pub async fn verify_checksum(&self, file_path: &Path, expected: &str) -> Result<bool> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher)?;
        let result = hasher.finalize();
        let calculated = hex::encode(result);
        Ok(calculated == expected)
    }
}