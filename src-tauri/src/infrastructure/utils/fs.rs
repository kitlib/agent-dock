use std::fs;
use std::io;
use std::path::Path;

/// Ensure parent directory exists for a given path
pub fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Recursively copy directory contents from source to destination
pub fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let next_dst = dst.join(entry.file_name());

        if path.is_dir() {
            copy_recursive(&path, &next_dst)?;
        } else {
            ensure_parent_dir(&next_dst)?;
            fs::copy(&path, &next_dst)?;
        }
    }

    Ok(())
}

/// Remove path (file or directory)
pub fn remove_path(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-utils-{name}-{unique}"))
    }

    #[test]
    fn test_ensure_parent_dir_creates_nested_dirs() {
        let root = temp_dir("ensure-parent");
        let nested_file = root.join("a/b/c/file.txt");

        ensure_parent_dir(&nested_file).expect("create parent dirs");
        assert!(nested_file.parent().unwrap().exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn test_copy_recursive_copies_directory_tree() {
        let root = temp_dir("copy-recursive");
        let src = root.join("src");
        let dst = root.join("dst");

        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("file1.txt"), "content1").expect("write file1");
        fs::create_dir_all(src.join("subdir")).expect("create subdir");
        fs::write(src.join("subdir/file2.txt"), "content2").expect("write file2");

        copy_recursive(&src, &dst).expect("copy recursive");

        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.join("file1.txt")).unwrap(),
            "content1"
        );

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn test_remove_path_removes_file() {
        let root = temp_dir("remove-file");
        fs::create_dir_all(&root).expect("create root");
        let file = root.join("test.txt");
        fs::write(&file, "test").expect("write file");

        remove_path(&file).expect("remove file");
        assert!(!file.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn test_remove_path_removes_directory() {
        let root = temp_dir("remove-dir");
        let dir = root.join("testdir");
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(dir.join("file.txt"), "test").expect("write file");

        remove_path(&dir).expect("remove dir");
        assert!(!dir.exists());

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn test_remove_path_succeeds_on_nonexistent() {
        let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");
        remove_path(&nonexistent).expect("should succeed on nonexistent");
    }
}
