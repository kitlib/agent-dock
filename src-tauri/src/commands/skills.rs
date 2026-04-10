use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri_plugin_opener::OpenerExt;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::services::skill_discovery_service;

const DISABLED_SUFFIX: &str = ".disabled";

fn entry_file_name(entry_path: &Path, entry_file_path: &str) -> Result<String, String> {
    entry_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| format!("Invalid skill entry file path: {entry_file_path}"))
}

fn enabled_entry_path(entry_path: &Path, entry_file_path: &str) -> Result<PathBuf, String> {
    let entry_name = entry_file_name(entry_path, entry_file_path)?;
    let enabled_name = entry_name.strip_suffix(DISABLED_SUFFIX).unwrap_or(&entry_name);
    Ok(entry_path.with_file_name(enabled_name))
}

fn disabled_entry_path(entry_path: &Path, entry_file_path: &str) -> Result<PathBuf, String> {
    let entry_name = entry_file_name(entry_path, entry_file_path)?;
    Ok(entry_path.with_file_name(format!("{}{}", entry_name, DISABLED_SUFFIX)))
}

fn validate_skill_path(skill_path: &str, canonical_entry_path: &Path) -> Result<(), String> {
    let skill_entry = Path::new(skill_path);
    if !skill_entry.exists() {
        return Err(format!("Skill path not found: {skill_path}"));
    }

    if skill_entry.is_dir() {
        let entry_parent = canonical_entry_path.parent().ok_or_else(|| {
            format!(
                "Skill entry file has no parent directory: {}",
                canonical_entry_path.display()
            )
        })?;
        if entry_parent != skill_entry {
            return Err(format!(
                "Skill entry file does not belong to skill directory: {}",
                canonical_entry_path.display()
            ));
        }
        return Ok(());
    }

    if skill_entry.is_file() {
        let canonical_entry_path_str = canonical_entry_path.to_string_lossy();
        let disabled_entry_candidate =
            disabled_entry_path(canonical_entry_path, canonical_entry_path_str.as_ref())?;
        if skill_entry != canonical_entry_path && skill_entry != disabled_entry_candidate {
            return Err(format!(
                "Skill file path does not match entry file path: {skill_path}"
            ));
        }
        return Ok(());
    }

    Err(format!("Skill path is neither a file nor a directory: {skill_path}"))
}

fn set_local_skill_enabled_at_path(
    skill_path: &str,
    entry_file_path: &str,
    enabled: bool,
) -> Result<(), String> {
    let entry_path = Path::new(entry_file_path);
    let active_entry_path = enabled_entry_path(entry_path, entry_file_path)?;
    let disabled_entry_path = disabled_entry_path(&active_entry_path, entry_file_path)?;

    validate_skill_path(skill_path, &active_entry_path)?;

    let active_exists = active_entry_path.is_file();
    let disabled_exists = disabled_entry_path.is_file();
    if !active_exists && !disabled_exists {
        return Err(format!("Skill entry file not found: {entry_file_path}"));
    }
    if active_exists && disabled_exists {
        let conflict_path = if enabled {
            &active_entry_path
        } else {
            &disabled_entry_path
        };
        return Err(format!(
            "Target entry file already exists: {}",
            conflict_path.display()
        ));
    }

    if enabled {
        if active_exists {
            return Ok(());
        }
        if active_entry_path.exists() {
            return Err(format!(
                "Target entry file already exists: {}",
                active_entry_path.display()
            ));
        }
        return fs::rename(&disabled_entry_path, &active_entry_path).map_err(|error| error.to_string());
    }

    if disabled_exists {
        return Ok(());
    }
    if disabled_entry_path.exists() {
        return Err(format!(
            "Target entry file already exists: {}",
            disabled_entry_path.display()
        ));
    }

    fs::rename(&active_entry_path, &disabled_entry_path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_local_skills(scan_targets: Vec<SkillScanTargetDto>) -> Result<Vec<LocalSkillSummaryDto>, String> {
    println!(
        "[skills] list_local_skills command targets: {:?}",
        scan_targets
            .iter()
            .map(|target| format!("{}|{}|{}", target.agent_id, target.agent_type, target.root_path))
            .collect::<Vec<_>>()
    );

    let skills = skill_discovery_service::list_local_skills(scan_targets);

    println!(
        "[skills] list_local_skills command result: {:?}",
        skills
            .iter()
            .map(|skill| format!(
                "{}|owner={}|agentType={}|path={}",
                skill.id, skill.owner_agent_id, skill.agent_type, skill.skill_path
            ))
            .collect::<Vec<_>>()
    );

    Ok(skills)
}

#[tauri::command]
pub fn get_local_skill_detail(
    scan_targets: Vec<SkillScanTargetDto>,
    skill_id: String,
) -> Result<LocalSkillDetailDto, String> {
    println!(
        "[skills] get_local_skill_detail command request: skill_id={}, targets={:?}",
        skill_id,
        scan_targets
            .iter()
            .map(|target| format!("{}|{}|{}", target.agent_id, target.agent_type, target.root_path))
            .collect::<Vec<_>>()
    );

    let detail = skill_discovery_service::get_local_skill_detail(scan_targets, &skill_id)?;

    println!(
        "[skills] get_local_skill_detail command result: {}|owner={}|agentType={}|path={}",
        detail.id, detail.owner_agent_id, detail.agent_type, detail.skill_path
    );

    Ok(detail)
}

#[tauri::command]
pub fn set_local_skill_enabled(
    skill_path: String,
    entry_file_path: String,
    enabled: bool,
) -> Result<(), String> {
    set_local_skill_enabled_at_path(&skill_path, &entry_file_path, enabled)
}

#[tauri::command]
pub fn open_skill_folder(app: tauri::AppHandle, skill_path: String) -> Result<(), String> {
    let path = Path::new(&skill_path);
    if !path.exists() {
        return Err(format!("Skill path not found: {skill_path}"));
    }

    let open_path = if path.is_dir() {
        &skill_path
    } else {
        path.parent()
            .and_then(|parent| parent.to_str())
            .ok_or_else(|| format!("Skill path has no parent directory: {skill_path}"))?
    };

    app.opener()
        .open_path(open_path, None::<&str>)
        .map_err(|error: tauri_plugin_opener::Error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::set_local_skill_enabled_at_path;
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
        std::env::temp_dir().join(format!("agent-dock-skills-command-{name}-{unique}"))
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

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
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

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
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

        let error = set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            false,
        )
        .expect_err("conflict should fail");

        assert!(error.contains("Target entry file already exists"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_missing_entry_file() {
        let root = temp_dir("missing-entry");
        let skill_dir = root.join("demo-skill");
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        let missing_entry = skill_dir.join(SKILL_ENTRY_FILE);

        let error = set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &missing_entry.to_string_lossy(),
            false,
        )
        .expect_err("missing entry should fail");

        assert!(error.contains("Skill entry file not found"));
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

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &stale_entry.to_string_lossy(),
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

        let error = set_local_skill_enabled_at_path(
            &skill_file.to_string_lossy(),
            &entry_file.to_string_lossy(),
            false,
        )
        .expect_err("mismatched file should fail");

        assert!(error.contains("Skill file path does not match entry file path"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn disable_skill_with_custom_entry_file() {
        let root = temp_dir("custom-disable");
        let skill_dir = root.join("demo-skill");
        let entry = write_entry(&skill_dir, "custom-skill.md");

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
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

        set_local_skill_enabled_at_path(&entry.to_string_lossy(), &entry.to_string_lossy(), false)
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

        set_local_skill_enabled_at_path(
            &disabled_entry.to_string_lossy(),
            &canonical_entry.to_string_lossy(),
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

        set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &entry.to_string_lossy(),
            true,
        )
        .expect("enable skill with custom entry");

        assert!(skill_dir.join("custom-skill.md").exists());
        assert!(!skill_dir.join("custom-skill.md.disabled").exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
