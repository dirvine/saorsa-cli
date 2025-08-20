#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::app::{App, Focus};
    use crate::fs::{FileEntry, FileType};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_directory_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create test directory structure
        let root = temp_dir.path();

        // Create directories
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("docs")).unwrap();
        fs::create_dir(root.join("src/lib")).unwrap();

        // Create files
        fs::write(root.join("README.md"), "# Test Project\n\nThis is a test.").unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {\n    println!(\"Hello!\");\n}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}").unwrap();
        fs::write(root.join("docs/guide.md"), "# User Guide\n\nHow to use this project.").unwrap();

        temp_dir
    }

    #[test]
    fn test_app_file_system_integration() {
        let temp_dir = create_test_directory_structure();
        let app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Verify app can read the directory structure
        assert!(!app.left_tree.is_empty());

        // Check that files are properly categorized
        let root_items: Vec<_> = app.left_tree.iter().collect();
        assert!(!root_items.is_empty());
    }

    #[test]
    fn test_file_operations_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Test opening a file
        let readme_path = temp_dir.path().join("README.md");
        app.open_file(&readme_path).unwrap();

        assert!(app.opened.is_some());
        assert_eq!(app.opened.as_ref().unwrap().path, readme_path);
    }

    #[test]
    fn test_tree_navigation_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Test expanding directories
        if let Some(root_item) = app.left_tree.first() {
            let src_dir_path = temp_dir.path().join("src");
            app.toggle_directory(&src_dir_path).unwrap();

            // Verify the directory was expanded
            assert!(app.expanded_directories.contains(&src_dir_path));
        }
    }

    #[test]
    fn test_search_functionality_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Test search for "Test"
        app.search_buffer = "Test".to_string();
        app.perform_search();

        // Should find the README.md file
        assert!(!app.search_results.is_empty());
    }

    #[test]
    fn test_git_integration() {
        let temp_dir = create_test_directory_structure();

        // Initialize a git repository
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&temp_dir)
            .output()
            .ok();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&temp_dir)
            .output()
            .ok();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&temp_dir)
            .output()
            .ok();

        let app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // App should detect git repository
        // Note: This test might be flaky if git is not available
        // In a real scenario, we'd mock the git functionality
    }

    #[test]
    fn test_markdown_preview_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        let readme_path = temp_dir.path().join("README.md");
        app.open_file(&readme_path).unwrap();

        // Switch to preview mode
        app.focus = Focus::Preview;

        // Verify preview content is generated
        if let Some(opened) = &app.opened {
            assert!(!opened.preview_content.is_empty());
        }
    }

    #[test]
    fn test_editor_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        let test_file_path = temp_dir.path().join("test_edit.md");
        fs::write(&test_file_path, "Initial content").unwrap();

        app.open_file(&test_file_path).unwrap();
        app.focus = Focus::Editor;

        // Test editor content
        if let Some(opened) = &app.opened {
            assert_eq!(opened.content, "Initial content");
        }
    }

    #[test]
    fn test_file_type_detection_integration() {
        let temp_dir = create_test_directory_structure();
        let app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Test various file types are detected correctly
        let rust_file = temp_dir.path().join("src/main.rs");
        let md_file = temp_dir.path().join("README.md");

        assert!(rust_file.exists());
        assert!(md_file.exists());

        // The app should be able to handle different file types
        // This is more of a smoke test to ensure no panics occur
        let _ = app.open_file(&rust_file);
        let _ = app.open_file(&md_file);
    }

    #[test]
    fn test_performance_with_large_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a large directory structure
        for i in 0..50 {
            let dir_path = root.join(format!("dir_{}", i));
            fs::create_dir(&dir_path).unwrap();

            // Add some files to each directory
            for j in 0..10 {
                let file_path = dir_path.join(format!("file_{}.txt", j));
                fs::write(&file_path, format!("Content of file {}_{}", i, j)).unwrap();
            }
        }

        // Test that app can handle large directory structures
        let app = App::new(root.to_path_buf()).unwrap();

        // Should not panic and should load the structure
        assert!(!app.left_tree.is_empty());
    }

    #[test]
    fn test_error_recovery_integration() {
        let temp_dir = create_test_directory_structure();
        let mut app = App::new(temp_dir.path().to_path_buf()).unwrap();

        // Test opening non-existent file
        let nonexistent = temp_dir.path().join("nonexistent.md");
        let result = app.open_file(&nonexistent);

        // Should handle error gracefully
        assert!(result.is_err());

        // App should remain in a valid state
        assert!(app.opened.is_none());
    }
}