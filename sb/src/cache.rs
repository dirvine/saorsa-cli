use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tui_tree_widget::TreeItem;

/// Directory cache entry with metadata
#[derive(Clone, Debug)]
pub struct CacheEntry {
    /// The cached tree items
    pub items: Vec<TreeItem<'static, String>>,
    /// When this entry was cached
    pub cached_at: SystemTime,
    /// Last modified time of the directory
    pub modified_time: SystemTime,
}

/// Directory cache with TTL and invalidation support
#[derive(Debug)]
pub struct DirectoryCache {
    /// Cache storage
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
    /// Time-to-live for cache entries
    ttl: Duration,
    /// Maximum number of cached directories
    max_entries: usize,
}

impl DirectoryCache {
    /// Creates a new directory cache with specified TTL and maximum entries.
    ///
    /// # Arguments
    ///
    /// * `ttl_seconds` - Time-to-live for cache entries in seconds
    /// * `max_entries` - Maximum number of directories to cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sb::DirectoryCache;
    ///
    /// // Cache directories for 5 minutes, max 50 entries
    /// let cache = DirectoryCache::new(300, 50);
    /// ```
    pub fn new(ttl_seconds: u64, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            max_entries,
        }
    }

    /// Get a cached directory tree if it exists and is still valid
    pub fn get(&self, path: &Path) -> Option<Vec<TreeItem<'static, String>>> {
        let cache = self.cache.lock().ok()?;
        let entry = cache.get(path)?;

        // Check if cache entry has expired
        let now = SystemTime::now();
        if now.duration_since(entry.cached_at).ok()? > self.ttl {
            return None;
        }

        // Check if directory has been modified since caching
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        if modified > entry.modified_time {
            return None;
        }

        Some(entry.items.clone())
    }

    /// Insert a directory tree into the cache
    pub fn insert(&self, path: PathBuf, items: Vec<TreeItem<'static, String>>) {
        let mut cache = match self.cache.lock() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Evict oldest entries if cache is full
        if cache.len() >= self.max_entries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, entry)| entry.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        // Get directory modification time
        let modified_time = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        cache.insert(
            path,
            CacheEntry {
                items,
                cached_at: SystemTime::now(),
                modified_time,
            },
        );
    }

    /// Invalidate a specific path in the cache
    pub fn invalidate(&self, path: &Path) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(path);

            // Also invalidate parent directory if it exists
            if let Some(parent) = path.parent() {
                cache.remove(parent);
            }
        }
    }

    /// Get the number of cached entries (used for testing)
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for DirectoryCache {
    fn default() -> Self {
        // Default: 5 minute TTL, max 100 directories
        Self::new(300, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Line;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_basic_operations() {
        use tempfile::tempdir;

        let cache = DirectoryCache::new(60, 10);
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let items = vec![
            TreeItem::new_leaf("test1".to_string(), Line::from("test1")),
            TreeItem::new_leaf("test2".to_string(), Line::from("test2")),
        ];

        // Insert and retrieve
        cache.insert(path.clone(), items.clone());
        assert!(cache.get(&path).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_expiration() {
        use tempfile::tempdir;

        let cache = DirectoryCache::new(1, 10); // 1 second TTL
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        let items = vec![TreeItem::new_leaf("test".to_string(), Line::from("test"))];

        cache.insert(path.clone(), items);
        assert!(cache.get(&path).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = DirectoryCache::new(60, 10);
        let path = PathBuf::from("/test/path");
        let parent = PathBuf::from("/test");

        let items = vec![TreeItem::new_leaf("test".to_string(), Line::from("test"))];

        cache.insert(path.clone(), items.clone());
        cache.insert(parent.clone(), items);

        assert_eq!(cache.len(), 2);

        // Invalidate child path should also invalidate parent
        cache.invalidate(&path);
        assert!(cache.get(&path).is_none());
        assert!(cache.get(&parent).is_none());
    }

    #[test]
    fn test_cache_max_entries() {
        let cache = DirectoryCache::new(60, 3); // Max 3 entries

        for i in 0..5 {
            let path = PathBuf::from(format!("/test/path{}", i));
            let items = vec![TreeItem::new_leaf(
                format!("test{}", i),
                Line::from(format!("test{}", i)),
            )];
            cache.insert(path, items);
        }

        // Should only have 3 entries (oldest evicted)
        assert_eq!(cache.len(), 3);
    }
}
