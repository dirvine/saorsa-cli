#[cfg(test)]
mod tests {
    use crate::app::{App, Focus};
    use crate::file_entry::FileEntry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_app() -> (App, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let app = App::new(temp_dir.path().to_path_buf());
        (app, temp_dir)
    }

    #[test]
    fn test_app_initialization() {
        let (app, _temp_dir) = create_test_app();

        assert!(app.opened.is_none());
        assert!(!app.show_raw_editor);
        assert!(app.show_left_pane);
        assert_eq!(app.focus, Focus::Left);
        assert!(!app.show_help);
        assert!(!app.command_mode);
        assert_eq!(app.command_buffer, String::new());
    }

    #[test]
    fn test_focus_switching() {
        let (mut app, _temp_dir) = create_test_app();

        // Test focus transitions
        app.focus = Focus::Left;
        assert_eq!(app.focus, Focus::Left);

        app.focus = Focus::Preview;
        assert_eq!(app.focus, Focus::Preview);

        app.focus = Focus::Editor;
        assert_eq!(app.focus, Focus::Editor);
    }

    #[test]
    fn test_editor_preference_persistence() {
        let (mut app, _temp_dir) = create_test_app();

        // Enable raw editor and set preference
        app.show_raw_editor = true;
        app.prefer_raw_editor = true;

        // Switch focus away and back
        app.focus = Focus::Left;
        app.focus = Focus::Editor;

        // Preference should persist
        assert!(app.prefer_raw_editor);
    }

    #[test]
    fn test_command_mode_toggle() {
        let (mut app, _temp_dir) = create_test_app();

        app.command_mode = false;
        app.command_mode = true;
        assert!(app.command_mode);

        app.command_mode = false;
        assert!(!app.command_mode);
    }

    #[test]
    fn test_command_buffer_operations() {
        let (mut app, _temp_dir) = create_test_app();

        app.command_buffer = "test".to_string();
        assert_eq!(app.command_buffer, "test");

        app.command_buffer.push_str(" command");
        assert_eq!(app.command_buffer, "test command");

        app.command_buffer.clear();
        assert_eq!(app.command_buffer, "");
    }

    #[test]
    fn test_left_pane_toggle() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(app.show_left_pane);

        app.show_left_pane = false;
        assert!(!app.show_left_pane);

        app.show_left_pane = true;
        assert!(app.show_left_pane);
    }

    #[test]
    fn test_help_toggle() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(!app.show_help);

        app.show_help = true;
        assert!(app.show_help);

        app.show_help = false;
        assert!(!app.show_help);
    }

    #[test]
    fn test_delete_confirmation() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(!app.show_delete_confirmation);
        assert!(app.delete_target.is_none());

        let test_path = PathBuf::from("/test/file.md");
        app.show_delete_confirmation = true;
        app.delete_target = Some(test_path.clone());

        assert!(app.show_delete_confirmation);
        assert_eq!(app.delete_target, Some(test_path));
    }

    #[test]
    fn test_video_controls() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(!app.playing);
        assert!(!app.autoplay);

        app.playing = true;
        assert!(app.playing);

        app.autoplay = true;
        assert!(app.autoplay);
    }

    #[test]
    fn test_scroll_state() {
        let (mut app, _temp_dir) = create_test_app();

        assert_eq!(app.scroll_state.offset, 0);
        assert_eq!(app.h_scroll, 0);

        app.scroll_state.offset = 10;
        app.h_scroll = 5;

        assert_eq!(app.scroll_state.offset, 10);
        assert_eq!(app.h_scroll, 5);
    }

    #[test]
    fn test_dialog_states() {
        let (mut app, _temp_dir) = create_test_app();

        // Test all dialog states
        assert!(!app.show_new_file_dialog);
        assert!(!app.show_copy_dialog);
        assert!(!app.show_move_dialog);
        assert!(!app.show_mkdir_dialog);

        app.show_new_file_dialog = true;
        assert!(app.show_new_file_dialog);

        app.show_copy_dialog = true;
        assert!(app.show_copy_dialog);

        app.show_move_dialog = true;
        assert!(app.show_move_dialog);

        app.show_mkdir_dialog = true;
        assert!(app.show_mkdir_dialog);
    }

    #[test]
    fn test_file_picker_mode() {
        let (mut app, _temp_dir) = create_test_app();

        assert!(!app.file_picker_mode);

        app.file_picker_mode = true;
        assert!(app.file_picker_mode);

        app.file_picker_mode = false;
        assert!(!app.file_picker_mode);
    }
}
