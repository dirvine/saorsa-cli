#[cfg(test)]
mod tests {
    use crate::file_entry::FileEntry;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create test file structure
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("subdir/nested.md"), "Nested").unwrap();

        temp_dir
    }

    #[test]
    fn test_file_entry_creation() {
        let path = PathBuf::from("/test/file.md");
        let entry = FileEntry {
            name: "file.md".to_string(),
            path: path.clone(),
            is_dir: false,
            children: vec![],
            expanded: false,
        };

        assert_eq!(entry.name, "file.md");
        assert_eq!(entry.path, path);
        assert!(!entry.is_dir);
        assert!(entry.children.is_empty());
        assert!(!entry.expanded);
    }

    #[test]
    fn test_directory_entry_creation() {
        let path = PathBuf::from("/test/dir");
        let entry = FileEntry {
            name: "dir".to_string(),
            path: path.clone(),
            is_dir: true,
            children: vec![],
            expanded: false,
        };

        assert_eq!(entry.name, "dir");
        assert_eq!(entry.path, path);
        assert!(entry.is_dir);
        assert!(entry.children.is_empty());
        assert!(!entry.expanded);
    }

    #[test]
    fn test_file_entry_with_children() {
        let parent_path = PathBuf::from("/test");
        let child_path = PathBuf::from("/test/child.md");

        let child = FileEntry {
            name: "child.md".to_string(),
            path: child_path,
            is_dir: false,
            children: vec![],
            expanded: false,
        };

        let parent = FileEntry {
            name: "test".to_string(),
            path: parent_path.clone(),
            is_dir: true,
            children: vec![child],
            expanded: true,
        };

        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name, "child.md");
        assert!(parent.expanded);
    }

    #[test]
    fn test_file_sorting() {
        let mut entries = vec![
            FileEntry {
                name: "z_file.md".to_string(),
                path: PathBuf::from("/z_file.md"),
                is_dir: false,
                children: vec![],
                expanded: false,
            },
            FileEntry {
                name: "a_dir".to_string(),
                path: PathBuf::from("/a_dir"),
                is_dir: true,
                children: vec![],
                expanded: false,
            },
            FileEntry {
                name: "b_file.md".to_string(),
                path: PathBuf::from("/b_file.md"),
                is_dir: false,
                children: vec![],
                expanded: false,
            },
        ];

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        assert_eq!(entries[0].name, "a_dir");
        assert_eq!(entries[1].name, "b_file.md");
        assert_eq!(entries[2].name, "z_file.md");
    }

    #[test]
    fn test_file_expansion_toggle() {
        let mut entry = FileEntry {
            name: "dir".to_string(),
            path: PathBuf::from("/dir"),
            is_dir: true,
            children: vec![],
            expanded: false,
        };

        assert!(!entry.expanded);

        entry.expanded = true;
        assert!(entry.expanded);

        entry.expanded = !entry.expanded;
        assert!(!entry.expanded);
    }

    #[test]
    fn test_nested_file_structure() {
        let root = FileEntry {
            name: "root".to_string(),
            path: PathBuf::from("/"),
            is_dir: true,
            children: vec![
                FileEntry {
                    name: "dir1".to_string(),
                    path: PathBuf::from("/dir1"),
                    is_dir: true,
                    children: vec![FileEntry {
                        name: "file1.md".to_string(),
                        path: PathBuf::from("/dir1/file1.md"),
                        is_dir: false,
                        children: vec![],
                        expanded: false,
                    }],
                    expanded: true,
                },
                FileEntry {
                    name: "file2.md".to_string(),
                    path: PathBuf::from("/file2.md"),
                    is_dir: false,
                    children: vec![],
                    expanded: false,
                },
            ],
            expanded: true,
        };

        assert_eq!(root.children.len(), 2);
        assert!(root.children[0].is_dir);
        assert!(!root.children[1].is_dir);
        assert_eq!(root.children[0].children.len(), 1);
    }

    #[test]
    fn test_file_path_handling() {
        let temp_dir = setup_test_directory();
        let base_path = temp_dir.path();

        assert!(base_path.join("test.md").exists());
        assert!(base_path.join("test.rs").exists());
        assert!(base_path.join("subdir").exists());
        assert!(base_path.join("subdir/nested.md").exists());
    }

    #[test]
    fn test_file_extension_detection() {
        let md_file = PathBuf::from("test.md");
        let rs_file = PathBuf::from("test.rs");
        let no_ext = PathBuf::from("test");

        assert_eq!(md_file.extension().and_then(|s| s.to_str()), Some("md"));
        assert_eq!(rs_file.extension().and_then(|s| s.to_str()), Some("rs"));
        assert_eq!(no_ext.extension().and_then(|s| s.to_str()), None);
    }
}
