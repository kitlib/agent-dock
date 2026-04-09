use std::{fs, path::Path};

use tauri_plugin_opener::OpenerExt;

use crate::dto::skills::{LocalSkillDetailDto, LocalSkillSummaryDto, SkillScanTargetDto};
use crate::services::skill_discovery_service;

const SKILL_ENTRY_FILE: &str = "SKILL.md";
const DISABLED_SKILL_ENTRY_FILE: &str = "SKILL.md.disabled";

fn set_local_skill_enabled_at_path(
    skill_path: &str,
    entry_file_path: &str,
    enabled: bool,
) -> Result<(), String> {
    let skill_dir = Path::new(skill_path);
    if !skill_dir.exists() {
        return Err(format!("Skill path not found: {skill_path}"));
    }
    if !skill_dir.is_dir() {
        return Err(format!("Skill path is not a directory: {skill_path}"));
    }

    let entry_path = Path::new(entry_file_path);
    let entry_name = entry_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid skill entry file path: {entry_file_path}"))?;
    if entry_name != SKILL_ENTRY_FILE && entry_name != DISABLED_SKILL_ENTRY_FILE {
        return Err(format!("Unsupported skill entry file: {entry_file_path}"));
    }

    let entry_parent = entry_path
        .parent()
        .ok_or_else(|| format!("Skill entry file has no parent directory: {entry_file_path}"))?;
    if entry_parent != skill_dir {
        return Err(format!(
            "Skill entry file does not belong to skill directory: {entry_file_path}"
        ));
    }

    if !entry_path.is_file() {
        return Err(format!("Skill entry file not found: {entry_file_path}"));
    }

    let enabled_entry = skill_dir.join(SKILL_ENTRY_FILE);
    let disabled_entry = skill_dir.join(DISABLED_SKILL_ENTRY_FILE);
    let enabled_exists = enabled_entry.is_file();
    let disabled_exists = disabled_entry.is_file();

    if enabled_exists && disabled_exists {
        return Err(format!("Conflicting skill entry files found in: {skill_path}"));
    }

    if enabled {
        if entry_path == enabled_entry && enabled_exists {
            return Ok(());
        }
        if entry_path != disabled_entry {
            return Err(format!("Skill entry file is not the disabled entry: {entry_file_path}"));
        }
        if !disabled_exists {
            return Err(format!("Disabled skill entry file not found in: {skill_path}"));
        }

        fs::rename(&disabled_entry, &enabled_entry).map_err(|error| error.to_string())?;
        return Ok(());
    }

    if entry_path == disabled_entry && disabled_exists {
        return Ok(());
    }
    if entry_path != enabled_entry {
        return Err(format!("Skill entry file is not the enabled entry: {entry_file_path}"));
    }
    if !enabled_exists {
        return Err(format!("Enabled skill entry file not found in: {skill_path}"));
    }

    fs::rename(&enabled_entry, &disabled_entry).map_err(|error| error.to_string())
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
    use super::{set_local_skill_enabled_at_path, DISABLED_SKILL_ENTRY_FILE, SKILL_ENTRY_FILE};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

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
        let entry = write_entry(&skill_dir, DISABLED_SKILL_ENTRY_FILE);

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

        assert!(error.contains("Conflicting skill entry files"));

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
    fn toggle_skill_rejects_stale_entry_file_path() {
        let root = temp_dir("stale-entry");
        let skill_dir = root.join("demo-skill");
        write_entry(&skill_dir, SKILL_ENTRY_FILE);
        let stale_entry = skill_dir.join(DISABLED_SKILL_ENTRY_FILE);

        let error = set_local_skill_enabled_at_path(
            &skill_dir.to_string_lossy(),
            &stale_entry.to_string_lossy(),
            false,
        )
        .expect_err("stale entry should fail");

        assert!(error.contains("Skill entry file not found"));
        assert!(skill_dir.join(SKILL_ENTRY_FILE).exists());
        assert!(!skill_dir.join(DISABLED_SKILL_ENTRY_FILE).exists());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn toggle_skill_rejects_non_directory_skill_path() {
        let root = temp_dir("nondir");
        fs::create_dir_all(&root).expect("create temp root");
        let file_path = root.join("feat.md");
        fs::write(&file_path, "# Command\n").expect("write markdown");

        let error = set_local_skill_enabled_at_path(
            &file_path.to_string_lossy(),
            &file_path.to_string_lossy(),
            false,
        )
        .expect_err("non-directory should fail");

        assert!(error.contains("not a directory"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
