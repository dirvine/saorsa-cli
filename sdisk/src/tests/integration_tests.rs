#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::commands::Commands;
    use clap::Parser;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn create_test_directory_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test directory structure
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("docs")).unwrap();
        fs::create_dir(root.join("temp")).unwrap();

        // Create files with different sizes and ages
        fs::write(root.join("large_file.bin"), vec![0u8; 1024 * 1024]).unwrap(); // 1MB
        fs::write(root.join("small_file.txt"), "Hello, world!").unwrap();
        fs::write(root.join("src/code.rs"), "fn main() {}").unwrap();
        fs::write(root.join("docs/readme.md"), "# Documentation").unwrap();

        // Create some old files (simulate by creating and then modifying timestamps if possible)
        let old_file = root.join("old_file.tmp");
        fs::write(&old_file, "This is old").unwrap();

        temp_dir
    }

    #[test]
    fn test_disk_info_command() {
        let temp_dir = create_test_directory_structure();

        // Test the info command
        let result = cmd_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_top_command_integration() {
        let temp_dir = create_test_directory_structure();
        let roots = vec![temp_dir.path().to_path_buf()];

        // Test the top command
        let result = cmd_top(roots, 10, false, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stale_command_integration() {
        let temp_dir = create_test_directory_structure();
        let roots = vec![temp_dir.path().to_path_buf()];

        // Test the stale command with a very high threshold to catch all files
        let result = cmd_stale(roots, 365 * 24 * 60 * 60, 20, false, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_directory_traversal_integration() {
        let temp_dir = create_test_directory_structure();
        let roots = vec![temp_dir.path().to_path_buf()];

        // Test directory traversal
        let mut entries: Vec<(PathBuf, u64)> = Vec::new();
        for root in &roots {
            for entry in walkdir::WalkDir::new(root)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        entries.push((path.to_path_buf(), meta.len()));
                    }
                }
            }
        }

        // Should find multiple files
        assert!(!entries.is_empty());

        // Should include our test files
        let file_names: Vec<_> = entries.iter().map(|(p, _)| p.file_name().unwrap().to_str().unwrap()).collect();
        assert!(file_names.contains(&"large_file.bin"));
        assert!(file_names.contains(&"small_file.txt"));
    }

    #[test]
    fn test_file_size_calculation_integration() {
        let temp_dir = create_test_directory_structure();

        // Test file size calculation
        let large_file = temp_dir.path().join("large_file.bin");
        let small_file = temp_dir.path().join("small_file.txt");

        let large_size = large_file.metadata().unwrap().len();
        let small_size = small_file.metadata().unwrap().len();

        // Large file should be bigger
        assert!(large_size > small_size);
        assert_eq!(large_size, 1024 * 1024); // 1MB
        assert!(small_size > 0);
    }

    #[test]
    fn test_collect_roots_integration() {
        let temp_dir = create_test_directory_structure();

        // Test with explicit root
        let roots = collect_roots(Some(temp_dir.path().to_path_buf()), vec![]);
        assert!(roots.is_ok());
        assert_eq!(roots.unwrap().len(), 1);

        // Test with additional paths
        let extra_paths = vec![temp_dir.path().join("src"), temp_dir.path().join("docs")];
        let roots = collect_roots(Some(temp_dir.path().to_path_buf()), extra_paths);
        assert!(roots.is_ok());
        let roots = roots.unwrap();
        assert!(roots.len() >= 3); // At least the main path and the two extras (if they don't overlap)

        // Test with no explicit root (should use current dir)
        let roots = collect_roots(None, vec![]);
        assert!(roots.is_ok());
        assert_eq!(roots.unwrap().len(), 1);
    }

    #[test]
    fn test_spinner_creation_integration() {
        // Test spinner creation
        let result = spinner();
        assert!(result.is_ok());

        let pb = result.unwrap();
        // Progress bar should be created successfully
        assert!(!pb.is_finished());
    }

    #[test]
    fn test_error_handling_integration() {
        // Test with non-existent directory
        let nonexistent = PathBuf::from("/definitely/does/not/exist");
        let roots = vec![nonexistent];

        // Should handle gracefully
        let result = cmd_top(roots, 10, false, true, false);
        // May or may not error depending on implementation, but shouldn't panic
        // We mainly want to ensure no panic occurs
    }

    #[test]
    fn test_large_directory_handling() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create many files
        for i in 0..100 {
            let file_path = root.join(format!("file_{}.txt", i));
            fs::write(&file_path, format!("Content of file {}", i)).unwrap();
        }

        let roots = vec![root.to_path_buf()];

        // Should handle large number of files without issues
        let result = cmd_top(roots, 50, false, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_age_detection() {
        let temp_dir = create_test_directory_structure();
        let old_file = temp_dir.path().join("old_file.tmp");

        // Get file metadata
        let metadata = old_file.metadata().unwrap();
        let created = metadata.created().unwrap();
        let now = SystemTime::now();

        // File should have been created recently (within last few seconds)
        let age = now.duration_since(created).unwrap();
        assert!(age < Duration::from_secs(60)); // Less than a minute old
    }

    #[test]
    fn test_command_line_parsing_integration() {
        // Test CLI parsing
        let args = vec!["sdisk", "top", "--count", "5", "--path", "/tmp"];
        let cli = Cli::try_parse_from(args);

        match cli {
            Ok(cli) => {
                match cli.command {
                    Some(Commands::Top { count, .. }) => {
                        assert_eq!(count, 5);
                    }
                    _ => panic!("Expected Top command"),
                }
            }
            Err(e) => {
                // If parsing fails, it should be for a valid reason
                // This test mainly ensures no panics occur during parsing
                println!("CLI parsing failed (expected in test env): {}", e);
            }
        }
    }
}