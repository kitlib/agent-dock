use std::fs;
use std::path::Path;
use crate::services::Skill;
use crate::services::ServiceError;
use crate::infrastructure::utils::fs::{copy_recursive, ensure_parent_dir, remove_path};

/// Rename skill entry file between enabled and disabled states
pub fn rename_entry_file(
    skill_path: &Path,
    entry_file_path: &Path,
    enabled: bool,
) -> Result<(), ServiceError> {
    let (active_entry_path, disabled_entry_path) =
        Skill::resolve_entry_paths_for(entry_file_path)?;

    Skill::validate_path_consistency(skill_path, &active_entry_path)?;

    match (active_entry_path.is_file(), disabled_entry_path.is_file()) {
        (false, false) => {
            return Err(ServiceError::SkillNotFound(entry_file_path.display().to_string()).into());
        }
        (true, true) => {
            let conflict_path = if enabled {
                &active_entry_path
            } else {
                &disabled_entry_path
            };
            return Err(ServiceError::ConflictingEntryFiles(
                conflict_path.display().to_string()
            ).into());
        }
        (true, false) if enabled => return Ok(()),
        (false, true) if !enabled => return Ok(()),
        _ => {}
    }

    let (source_path, target_path) = if enabled {
        (&disabled_entry_path, &active_entry_path)
    } else {
        (&active_entry_path, &disabled_entry_path)
    };

    if target_path.exists() {
        return Err(ServiceError::ConflictingEntryFiles(
            target_path.display().to_string()
        ).into());
    }

    fs::rename(source_path, target_path).map_err(|error| ServiceError::Internal(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    const SKILL_ENTRY_FILE: &str = "SKILL.md";
    const DISABLED_SKILL_ENTRY_FILE: &str = "SKILL.md.disabled";

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-dock-skill-fs-test-{name}-{unique}"))
    }

    fn write_entry(skill_dir: &PathBuf, name: &str) -> PathBuf {
        fs::create_dir_all(skill_dir).expect("create skill dir");
        let path = skill_dir.join(name);
        fs::write(&path, "# Test skill\n\nBody.").expect("write skill entry");
        path
    }

    #[test]
    fn disable_skill_renames_enabled_entry() {
        let root = temp_dir("disable");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);

        rename_entry_file(
            &skill_dir,
            &entry,
            false,
        )
        .expect("disable skill");

        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_skill_renames_disabled_entry() {
        let root = temp_dir("enable");
        let skill_dir = root.join("demo-skill");
        let entry = skill_dir.join(SKILL_ENTRY_FILE);
        write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        rename_entry_file(
            &skill_dir,
            &entry,
            true,
        )
        .expect("enable skill");

        assert!(skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(!skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_conflicting_entries() {
        let root = temp_dir("conflict");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);
        write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

        let error = rename_entry_file(
            &skill_dir,
            &entry,
            false,
        )
        .expect_err("conflict should fail");

        assert!(error.to_string().contains("Conflicting entry files"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_missing_entry_file() {
        let root = temp_dir("missing-entry");
        let skill_dir = root.join("demo-skill");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        let missing_entry = skill_dir.join(SKILL_ENTRY_FILE);

        let error = rename_entry_file(
            &skill_dir,
            &missing_entry,
            false,
        )
        .expect_err("missing entry should fail");

        assert!(error.to_string().contains("Skill not found"));
        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(!skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_accepts_disabled_entry_path_when_disabling() {
        let root = temp_dir("stale-entry");
        let skill_dir = root.join("demo-skill");
        write_entry(&skill_dir, SKILL_ENTRY_FILE);
        let stale_entry = skill_dir.join(DISABLED_SKILL_ENTRY_FILE);

        rename_entry_file(
            &skill_dir,
            &stale_entry,
            false,
        )
        .expect("disable using disabled entry path");

        assert!(!skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_mismatched_skill_file_path() {
        let root = temp_dir("mismatched-file");
        fs::create_dir_all(&root).expect("create temp root");
        let skill_file = root.join("feat.md");
        let entry_file = root.join("other.md");
        fs::write(&skill_file, "# Command\n").expect("write skill markdown");
        fs::write(&entry_file, "# Other\n").expect("write entry markdown");

        let error = rename_entry_file(
            &skill_file,
            &entry_file,
            false,
        )
        .expect_err("mismatched file should fail");

        assert!(error.to_string().contains("Skill file path does not match entry file path"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn disable_skill_with_custom_entry_file() {
        let root = temp_dir("custom-disable");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, "custom-skill.md");

        rename_entry_file(
            &skill_dir,
            &entry,
            false,
        )
        .expect("disable skill with custom entry");

        assert!(!skill_dir.join("custom-skill.md").exists());
        assert!(skill_dir.join("custom-skill.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn disable_single_file_skill_renames_entry() {
        let root = temp_dir("single-file-disable");
        fs::create_dir_all(&root).expect("create temp root");
        let entry = root.join("feat.md");
        fs::write(&entry, "# Command\n").expect("write markdown");

        rename_entry_file(&entry, &entry, false)
            .expect("disable single-file skill");

        assert!(!entry.exists());
        assert!(root.join("feat.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_single_file_skill_uses_canonical_entry_path() {
        let root = temp_dir("single-file-enable");
        fs::create_dir_all(&root).expect("create temp root");
        let canonical_entry = root.join("feat.md");
        let disabled_entry = root.join("feat.md.disabled");
        fs::write(&disabled_entry, "# Command\n").expect("write markdown");

        rename_entry_file(
            &disabled_entry,
            &canonical_entry,
            true,
        )
        .expect("enable single-file skill");

        assert!(canonical_entry.exists());
        assert!(!disabled_entry.exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn enable_skill_with_custom_entry_file_uses_canonical_path() {
        let root = temp_dir("custom-enable");
        let skill_dir = root.join("demo-skill");
        let entry = skill_dir.join("custom-skill.md");
        write_entry(&skill_dir, "custom-skill.md.disabled");

        rename_entry_file(
            &skill_dir,
            &entry,
            true,
        )
        .expect("enable skill with custom entry");

        assert!(skill_dir.join("custom-skill.md").exists());
        assert!(!skill_dir.join("custom-skill.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn delete_local_skill_removes_directory_skill() {
        let root = temp_dir("delete-directory");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, SKILL_ENTRY_FILE);

        delete_skill(&skill_dir, &entry)
            .expect("delete skill directory");

        assert!(!skill_dir.exists());
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}

/// Delete skill at path (file or directory)
pub fn delete_skill(skill_path: &Path, entry_file_path: &Path) -> Result<(), ServiceError> {
    let (active_entry_path, disabled_entry_path) =
        Skill::resolve_entry_paths_for(entry_file_path)?;

    Skill::validate_path_consistency(skill_path, &active_entry_path)?;

    let skill_entry = skill_path;
    if skill_entry.is_dir() {
        if !active_entry_path.is_file() && !disabled_entry_path.is_file() {
            return Err(ServiceError::SkillNotFound(entry_file_path.display().to_string()).into());
        }

        return fs::remove_dir_all(skill_entry).map_err(|error| ServiceError::Internal(error.to_string()));
    }

    let existing_entry_path = Skill::resolve_existing_entry_path(skill_path, entry_file_path)?;
    fs::remove_file(existing_entry_path).map_err(|error| ServiceError::Internal(error.to_string()))
}

/// Copy skill from source to destination
pub fn copy_skill(source_path: &Path, destination_path: &Path) -> Result<(), ServiceError> {
    if source_path.is_dir() {
        return copy_recursive(source_path, destination_path)
            .map_err(|e| ServiceError::Internal(e.to_string()));
    }

    ensure_parent_dir(destination_path).map_err(|e| ServiceError::Internal(e.to_string()))?;
    fs::copy(source_path, destination_path).map_err(|error| ServiceError::Internal(error.to_string()))?;
    Ok(())
}

/// Remove existing skill at destination path
pub fn remove_existing_skill(existing_path: &Path) -> Result<(), ServiceError> {
    remove_path(existing_path).map_err(|e| ServiceError::Internal(e.to_string()))
}
