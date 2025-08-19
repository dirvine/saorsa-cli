//! Git integration for saorsa-browser
//!
//! This module provides Git repository detection, status tracking, and diff functionality.

use git2::{Repository, Status, StatusOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GitError {
    #[error("Git repository not found")]
    NotARepository,
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FileStatus {
    Unmodified,
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

impl From<Status> for FileStatus {
    fn from(status: Status) -> Self {
        if status == Status::CURRENT {
            FileStatus::Unmodified
        } else if status.intersects(Status::WT_NEW | Status::INDEX_NEW) {
            FileStatus::Added
        } else if status.intersects(Status::WT_MODIFIED | Status::INDEX_MODIFIED) {
            FileStatus::Modified
        } else if status.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
            FileStatus::Deleted
        } else if status.intersects(Status::WT_RENAMED | Status::INDEX_RENAMED) {
            FileStatus::Renamed
        } else if status.intersects(Status::IGNORED) {
            FileStatus::Ignored
        } else if status.intersects(Status::CONFLICTED) {
            FileStatus::Conflicted
        } else {
            FileStatus::Untracked
        }
    }
}

pub struct GitRepository {
    repo: Repository,
    root: PathBuf,
}

impl std::fmt::Debug for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepository")
            .field("root", &self.root)
            .finish()
    }
}

impl GitRepository {
    /// Try to open a Git repository from the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let repo = Repository::discover(path.as_ref())?;
        let root = repo
            .workdir()
            .ok_or_else(|| GitError::Git(git2::Error::from_str("No working directory")))?
            .to_path_buf();

        Ok(GitRepository { repo, root })
    }

    /// Check if a path is within a Git repository
    #[allow(dead_code)]
    pub fn is_git_repo<P: AsRef<Path>>(path: P) -> bool {
        Repository::discover(path.as_ref()).is_ok()
    }

    /// Get the root directory of the Git repository
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the Git status of all files in the repository
    pub fn status(&self) -> Result<HashMap<PathBuf, FileStatus>, GitError> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);
        status_opts.recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut status_opts))?;
        let mut result = HashMap::new();

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let path = self.root.join(path);
                let status = FileStatus::from(entry.status());

                // Insert both the path and its canonical version for better matching
                result.insert(path.clone(), status);
                if let Ok(canonical) = path.canonicalize() {
                    result.insert(canonical, status);
                }
            }
        }

        Ok(result)
    }

    /// Get the status of a specific file
    #[allow(dead_code)]
    pub fn file_status<P: AsRef<Path>>(&self, path: P) -> Result<FileStatus, GitError> {
        let relative_path = path
            .as_ref()
            .strip_prefix(&self.root)
            .map_err(|_| GitError::Git(git2::Error::from_str("Path not in repository")))?;

        let status = self.repo.status_file(relative_path)?;
        Ok(FileStatus::from(status))
    }

    /// Get the diff for a specific file
    pub fn file_diff<P: AsRef<Path>>(&self, path: P) -> Result<String, GitError> {
        let relative_path = path
            .as_ref()
            .strip_prefix(&self.root)
            .map_err(|_| GitError::Git(git2::Error::from_str("Path not in repository")))?;

        // Get the diff between the working directory and the index
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.pathspec(relative_path);

        let diff = self
            .repo
            .diff_index_to_workdir(None, Some(&mut diff_opts))?;

        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            match line.origin() {
                '+' => diff_output.push_str(&format!("+{}", content)),
                '-' => diff_output.push_str(&format!("-{}", content)),
                ' ' => diff_output.push_str(&format!(" {}", content)),
                '@' => diff_output.push_str(&format!("@{}", content)),
                _ => diff_output.push_str(content),
            }
            true
        })?;

        // If no working directory changes, try staged changes
        if diff_output.is_empty() {
            let tree = self.repo.head()?.peel_to_tree()?;
            let diff = self
                .repo
                .diff_tree_to_index(Some(&tree), None, Some(&mut diff_opts))?;

            diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                let content = std::str::from_utf8(line.content()).unwrap_or("");
                match line.origin() {
                    '+' => diff_output.push_str(&format!("+{}", content)),
                    '-' => diff_output.push_str(&format!("-{}", content)),
                    ' ' => diff_output.push_str(&format!(" {}", content)),
                    '@' => diff_output.push_str(&format!("@{}", content)),
                    _ => diff_output.push_str(content),
                }
                true
            })?;
        }

        Ok(diff_output)
    }

    /// Get a formatted Git status summary
    pub fn status_summary(&self) -> Result<String, GitError> {
        let statuses = self.status()?;

        if statuses.is_empty() {
            return Ok("Working tree clean".to_string());
        }

        let mut summary = String::new();
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut untracked = 0;
        let mut conflicted = 0;

        for status in statuses.values() {
            match status {
                FileStatus::Added => added += 1,
                FileStatus::Modified => modified += 1,
                FileStatus::Deleted => deleted += 1,
                FileStatus::Untracked => untracked += 1,
                FileStatus::Conflicted => conflicted += 1,
                _ => {}
            }
        }

        if added > 0 {
            summary.push_str(&format!("Added: {} ", added));
        }
        if modified > 0 {
            summary.push_str(&format!("Modified: {} ", modified));
        }
        if deleted > 0 {
            summary.push_str(&format!("Deleted: {} ", deleted));
        }
        if untracked > 0 {
            summary.push_str(&format!("Untracked: {} ", untracked));
        }
        if conflicted > 0 {
            summary.push_str(&format!("Conflicted: {} ", conflicted));
        }

        if summary.is_empty() {
            summary = "Working tree clean".to_string();
        }

        Ok(summary.trim().to_string())
    }

    /// Remove a file using Git
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<(), GitError> {
        let relative_path = path
            .as_ref()
            .strip_prefix(&self.root)
            .map_err(|_| GitError::Git(git2::Error::from_str("Path not in repository")))?;

        let mut index = self.repo.index()?;
        index.remove_path(relative_path)?;
        index.write()?;

        // Also remove from working directory
        if path.as_ref().exists() {
            std::fs::remove_file(path.as_ref())?;
        }

        Ok(())
    }

    /// Move a file using Git (git mv)
    pub fn move_file<P: AsRef<Path>>(&self, from: P, to: P) -> Result<(), GitError> {
        let from_relative = from
            .as_ref()
            .strip_prefix(&self.root)
            .map_err(|_| GitError::Git(git2::Error::from_str("Source path not in repository")))?;

        let to_relative = to.as_ref().strip_prefix(&self.root).map_err(|_| {
            GitError::Git(git2::Error::from_str("Destination path not in repository"))
        })?;

        // Move the file in the filesystem first
        std::fs::rename(from.as_ref(), to.as_ref())?;

        // Update the Git index
        let mut index = self.repo.index()?;
        index.remove_path(from_relative)?;
        index.add_path(to_relative)?;
        index.write()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_repo_detection() {
        // Test with non-git directory
        let temp_dir = TempDir::new().unwrap();
        assert!(!GitRepository::is_git_repo(temp_dir.path()));
    }

    #[test]
    fn test_file_status_conversion() {
        let status = Status::WT_NEW;
        let file_status = FileStatus::from(status);
        assert_eq!(file_status, FileStatus::Added);

        let status = Status::WT_MODIFIED;
        let file_status = FileStatus::from(status);
        assert_eq!(file_status, FileStatus::Modified);
    }
}
