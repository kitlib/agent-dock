use std::env;
use std::path::{Path, PathBuf};
use dirs;

/// Get user home directory, fallback to current directory if not available
pub fn user_home_dir() -> PathBuf {
    if let Ok(home) = env::var("AGENT_DOCK_TEST_HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Normalize path by replacing backslashes with forward slashes
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

/// Resolve agent root path, handling tilde prefix and relative paths
pub fn resolve_agent_root(root_path: &str) -> PathBuf {
    resolve_path(root_path)
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

/// Display path with tilde prefix for user home directory
pub fn display_path(relative_path: &Path) -> String {
    PathBuf::from("~")
        .join(relative_path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let path = PathBuf::from("C:\\Users\\test\\file.txt");
        assert_eq!(normalize_path(&path), "C:/Users/test/file.txt");
    }

    #[test]
    fn test_resolve_agent_root_absolute() {
        let absolute = "C:/Users/test/.claude";
        let resolved = resolve_agent_root(absolute);
        assert_eq!(resolved, PathBuf::from(absolute));
    }

    #[test]
    fn test_resolve_agent_root_tilde() {
        let tilde_path = "~/.claude";
        let resolved = resolve_agent_root(tilde_path);
        assert!(resolved.ends_with(".claude"));
        assert!(!resolved.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_user_home_dir_returns_valid_path() {
        let home = user_home_dir();
        assert!(home.is_absolute() || home == PathBuf::from("."));
    }

    #[test]
    fn test_display_path() {
        let relative = Path::new(".claude/skills");
        let displayed = display_path(relative);
        assert!(displayed.starts_with("~"));
        assert!(displayed.contains("/.claude/skills"));
    }
}
