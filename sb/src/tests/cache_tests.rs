#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    struct CacheEntry<T> {
        value: T,
        timestamp: Instant,
        ttl: Duration,
    }

    struct Cache<K, V> {
        entries: HashMap<K, CacheEntry<V>>,
        max_size: usize,
    }

    impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
        fn new(max_size: usize) -> Self {
            Self {
                entries: HashMap::new(),
                max_size,
            }
        }

        fn insert(&mut self, key: K, value: V, ttl: Duration) {
            if self.entries.len() >= self.max_size {
                // In real implementation, would evict LRU
                self.entries.clear();
            }

            self.entries.insert(
                key,
                CacheEntry {
                    value,
                    timestamp: Instant::now(),
                    ttl,
                },
            );
        }

        fn get(&self, key: &K) -> Option<&V> {
            self.entries.get(key).and_then(|entry| {
                if entry.timestamp.elapsed() < entry.ttl {
                    Some(&entry.value)
                } else {
                    None
                }
            })
        }

        fn is_expired(&self, key: &K) -> bool {
            self.entries
                .get(key)
                .map(|entry| entry.timestamp.elapsed() >= entry.ttl)
                .unwrap_or(true)
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache: Cache<String, String> = Cache::new(100);
        assert_eq!(cache.entries.len(), 0);
        assert_eq!(cache.max_size, 100);
    }

    #[test]
    fn test_cache_insertion() {
        let mut cache = Cache::new(10);
        cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_secs(60),
        );

        assert_eq!(cache.entries.len(), 1);
        assert!(cache.entries.contains_key("key1"));
    }

    #[test]
    fn test_cache_retrieval() {
        let mut cache = Cache::new(10);
        cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_secs(60),
        );

        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some(&"value1".to_string()));

        let missing = cache.get(&"nonexistent".to_string());
        assert_eq!(missing, None);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = Cache::new(10);

        // Insert with very short TTL
        cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_millis(1),
        );

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));

        assert!(cache.is_expired(&"key1".to_string()));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_size_limit() {
        let mut cache = Cache::new(2);

        cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_secs(60),
        );
        cache.insert(
            "key2".to_string(),
            "value2".to_string(),
            Duration::from_secs(60),
        );

        assert_eq!(cache.entries.len(), 2);

        // Adding third item should trigger eviction
        cache.insert(
            "key3".to_string(),
            "value3".to_string(),
            Duration::from_secs(60),
        );

        // Simple implementation clears all - real LRU would keep most recent
        assert!(cache.entries.len() <= 2);
    }

    #[test]
    fn test_directory_cache_ttl() {
        const DIRECTORY_TTL: Duration = Duration::from_secs(30);

        let mut cache = Cache::new(100);
        cache.insert(
            "/path/to/dir".to_string(),
            vec!["file1", "file2"],
            DIRECTORY_TTL,
        );

        // Should be valid immediately
        assert!(!cache.is_expired(&"/path/to/dir".to_string()));

        // In real test, would mock time progression
        // For now, just verify TTL is set correctly
        if let Some(entry) = cache.entries.get(&"/path/to/dir".to_string()) {
            assert_eq!(entry.ttl, DIRECTORY_TTL);
        }
    }

    #[test]
    fn test_file_preview_cache_ttl() {
        const PREVIEW_TTL: Duration = Duration::from_secs(60);

        let mut cache = Cache::new(100);
        cache.insert(
            "/file.md".to_string(),
            "preview content".to_string(),
            PREVIEW_TTL,
        );

        // Should be valid immediately
        assert!(!cache.is_expired(&"/file.md".to_string()));

        if let Some(entry) = cache.entries.get(&"/file.md".to_string()) {
            assert_eq!(entry.ttl, PREVIEW_TTL);
        }
    }

    #[test]
    fn test_cache_hit_rate_tracking() {
        struct CacheWithMetrics<K, V> {
            cache: Cache<K, V>,
            hits: usize,
            misses: usize,
        }

        impl<K: std::hash::Hash + Eq, V> CacheWithMetrics<K, V> {
            fn new(max_size: usize) -> Self {
                Self {
                    cache: Cache::new(max_size),
                    hits: 0,
                    misses: 0,
                }
            }

            fn get(&mut self, key: &K) -> Option<&V> {
                if self.cache.get(key).is_some() {
                    self.hits += 1;
                    self.cache.get(key)
                } else {
                    self.misses += 1;
                    None
                }
            }

            fn hit_rate(&self) -> f64 {
                let total = self.hits + self.misses;
                if total == 0 {
                    0.0
                } else {
                    self.hits as f64 / total as f64
                }
            }
        }

        let mut metrics_cache = CacheWithMetrics::new(10);
        metrics_cache.cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_secs(60),
        );

        // Hit
        let _ = metrics_cache.get(&"key1".to_string());
        assert_eq!(metrics_cache.hits, 1);
        assert_eq!(metrics_cache.misses, 0);

        // Miss
        let _ = metrics_cache.get(&"key2".to_string());
        assert_eq!(metrics_cache.hits, 1);
        assert_eq!(metrics_cache.misses, 1);

        assert_eq!(metrics_cache.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_memory_bounds() {
        // Test that cache respects memory bounds
        const MAX_CACHE_MEMORY: usize = 1024 * 1024; // 1MB

        struct MemoryBoundedCache {
            cache: Cache<String, Vec<u8>>,
            current_size: usize,
            max_memory: usize,
        }

        impl MemoryBoundedCache {
            fn new(max_items: usize, max_memory: usize) -> Self {
                Self {
                    cache: Cache::new(max_items),
                    current_size: 0,
                    max_memory,
                }
            }

            fn can_insert(&self, size: usize) -> bool {
                self.current_size + size <= self.max_memory
            }
        }

        let cache = MemoryBoundedCache::new(100, MAX_CACHE_MEMORY);

        // Test boundary conditions
        assert!(cache.can_insert(MAX_CACHE_MEMORY - 1));
        assert!(cache.can_insert(MAX_CACHE_MEMORY));
        assert!(!cache.can_insert(MAX_CACHE_MEMORY + 1));
    }
}
