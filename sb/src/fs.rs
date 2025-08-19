use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub fn resolve_link(current_file: &Path, link: &str) -> PathBuf {
    let p = std::path::Path::new(link);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        current_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(p)
    }
}
