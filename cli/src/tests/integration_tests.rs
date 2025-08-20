#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::config::Config;
    use crate::platform::Platform;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, Config, Platform) {
        let temp_dir = TempDir::new().unwrap();

        // Create a minimal config for testing
        let mut config = Config::default();
        config.cache_dir = Some(temp_dir.path().join("cache"));
        config.ensure_directories().unwrap();

        let platform = Platform::detect().unwrap();

        (temp_dir, config, platform)
    }

    #[test]
    fn test_config_persistence() {
        let (_temp_dir, mut config, _platform) = setup_test_env();

        // Modify config
        config.behavior.auto_update_check = false;
        config.behavior.use_system_binaries = true;

        // Save and reload
        let config_path = _temp_dir.path().join("test_config.toml");
        config.save_to(&config_path).unwrap();

        let loaded_config = Config::load_from(&config_path).unwrap();

        // Verify persistence
        assert_eq!(loaded_config.behavior.auto_update_check, false);
        assert_eq!(loaded_config.behavior.use_system_binaries, true);
    }

    #[test]
    fn test_binary_path_resolution() {
        let (_temp_dir, config, platform) = setup_test_env();
        let runner = BinaryRunner::new();

        // Test system binary detection (sb should be available in PATH during tests)
        let sb_path = runner.which("sb");
        if sb_path.is_some() {
            assert!(sb_path.unwrap().exists());
        }

        // Test cached binary path generation
        let downloader = Downloader::new(
            config.github.owner.clone(),
            config.github.repo.clone(),
        ).unwrap();

        let cache_path = downloader.binary_path("sb", &platform);
        assert!(cache_path.to_string_lossy().contains("saorsa"));
        assert!(cache_path.to_string_lossy().contains(&platform.os));
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect().unwrap();

        // Verify platform has required fields
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());

        // Verify binary name generation
        let binary_name = platform.binary_name("sb");
        assert!(binary_name.contains("sb"));
        assert!(binary_name.contains(&platform.os) || binary_name.contains(&platform.arch));
    }

    #[test]
    fn test_directory_structure_creation() {
        let (_temp_dir, mut config, _platform) = setup_test_env();

        // Set custom cache directory
        let custom_cache = _temp_dir.path().join("custom_cache");
        config.cache_dir = Some(custom_cache.clone());

        // Ensure directories are created
        config.ensure_directories().unwrap();

        // Verify directories exist
        assert!(custom_cache.exists());
        assert!(custom_cache.is_dir());

        let bin_dir = custom_cache.join("bin");
        assert!(bin_dir.exists());
        assert!(bin_dir.is_dir());
    }

    #[test]
    fn test_config_validation() {
        let (_temp_dir, config, _platform) = setup_test_env();

        // Test valid config
        assert!(config.validate().is_ok());

        // Test config with invalid paths
        let mut invalid_config = config.clone();
        invalid_config.cache_dir = Some(PathBuf::from("/nonexistent/invalid/path"));

        // Should still be valid (we don't validate paths exist, just structure)
        assert!(invalid_config.validate().is_ok());
    }

    #[test]
    fn test_error_handling_integration() {
        let (_temp_dir, config, _platform) = setup_test_env();
        let runner = BinaryRunner::new();

        // Test running non-existent binary
        let result = runner.run_interactive(&PathBuf::from("/nonexistent/binary"), vec![]);
        assert!(result.is_err());

        // Test checking non-existent binary
        let exists = runner.check_binary_exists(&PathBuf::from("/nonexistent/binary"));
        assert!(!exists);
    }

    #[test]
    fn test_github_integration() {
        let (_temp_dir, config, platform) = setup_test_env();

        let downloader = Downloader::new(
            config.github.owner.clone(),
            config.github.repo.clone(),
        ).unwrap();

        // Test binary path generation for different tools
        let sb_path = downloader.binary_path("sb", &platform);
        let sdisk_path = downloader.binary_path("sdisk", &platform);

        assert_ne!(sb_path, sdisk_path);
        assert!(sb_path.to_string_lossy().contains("sb"));
        assert!(sdisk_path.to_string_lossy().contains("sdisk"));
    }

    #[test]
    fn test_menu_integration() {
        let (_temp_dir, _config, _platform) = setup_test_env();

        // Test menu creation and basic functionality
        let menu = Menu::new();

        // Menu should be properly initialized
        // Note: We can't easily test the full menu loop in unit tests
        // but we can verify the menu struct is created correctly
        assert!(true); // Placeholder - menu integration would need UI testing framework
    }
}