//! Path utility functions
use std::env;
use std::path::{Path, PathBuf};
use dirs;

/// Get user home directory, with test override support
pub fn user_home_dir() -> PathBuf {
    if let Ok(home) = env::var("AGENT_DOCK_TEST_HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Normalize path, convert Windows backslashes to forward slashes
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Resolve path, support ~ as home directory
pub fn resolve_path(path: &str) -> PathBuf {
    // Handle ~ prefix
    if let Some(relative_path) = path
        .strip_prefix("~/")
        .or_else(|| path.strip_prefix("~\\"))
    {
        return user_home_dir().join(relative_path);
    }

    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        return path_buf;
    }

    // Relative path, resolve from home directory
    user_home_dir().join(path_buf)
}

/// Resolve agent root path
pub fn resolve_agent_root(root_path: &str) -> PathBuf {
    resolve_path(root_path)
}

/// Ensure parent directory exists
pub fn ensure_parent_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!(
            "Path has no parent directory: {}",
            normalize_path(path)
        ));
    };

    std::fs::create_dir_all(parent).map_err(|e| e.to_string())
}

/// Atomic write to file: write to temp file first, then rename to target
pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    // Generate temporary file path in the same directory
    let mut tmp_path = path.to_path_buf();
    let rand_suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() % 1000000;
    tmp_path.set_extension(format!("tmp_{}", rand_suffix));

    // Write to temp file
    std::fs::write(&tmp_path, content).map_err(|e| e.to_string())?;

    // Atomic rename to target file
    std::fs::rename(&tmp_path, path).map_err(|e| e.to_string())
}